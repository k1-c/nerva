use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "open_path".into(),
    name: "Open Path".into(),
    description: "Open a file or URL with the default application".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct OpenPathSkill;

#[async_trait::async_trait]
impl Skill for OpenPathSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NervaError::InvalidInput("missing 'path' field".into()))?;

        nerva_os::notification::open_path(path).await?;

        Ok(serde_json::json!({ "opened": path }))
    }
}
