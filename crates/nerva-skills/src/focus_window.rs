use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "focus_window".into(),
    name: "Focus Window".into(),
    description: "Focus a specific window by address".into(),
    risk: RiskTier::Caution,
    confirmation_required: true,
});

pub struct FocusWindowSkill;

#[async_trait::async_trait]
impl Skill for FocusWindowSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let address = input
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NervaError::InvalidInput("missing 'address' field".into()))?;

        nerva_os::wayland::focus_window(address).await?;

        Ok(serde_json::json!({ "focused": address }))
    }
}
