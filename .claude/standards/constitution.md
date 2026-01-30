# Project Constitution

> **Version**: 1.0.0
> **Project**: Cherry2K.sh.sh - Zsh Terminal AI Assistant
> **Language**: Rust
> **Owner**: David Dunnock
> **Created**: 2026-01-29

---

## Document Structure

This constitution is organized as a modular system. Global principles live here; specific standards are in referenced files.

| File | Scope | When to Read |
|------|-------|--------------|
| `constitution.md` | Global principles, quality gates, workflows | Always |
| `rust.md` | Rust ≥1.75, Cargo workspace, Clippy, async patterns | All Rust development |
| `testing.md` | Coverage requirements, test patterns, mocking | Writing tests |
| `documentation.md` | Doc comments, README, rustdoc | Writing documentation |
| `git-cicd.md` | Commits, branches, PRs, GitHub Actions | Version control, deployment |
| `security.md` | API keys, secrets, input validation | Security concerns |

---

## 1. Purpose and Scope

This constitution establishes the authoritative standards for all development activities within Cherry2K.sh. It serves as the single source of truth for architectural decisions, coding standards, and quality gates.

### 1.1 Applicability

This constitution applies to:

- All Rust source code in `crates/`
- Zsh integration scripts in `zsh/`
- Configuration files and schemas
- All documentation and technical specifications
- All automated tests and verification procedures
- CI/CD pipelines and release processes

### 1.2 Normative Language

This document uses RFC 2119 terminology:

| Term | Meaning |
|------|---------|
| **MUST** | Absolute requirement—violation blocks merge |
| **MUST NOT** | Absolute prohibition—violation blocks merge |
| **SHOULD** | Strongly recommended—deviation requires documented rationale |
| **SHOULD NOT** | Discouraged—use only with justification |
| **MAY** | Optional—developer preference determines behavior |

---

## 2. Global Quality Standards

### 2.1 Test Coverage Requirements

All code **MUST** meet or exceed the following coverage thresholds:

| Metric | Minimum | Target | Blocking |
|--------|---------|--------|----------|
| Line Coverage | 80% | 90% | Yes |
| Branch Coverage | 75% | 85% | Yes |
| Function Coverage | 85% | 95% | Yes |

**Exclusions** (MUST be documented with rationale):

- Generated code (e.g., build scripts)
- Integration test fixtures
- Debug-only code paths

> See `testing.md` for enforcement commands and test patterns.

### 2.2 Documentation Requirements

All public APIs **MUST** have 100% documentation coverage.

| Scope | Requirement |
|-------|-------------|
| Public functions | **MUST** have doc comments with examples |
| Public types | **MUST** have doc comments explaining purpose |
| Modules | **MUST** have module-level documentation |
| Error types | **MUST** document all variants |

### 2.3 Type Safety and Linting

All code **MUST** pass with zero errors:

| Tool | Configuration | Blocking |
|------|--------------|----------|
| `cargo fmt` | Default | Yes |
| `cargo clippy` | `pedantic` + `nursery` | Yes |
| `cargo test` | All tests | Yes |
| `cargo doc` | No warnings | Yes |

---

## 3. Dependency Management

### 3.1 Freshness Policy

All dependencies **MUST** use the latest stable version unless a specific conflict exists.

| Rule | Enforcement |
|------|-------------|
| Default to latest | All new dependencies at latest stable version |
| Verify via crates.io | Version selection **MUST** be verified via registry lookup |
| Document conflicts | Any pinned dependency **MUST** document the reason |
| Security audits | Run `cargo audit` weekly |

### 3.2 Verification Commands

```bash
# Check for outdated dependencies
cargo outdated

# Search for latest version
cargo search <crate>

# Security audit
cargo audit
```

### 3.3 Conflict Documentation

Document pinned versions in `Cargo.toml` with inline comments:

```toml
# PINNED: v2.x has breaking changes to streaming API, see issue #42
reqwest = ">=0.11.0,<0.12.0"
```

---

## 4. Project Structure

### 4.1 Workspace Layout

