use std::path::Path;

use nerva_core::NervaError;
use tokio::process::Command;

/// Extract text from an image using tesseract OCR.
///
/// Requires `tesseract` to be installed on the system.
/// Supports language selection (default: "eng").
pub async fn extract_text(image_path: &Path, lang: Option<&str>) -> Result<String, NervaError> {
    let lang = lang.unwrap_or("eng");

    let output = Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .args(["-l", lang])
        .arg("--psm")
        .arg("3") // fully automatic page segmentation
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to run tesseract: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(format!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

/// Check if tesseract is available on the system.
pub async fn is_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .output()
        .await
        .is_ok_and(|o| o.status.success())
}

/// List available tesseract languages.
pub async fn list_languages() -> Result<Vec<String>, NervaError> {
    let output = Command::new("tesseract")
        .arg("--list-langs")
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to query tesseract languages: {e}")))?;

    if !output.status.success() {
        return Err(NervaError::OsError(
            "tesseract --list-langs failed".into(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let langs: Vec<String> = stdout
        .lines()
        .skip(1) // first line is the tessdata path
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(langs)
}
