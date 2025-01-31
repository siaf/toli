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
    async fn translate_to_command(&self, query: &str, additional_context: &str) -> Result<Vec<ResponseType>> {
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
                "You are a command-line assistant. Your task is to translate user queries into appropriate shell commands or recommend to use scripts for complex tasks. Details about user's environment: {}. RESPOND ONLY WITH A VALID JSON ARRAY OF OPTIONS.\n\nIMPORTANT: Only suggest commands for operations that can be completed in a single shot, piping is okay. For any task requiring multiple steps, dependencies, or complex setup, recommend to use a script instead.\n\nEach command option must have these fields:\n- 'command': For single-shot tasks: the exact shell command. For complex tasks: a suggested script name\n- 'explanation': A brief description of what the command does and why it's recommended, for scripts an high level description of what it should do\n- 'confidence': A float between 0 and 1:\n  - >= 0.8 ONLY for simple, direct commands that can be executed in one shot\n  - 0.5-0.7 for tasks requiring scripts (multiple steps, dependencies, or complex setup)\n  - < 0.5 for uncertain suggestions\n\nExample response format:\n[{{\"command\": \"#!/bin/bash\necho 'Installing Docker...'\nbrew install docker\nbrew install docker-compose\", \"explanation\": \"Script recommended: Docker installation requires multiple steps and dependency management\", \"confidence\": 0.6}}]\n\nProvide up to 5 options. DO NOT include any text before or after the JSON array. Here's the query: {}",
                additional_context,
                query
            );

            if !failed_responses.is_empty() {
                prompt.push_str("\n\nPrevious attempts failed to generate valid JSON. Here are the failed responses:\n");
                for (i, response) in failed_responses.iter().enumerate() {
                    prompt.push_str(&format!("\nAttempt {}: {}\n", i + 1, response));
                }
                prompt.push_str("\nPlease ensure your response is a valid JSON array.");
            }

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
        }

        Ok(vec![ResponseType::Uncertain(String::from(
            "Failed to generate valid command options after multiple attempts."
        ))])
    }

    async fn suggest_aliases(&self, command: &str, additional_context: &str) -> Result<Vec<CommandOption>> {
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
                "You are a command-line expert. \
                Only For the command '{}', suggest (up to 3) useful aliases that would make working with this command more efficient.\n\n\
                Don't suggest alias for other commands. Only the one provided earlier. \
                Consider the following context about the user's environment, but only when applicable: {}. \
                IMPORTANT: Your response must be a valid JSON array containing objects with exactly these fields:\n\
                - 'command' (string with alias definition)\n\
                - 'explanation' (string describing what it does)\n\
                - 'confidence' (number between 0 and 1)\n\n\
                Example response: [{{\
                \"command\": \"alias ll='ls -la'\", \
                \"explanation\": \"Lists all files in long format\", \
                \"confidence\": 1.0\
                }}]. \
                Ensure proper JSON escaping and shell syntax conventions. \
                RESPOND ONLY IN JSON. DO NOT INCLUDE ANYTHING ELSE BESIDE JSON.",
                command,
                additional_context
            );

            if !failed_responses.is_empty() {
                prompt.push_str("\n\nPrevious attempts failed to generate valid JSON. Here are the failed responses:\n");
                for (i, response) in failed_responses.iter().enumerate() {
                    prompt.push_str(&format!("\nAttempt {}: {}\n", i + 1, response));
                }
                prompt.push_str("\nPlease ensure your response is a valid JSON array.");
            }

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

            // Clean up the response string to handle escaping issues
            let cleaned_response = response_str
                .trim()
                .replace("\\'", "'")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
                .replace("\n", "")
                .replace("\r", "");

            //println!("Attempt {}: {}", attempts + 1, cleaned_response);

            // Try parsing with serde_json first
            let parse_result = serde_json::from_str::<Vec<CommandOption>>(&cleaned_response);
            
            // If that fails, try to extract JSON array from the response
            let parse_result = if parse_result.is_err() {
                if let Some(start) = cleaned_response.find('[') {
                    if let Some(end) = cleaned_response.rfind(']') {
                        let json_str = &cleaned_response[start..=end];
                        serde_json::from_str::<Vec<CommandOption>>(json_str)
                    } else {
                        parse_result
                    }
                } else {
                    parse_result
                }
            } else {
                parse_result
            };

            match parse_result {
                Ok(options) => {
                    if options.is_empty() {
                        failed_responses.push(cleaned_response.to_string());
                    } else {
                        return Ok(options);
                    }
                }
                Err(e) => {
                    //println!("Failed to parse JSON: {}", e);
                    failed_responses.push(cleaned_response.to_string());
                }
            }

            attempts += 1;
        }

        Ok(vec![])
    }

    async fn explain_command(&self, command: &str, additional_context: &str) -> Result<ResponseType> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "You are a command-line expert. Explain briefly what this command does: '{}'. Consider the following context about the user's environment: {}. \
             Keep it brief to a paragraph.",
            command, additional_context
        );

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

        let explanation = response_data["response"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?;

        Ok(ResponseType::Command(CommandOption {
            command: command.to_string(),
            explanation: explanation.to_string(),
            confidence: 1.0
        }))
    }
}