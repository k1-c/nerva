use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "clipboard_read".into(),
    name: "Read Clipboard".into(),
    description: "Read the current clipboard content".into(),
    risk: RiskTier::Safe,
    confirmation_required: false,
});

pub struct ClipboardReadSkill;

#[async_trait::async_trait]
impl Skill for ClipboardReadSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let content = nerva_os::clipboard::read_clipboard().await?;
        Ok(serde_json::json!({ "content": content }))
    }
}
