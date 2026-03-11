use crate::error::NervaError;
use crate::types::ToolMetadata;

#[async_trait::async_trait]
pub trait Skill: Send + Sync {
    fn metadata(&self) -> &ToolMetadata;

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError>;
}
