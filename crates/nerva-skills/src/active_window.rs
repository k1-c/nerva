use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "get_active_window".into(),
    name: "Get Active Window".into(),
    description: "Get information about the currently focused window".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct GetActiveWindowSkill;

#[async_trait::async_trait]
impl Skill for GetActiveWindowSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let window = nerva_os::wayland::get_active_window().await?;

        match window {
            Some(w) => Ok(serde_json::json!({
                "found": true,
                "window": w,
            })),
            None => Ok(serde_json::json!({
                "found": false,
                "window": null,
            })),
        }
    }
}
