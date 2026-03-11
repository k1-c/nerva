use std::sync::Arc;

use nerva_core::agent::AgentRuntime;
use nerva_core::watcher::Suggestion;
use nerva_core::{CapabilityBus, ExecutionRequest};
use tokio::sync::Mutex;

use crate::protocol::{Request, Response};

pub async fn handle(
    bus: &Arc<CapabilityBus>,
    agent: &Option<Arc<AgentRuntime>>,
    suggestions: &Arc<Mutex<Vec<Suggestion>>>,
    request: Request,
) -> Response {
    match request {
        Request::Execute { tool_id, input } => {
            let req = ExecutionRequest::new(tool_id, input);
            let result = bus.execute(req).await;
            Response::success(serde_json::to_value(&result).unwrap())
        }
        Request::Ask { query } => {
            let Some(agent) = agent else {
                return Response::error(
                    "Agent runtime not available. Configure [llm] in config.toml to enable.",
                );
            };

            match agent.ask(&query, None).await {
                Ok(response) => Response::success(serde_json::json!({
                    "answer": response.answer,
                    "steps": response.steps,
                })),
                Err(e) => Response::error(format!("Agent error: {e}")),
            }
        }
        Request::ListTools => {
            let tools = bus.registry().list().await;
            Response::success(serde_json::json!({ "tools": tools }))
        }
        Request::GetLog { count } => {
            let records = bus.recent_log(count).await;
            Response::success(serde_json::json!({ "records": records }))
        }
        Request::GetSuggestions { count } => {
            let store = suggestions.lock().await;
            let recent: Vec<_> = store.iter().rev().take(count).cloned().collect();
            Response::success(serde_json::json!({ "suggestions": recent }))
        }
        Request::Status => {
            let tool_count = bus.registry().list().await.len();
            let agent_info = agent.as_ref().map(|a| a.provider_name());
            let suggestion_count = suggestions.lock().await.len();
            Response::success(serde_json::json!({
                "status": "running",
                "tools_registered": tool_count,
                "agent_enabled": agent.is_some(),
                "agent_provider": agent_info,
                "suggestions_buffered": suggestion_count,
            }))
        }
    }
}
