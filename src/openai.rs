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
                        "content": format!("You are a helpful command-line assistant. Your task is to translate user queries into appropriate shell commands or recommend scripts for complex tasks. Details about user's environment: {}. RESPOND ONLY WITH A VALID JSON ARRAY OF COMMAND OPTIONS.\n\nIMPORTANT: Only suggest direct commands for operations that can be completed in a single shot. For any task requiring multiple steps, dependencies, or complex setup, recommend a script instead.\n\nEach command option must have these fields:\n- 'command': For single-shot tasks: the exact shell command. For complex tasks: a descriptive script outline\n- 'explanation': A brief description of what the command/script does and why it's recommended\n- 'confidence': A float between 0 and 1:\n  - >= 0.8 ONLY for simple, direct commands that can be executed in one shot\n  - 0.5-0.7 for tasks requiring scripts (multiple steps, dependencies, or complex setup)\n  - < 0.5 for uncertain suggestions\n\nExample response format:\n[{{\"command\": \"#!/bin/bash\necho 'Installing Docker...'\nbrew install docker\nbrew install docker-compose\", \"explanation\": \"Script recommended: Docker installation requires multiple steps and dependency management\", \"confidence\": 0.6}}]\n\nProvide 1-3 options. DO NOT include any text before or after the JSON array.", additional_context)
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

    async fn suggest_aliases(&self, command: &str, additional_context: &str) -> Result<Vec<CommandOption>> {
        let client = reqwest::Client::new();
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": format!(
                    "You are a command-line expert. Your task is to suggest useful aliases for shell commands. Consider the following context about the user's environment: {}.",
                    additional_context
                )
            }),
            serde_json::json!({
                "role": "user",
                "content": format!(
                    "For the command '{}', suggest up to 5 useful aliases that would make working with this command more efficient. Format your response as a valid JSON array where each item has a 'command' field with the alias definition (e.g., 'alias ll=\'ls -la\''), an 'explanation' field describing what it does, and a 'confidence' field between 0 and 1 (1.0 for common aliases, 0.8-0.9 for useful but less common ones). Ensure they follow shell syntax conventions.",
                    command
                )
            })
        ];

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.7
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("API request failed with status: {}", response.status()));
        }

        let response_data: Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?;

        let aliases: Vec<CommandOption> = serde_json::from_str(content)
            .map_err(|e| anyhow!("Failed to parse aliases: {}", e))?;

        Ok(aliases)
    }

    async fn explain_command(&self, command: &str, additional_context: &str) -> Result<ResponseType> {
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
                        "content": format!("You are a command-line expert. Explain in detail what this command does: '{}'. Consider the following context about the user's environment: {}. Format your explanation to cover: 1) Main purpose 2) How it works 3) Important flags/options 4) Potential risks or considerations", command, additional_context)
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

        let explanation = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();

        Ok(ResponseType::Command(CommandOption {
            command: command.to_string(),
            explanation,
            confidence: 1.0
        }))
    }
}