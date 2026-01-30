# Git & CI/CD Standards

> **Applies to**: All version control and deployment workflows for Cherry2K.sh
> **Parent**: `constitution.md`

---

## 1. Commit Message Format

All commits **MUST** follow the Conventional Commits specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### 1.1 Types

| Type | Purpose | Example |
|------|---------|---------|
| `feat` | New feature | `feat(provider): add Ollama support` |
| `fix` | Bug fix | `fix(storage): handle SQLite busy timeout` |
| `docs` | Documentation only | `docs(readme): add installation steps` |
| `style` | Code formatting | `style: apply rustfmt` |
| `refactor` | Code refactoring | `refactor(cli): extract output formatting` |
| `perf` | Performance improvement | `perf(provider): add response caching` |
| `test` | Adding/updating tests | `test(openai): add rate limit handling tests` |
| `build` | Build system changes | `build(deps): upgrade tokio to 1.36` |
| `ci` | CI/CD changes | `ci: add coverage reporting` |
| `chore` | Maintenance tasks | `chore: update Cargo.lock` |

### 1.2 Scopes

| Scope | Description |
|-------|-------------|
| `provider` | AI provider implementations |
| `openai` | OpenAI-specific code |
| `anthropic` | Anthropic-specific code |
| `ollama` | Ollama-specific code |
| `storage` | SQLite/rusqlite code |
| `cli` | CLI commands and REPL |
| `zsh` | Zsh integration scripts |
| `config` | Configuration handling |
| `deps` | Dependency updates |

### 1.3 Examples

```bash
# Feature
feat(provider): add streaming response support

Implement Server-Sent Events parsing for real-time
response streaming from OpenAI and Anthropic APIs.

# Bug fix
fix(storage): handle concurrent database access

Add busy_timeout pragma to prevent SQLITE_BUSY errors
when multiple processes access the database.

Closes #42

# Breaking change
feat(cli)!: change output format for JSON mode

BREAKING CHANGE: The --json flag now outputs newline-delimited
JSON instead of a single array. Use --json-array for old behavior.
```

---

## 2. Branch Strategy

```
main                        # Production-ready code
├── develop                 # Integration branch
│   ├── feature/add-ollama-provider
│   ├── feature/streaming-responses
│   └── fix/sqlite-busy-timeout
├── release/v0.2.0          # Release preparation
└── hotfix/critical-api-fix
```

### 2.1 Branch Rules

| Rule | Enforcement |
|------|-------------|
| `main` always deployable | **MUST** pass all checks before merge |
| Changes via PR only | Direct pushes to `main` **MUST NOT** be allowed |
| Rebase before merge | Feature branches **MUST** be rebased on `develop` |
| Squash merge | Feature branches **SHOULD** use squash merge |

### 2.2 Branch Naming

```
feature/short-description
fix/issue-description
hotfix/critical-fix-description
release/vX.Y.Z
chore/maintenance-description
```

---

## 3. Pull Request Requirements

### 3.1 Before Opening PR

- [ ] All tests pass locally (`cargo test`)
- [ ] Coverage thresholds met (`cargo llvm-cov --fail-under-lines 80`)
- [ ] Linting passes (`cargo clippy -- -D warnings`)
- [ ] Formatting applied (`cargo fmt`)
- [ ] Documentation updated if needed
- [ ] Self-review completed

### 3.2 PR Template

```markdown
## Summary

Brief description of changes.

## Type of Change

- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing

Describe the tests added or modified.

## Checklist

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review
- [ ] I have added tests proving my fix/feature works
- [ ] All new and existing tests pass
- [ ] I have updated relevant documentation

## Related Issues

Closes #[issue number]
```

### 3.3 Review Requirements

| Check | Requirement |
|-------|-------------|
| Approvals | At least 1 approval required |
| CI Status | All checks must pass |
| Up to date | Branch must be up to date with base |

---

## 4. CI Pipeline

### 4.1 Main Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Run cargo check
        run: cargo check --all-targets

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    needs: [check, fmt, clippy]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Run tests with coverage
        run: cargo llvm-cov --lcov --output-path lcov.info --fail-under-lines 80

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  doc:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build docs
        run: cargo doc --no-deps --document-private-items
        env:
          RUSTDOCFLAGS: -D warnings

  build:
    name: Build
    runs-on: ${{ matrix.os }}
    needs: test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build release
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: cherry2k-${{ matrix.os }}
          path: target/release/cherry2k
