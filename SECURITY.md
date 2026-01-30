# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Do NOT open a public issue for security vulnerabilities.**

Instead:

1. **Email:** Send details to the maintainers privately
2. **GitHub Security Advisories:** Use GitHub's private vulnerability reporting feature

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial assessment:** Within 7 days
- **Resolution timeline:** Depends on severity, typically 30-90 days

### After Reporting

1. We'll acknowledge receipt and begin investigation
2. We'll keep you informed of progress
3. Once fixed, we'll coordinate disclosure timing with you
4. We'll credit you in the security advisory (unless you prefer anonymity)

## Security Best Practices for Users

### API Key Management

- **Never commit API keys** to version control
- Store keys in environment variables:
  ```bash
  export OPENAI_API_KEY="sk-..."
  export ANTHROPIC_API_KEY="sk-ant-..."
  ```
- Or use a config file with restricted permissions:
  ```bash
  chmod 600 ~/.config/cherry2k/config.toml
  ```

### Configuration File Security

Cherry2K config files may contain sensitive data. Ensure:

```bash
# Config directory permissions
chmod 700 ~/.config/cherry2k

# Config file permissions
chmod 600 ~/.config/cherry2k/config.toml
```

### Local Model Security (Ollama)

When using Ollama for local inference:

- Keep Ollama updated to the latest version
- Bind to localhost only (default behavior)
- Don't expose the Ollama API to untrusted networks

### SQLite Database

Conversation history is stored locally in SQLite:

- The database may contain sensitive conversation content
- Ensure appropriate file permissions on the database file
- Consider the implications before sharing or backing up

## Security Design Principles

### What Cherry2K Does

- Stores API keys in memory only during runtime (not persisted)
- Uses HTTPS for all cloud API communications
- Validates and sanitizes user input before sending to providers
- Logs at configurable levels (never logs API keys)

### What Cherry2K Does NOT Do

- Send telemetry or usage data
- Store API keys in plaintext (expects env vars or secure config)
- Make network requests to unexpected endpoints
- Execute arbitrary code from AI responses

## Dependency Security

We monitor dependencies for known vulnerabilities:

```bash
cargo audit
```

Dependencies are reviewed for security implications before adoption.

## Scope

This security policy covers:

- The Cherry2K CLI application
- The Rust crates (`core`, `storage`, `cli`)
- The Zsh integration scripts

Out of scope:

- Third-party AI providers (OpenAI, Anthropic, Ollama)
- User's system configuration
- Network infrastructure