# Contributing to Cherry2K

Thank you for your interest in contributing to Cherry2K! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## Getting Started

1. **Fork the repository** and clone your fork locally
2. **Set up the development environment:**
   ```bash
   cargo build
   cargo test
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feat/your-feature-name
   ```

## Development Workflow

### Before You Start

- Check existing issues and PRs to avoid duplicate work
- For significant changes, open an issue first to discuss the approach
- Read the [CLAUDE.md](CLAUDE.md) for project standards and architecture

### Code Quality Requirements

All contributions must pass these checks:

```bash
cargo fmt --check              # Formatting
cargo clippy -- -D warnings    # Linting (warnings = errors)
cargo test                     # All tests pass
cargo llvm-cov --fail-under-lines 80  # 80% line coverage minimum
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

feat(provider): add Ollama support for local models
fix(storage): handle SQLite busy timeout
docs(readme): add installation instructions
test(cli): add integration tests for REPL
```

**Types:** `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

**Scopes:** `provider`, `storage`, `cli`, `zsh`, `config`

### Pull Request Process

1. **Update documentation** if you change public APIs
2. **Add tests** for new functionality
3. **Ensure all checks pass** before requesting review
4. **Keep PRs focused** - one feature or fix per PR
5. **Write a clear description** explaining what and why

### PR Title Format

```
feat(provider): add streaming support for Anthropic
fix(cli): handle missing config file gracefully
```

## Architecture Guidelines

### Adding a New AI Provider

1. Create `crates/core/src/provider/new_provider.rs`
2. Implement the `AiProvider` trait
3. Add to `ProviderFactory` in `mod.rs`
4. Add configuration schema
5. Write tests with mocked HTTP responses
6. Update documentation

### Error Handling

- Use `thiserror` for library errors
- Propagate errors with `?`
- Never use `.unwrap()` or `.expect()` in library code

```rust
#[derive(Debug, Error)]
pub enum MyError {
    #[error("description: {0}")]
    VariantName(#[from] SourceError),
}
```

### Testing

- Place unit tests in `#[cfg(test)]` modules within source files
- Place integration tests in `tests/` directories
- Use the Arrange-Act-Assert pattern
- Mock external dependencies (HTTP, filesystem)

## Reporting Issues

### Bug Reports

Include:
- Cherry2K version (`cherry2k --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Feature Requests

Include:
- Clear description of the feature
- Use case and motivation
- Proposed implementation approach (if any)

## Getting Help

- Open an issue for questions
- Check existing documentation in `CLAUDE.md`
- Review the standards in `.claude/standards/`

## License

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 License.