use nerva_core::{NervaError, WindowInfo};
use tokio::process::Command;

pub async fn list_windows() -> Result<Vec<WindowInfo>, NervaError> {
    // Try hyprctl first (Hyprland)
    let output = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let json: serde_json::Value =
                serde_json::from_slice(&o.stdout).map_err(|e| {
                    NervaError::OsError(format!("Failed to parse hyprctl output: {e}"))
                })?;

            let windows = json
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|w| WindowInfo {
                    id: w["address"].as_str().unwrap_or("").to_string(),
                    title: w["title"].as_str().unwrap_or("").to_string(),
                    app_id: w["class"].as_str().unwrap_or("").to_string(),
                    focused: w["focusHistoryID"].as_i64() == Some(0),
                })
                .collect();

            return Ok(windows);
        }
        _ => {}
    }

    tracing::warn!("No supported compositor detected for window listing");
    Ok(vec![])
}
