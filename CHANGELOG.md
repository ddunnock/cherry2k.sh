# Changelog

All notable changes to Cherry2K will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with workspace layout
- Provider-agnostic AI abstraction (`AiProvider` trait)
- OpenAI provider implementation
- Anthropic provider implementation
- Ollama provider implementation for local models
- SQLite storage for conversation persistence
- CLI with REPL mode
- Zsh integration with ZLE widgets
- Configuration system with environment variable support
- `.env` file support via dotenvy
- Sentry error tracking integration
- Sentry test command for integration verification

### Changed
- Consistent environment variable overrides for configuration
- Valid fallback path for configuration loading

### Fixed
- Configuration environment variable override consistency

## [0.1.0] - TBD

### Added
- Initial release
- Multi-provider AI support (OpenAI, Anthropic, Ollama)
- Conversation history with SQLite
- Interactive REPL mode
- Zsh shell integration
- Streaming response support

---

## Version History Format

### Types of Changes

- **Added** - New features
- **Changed** - Changes in existing functionality
- **Deprecated** - Soon-to-be removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Vulnerability fixes