```
cherry2k/
├── .claude/
│   ├── standards/           # This directory
│   │   ├── constitution.md
│   │   ├── rust.md
│   │   ├── testing.md
│   │   ├── documentation.md
│   │   ├── git-cicd.md
│   │   └── security.md
│   └── memory/              # Persistent project state (GSD)
├── crates/
│   ├── core/                # Domain logic + provider abstraction
│   ├── storage/             # SQLite persistence
│   └── cli/                 # Terminal interface
├── zsh/                     # Zsh integration
│   ├── cherry2k.plugin.zsh
│   ├── widgets/
│   └── completions/
├── tests/                   # Workspace-level integration tests
├── Cargo.toml               # Workspace root
├── CLAUDE.md
├── README.md
└── CHANGELOG.md
```

### 4.2 Crate Organization

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `cherry2k-core` | Provider trait, conversation logic, config | `tokio`, `reqwest`, `serde` |
| `cherry2k-storage` | SQLite via rusqlite, migrations | `rusqlite`, `cherry2k-core` |
| `cherry2k-cli` | CLI commands, REPL, output formatting | `clap`, `cherry2k-core`, `cherry2k-storage` |

---

## 5. Exception Handling

### 5.1 Deviation Process

Any deviation from this constitution **MUST** follow:

1. **Document the deviation** with rationale in code comments or PR description
2. **Justify the deviation** with technical rationale
3. **Set an expiration** for temporary deviations
4. **Track the deviation** in the relevant issue

### 5.2 Deviation Template

```markdown
## Deviation: [Brief Description]

**Constitution Section**: [e.g., 2.1 Test Coverage]
**Reason**: [Why deviation is necessary]
**Scope**: [What code is affected]
**Expiration**: [When to revisit, or "Permanent"]
**Approved**: [Date]
```

---

## 6. Workflow: Get Shit Done (GSD)

Cherry2K.sh development follows the GSD methodology with Claude Code.

### 6.1 GSD Principles

1. **Plan before coding** - Use `/gsd:plan-phase` to design before implementing
2. **Atomic commits** - Each commit should be a complete, working change
3. **Verify completeness** - Use `/gsd:verify-work` before marking done
4. **Debug systematically** - Use `/gsd:debug` for structured problem-solving

### 6.2 Development Cycle

```
/gsd:progress → /gsd:plan-phase → /gsd:execute-phase → /gsd:verify-work
      ↑                                                        │
      └────────────────────────────────────────────────────────┘
```

### 6.3 Key Commands

| Command | When to Use |
|---------|-------------|
| `/gsd:new-project` | Initial project setup |
| `/gsd:progress` | Check current state, find next task |
| `/gsd:plan-phase` | Design implementation approach |
| `/gsd:execute-phase` | Implement planned work |
| `/gsd:verify-work` | Validate completion |
| `/gsd:debug` | Systematic issue investigation |

---

## 7. Architecture Principles

### 7.1 Provider Abstraction

The AI provider system **MUST** follow these principles:

- **Trait-based polymorphism** - All providers implement `AiProvider` trait
- **Async-first** - Use async/await for all I/O operations
- **Error propagation** - Use `Result<T, ProviderError>` consistently
- **Configuration validation** - Validate before first use

### 7.2 Storage Layer

The SQLite storage layer **MUST**:

- Use migrations for all schema changes
- Support offline operation
- Handle concurrent access gracefully
- Never store secrets in the database

### 7.3 CLI Design

The CLI **MUST**:

- Support both interactive (REPL) and one-shot modes
- Provide structured output (JSON) for scripting
- Integrate cleanly with zsh widgets
- Handle terminal resize and signals properly

---

## Glossary

| Term | Definition |
|------|------------|
| **GSD** | Get Shit Done - development methodology for Claude Code |
| **Provider** | AI service backend (OpenAI, Anthropic, Ollama) |
| **Widget** | Zsh ZLE widget for keyboard shortcuts |
| **REPL** | Read-Eval-Print Loop - interactive mode |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-29 | Claude | Initial constitution for Cherry2K.sh |

---

*This constitution is a living document. Update via pull request.*