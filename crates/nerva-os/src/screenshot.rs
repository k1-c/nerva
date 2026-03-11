use std::path::{Path, PathBuf};

use nerva_core::NervaError;
use tokio::process::Command;

/// Capture the screen, trying xdg-desktop-portal first, then falling back to grim.
pub async fn capture_screen(output_path: Option<&Path>) -> Result<PathBuf, NervaError> {
    let path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_screenshot_path);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    // Try xdg-desktop-portal screenshot first (works across compositors)
    match capture_via_portal(&path).await {
        Ok(()) => return Ok(path),
        Err(e) => {
            tracing::debug!(error = %e, "Portal screenshot failed, falling back to grim");
        }
    }

    // Fallback to grim (wlroots-based compositors)
    capture_via_grim(&path).await?;
    Ok(path)
}

/// Capture a specific region of the screen.
/// `region` format: "X,Y WxH" (e.g., "100,200 640x480")
pub async fn capture_region(
    region: &str,
    output_path: Option<&Path>,
) -> Result<PathBuf, NervaError> {
    let path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_screenshot_path);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let output = Command::new("grim")
        .args(["-g", region])
        .arg(&path)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to capture region (grim): {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "grim region capture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(path)
}

/// Capture the active window only.
pub async fn capture_active_window(output_path: Option<&Path>) -> Result<PathBuf, NervaError> {
    let path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_screenshot_path);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    // Get active window geometry from hyprctl
    let geometry = active_window_geometry().await?;

    let output = Command::new("grim")
        .args(["-g", &geometry])
        .arg(&path)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to capture window (grim): {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "grim window capture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(path)
}

async fn capture_via_portal(path: &Path) -> Result<(), NervaError> {
    // Use xdg-desktop-portal Screenshot via gdbus
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest=org.freedesktop.portal.Desktop",
            "--object-path=/org/freedesktop/portal/desktop",
            "--method=org.freedesktop.portal.Screenshot.Screenshot",
            "",
            "{}",
        ])
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("gdbus call failed: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "Portal screenshot failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Portal returns a URI to the screenshot, parse it and copy to desired path
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(uri) = extract_uri_from_portal_response(&stdout) {
        let source = uri.strip_prefix("file://").unwrap_or(&uri);
        tokio::fs::copy(source, path)
            .await
            .map_err(|e| NervaError::OsError(format!("Failed to copy portal screenshot: {e}")))?;
        // Clean up the temporary portal file
        let _ = tokio::fs::remove_file(source).await;
        Ok(())
    } else {
        Err(NervaError::OsError(
            "Could not extract URI from portal response".into(),
        ))
    }
}

async fn capture_via_grim(path: &Path) -> Result<(), NervaError> {
    let output = Command::new("grim")
        .arg(path)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to capture screen (grim): {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "grim failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

async fn active_window_geometry() -> Result<String, NervaError> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("hyprctl failed: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(
            "Failed to get active window info".into(),
        ));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| NervaError::OsError(format!("Failed to parse hyprctl output: {e}")))?;

    let x = json["at"][0].as_i64().unwrap_or(0);
    let y = json["at"][1].as_i64().unwrap_or(0);
    let w = json["size"][0].as_i64().unwrap_or(800);
    let h = json["size"][1].as_i64().unwrap_or(600);

    Ok(format!("{x},{y} {w}x{h}"))
}

fn extract_uri_from_portal_response(response: &str) -> Option<String> {
    // Portal response format: (<object path>, {'uri': <s 'file:///path/to/screenshot.png'>})
    response
        .find("file://")
        .map(|start| {
            let rest = &response[start..];
            let end = rest.find('\'').or_else(|| rest.find('"')).unwrap_or(rest.len());
            rest[..end].to_string()
        })
}

fn default_screenshot_path() -> PathBuf {
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    std::env::temp_dir().join(format!("nerva_screenshot_{ts}.png"))
}
