# Codebase Structure

**Analysis Date:** 2026-01-29

## Directory Layout

```
cherry2k/
├── crates/                     # Rust workspace crates
│   ├── core/                   # Domain logic & provider abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Public API exports
│   │       ├── provider/        # AI provider implementations
│   │       │   ├── mod.rs
│   │       │   ├── trait.rs     # AiProvider trait definition
│   │       │   ├── openai.rs
│   │       │   ├── anthropic.rs
│   │       │   └── ollama.rs
│   │       ├── conversation/    # Conversation & message models
│   │       │   ├── mod.rs
│   │       │   ├── message.rs
│   │       │   └── context.rs
│   │       ├── config/          # Configuration loading & schema
│   │       │   ├── mod.rs
│   │       │   └── provider_config.rs
│   │       └── error.rs         # Error types (thiserror)
│   ├── storage/                # SQLite persistence layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Public repository API
│   │       ├── schema.rs       # Database schema definitions
│   │       ├── migrations/     # SQL migrations directory
│   │       │   ├── mod.rs
│   │       │   ├── 0001_initial.sql
│   │       │   └── 0002_conversations.sql
│   │       └── repository.rs   # Data access layer (CRUD)
│   └── cli/                    # Terminal interface
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # Entry point & command dispatcher
│           ├── commands/       # Command handlers
│           │   ├── mod.rs
│           │   ├── chat.rs
│           │   ├── config.rs
│           │   ├── history.rs
│           │   └── repl.rs
│           ├── repl/           # Interactive REPL mode
│           │   ├── mod.rs
│           │   └── readline.rs
│           └── output/         # Terminal formatting
│               ├── mod.rs
│               └── formatter.rs
├── zsh/                        # Zsh shell integration
│   ├── cherry2k.plugin.zsh     # Main plugin file (shell functions)
│   ├── widgets/                # ZLE widget functions
│   │   ├── cherry2k-assist-widget.zsh
│   │   └── cherry2k-explain-widget.zsh
│   └── completions/            # Tab completion functions
│       └── _cherry2k
├── .claude/                    # Project standards & documentation
│   └── standards/              # Quality gates & conventions
│       ├── constitution.md
│       ├── rust.md
│       ├── testing.md
│       ├── security.md
│       ├── git-cicd.md
│       └── documentation.md
├── .planning/                  # GSD planning artifacts
│   └── codebase/              # Codebase analysis documents
├── docs/                       # User-facing documentation
│   ├── explanation/
│   ├── how-to/
│   ├── tutorials/
│   └── reference/
├── assets/                     # Project assets (logo, images)
│   └── Cherry2K.sh-logo.png
├── Cargo.toml                  # Workspace root configuration
├── CLAUDE.md                   # Project instructions & architecture overview
└── README.md                   # User documentation
```

## Directory Purposes

**crates/:**
- Purpose: Rust workspace containing all compiled crates
- Contains: Three independent library/binary crates sharing dependencies
- Key files: Workspace-level `Cargo.toml` defines shared dependencies and lints

**crates/core/:**
- Purpose: Domain logic and AI provider abstraction
- Contains: Provider trait implementations, conversation models, configuration parsing, error types
- Key files: `crates/core/src/lib.rs` (exports public API), `crates/core/src/provider/trait.rs` (core abstraction)

**crates/core/src/provider/:**
- Purpose: AI provider implementations
- Contains: Separate files for OpenAI, Anthropic, Ollama with HTTP request/response handling
- Key files: `mod.rs` (re-exports), `trait.rs` (AiProvider trait), `openai.rs`, `anthropic.rs`, `ollama.rs`

**crates/core/src/conversation/:**
- Purpose: Conversation and message domain models
- Contains: Message structs, conversation context management, history building
- Key files: `mod.rs` (public exports), `message.rs` (message types), `context.rs` (conversation state)

**crates/core/src/config/:**
- Purpose: Configuration loading and validation
- Contains: Config file parsing (TOML), environment variable reading, provider-specific settings
- Key files: `mod.rs` (config loader), `provider_config.rs` (per-provider settings)

**crates/storage/:**
- Purpose: SQLite persistence layer
- Contains: Database schema, migrations, repository pattern data access
- Key files: `crates/storage/src/lib.rs` (exports), `schema.rs` (table definitions), `repository.rs` (CRUD)

