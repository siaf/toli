use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::llm::{LLMBackend, CommandOption, ResponseType};

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
    async fn translate_to_command(&self, query: &str, additional_context: &str) -> Result<Vec<ResponseType>> {
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
                        "content": format!("You are a helpful command-line assistant. Your task is to translate user queries into appropriate shell commands. Details about user's environment: {}. RESPOND ONLY WITH A VALID JSON ARRAY OF COMMAND OPTIONS. Each command option must have these fields:\n\n- 'command': The exact shell command to run\n- 'explanation': A brief description of what the command does and why it's recommended\n- 'confidence': A float between 0 and 1 indicating your confidence in the command (>= 0.8 for direct commands, >= 0.5 for script recommendations, < 0.5 for uncertain suggestions)\n\nExample response format:\n[{{\"command\": \"ls -la\", \"explanation\": \"List all files with detailed information\", \"confidence\": 0.9}}]\n\nProvide 2-3 command options. DO NOT include any text before or after the JSON array.", additional_context)
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