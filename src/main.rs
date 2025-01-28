use clap::Parser;
use anyhow::Result;
use std::process::Command;
mod config;
mod llm;
mod openai;
mod ollama;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// The query to translate into a shell command
    #[arg(required = true)]
    query: Vec<String>,

    /// Execute the command instead of just showing it
    #[arg(short, long)]
    execute: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let query = cli.query.join(" ");

    // Load configuration
    let config = config::Config::load()?;

    // Initialize the appropriate LLM backend
    let llm: Box<dyn llm::LLMBackend> = match config.backend {
        config::LlmBackend::OpenAI => {
            let openai_config = config.openai.ok_or_else(|| anyhow::anyhow!("OpenAI config missing"))?;
            Box::new(openai::OpenAIBackend::new(openai_config.api_key, Some(openai_config.model)))
        }
        config::LlmBackend::Ollama => {
            let ollama_config = config.ollama.ok_or_else(|| anyhow::anyhow!("Ollama config missing"))?;
            Box::new(ollama::OllamaBackend::new(ollama_config.endpoint, Some(ollama_config.model)))
        }
    };

    // Get command translation from LLM
    let command = llm.translate_to_command(&query).await?;

    if cli.execute {
        let args = shell_words::split(&command)?;
        let status = Command::new(&args[0])
            .args(&args[1..])
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    } else {
        println!("{}", command);
    }

    Ok(())
}