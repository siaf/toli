use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::llm::LLMBackend;

pub struct OllamaBackend {
    endpoint: String,
    model: String,
}

impl OllamaBackend {
    pub fn new(endpoint: String, model: Option<String>) -> Self {
        Self {
            endpoint,
            model: model.unwrap_or_else(|| String::from("llama2")),
        }
    }
}

#[async_trait]
impl LLMBackend for OllamaBackend {
    async fn translate_to_command(&self, query: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": format!(
                    "You are a helpful command-line assistant. Translate the following query into the most appropriate shell command: {}",
                    query
                ),
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("API request failed with status: {}", response.status()));
        }

        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        let response_data: Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        response_data["response"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow!("Invalid response format"))
    }
}