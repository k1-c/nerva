use std::sync::LazyLock;

use nerva_core::vlm::VlmClient;
use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "summarize_screen".into(),
    name: "Summarize Screen".into(),
    description: "Capture the screen and summarize its contents using OCR or VLM".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct SummarizeScreenSkill {
    vlm_client: Option<VlmClient>,
}

impl SummarizeScreenSkill {
    pub fn new(vlm_client: Option<VlmClient>) -> Self {
        Self { vlm_client }
    }
}

#[async_trait::async_trait]
impl Skill for SummarizeScreenSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let prompt = input
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Describe what is shown on this screen. Be concise.");

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

        // Try VLM first, fall back to OCR
        let (summary, method) = if let Some(ref client) = self.vlm_client {
            match client.describe_image(&screenshot_path, prompt).await {
                Ok(description) => (description, "vlm"),
                Err(e) => {
                    tracing::warn!(error = %e, "VLM failed, falling back to OCR");
                    let text = nerva_os::ocr::extract_text(&screenshot_path, None).await?;
                    (text, "ocr")
                }
            }
        } else {
            let text = nerva_os::ocr::extract_text(&screenshot_path, None).await?;
            (text, "ocr")
        };

        // Clean up temporary screenshot
        let _ = tokio::fs::remove_file(&screenshot_path).await;

        Ok(serde_json::json!({
            "summary": summary,
            "method": method,
            "window_only": window_only,
        }))
    }
}
