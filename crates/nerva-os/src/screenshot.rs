use std::path::{Path, PathBuf};

use nerva_core::NervaError;
use tokio::process::Command;

pub async fn capture_screen(output_path: Option<&Path>) -> Result<PathBuf, NervaError> {
    let path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            std::env::temp_dir().join(format!("nerva_screenshot_{ts}.png"))
        });

    let output = Command::new("grim")
        .arg(&path)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to capture screen (grim): {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "grim failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(path)
}
