use std::sync::Arc;

use nerva_core::{CapabilityBus, ExecutionRequest};

use crate::protocol::{Request, Response};

pub async fn handle(bus: &Arc<CapabilityBus>, request: Request) -> Response {
    match request {
        Request::Execute { tool_id, input } => {
            let req = ExecutionRequest::new(tool_id, input);
            let result = bus.execute(req).await;
            Response::success(serde_json::to_value(&result).unwrap())
        }
        Request::ListTools => {
            let tools = bus.registry().list().await;
            Response::success(serde_json::json!({ "tools": tools }))
        }
        Request::GetLog { count } => {
            let records = bus.recent_log(count).await;
            Response::success(serde_json::json!({ "records": records }))
        }
        Request::Status => {
            let tool_count = bus.registry().list().await.len();
            Response::success(serde_json::json!({
                "status": "running",
                "tools_registered": tool_count,
            }))
        }
    }
}