**crates/storage/src/migrations/:**
- Purpose: Versioned database schema changes
- Contains: Numbered SQL migration files applied in order
- Key files: `0001_initial.sql` (schema creation), `0002_conversations.sql` (schema updates)

**crates/cli/:**
- Purpose: Binary entry point and terminal interface
- Contains: Command handlers, REPL implementation, output formatting
- Key files: `src/main.rs` (entry point), command handlers in `commands/` directory

**crates/cli/src/commands/:**
- Purpose: Individual command implementations
- Contains: `chat.rs` (one-shot queries), `repl.rs` (interactive mode), `config.rs` (config management), `history.rs` (conversation history)
- Key files: Each command has handler function matching clap command definition

**crates/cli/src/repl/:**
- Purpose: Interactive REPL mode
- Contains: Readline integration, prompt rendering, command parsing, session state
- Key files: `mod.rs` (REPL loop), `readline.rs` (line editing)

**crates/cli/src/output/:**
- Purpose: Terminal output formatting
- Contains: Response formatting (plain text, JSON), streaming output, terminal colors
- Key files: `formatter.rs` (format selection and rendering)

**zsh/:**
- Purpose: Shell integration without external dependencies
- Contains: Pure zsh functions and ZLE widgets
- Key files: `cherry2k.plugin.zsh` (main plugin), widget files in `widgets/`

**zsh/cherry2k.plugin.zsh:**
- Purpose: Main plugin file sourced by user's .zshrc
- Contains: Function definitions, widget registration, completion sourcing
- Key files: Single file containing all shell functions and widget setup

**zsh/widgets/:**
- Purpose: ZLE widget implementations
- Contains: Keybinding handlers for AI assist and explain functions
- Key files: `cherry2k-assist-widget.zsh`, `cherry2k-explain-widget.zsh`

**zsh/completions/:**
- Purpose: Tab completion for cherry2k command
- Contains: zsh completion function for command, subcommands, options
- Key files: `_cherry2k` (main completion function)

**.claude/standards/:**
- Purpose: Project quality standards and conventions
- Contains: Quality gates, coding standards, security requirements, testing patterns
- Key files: `constitution.md` (global), `rust.md` (Rust-specific), `testing.md` (test patterns)

**.planning/codebase/:**
- Purpose: GSD codebase analysis documents
- Contains: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, CONCERNS.md, STACK.md, INTEGRATIONS.md
- Key files: Analysis documents consumed by `/gsd:plan-phase` and `/gsd:execute-phase`

**docs/:**
- Purpose: User-facing documentation
- Contains: How-to guides, tutorials, reference docs, explanations
- Key files: Organized by doc type (tutorial, how-to, explanation, reference)

## Key File Locations

**Entry Points:**
- `crates/cli/src/main.rs`: Binary entry point; sets up logging, parses CLI args, dispatches to command handlers
- `crates/core/src/lib.rs`: Library entry point; exports AiProvider trait, ProviderConfig, error types
- `zsh/cherry2k.plugin.zsh`: Shell integration entry point; sourced by user's .zshrc

**Configuration:**
- `Cargo.toml`: Workspace root; defines shared dependencies, workspace members, lints
- `crates/core/Cargo.toml`: Core crate with provider implementations and domain logic
- `crates/storage/Cargo.toml`: Storage crate with rusqlite dependency
- `crates/cli/Cargo.toml`: CLI crate with clap and terminal formatting dependencies
- `CLAUDE.md`: Project instructions and architecture overview

**Core Logic:**
- `crates/core/src/provider/trait.rs`: AiProvider trait definition (core abstraction)
- `crates/core/src/provider/openai.rs`: OpenAI provider implementation
- `crates/core/src/provider/anthropic.rs`: Anthropic provider implementation
- `crates/core/src/provider/ollama.rs`: Ollama (local) provider implementation
- `crates/core/src/config/mod.rs`: Configuration loading and environment setup

**Testing:**
- Tests colocated in `#[cfg(test)]` modules within implementation files
- No separate `tests/` directory; unit tests live in same file as implementation
- Fixtures defined in submodules within test modules (e.g., `fixtures::mock_config()`)

**Database:**
- `crates/storage/src/schema.rs`: SQLite table and schema definitions
- `crates/storage/src/migrations/0001_initial.sql`: Initial schema creation
- `crates/storage/src/repository.rs`: Data access layer with CRUD methods

