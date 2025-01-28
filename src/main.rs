use clap::Parser;
use anyhow::Result;
use std::process::Command;
use std::io::{self, Write};
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

    // Get command options from LLM
    let options = llm.translate_to_command(&query).await?;

    // Display command options
    for (i, option) in options.iter().enumerate() {
        println!("");
        println!("{}) {}", i + 1, option.command);
        println!("{}", option.explanation);
    }

    let selected_command = if cli.execute {
        if options.len() > 1 {
            // Prompt user to select a command
            print!("\nSelect a command to execute (1-{}) or 0 to skip: ", options.len());
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let selection: usize = input.trim().parse()?;
            if selection == 0 {
                println!("\nSkipping command execution.");
                return Ok(());
            }
            if selection < 1 || selection > options.len() {
                return Err(anyhow::anyhow!("Invalid selection"));
            }
            &options[selection - 1].command
        } else {
            print!("\nExecute this command? [Y/n]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "n" {
                println!("\nSkipping command execution.");
                return Ok(());
            }
            &options[0].command
        }
    } else {
        return Ok(());
    };

    // Execute the selected command
    let args = shell_words::split(selected_command)?;
    let status = Command::new(&args[0])
        .args(&args[1..])
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}