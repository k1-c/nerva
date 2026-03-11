#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bus::CapabilityBus;
    use crate::error::NervaError;
    use crate::policy::{PolicyConfig, PolicyEngine};
    use crate::registry::ToolRegistry;
    use crate::skill::Skill;
    use crate::types::*;

    struct MockSkill {
        meta: ToolMetadata,
    }

    impl MockSkill {
        fn safe(id: &str) -> Self {
            Self {
                meta: ToolMetadata {
                    id: id.into(),
                    name: id.into(),
                    description: "mock".into(),
                    risk: RiskTier::Safe,
                    confirmation_required: false,
                },
            }
        }
    }

    #[async_trait::async_trait]
    impl Skill for MockSkill {
        fn metadata(&self) -> &ToolMetadata {
            &self.meta
        }

        async fn execute(
            &self,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, NervaError> {
            Ok(serde_json::json!({ "echo": input }))
        }
    }

    async fn setup_bus() -> Arc<CapabilityBus> {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockSkill::safe("test_tool"))).await;
        let policy = PolicyEngine::default();
        Arc::new(CapabilityBus::new(registry, policy))
    }

    #[tokio::test]
    async fn test_execute_success() {
        let bus = setup_bus().await;
        let req = ExecutionRequest::new("test_tool", serde_json::json!({"key": "value"}));
        let result = bus.execute(req).await;

        assert_eq!(result.status, ExecutionStatus::Success);
        assert!(result.output.is_some());
        let output = result.output.unwrap();
        assert_eq!(output["echo"]["key"], "value");
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let bus = setup_bus().await;
        let req = ExecutionRequest::new("nonexistent", serde_json::json!({}));
        let result = bus.execute(req).await;

        assert_eq!(result.status, ExecutionStatus::Failed);
        assert!(result.error.unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_execute_blocked_tool() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockSkill::safe("blocked_tool"))).await;

        let config = PolicyConfig {
            blocked_tools: std::collections::HashSet::from(["blocked_tool".into()]),
            ..Default::default()
        };
        let policy = PolicyEngine::new(config);
        let bus = CapabilityBus::new(registry, policy);

        let req = ExecutionRequest::new("blocked_tool", serde_json::json!({}));
        let result = bus.execute(req).await;

        assert_eq!(result.status, ExecutionStatus::Denied);
    }

    #[tokio::test]
    async fn test_audit_log() {
        let bus = setup_bus().await;

        let req1 = ExecutionRequest::new("test_tool", serde_json::json!({"n": 1}));
        let req2 = ExecutionRequest::new("test_tool", serde_json::json!({"n": 2}));
        bus.execute(req1).await;
        bus.execute(req2).await;

        let log = bus.recent_log(10).await;
        assert_eq!(log.len(), 2);
        // recent_log returns most recent first
        assert_eq!(log[0].request.input["n"], 2);
        assert_eq!(log[1].request.input["n"], 1);
    }

    #[tokio::test]
    async fn test_registry_list() {
        let bus = setup_bus().await;
        let tools = bus.registry().list().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "test_tool");
    }
}
