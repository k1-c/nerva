use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;

use crate::policy::{PolicyDecision, PolicyEngine};
use crate::registry::ToolRegistry;
use crate::types::{ExecutionRecord, ExecutionRequest, ExecutionResult, ExecutionStatus};

pub struct CapabilityBus {
    registry: Arc<ToolRegistry>,
    policy: PolicyEngine,
    log: Mutex<Vec<ExecutionRecord>>,
}

impl CapabilityBus {
    pub fn new(registry: Arc<ToolRegistry>, policy: PolicyEngine) -> Self {
        Self {
            registry,
            policy,
            log: Mutex::new(Vec::new()),
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub async fn execute(&self, request: ExecutionRequest) -> ExecutionResult {
        let started_at = Utc::now();

        let skill = match self.registry.get(&request.tool_id).await {
            Some(s) => s,
            None => {
                let result = ExecutionResult {
                    request_id: request.id,
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Tool not found: {}", request.tool_id)),
                    started_at,
                    completed_at: Utc::now(),
                };
                self.record(request, result.clone()).await;
                return result;
            }
        };

        let decision = self.policy.evaluate(skill.metadata());
        match decision {
            PolicyDecision::Deny(reason) => {
                let result = ExecutionResult {
                    request_id: request.id,
                    status: ExecutionStatus::Denied,
                    output: None,
                    error: Some(reason),
                    started_at,
                    completed_at: Utc::now(),
                };
                self.record(request, result.clone()).await;
                return result;
            }
            PolicyDecision::RequireConfirmation => {
                // In daemon mode, auto-approve with a warning.
                // Full confirmation flow requires a UI (Phase 6).
                tracing::warn!(
                    tool_id = %request.tool_id,
                    risk = ?skill.metadata().risk,
                    "Tool requires confirmation — auto-approving in headless mode"
                );
            }
            PolicyDecision::Allow => {}
        }

        let result = match skill.execute(request.input.clone()).await {
            Ok(output) => ExecutionResult {
                request_id: request.id,
                status: ExecutionStatus::Success,
                output: Some(output),
                error: None,
                started_at,
                completed_at: Utc::now(),
            },
            Err(e) => ExecutionResult {
                request_id: request.id,
                status: ExecutionStatus::Failed,
                output: None,
                error: Some(e.to_string()),
                started_at,
                completed_at: Utc::now(),
            },
        };

        self.record(request, result.clone()).await;
        result
    }

    pub async fn recent_log(&self, count: usize) -> Vec<ExecutionRecord> {
        let log = self.log.lock().await;
        log.iter().rev().take(count).cloned().collect()
    }

    async fn record(&self, request: ExecutionRequest, result: ExecutionResult) {
        tracing::info!(
            tool_id = %request.tool_id,
            status = ?result.status,
            "Execution completed"
        );
        self.log.lock().await.push(ExecutionRecord { request, result });
    }
}
