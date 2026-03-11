use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "ocr_screen".into(),
    name: "OCR Screen".into(),
    description: "Capture the screen and extract text via OCR (tesseract)".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct OcrScreenSkill;

#[async_trait::async_trait]
impl Skill for OcrScreenSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let lang = input.get("lang").and_then(|v| v.as_str());
        let window_only = input
            .get("window_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Capture screenshot
        let screenshot_path = if window_only {
            nerva_os::screenshot::capture_active_window(None).await?
        } else {
            nerva_os::screenshot::capture_screen(None).await?
        };

        // Run OCR
        let text = nerva_os::ocr::extract_text(&screenshot_path, lang).await?;

        // Clean up temporary screenshot
        let _ = tokio::fs::remove_file(&screenshot_path).await;

        Ok(serde_json::json!({
            "text": text,
            "lang": lang.unwrap_or("eng"),
            "window_only": window_only,
        }))
    }
}
