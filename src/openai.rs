use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::llm::LLMBackend;

pub struct OpenAIBackend {
    api_key: String,
    model: String,
}

impl OpenAIBackend {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| String::from("gpt-3.5-turbo")),
        }
    }
}

#[async_trait]
impl LLMBackend for OpenAIBackend {
    async fn translate_to_command(&self, query: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a helpful command-line assistant. Translate the user's natural language query into the most appropriate shell command. Respond with ONLY the command, no explanations."
                    },
                    {
                        "role": "user",
                        "content": query
                    }
                ]
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

        response_data["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow!("Invalid response format"))
    }
}