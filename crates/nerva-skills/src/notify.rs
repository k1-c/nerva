use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "notify".into(),
    name: "Send Notification".into(),
    description: "Send a desktop notification".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct NotifySkill;

#[async_trait::async_trait]
impl Skill for NotifySkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let summary = input
            .get("summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NervaError::InvalidInput("missing 'summary' field".into()))?;

        let body = input.get("body").and_then(|v| v.as_str());

        nerva_os::notification::send_notification(summary, body).await?;

        Ok(serde_json::json!({ "sent": true }))
    }
}
