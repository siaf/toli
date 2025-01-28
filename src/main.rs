use clap::Parser;
use anyhow::Result;
use std::process::Command;
use std::io::{self, Write};
use crate::llm::ResponseType;
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
        match option {
            ResponseType::Command(cmd) => {
                println!("{}) {}", i + 1, cmd.command);
                println!("{}", cmd.explanation);
            },
            ResponseType::ScriptRecommended(cmd) => {
                println!("{}) {}", i + 1, cmd);
                println!("This command might need to be part of a script");
            },
            ResponseType::Uncertain(msg) => {
                println!("{}) Uncertain command", i + 1);
                println!("{}", msg);
            }
        }
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
            match &options[selection - 1] {
                ResponseType::Command(cmd) => &cmd.command,
                ResponseType::ScriptRecommended(cmd) => cmd,
                ResponseType::Uncertain(msg) => {
                    return Err(anyhow::anyhow!("Cannot execute uncertain command: {}", msg));
                }
            }
        } else {
            print!("\nExecute this command? [Y/n]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "n" {
                println!("\nSkipping command execution.");
                return Ok(());
            }
            match &options[0] {
                ResponseType::Command(cmd) => &cmd.command,
                ResponseType::ScriptRecommended(cmd) => cmd,
                ResponseType::Uncertain(msg) => {
                    return Err(anyhow::anyhow!("Cannot execute uncertain command: {}", msg));
                }
            }
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