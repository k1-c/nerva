use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "list_windows".into(),
    name: "List Windows".into(),
    description: "List all open windows with their titles and app IDs".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct ListWindowsSkill;

#[async_trait::async_trait]
impl Skill for ListWindowsSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let windows = nerva_os::wayland::list_windows().await?;
        Ok(serde_json::json!({ "windows": windows }))
    }
}
