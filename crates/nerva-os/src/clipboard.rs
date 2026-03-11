use nerva_core::NervaError;
use tokio::process::Command;

pub async fn read_clipboard() -> Result<String, NervaError> {
    // Try wl-paste first (Wayland)
    let output = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            return Ok(String::from_utf8_lossy(&o.stdout).to_string());
        }
        _ => {}
    }

    // Fallback to xclip (X11)
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to read clipboard: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(
            "Failed to read clipboard: no supported clipboard tool found".into(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
