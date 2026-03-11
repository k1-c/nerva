use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::bus::CapabilityBus;
use crate::context::DesktopContext;
use crate::error::NervaError;
use crate::llm::{ChatMessage, LlmBackend};
use crate::types::ExecutionRequest;

/// Maximum number of tool-call rounds before the agent stops.
const MAX_ROUNDS: usize = 10;

/// The Agent Runtime: interprets natural language, plans tool calls, and executes them.
pub struct AgentRuntime {
    llm: Box<dyn LlmBackend>,
    bus: Arc<CapabilityBus>,
}

/// Result of an agent `ask` invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Final text response from the agent.
    pub answer: String,
    /// Steps executed during this invocation.
    pub steps: Vec<AgentStep>,
}

/// A single step in the agent's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub tool_id: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl AgentRuntime {
    pub fn new(llm: Box<dyn LlmBackend>, bus: Arc<CapabilityBus>) -> Self {
        Self { llm, bus }
    }

    /// Process a natural language query: interpret → plan → execute → respond.
    pub async fn ask(
        &self,
        query: &str,
        context: Option<&DesktopContext>,
    ) -> Result<AgentResponse, NervaError> {
        let tools = self.bus.registry().list().await;
        let mut steps = Vec::new();

        // Build system prompt
        let system_prompt = build_system_prompt(context);
        let mut messages = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(query),
        ];

        for _round in 0..MAX_ROUNDS {
            let response = self.llm.chat(messages.clone(), Some(&tools)).await?;

            if !response.has_tool_calls() {
                // No more tool calls — we have the final answer
                return Ok(AgentResponse {
                    answer: response.content.clone(),
                    steps,
                });
            }

            // Execute each tool call
            let tool_calls = response.tool_calls.clone().unwrap_or_default();
            messages.push(response);

            for call in &tool_calls {
                if call.function.name.is_empty() {
                    continue;
                }
                let tool_id = &call.function.name;
                let input = call.function.arguments.clone();

                tracing::info!(tool_id, "Agent executing tool call");

                let request = ExecutionRequest::new(tool_id, input.clone());
                let result = self.bus.execute(request).await;

                let step = AgentStep {
                    tool_id: tool_id.clone(),
                    input,
                    output: result.output.clone(),
                    error: result.error.clone(),
                };
                steps.push(step);

                // Feed result back to LLM
                let tool_result = if let Some(ref output) = result.output {
                    serde_json::to_string(output).unwrap_or_default()
                } else if let Some(ref error) = result.error {
                    format!("Error: {error}")
                } else {
                    "No output".to_string()
                };

                messages.push(ChatMessage::tool_result(&call.id, tool_result));
            }
        }

        // Exceeded max rounds
        Ok(AgentResponse {
            answer: "I reached the maximum number of steps. Here's what I accomplished so far."
                .to_string(),
            steps,
        })
    }

    pub fn provider_name(&self) -> &str {
        self.llm.provider_name()
    }
}

fn build_system_prompt(context: Option<&DesktopContext>) -> String {
    let mut prompt = String::from(
        "You are Nerva, an AI desktop agent for Linux. \
         You help the user by calling tools to interact with their desktop environment.\n\n\
         Guidelines:\n\
         - Call tools when you need to perform actions or gather information.\n\
         - If no tool call is needed, respond directly with text.\n\
         - Be concise and helpful.\n\
         - When a tool call fails, explain the error and suggest alternatives.\n",
    );

    if let Some(ctx) = context {
        let context_text = ctx.to_prompt_text();
        if context_text != "No context available." {
            prompt.push_str("\nCurrent desktop context:\n");
            prompt.push_str(&context_text);
            prompt.push('\n');
        }
    }

    prompt
}
