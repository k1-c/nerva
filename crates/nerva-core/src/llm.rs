use serde::{Deserialize, Serialize};

use crate::error::NervaError;
use crate::types::ToolMetadata;

// ─── Common types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into(), tool_calls: None }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into(), tool_calls: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into(), tool_calls: None }
    }
    pub fn tool(content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: content.into(), tool_calls: None }
    }
    pub fn tool_result(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: content.into(), tool_calls: Some(vec![ToolCall { id: id.into(), function: FunctionCall { name: String::new(), arguments: serde_json::Value::Null } }]) }
    }
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls.as_ref().is_some_and(|tc| !tc.is_empty() && tc.iter().any(|c| !c.function.name.is_empty()))
    }
}

// ─── Trait ───────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<&[ToolMetadata]>,
    ) -> Result<ChatMessage, NervaError>;

    async fn is_available(&self) -> bool;

    fn provider_name(&self) -> &str;
}

// ─── Claude (Anthropic) backend ─────────────────────────────────────

pub struct ClaudeBackend {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeBackend {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl LlmBackend for ClaudeBackend {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<&[ToolMetadata]>,
    ) -> Result<ChatMessage, NervaError> {
        // Separate system message from conversation
        let system_text: String = messages
            .iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| msg_to_claude(m))
            .collect();

        let claude_tools: Option<Vec<serde_json::Value>> = tools.map(|t| {
            t.iter()
                .map(|meta| {
                    serde_json::json!({
                        "name": meta.id,
                        "description": meta.description,
                        "input_schema": { "type": "object", "properties": {} },
                    })
                })
                .collect()
        });

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": api_messages,
        });
        if !system_text.is_empty() {
            body["system"] = serde_json::Value::String(system_text);
        }
        if let Some(tools) = claude_tools {
            body["tools"] = serde_json::Value::Array(tools);
        }

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| NervaError::ExecutionError(format!("Claude request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(NervaError::ExecutionError(format!(
                "Claude returned {status}: {body}"
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            NervaError::ExecutionError(format!("Failed to parse Claude response: {e}"))
        })?;

        parse_claude_response(&body)
    }

    async fn is_available(&self) -> bool {
        // Simple check: try to reach the API
        self.client
            .get("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .is_ok()
    }

    fn provider_name(&self) -> &str {
        "claude"
    }
}

fn msg_to_claude(msg: &ChatMessage) -> serde_json::Value {
    match msg.role.as_str() {
        "assistant" if msg.has_tool_calls() => {
            let mut content = Vec::new();
            if !msg.content.is_empty() {
                content.push(serde_json::json!({ "type": "text", "text": msg.content }));
            }
            if let Some(ref calls) = msg.tool_calls {
                for call in calls {
                    if !call.function.name.is_empty() {
                        content.push(serde_json::json!({
                            "type": "tool_use",
                            "id": call.id,
                            "name": call.function.name,
                            "input": call.function.arguments,
                        }));
                    }
                }
            }
            serde_json::json!({ "role": "assistant", "content": content })
        }
        "tool" => {
            // Extract tool_use_id from the tool_calls field (we stash it there)
            let tool_use_id = msg
                .tool_calls
                .as_ref()
                .and_then(|tc| tc.first())
                .map(|c| c.id.as_str())
                .unwrap_or("unknown");
            serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": tool_use_id,
                    "content": msg.content,
                }]
            })
        }
        _ => serde_json::json!({
            "role": msg.role,
            "content": msg.content,
        }),
    }
}

fn parse_claude_response(body: &serde_json::Value) -> Result<ChatMessage, NervaError> {
    let content_blocks = body["content"]
        .as_array()
        .ok_or_else(|| NervaError::ExecutionError("No content in Claude response".into()))?;

    let mut text_parts = Vec::new();
    let mut tool_calls = Vec::new();

    for block in content_blocks {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(text) = block["text"].as_str() {
                    text_parts.push(text.to_string());
                }
            }
            Some("tool_use") => {
                let id = block["id"].as_str().unwrap_or("").to_string();
                let name = block["name"].as_str().unwrap_or("").to_string();
                let input = block["input"].clone();
                tool_calls.push(ToolCall {
                    id,
                    function: FunctionCall {
                        name,
                        arguments: input,
                    },
                });
            }
            _ => {}
        }
    }

    Ok(ChatMessage {
        role: "assistant".into(),
        content: text_parts.join("\n"),
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
    })
}

// ─── OpenAI backend ─────────────────────────────────────────────────

pub struct OpenAiBackend {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiBackend {
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".into()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl LlmBackend for OpenAiBackend {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<&[ToolMetadata]>,
    ) -> Result<ChatMessage, NervaError> {
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| msg_to_openai(m))
            .collect();

