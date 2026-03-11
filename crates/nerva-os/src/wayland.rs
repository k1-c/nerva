use nerva_core::{NervaError, WindowInfo};
use tokio::process::Command;

pub async fn list_windows() -> Result<Vec<WindowInfo>, NervaError> {
    let json = hyprctl_json(&["clients", "-j"]).await?;

    let Some(arr) = json.as_array() else {
        return Ok(vec![]);
    };

    let windows = arr
        .iter()
        .map(|w| WindowInfo {
            id: w["address"].as_str().unwrap_or("").to_string(),
            title: w["title"].as_str().unwrap_or("").to_string(),
            app_id: w["class"].as_str().unwrap_or("").to_string(),
            focused: w["focusHistoryID"].as_i64() == Some(0),
        })
        .collect();

    Ok(windows)
}

pub async fn get_active_window() -> Result<Option<WindowInfo>, NervaError> {
    let json = hyprctl_json(&["activewindow", "-j"]).await?;

    if json.is_null() || json.get("address").is_none() {
        return Ok(None);
    }

    Ok(Some(WindowInfo {
        id: json["address"].as_str().unwrap_or("").to_string(),
        title: json["title"].as_str().unwrap_or("").to_string(),
        app_id: json["class"].as_str().unwrap_or("").to_string(),
        focused: true,
    }))
}

pub async fn focus_window(address: &str) -> Result<(), NervaError> {
    let output = Command::new("hyprctl")
        .args(["dispatch", "focuswindow", &format!("address:{address}")])
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("hyprctl dispatch failed: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "Failed to focus window: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

async fn hyprctl_json(args: &[&str]) -> Result<serde_json::Value, NervaError> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => serde_json::from_slice(&o.stdout)
            .map_err(|e| NervaError::OsError(format!("Failed to parse hyprctl output: {e}"))),
        Ok(o) => Err(NervaError::OsError(format!(
            "hyprctl failed: {}",
            String::from_utf8_lossy(&o.stderr)
        ))),
        Err(_) => {
            tracing::warn!("hyprctl not available");
            Ok(serde_json::Value::Null)
        }
    }
}
