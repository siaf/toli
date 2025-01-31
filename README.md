# HowTo CLI Tool

# TOLI (Terminal Intelligence & Learning Operator)

A command-line interface tool that translates natural language queries into shell commands using AI language models.

## Features

- Natural language to shell command translation
- Support for multiple LLM backends (OpenAI and Ollama)
- Configurable settings via TOML configuration file
- Option to execute translated commands directly

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/toli.git
cd toli
```

2. Build the project:
```bash
cargo build --release
```

3. (Optional) Add to your PATH for global access

4. Set up convenient aliases (recommended):

Add these lines to your shell configuration file (e.g., ~/.bashrc, ~/.zshrc):
```bash
alias howto='toli --how'
alias do='toli --do'
```

## Configuration

The tool uses a `config.toml` file for configuration. On first run, a default configuration file will be created with the following structure:

```toml
# Select which LLM backend to use
# Available options: "OpenAI" or "Ollama"
backend = "Ollama"

# OpenAI configuration
[openai]
api_key = "your-openai-api-key-here"
model = "gpt-3.5-turbo"

# Ollama configuration
[ollama]
endpoint = "http://localhost:11434"
model = "llama2"
```

### Configuration Options

- `backend`: Choose between "OpenAI" or "Ollama" as your LLM provider
- `openai.api_key`: Your OpenAI API key (required for OpenAI backend)
- `openai.model`: OpenAI model to use (e.g., "gpt-3.5-turbo", "gpt-4")
- `ollama.endpoint`: URL of your Ollama instance
- `ollama.model`: Ollama model to use

## Usage

```bash
# Get a command without executing it
toli --how "show all running docker containers"

# Show command with explanation
toli --how -s "show all running docker containers"

# Execute the command after confirmation
toli --do "show all running docker containers"

# Using aliases (if configured)
howto "show all running docker containers"  # Same as toli --how
do "show all running docker containers"      # Same as toli --do
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.