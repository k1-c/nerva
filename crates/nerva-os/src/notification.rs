use nerva_core::NervaError;
use tokio::process::Command;

pub async fn send_notification(summary: &str, body: Option<&str>) -> Result<(), NervaError> {
    let mut cmd = Command::new("notify-send");
    cmd.arg(summary);
    if let Some(body) = body {
        cmd.arg(body);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to send notification: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "notify-send failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

pub async fn open_path(path: &str) -> Result<(), NervaError> {
    let output = Command::new("xdg-open")
        .arg(path)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to open path: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "xdg-open failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