        let openai_tools: Option<Vec<serde_json::Value>> = tools.map(|t| {
            t.iter()
                .map(|meta| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": meta.id,
                            "description": meta.description,
                            "parameters": { "type": "object", "properties": {} },
                        }
                    })
                })
                .collect()
        });

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": api_messages,
        });
        if let Some(tools) = openai_tools {
            body["tools"] = serde_json::Value::Array(tools);
        }

        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| NervaError::ExecutionError(format!("OpenAI request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(NervaError::ExecutionError(format!(
                "OpenAI returned {status}: {body}"
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            NervaError::ExecutionError(format!("Failed to parse OpenAI response: {e}"))
        })?;

        parse_openai_response(&body)
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/models", self.base_url);
        self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

fn msg_to_openai(msg: &ChatMessage) -> serde_json::Value {
    match msg.role.as_str() {
        "assistant" if msg.has_tool_calls() => {
            let tool_calls: Vec<serde_json::Value> = msg
                .tool_calls
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .filter(|c| !c.function.name.is_empty())
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "type": "function",
                        "function": {
                            "name": c.function.name,
                            "arguments": serde_json::to_string(&c.function.arguments).unwrap_or_default(),
                        }
                    })
                })
                .collect();
            let mut val = serde_json::json!({ "role": "assistant" });
            if !msg.content.is_empty() {
                val["content"] = serde_json::Value::String(msg.content.clone());
            }
            val["tool_calls"] = serde_json::Value::Array(tool_calls);
            val
        }
        "tool" => {
            let tool_call_id = msg
                .tool_calls
                .as_ref()
                .and_then(|tc| tc.first())
                .map(|c| c.id.as_str())
                .unwrap_or("unknown");
            serde_json::json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": msg.content,
            })
        }
        _ => serde_json::json!({
            "role": msg.role,
            "content": msg.content,
        }),
    }
}

fn parse_openai_response(body: &serde_json::Value) -> Result<ChatMessage, NervaError> {
    let choice = body["choices"]
        .as_array()
        .and_then(|c| c.first())
        .ok_or_else(|| NervaError::ExecutionError("No choices in OpenAI response".into()))?;

    let message = &choice["message"];
    let content = message["content"].as_str().unwrap_or("").to_string();

    let tool_calls = message["tool_calls"]
        .as_array()
        .map(|calls| {
            calls
                .iter()
                .map(|c| {
                    let id = c["id"].as_str().unwrap_or("").to_string();
                    let name = c["function"]["name"].as_str().unwrap_or("").to_string();
                    let args_str = c["function"]["arguments"].as_str().unwrap_or("{}");
                    let arguments = serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
                    ToolCall {
                        id,
                        function: FunctionCall { name, arguments },
                    }
                })
                .collect::<Vec<_>>()
        })
        .filter(|tc| !tc.is_empty());

    Ok(ChatMessage {
        role: "assistant".into(),
        content,
        tool_calls,
    })
}

// ─── Ollama backend ─────────────────────────────────────────────────

pub struct OllamaBackend {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaBackend {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl LlmBackend for OllamaBackend {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<&[ToolMetadata]>,
    ) -> Result<ChatMessage, NervaError> {
        let ollama_tools: Option<Vec<serde_json::Value>> = tools.map(|t| {
            t.iter()
                .map(|meta| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": meta.id,
                            "description": meta.description,
                            "parameters": { "type": "object", "properties": {} },
                        }
                    })
                })
                .collect()
        });

        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| msg_to_openai(m)) // Ollama uses OpenAI-compatible format
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": api_messages,
            "stream": false,
        });
        if let Some(tools) = ollama_tools {
            body["tools"] = serde_json::Value::Array(tools);
        }

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NervaError::ExecutionError(format!("Ollama request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(NervaError::ExecutionError(format!(
                "Ollama returned {status}: {body}"
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            NervaError::ExecutionError(format!("Failed to parse Ollama response: {e}"))
        })?;

        // Ollama wraps in { message: { ... } }
        let message = &body["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();

        let tool_calls = message["tool_calls"]
            .as_array()
            .map(|calls| {
                calls
                    .iter()
                    .map(|c| {
                        let name = c["function"]["name"].as_str().unwrap_or("").to_string();
                        let arguments = c["function"]["arguments"].clone();
                        ToolCall {
                            id: String::new(),
                            function: FunctionCall { name, arguments },
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|tc| !tc.is_empty());

        Ok(ChatMessage {
            role: "assistant".into(),
            content,
            tool_calls,
        })
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}
