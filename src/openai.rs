use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::llm::{LLMBackend, CommandOption};

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
    async fn translate_to_command(&self, query: &str) -> Result<Vec<ResponseType>> {
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
                        "content": "You are a helpful command-line assistant. Translate the user's query into appropriate shell commands. Provide 2-3 different command options with explanations. Format your response as a JSON array of objects, where each object has 'command' and 'explanation' fields. The command should be the exact shell command to run, and the explanation should briefly describe what the command does and why it might be preferred."
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

        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?;

        match serde_json::from_str::<Vec<CommandOption>>(content) {
            Ok(options) => {
                if options.is_empty() {
                    return Err(anyhow!("No valid command options generated"));
                }
                let responses: Vec<ResponseType> = options.into_iter()
                    .map(|opt| {
                        if opt.confidence >= 0.8 {
                            ResponseType::Command(opt)
                        } else if opt.confidence >= 0.5 {
                            ResponseType::ScriptRecommended(opt.command)
                        } else {
                            ResponseType::Uncertain(format!("Uncertain about command: {}", opt.command))
                        }
                    })
                    .collect();
                Ok(responses)
            }
            Err(_) => {
                Ok(vec![ResponseType::Uncertain(String::from("Unable to parse response as valid command options."))])
            }
        }
    }
}