use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::io::Write;
use crate::llm::{LLMBackend, CommandOption, ResponseType};

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
    async fn translate_to_command(&self, query: &str) -> Result<Vec<ResponseType>> {
        let mut attempts = 0;
        let max_attempts = 5;
        let mut failed_responses = Vec::new();
        let feedback = [".   ", "..  ", "... ", "...."];

        while attempts < max_attempts {
            if attempts > 0 {
                print!("\rThinking{}", feedback[attempts % feedback.len()]);
                std::io::stdout().flush().ok();
            }

            let client = reqwest::Client::new();
            let mut prompt = format!(
                "You are a helpful command-line assistant. Your task is to translate user queries into appropriate shell commands. Details about users environment: running macos and generally zsh, is a developer, and uses brew. RESPOND ONLY WITH A VALID JSON ARRAY OF COMMAND OPTIONS. Each command option must have these fields:\n\n- 'command': The exact shell command to run\n- 'explanation': A brief description of what the command does and why it's recommended\n- 'confidence': A float between 0 and 1 indicating your confidence in the command (>= 0.8 for direct commands, >= 0.5 for script recommendations, < 0.5 for uncertain suggestions)\n\nExample response format:\n[{{\"command\": \"ls -la\", \"explanation\": \"List all files with detailed information\", \"confidence\": 0.9}}]\n\nProvide up to 5 command options. DO NOT include any text before or after the JSON array.\n\nHere's the query: {}",
                query
            );

            if !failed_responses.is_empty() {
                prompt.push_str("\n\nPrevious attempts failed to generate valid JSON. Here are the failed responses:\n");
                for (i, response) in failed_responses.iter().enumerate() {
                    prompt.push_str(&format!("\nAttempt {}: {}\n", i + 1, response));
                }
                prompt.push_str("\nPlease ensure your response is a valid JSON array.");
            }

            prompt.push_str(&format!("\n\nHere's the query: {}", query));

            let response = client
                .post(format!("{}/api/generate", self.endpoint))
                .json(&serde_json::json!({
                    "model": self.model,
                    "prompt": prompt,
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

            let response_str = response_data["response"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            let parsed_result = serde_json::from_str::<Vec<CommandOption>>(response_str);
            match parsed_result {
                Ok(options) => {
                    if options.is_empty() {
                        failed_responses.push(response_str.to_string());
                    } else {
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
                        return Ok(responses);
                    }
                }
                Err(_) => {
                    failed_responses.push(response_str.to_string());
                }
            }

            attempts += 1;
            print!("\r{}{}", "Thinking", feedback[attempts % feedback.len()]);
            std::io::stdout().flush().ok();
        }

        // If all attempts failed, fall back to using the last response as a direct command
        Ok(vec![ResponseType::Uncertain(String::from("Unable to generate a valid command after multiple attempts."))])
    }
}