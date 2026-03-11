use std::sync::LazyLock;

use nerva_core::context::DesktopContext;
use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "gather_context".into(),
    name: "Gather Context".into(),
    description: "Assemble current desktop context (active window + clipboard + screen text)"
        .into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct GatherContextSkill;

#[async_trait::async_trait]
impl Skill for GatherContextSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let include_ocr = input
            .get("include_ocr")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut ctx = DesktopContext::new();

        // Get active window (best-effort)
        match nerva_os::wayland::get_active_window().await {
            Ok(Some(window)) => {
                ctx = ctx.with_active_window(window);
            }
            Ok(None) => {}
            Err(e) => {
                tracing::debug!(error = %e, "Failed to get active window");
            }
        }

        // Get clipboard (best-effort)
        match nerva_os::clipboard::read_clipboard().await {
            Ok(text) => {
                if !text.is_empty() {
                    ctx = ctx.with_clipboard(text);
                }
            }
            Err(e) => {
                tracing::debug!(error = %e, "Failed to read clipboard");
            }
        }

        // OCR screen text (optional, slower)
        if include_ocr {
            match nerva_os::screenshot::capture_screen(None).await {
                Ok(path) => {
                    match nerva_os::ocr::extract_text(&path, None).await {
                        Ok(text) => {
                            if !text.is_empty() {
                                ctx = ctx.with_screen_text(text);
                            }
                            ctx = ctx.with_screenshot_path(path.to_string_lossy().to_string());
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "OCR failed");
                        }
                    }
                    let _ = tokio::fs::remove_file(&path).await;
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Screenshot failed");
                }
            }
        }

        serde_json::to_value(&ctx)
            .map_err(|e| NervaError::ExecutionError(format!("Failed to serialize context: {e}")))
    }
}
