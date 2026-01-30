<p align="center">
  <img src="assets/Cherry2K.sh-logo.png" alt="Cherry2K.sh Logo" width="400">
</p>

<h1 align="center">Cherry2K.sh</h1>

<p align="center">
  <strong>AI assistant for your zsh terminal</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#providers">Providers</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#zsh-integration">Zsh Integration</a>
</p>

---

Cherry2K.sh is a terminal-based AI assistant built in Rust with a provider-agnostic architecture. Switch seamlessly between OpenAI, Anthropic, and Ollama without changing your workflow.

## Features

- **Multiple AI Providers** - OpenAI, Anthropic, and Ollama support out of the box
- **Streaming Responses** - Real-time output as the AI generates responses
- **Conversation History** - SQLite-backed persistence across sessions
- **Zsh Integration** - Native widgets and completions for seamless terminal use
- **Offline Mode** - Use Ollama for fully local, private AI assistance
- **Fast & Lightweight** - Built in Rust for minimal resource usage

## Installation

### Homebrew (Recommended)

```bash
brew tap dunnock/tap
brew install cherry2k
```

### From Source

```bash
# Clone the repository
git clone https://github.com/dunnock/cherry2k.git
cd cherry2k

# Build and install
cargo build --release
cp target/release/cherry2k ~/.local/bin/
```

### Requirements

- macOS or Linux
- Rust 1.75+ (for building from source)
- SQLite 3 (installed via Homebrew on macOS)
- One of: OpenAI API key, Anthropic API key, or Ollama running locally

## Quick Start

### 1. Set up your API key

```bash
# For OpenAI
export OPENAI_API_KEY=sk-...

# For Anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# For Ollama (no key needed, just start the server)
ollama serve
```

### 2. Send a message

```bash
# One-shot query
cherry2k chat "Explain Rust ownership in simple terms"

# Interactive REPL
cherry2k repl
```

### 3. Enable zsh integration (optional)

```bash
# Add to your .zshrc
source $(brew --prefix)/share/cherry2k/cherry2k.plugin.zsh

# Or if installed from source
source /path/to/cherry2k/zsh/cherry2k.plugin.zsh
```

## Providers

### OpenAI

```bash
export OPENAI_API_KEY=sk-...
cherry2k chat -p openai "Hello!"
```

Supports: GPT-4, GPT-4 Turbo, GPT-3.5 Turbo

### Anthropic

```bash
export ANTHROPIC_API_KEY=sk-ant-...
cherry2k chat -p anthropic "Hello!"
```

Supports: Claude 3 Opus, Claude 3 Sonnet, Claude 3 Haiku

### Ollama (Local)

```bash
# Start Ollama server
ollama serve

# Pull a model
ollama pull llama2

# Use with Cherry2K.sh
cherry2k chat -p ollama "Hello!"
```

Supports: Any model available in Ollama (Llama 2, Mistral, CodeLlama, etc.)

## Configuration

### Config File

Create `~/.config/cherry2k/config.toml`:

```toml
# Default provider
default_provider = "openai"

[providers.openai]
model = "gpt-4-turbo"
max_tokens = 2000
temperature = 0.7

[providers.anthropic]
model = "claude-3-sonnet-20240229"
max_tokens = 2000
temperature = 0.7

[providers.ollama]
model = "llama2"
host = "http://localhost:11434"

[storage]
path = "~/.local/share/cherry2k/conversations.db"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OLLAMA_HOST` | Ollama server URL | `http://localhost:11434` |
| `CHERRY2K_CONFIG_PATH` | Config file path | `~/.config/cherry2k/config.toml` |
| `CHERRY2K_LOG_LEVEL` | Log verbosity | `info` |

## Usage

### Commands

```bash
# Send a one-shot message
cherry2k chat "Your message here"

# Start interactive REPL
cherry2k repl

# Specify provider
cherry2k chat -p anthropic "Hello!"

# JSON output (for scripting)
cherry2k chat --format json "Hello!"

# View conversation history
cherry2k history

# Show last 5 conversations
cherry2k history -l 5

# Manage configuration
cherry2k config show
cherry2k config set default_provider anthropic
```

### Interactive REPL

```
$ cherry2k repl

Cherry2K.sh v0.1.0 - Type /help for commands, /quit to exit

> Hello!
Hello! How can I help you today?

> /provider anthropic
Switched to anthropic

> /history
Showing conversation history...

> /quit
Goodbye!
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/quit` | Exit the REPL |
| `/clear` | Clear conversation context |
| `/provider <name>` | Switch AI provider |
| `/model <name>` | Switch model |
| `/history` | Show conversation history |
| `/export` | Export current conversation |

## Zsh Integration

### Widgets

Cherry2K.sh provides ZLE widgets for quick AI access:

| Keybinding | Widget | Description |
|------------|--------|-------------|
| `Ctrl+G` | `cherry2k-assist-widget` | Send current line to AI |
| `Ctrl+X Ctrl+A` | `cherry2k-explain-widget` | Explain the current command |

### Custom Keybindings

```zsh
# Add to your .zshrc
bindkey '^[a' cherry2k-assist-widget  # Alt+A
```

### Tab Completion

Full tab completion for commands and options:

```bash
cherry2k <TAB>
# chat    config    history    repl

cherry2k chat -p <TAB>
# anthropic    ollama    openai
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with coverage
cargo llvm-cov --fail-under-lines 80

# Check linting
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Project Structure

```
cherry2k/
├── crates/
│   ├── core/       # Provider abstraction, conversation logic
│   ├── storage/    # SQLite persistence
│   └── cli/        # Terminal interface
├── zsh/            # Zsh integration
│   ├── cherry2k.plugin.zsh
│   ├── widgets/
│   └── completions/
└── tests/          # Integration tests
```

## Contributing

Contributions are welcome! Please read the [contributing guidelines](CONTRIBUTING.md) first.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Inspired by the need for a fast, flexible terminal AI assistant