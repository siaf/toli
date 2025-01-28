use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub enum LlmBackend {
    OpenAI,
    Ollama,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub backend: LlmBackend,
    pub openai: Option<OpenAIConfig>,
    pub ollama: Option<OllamaConfig>,
    pub additional_context: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_file_path();
        if !config_path.exists() {
            return Self::create_default_config(&config_path);
        }
        let config_str = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&config_str)?)
    }

    fn create_default_config(config_path: &PathBuf) -> Result<Self> {
        let default_config = Config {
            backend: LlmBackend::Ollama,
            openai: Some(OpenAIConfig {
                api_key: String::from("your-openai-api-key-here"),
                model: String::from("gpt-3.5-turbo"),
            }),
            ollama: Some(OllamaConfig {
                endpoint: String::from("http://localhost:11434"),
                model: String::from("llama3.2"),
            }),
            additional_context: String::from("running macos and generally zsh, is a developer, and uses brew"),
        };

        let config_str = toml::to_string_pretty(&default_config)?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(config_path, config_str)?;

        Ok(default_config)
    }
}

fn get_config_file_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("howto").join("config.toml")
}