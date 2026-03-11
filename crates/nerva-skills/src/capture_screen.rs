use std::path::PathBuf;
use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "capture_screen".into(),
    name: "Capture Screen".into(),
    description: "Take a screenshot of the current screen".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct CaptureScreenSkill;

#[async_trait::async_trait]
impl Skill for CaptureScreenSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let output_path = input
            .get("output_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        let path =
            nerva_os::screenshot::capture_screen(output_path.as_deref()).await?;

        Ok(serde_json::json!({ "path": path.to_string_lossy() }))
    }
}