```

### 4.2 Quality Gates

**Blocking** (MUST pass to merge):

- `cargo check` succeeds
- `cargo fmt --check` passes
- `cargo clippy -- -D warnings` passes
- All tests pass
- Code coverage ≥80%
- `cargo doc` builds without warnings

**Non-Blocking** (warnings only):

- Coverage <90% (target)
- Slow tests

---

## 5. Release Process

### 5.1 Versioning

Follow Semantic Versioning (SemVer):

```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes to CLI or public API
MINOR: New features (backward compatible)
PATCH: Bug fixes (backward compatible)
```

### 5.2 Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    name: Build Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: cherry2k-linux-amd64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: cherry2k-darwin-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: cherry2k-darwin-arm64

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        run: |
          mkdir -p dist
          cp target/${{ matrix.target }}/release/cherry2k dist/
          cp README.md LICENSE dist/
          tar -czvf ${{ matrix.artifact }}.tar.gz -C dist .

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}.tar.gz

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
          files: artifacts/**/*.tar.gz

  homebrew:
    name: Update Homebrew
    needs: release
    runs-on: ubuntu-latest
    steps:
      - name: Update Homebrew formula
        uses: mislav/bump-homebrew-formula-action@v3
        with:
          formula-name: cherry2k
          homebrew-tap: dunnock/homebrew-tap
        env:
          COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
```

### 5.3 Changelog

Maintain `CHANGELOG.md` following Keep a Changelog format:

```markdown
# Changelog

All notable changes to Cherry2K.sh will be documented in this file.

## [Unreleased]

### Added
- Feature description

### Changed
- Change description

### Fixed
- Fix description

## [0.1.0] - 2026-01-29

### Added
- Initial release
- OpenAI provider support
- Anthropic provider support
- Ollama provider support
- SQLite conversation storage
- Zsh integration with widgets
```

---

## 6. Branch Protection

### 6.1 GitHub Settings

Configure branch protection for `main`:

```yaml
# Branch protection rules
protection:
  required_pull_request_reviews:
    required_approving_review_count: 1
    dismiss_stale_reviews: true
  required_status_checks:
    strict: true
    contexts:
      - "Check"
      - "Format"
      - "Clippy"
      - "Test"
      - "Documentation"
      - "Build (ubuntu-latest)"
      - "Build (macos-latest)"
  enforce_admins: true
  allow_force_pushes: false
  allow_deletions: false
```

---

## 7. Homebrew Distribution

### 7.1 Formula Template

```ruby
# Formula/cherry2k.rb
class Cherry2k < Formula
  desc "AI assistant for your zsh terminal"
  homepage "https://github.com/dunnock/cherry2k"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/dunnock/cherry2k/releases/download/v#{version}/cherry2k-darwin-arm64.tar.gz"
      sha256 "..."
    else
      url "https://github.com/dunnock/cherry2k/releases/download/v#{version}/cherry2k-darwin-amd64.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/dunnock/cherry2k/releases/download/v#{version}/cherry2k-linux-amd64.tar.gz"
    sha256 "..."
  end

  def install
    bin.install "cherry2k"

    # Install zsh integration
    zsh_completion.install "completions/_cherry2k"
    (share/"cherry2k").install "cherry2k.plugin.zsh"
  end

  def caveats
    <<~EOS
      To enable zsh integration, add to your .zshrc:
        source #{opt_share}/cherry2k/cherry2k.plugin.zsh

      Set your API keys:
        export OPENAI_API_KEY=sk-...
        export ANTHROPIC_API_KEY=sk-ant-...
    EOS
  end

  test do
    assert_match "cherry2k", shell_output("#{bin}/cherry2k --version")
  end
end
```

### 7.2 Installation

```bash
# Add tap
brew tap dunnock/tap

# Install
brew install cherry2k

# Enable zsh integration
echo 'source $(brew --prefix)/share/cherry2k/cherry2k.plugin.zsh' >> ~/.zshrc
```

---

## 8. Local Development

### 8.1 Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt -- --check

# Clippy
cargo clippy -- -D warnings

# Tests
cargo test --quiet

echo "All checks passed!"
```

### 8.2 Install Hooks

```bash
# Install pre-commit hook
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```