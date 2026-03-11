use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "launch_app".into(),
    name: "Launch Application".into(),
    description: "Launch a desktop application by name".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct LaunchAppSkill;

#[async_trait::async_trait]
impl Skill for LaunchAppSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let app = input
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NervaError::InvalidInput("missing 'app' field".into()))?;

        nerva_os::process::launch_app(app).await?;

        Ok(serde_json::json!({ "launched": app }))
    }
}
