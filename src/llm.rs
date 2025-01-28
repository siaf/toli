use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait LLMBackend {
    async fn translate_to_command(&self, query: &str) -> Result<String>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
}