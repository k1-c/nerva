use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::NervaError;

/// VLM (Vision Language Model) client for interpreting images.
///
/// Currently supports Ollama's API with vision-capable models
/// (e.g., llava, llava-llama3, moondream).
pub struct VlmClient {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

impl VlmClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a client with default Ollama settings.
    pub fn default_ollama(model: impl Into<String>) -> Self {
        Self::new("http://localhost:11434", model)
    }

    /// Describe an image using the VLM.
    pub async fn describe_image(
        &self,
        image_path: &Path,
        prompt: &str,
    ) -> Result<String, NervaError> {
        let image_data = tokio::fs::read(image_path)
            .await
            .map_err(|e| NervaError::OsError(format!("Failed to read image: {e}")))?;

        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &image_data,
        );

        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            images: vec![encoded],
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| NervaError::OsError(format!("Ollama request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(NervaError::OsError(format!(
                "Ollama returned {status}: {body}"
            )));
        }

        let body: OllamaGenerateResponse = resp
            .json()
            .await
            .map_err(|e| NervaError::OsError(format!("Failed to parse Ollama response: {e}")))?;

        Ok(body.response)
    }

    /// Check if the Ollama server is reachable and the model is available.
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        let Ok(resp) = self.client.get(&url).send().await else {
            return false;
        };

        if !resp.status().is_success() {
            return false;
        }

        // Check if our model is in the list
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }
        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let Ok(tags) = resp.json::<TagsResponse>().await else {
            return false;
        };

        tags.models.iter().any(|m| m.name.starts_with(&self.model))
    }
}
