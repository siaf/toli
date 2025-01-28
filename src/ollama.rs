use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::io::Write;
use crate::llm::{LLMBackend, CommandOption};

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
                "RESPOND ONLY IN JSON THAT IS PASSED TO A PARSER. DONT ADD anthing before or after the json response. You are a helpful command-line assistant. Translate the following query into the most appropriate shell commands. Provide command options with explanations. Format your response as a JSON array of objects, where each object has 'command' and 'explanation' fields. The command should be the exact shell command to run, and the explanation should briefly describe what the command does and why it might be preferred. We want to parse your responce using a json parser so don't include anything that can't be parsed."
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