**Shell Integration:**
- `zsh/cherry2k.plugin.zsh`: Main plugin; defines functions and widgets
- `zsh/widgets/cherry2k-assist-widget.zsh`: Ctrl+G widget for AI assist
- `zsh/widgets/cherry2k-explain-widget.zsh`: Command explanation widget
- `zsh/completions/_cherry2k`: Tab completion function

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (e.g., `openai.rs`, `provider_config.rs`)
- Zsh functions: `kebab-case.zsh` (e.g., `cherry2k-assist-widget.zsh`)
- Config files: `snake_case.toml` or `snake_case` (e.g., `cherry2k_config.toml`)
- Migrations: `NNNN_description.sql` (e.g., `0001_initial.sql`)

**Directories:**
- Rust modules: `snake_case/` (e.g., `src/provider/`, `src/conversation/`)
- Zsh subdirectories: `kebab-case/` (e.g., `widgets/`, `completions/`)

**Rust Identifiers:**
- Functions: `snake_case` (e.g., `validate_config()`, `complete_request()`)
- Types/Traits: `PascalCase` (e.g., `AiProvider`, `CompletionRequest`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`, `DEFAULT_TIMEOUT`)
- Private fields: `_prefix` or no prefix with doc comments (e.g., `_client`, `api_key`)

**Zsh Identifiers:**
- Functions: `cherry2k-<action>` (e.g., `cherry2k-complete`, `cherry2k-assist-widget`)
- Variables: `CHERRY2K_<NAME>` (e.g., `CHERRY2K_CONFIG_PATH`)

## Where to Add New Code

**New Provider Implementation:**
- Primary code: Create `crates/core/src/provider/new_provider.rs` implementing AiProvider trait
- Export: Add module to `crates/core/src/provider/mod.rs`
- Tests: Add `#[cfg(test)] mod tests { ... }` in same file with fixtures
- Integration: Update provider factory logic in command handlers

**New CLI Command:**
- Primary code: Create `crates/cli/src/commands/new_command.rs` with handler function
- Registration: Add clap command definition to `crates/cli/src/main.rs`
- Tests: Add integration tests in `#[cfg(test)]` module within command file
- Output: Use formatter from `crates/cli/src/output/` for terminal display
- Shell completion: Update `zsh/completions/_cherry2k` with new command options

**New Conversation Feature (e.g., custom context):**
- Primary code: Extend models in `crates/core/src/conversation/message.rs`
- Persistence: Add database columns in new migration `crates/storage/src/migrations/NNNN_*.sql`
- Repository: Add CRUD methods to `crates/storage/src/repository.rs`
- CLI usage: Update command handlers to accept and pass new context

**New Configuration Option:**
- Primary code: Add field to config struct in `crates/core/src/config/provider_config.rs`
- Loading: Update TOML parsing and env var override logic in `crates/core/src/config/mod.rs`
- Validation: Add checks in `validate_config()` for provider implementations
- Documentation: Update default config template in README.md

**New Zsh Widget:**
- Primary code: Create `zsh/widgets/cherry2k-<action>-widget.zsh` with widget function
- Registration: Source in `zsh/cherry2k.plugin.zsh` and bind with `bindkey`
- Completion: Update `zsh/completions/_cherry2k` if widget has options

**Shared Utilities:**
- Location: `crates/core/src/` (new module) or existing module if thematically related
- Export: Add to `crates/core/src/lib.rs` if used externally
- Tests: Colocated in `#[cfg(test)]` module in same file

## Special Directories

**crates/target/:**
- Purpose: Cargo build artifacts (compiled binaries, objects, dependencies)
- Generated: Yes (automatically by `cargo build`)
- Committed: No (.gitignored)

**.planning/codebase/:**
- Purpose: GSD codebase analysis documents used by planning/execution commands
- Generated: No (manually written by GSD mappers)
- Committed: Yes (part of project documentation)

**crates/storage/src/migrations/:**
- Purpose: Versioned SQL migrations for schema evolution
- Generated: No (hand-written SQL)
- Committed: Yes (critical for database reproducibility)

**zsh/completions/:**
- Purpose: Zsh-specific completion definitions
- Generated: No (hand-written zsh completion functions)
- Committed: Yes (required for shell integration)

---

*Structure analysis: 2026-01-29*
