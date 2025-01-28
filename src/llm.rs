use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Command(CommandOption),
    ScriptRecommended(String),
    Uncertain(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    pub command: String,
    pub explanation: String,
    pub confidence: f32,
}

#[async_trait]
pub trait LLMBackend {
    async fn translate_to_command(&self, query: &str) -> Result<Vec<ResponseType>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
}