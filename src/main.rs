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
#[command(author, version, about = "A CLI tool that translates natural language queries into shell commands")]
#[command(help_template = "{about-section}\n\nUsage: {usage}\n\n{options}\n\nExamples:\n  toli --how 'find all pdf files in current directory'\n  toli --do 'list all running docker containers'\n  toli 'show system memory usage'\n\nNote: By default, commands are displayed with explanations but not executed.")]
#[command(after_help = "Run 'howto --help' for more information about available options.")]
#[command(arg_required_else_help = true)]
struct Cli {
    /// The query to translate into a shell command
    #[arg(required = true, value_name = "QUERY")]
    query: Vec<String>,

    /// Show command options without executing
    #[arg(short = 's', long = "how", default_value_t = true,
          help = "Display the command explanation without executing it")]
    how: bool,

    /// Execute the selected command
    #[arg(short = 'd', long = "do", default_value_t = false,
          help = "Execute the command after displaying it")]
    do_execute: bool,

    /// Explain what a command does
    #[arg(short = 'e', long = "explain", default_value_t = false,
          help = "Explain what a given command does")]
    explain: bool,

    /// Suggest aliases for a command
    #[arg(short = 'a', long = "alias", default_value_t = false,
          help = "Suggest aliases for a given command")]
    alias: bool,
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
    let options = if cli.explain {
        // For explain flag, use the dedicated explain_command method
        vec![llm.explain_command(&query, &config.additional_context).await?]
    } else if cli.alias {
        // For alias flag, use the suggest_aliases method
        match llm.suggest_aliases(&query, &config.additional_context).await {
            Ok(aliases) => {
                println!("\nSuggested aliases for '{}':"  , query);
                for alias in aliases {
                    println!("\nAlias: {}", alias.command);
                    println!("Description: {}", alias.explanation);
                    println!("---");
                }
                return Ok(());
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get alias suggestions: {}", e));
            }
        }
    } else {
        llm.translate_to_command(&query, &config.additional_context).await?
    };

    // Display command options
    if cli.explain {
        // For explain flag, we only expect and show a single explanation
        if let Some(option) = options.first() {
            match option {
                ResponseType::Command(cmd) => {
                   
                    println!("{}", cmd.explanation);
                },
                ResponseType::ScriptRecommended(_) | ResponseType::Uncertain(_) => {
                    return Err(anyhow::anyhow!("Unable to explain the command. Please check if the command is valid."));
                }
            }
        } else {
            return Err(anyhow::anyhow!("No explanation available for the command."));
        }
    } else {
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
    }

    let selected_command = if cli.do_execute {
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