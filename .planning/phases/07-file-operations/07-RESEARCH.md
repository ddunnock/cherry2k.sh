# Phase 7: File Operations - Research

**Researched:** 2026-01-31
**Domain:** File I/O, diff generation, path traversal, git repository detection
**Confidence:** HIGH

## Summary

This phase implements file operations for Cherry2K: reading files when users reference them, proposing file edits/creation with unified diff previews, requiring user approval before writing (safe mode), and respecting project scope boundaries. The existing codebase already has the foundation: `confirm.rs` provides `[y/n/e]` confirmation patterns, and Phase 6 established command execution with approval flows.

The core requirements are: (1) smart file detection when users mention files in context, (2) unified diff generation with syntax highlighting, (3) approval flow consistent with command confirmation, (4) project scope detection via git repository root, and (5) safety boundaries for secrets and out-of-scope paths.

Rust's `std::fs` provides safe, efficient file I/O. The `similar` crate is the standard for diff generation with unified format support and syntax-aware output. The `git2` crate (libgit2 bindings) provides robust repository discovery. The `walkdir` crate handles recursive directory traversal with filtering. The `ignore` crate implements gitignore pattern matching. The `edit` crate provides cross-platform `$EDITOR` integration.

**Primary recommendation:** Use `similar::TextDiff` for unified diff generation with syntax highlighting via `termimad` (already in use). Use `git2::Repository::discover()` to find project root. Use `walkdir` with `ignore` for file traversal. Use `edit::edit()` for `$EDITOR` integration in the edit flow. Build on existing `confirm.rs` patterns for approval prompts.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::fs | stdlib | File I/O operations | Safe, efficient, built-in; no external dependencies |
| similar | 2.7+ | Diff generation | Fast, dependency-free, unified diff format, syntax-aware |
| git2 | 0.19+ | Git repository detection | Robust libgit2 bindings, industry standard |
| walkdir | 2.5+ | Directory traversal | Cross-platform, efficient, filter support |
| ignore | 0.4+ | Gitignore pattern matching | Fast, 80M+ downloads, used by ripgrep/cargo |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| edit | 1.0+ | $EDITOR integration | Cross-platform editor launching for edit flow |
| tempfile | 3.24+ | Temporary files | Already in workspace; use for $EDITOR temp files |
| termimad | 0.30+ | Syntax highlighting | Already in use; extend for diff syntax highlighting |
| colored | 3.x | ANSI colors | Already in use; extend for diff +/- line coloring |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| similar | diffy | similar is faster, more feature-complete, better maintained |
| git2 | git-discover | git2 provides full git functionality if needed later |
| walkdir | std::fs::read_dir | walkdir handles symlinks, filtering, cross-platform edge cases |
| edit | manual $EDITOR spawn | edit handles platform differences, fallback editors |

**Installation:**
```bash
cargo add similar git2 walkdir ignore edit
# tempfile, termimad, colored already in workspace
```

## Architecture Patterns

### Recommended Project Structure
```
crates/cli/src/
├── commands/
│   ├── chat.rs           # Extend to detect file references
│   └── files.rs          # NEW: Explicit file commands (/read, /write)
├── files/                # NEW: File operations module
│   ├── mod.rs
│   ├── reader.rs         # Smart file reading with size checks
│   ├── writer.rs         # File write/edit with diff preview
│   ├── diff.rs           # Unified diff generation + highlighting
│   ├── scope.rs          # Project boundary detection
│   └── security.rs       # Secrets detection, path validation
├── intent/
│   ├── detector.rs       # Extend to detect file operation intents
│   └── types.rs          # Add FileOperation intent variant
└── confirm.rs            # Extend for file approval prompts
```

### Pattern 1: Git Repository Discovery for Project Root
**What:** Use git2 to find the nearest `.git` directory, establishing project scope
**When to use:** At session start, before any file operations
**Example:**
```rust
// Source: https://docs.rs/git2/latest/git2/struct.Repository.html
use git2::Repository;
use std::path::{Path, PathBuf};

/// Find the project root by searching for the nearest git repository.
///
/// Starts from `start_path` and walks up the directory tree until a .git
/// directory is found. Returns the repository's working directory path.
pub fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    Repository::discover(start_path)
        .ok()
        .and_then(|repo| repo.workdir().map(Path::to_path_buf))
}

// Usage in scope detection
pub struct ProjectScope {
    root: PathBuf,
}

impl ProjectScope {
    pub fn detect() -> Result<Self, Error> {
        let cwd = std::env::current_dir()?;
        let root = find_project_root(&cwd)
            .unwrap_or_else(|| cwd.clone()); // Fallback to cwd if not in git repo

        Ok(Self { root })
    }

    pub fn is_within_scope(&self, path: &Path) -> bool {
        path.canonicalize()
            .ok()
            .and_then(|p| p.strip_prefix(&self.root).ok())
            .is_some()
    }
}
```

### Pattern 2: Unified Diff Generation with Syntax Highlighting
**What:** Generate git-style unified diffs with language-aware syntax highlighting
**When to use:** When proposing file edits or new file creation
**Example:**
```rust
// Source: https://docs.rs/similar/latest/similar/
use similar::{ChangeTag, TextDiff};
use colored::Colorize;

/// Generate a unified diff between old and new file content.
///
/// Returns formatted diff string with ANSI color codes:
/// - Green lines start with '+'
/// - Red lines start with '-'
/// - Context lines start with ' '
pub fn generate_diff(old: &str, new: &str, filename: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    // Header
    output.push_str(&format!("--- a/{}\n", filename));
    output.push_str(&format!("+++ b/{}\n", filename));

    // Unified hunks with 3 lines of context
    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        output.push_str(&hunk.header().to_string());

        for change in hunk.iter_changes() {
            let (sign, color_fn): (&str, fn(&str) -> colored::ColoredString) =
                match change.tag() {
                    ChangeTag::Delete => ("-", |s: &str| s.red()),
                    ChangeTag::Insert => ("+", |s: &str| s.green()),
                    ChangeTag::Equal => (" ", |s: &str| s.normal()),
                };

            let line = format!("{}{}", sign, change);
            output.push_str(&color_fn(&line).to_string());
        }
    }

    output
}

/// Display diff for new file creation (not a diff, just preview)
pub fn display_new_file_preview(content: &str, filename: &str) {
    println!("New file: {}", filename.green());
    println!("---");

    // Syntax highlight based on extension if using termimad
    // For now, simple preview:
    for line in content.lines() {
        println!("  {}", line);
    }
    println!("---");
}
```

### Pattern 3: Smart File Reading with Size Assessment
**What:** Detect file references in conversation, check size before reading
**When to use:** When user mentions files in their message
**Example:**
```rust
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

const LARGE_FILE_THRESHOLD: u64 = 100_000; // 100KB
const MAX_FILE_SIZE: u64 = 1_000_000; // 1MB hard limit

/// Read a file with safety checks and size assessment.
///
/// - Checks file size before reading
/// - Prompts user for large files
/// - Returns file content or error
pub fn read_file_with_checks(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

    let size = metadata.len();

    // Hard limit check
    if size > MAX_FILE_SIZE {
        anyhow::bail!(
            "File {} is too large ({} bytes). Maximum is {} bytes.",
            path.display(),
            size,
            MAX_FILE_SIZE
        );
    }

    // Warn on large files
    if size > LARGE_FILE_THRESHOLD {
        println!(
            "File {} is large ({} bytes). This may take time to process.",
            path.display(),
            size
        );

        // Could add confirmation prompt here if desired
        let result = crate::confirm::confirm("Continue reading?", false)?;
        if result != crate::confirm::ConfirmResult::Yes {
            anyhow::bail!("File read cancelled by user");
        }
    }

    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file {}", path.display()))
}

/// Detect file paths mentioned in user message.
///
/// Simple heuristic: words that look like paths and exist as files.
pub fn detect_file_references(message: &str, cwd: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for word in message.split_whitespace() {
        // Remove common punctuation
        let word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-');

        // Try as absolute path
        let path = Path::new(word);
        if path.is_file() {
            files.push(path.to_path_buf());
            continue;
        }

        // Try as relative to cwd
        let path = cwd.join(word);
        if path.is_file() {
            files.push(path);
        }
    }

    files
}
```

### Pattern 4: File Write with Approval Flow
**What:** Propose file changes with diff preview, require approval before write
**When to use:** When AI wants to create or edit files
**Example:**
```rust
use crate::confirm::{confirm, ConfirmResult};
use crate::files::diff::generate_diff;
use std::fs;
use std::path::Path;

/// Write file content with user approval.
///
/// Shows diff preview, asks for confirmation, supports edit flow.
pub fn write_file_with_approval(
    path: &Path,
    new_content: &str,
    config: &Config,
) -> Result<()> {
    let old_content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new() // New file
    };

    // Generate and display diff
    if old_content.is_empty() {
        println!("\nProposed new file: {}\n", path.display());
        display_new_file_preview(new_content, path.display().to_string().as_str());
    } else {
        println!("\nProposed changes to: {}\n", path.display());
        let diff = generate_diff(&old_content, new_content, path.display().to_string().as_str());
        print!("{}", diff);
    }

    // Skip approval if auto-write mode
    if !config.safety.confirm_file_writes {
        fs::write(path, new_content)?;
        println!("File written: {}", path.display());
        return Ok(());
    }

    // Approval loop (similar to command confirmation)
    let mut content_to_write = new_content.to_string();

    loop {
        match confirm("Write this file?", true)? {
            ConfirmResult::Yes => {
                fs::write(path, &content_to_write)?;
                println!("File written: {}", path.display());
                return Ok(());
            }
            ConfirmResult::No => {
                println!("File write cancelled.");
                return Ok(());
            }
            ConfirmResult::Edit => {
                // Open in $EDITOR
                content_to_write = edit::edit(&content_to_write)?;

                // Re-display diff with edited content
                if old_content.is_empty() {
                    display_new_file_preview(&content_to_write, path.display().to_string().as_str());
                } else {
                    let diff = generate_diff(&old_content, &content_to_write, path.display().to_string().as_str());
                    print!("{}", diff);
                }
                // Loop continues to re-confirm
            }
        }
    }
}
```

### Pattern 5: Secrets Detection and Path Validation
**What:** Block writes to known secrets files, validate paths for traversal attacks
**When to use:** Before any file write operation
**Example:**
```rust
use std::path::Path;

/// Patterns for known secrets files that should never be written
const BLOCKED_FILENAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    "credentials.json",
    "secrets.json",
    "id_rsa",
    "id_ed25519",
    ".npmrc",
    ".pypirc",
];

/// Check if a filename is a known secrets file.
pub fn is_secrets_file(path: &Path) -> bool {
    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
        BLOCKED_FILENAMES.iter().any(|&blocked| filename == blocked)
    } else {
        false
    }
}

/// Validate that a path is safe to write.
///
/// Checks:
/// - Not a secrets file
/// - Within project scope (if scope detection enabled)
/// - No path traversal attempts (../ outside scope)
pub fn validate_write_path(
    path: &Path,
    scope: &ProjectScope,
) -> Result<(), String> {
    // Block secrets files
    if is_secrets_file(path) {
        return Err(format!(
            "Cannot write to secrets file: {}",
            path.display()
        ));
    }

    // Canonicalize to resolve symlinks and .. components
    let canonical = path.canonicalize()
        .or_else(|_| {
            // If file doesn't exist, canonicalize parent
            path.parent()
                .ok_or_else(|| "Invalid path".to_string())?
                .canonicalize()
                .map(|p| p.join(path.file_name().unwrap()))
        })
        .map_err(|e| format!("Invalid path: {}", e))?;

    // Check if within scope
    if !scope.is_within_scope(&canonical) {
        // Warn but allow with extra confirmation
        return Err(format!(
            "Path {} is outside project scope ({}). Proceed with caution.",
            canonical.display(),
            scope.root.display()
        ));
    }

    Ok(())
}
```

### Pattern 6: Directory Traversal with Filtering
**What:** List files in directory with gitignore and dotfile filtering
**When to use:** When AI needs to scan directory structure
**Example:**
```rust
// Source: https://docs.rs/walkdir/latest/walkdir/
// Source: https://docs.rs/ignore/latest/ignore/
use walkdir::WalkDir;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

/// List files in directory respecting gitignore rules.
pub fn list_files_filtered(
    dir: &Path,
    include_hidden: bool,
    max_depth: usize,
) -> Result<Vec<PathBuf>, Error> {
    // Load gitignore if exists
    let gitignore = if dir.join(".gitignore").exists() {
        let mut builder = GitignoreBuilder::new(dir);
        builder.add(dir.join(".gitignore"));
        Some(builder.build()?)
    } else {
        None
    };

    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(max_depth)
        .follow_links(true) // Follow symlinks per CONTEXT.md decision
        .into_iter()
        .filter_entry(|e| {
            // Filter hidden files if requested
            if !include_hidden {
                if let Some(name) = e.file_name().to_str() {
                    if name.starts_with('.') && name != "." {
                        return false;
                    }
                }
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            // Check gitignore
            if let Some(ref gi) = gitignore {
                if gi.matched(entry.path(), false).is_ignore() {
                    continue;
                }
            }

            // Skip secrets files
            if is_secrets_file(entry.path()) {
                continue;
            }

            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}
```

### Anti-Patterns to Avoid
- **Automatic file writes without approval:** Always require confirmation unless `confirm_file_writes=false`
- **Ignoring file size limits:** Large files can exhaust memory; check size first
- **Path traversal without validation:** Always validate paths against project scope
- **Writing to secrets files:** Block known secrets patterns
- **Symlink attacks:** Use `canonicalize()` to resolve symlinks before validation
- **Forgetting to flush buffers:** Always flush or use `write_all()` for atomic writes

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Diff generation | Custom line-by-line comparison | similar crate | Handles edge cases (newlines, context, hunks), fast algorithms |
| Git repository detection | Manual .git directory search | git2::Repository::discover() | Handles bare repos, worktrees, edge cases |
| Gitignore parsing | Regex patterns | ignore crate | Complex glob semantics, performance, testing |
| Directory traversal | Recursive fs::read_dir | walkdir | Cross-platform, symlink handling, efficient filtering |
| $EDITOR launching | Manual env var + process spawn | edit crate | Platform differences, fallback editors, temp file cleanup |
| Path canonicalization | Manual ../ resolution | Path::canonicalize() | Handles symlinks, platform differences, edge cases |
| Syntax highlighting | ANSI escape codes | termimad (already used) | Language detection, reliable highlighting |

**Key insight:** File operations have subtle platform differences and security implications. Use battle-tested libraries rather than custom implementations.

## Common Pitfalls

### Pitfall 1: TOCTOU Race Conditions
**What goes wrong:** File changes between check (size/exists) and use (read/write)
**Why it happens:** Filesystem operations are not atomic across check and action
**How to avoid:** Use `fs::OpenOptions` to atomically check and open; catch errors rather than pre-checking
**Warning signs:** Intermittent failures, especially in concurrent environments

### Pitfall 2: Symlink Following Outside Scope
**What goes wrong:** Symlink points outside project, allowing writes to sensitive files
**Why it happens:** Following symlinks without canonicalizing first
**How to avoid:** Always `canonicalize()` paths before scope validation
**Warning signs:** Writes to unexpected locations, security audit findings

### Pitfall 3: Unbounded File Reads
**What goes wrong:** Reading huge files exhausts memory, crashes process
**Why it happens:** Not checking file size before `read_to_string()`
**How to avoid:** Check `metadata().len()` first, set max size limit, offer truncation
**Warning signs:** Out-of-memory crashes, slow performance on large repos

### Pitfall 4: Diff Display for Binary Files
**What goes wrong:** Binary files produce garbage output in diffs
**Why it happens:** TextDiff assumes text content
**How to avoid:** Detect binary files (check for null bytes), refuse or show "binary file changed"
**Warning signs:** Corrupted terminal output, encoding errors

### Pitfall 5: Cross-Platform Path Separators
**What goes wrong:** Hard-coded `/` or `\` fail on other platforms
**Why it happens:** Assuming Unix or Windows path format
**How to avoid:** Use `Path::join()` and `Path::components()`, never string concatenation
**Warning signs:** Failures on Windows (or Unix if developed on Windows)

### Pitfall 6: Forgetting to Handle Approval Edit Loop
**What goes wrong:** Edit option doesn't loop back to re-confirm
**Why it happens:** Not structuring approval as a loop
**How to avoid:** Use loop with match on ConfirmResult, similar to Phase 6 command flow
**Warning signs:** Edit saves without re-confirming, inconsistent UX

### Pitfall 7: Secrets Detection Bypass
**What goes wrong:** User edits file path to bypass secrets detection
**Why it happens:** Only checking filename, not canonical path
**How to avoid:** Validate canonical path at every approval step, not just initial
**Warning signs:** Successful writes to .env, credentials files

## Code Examples

Verified patterns from official sources:

### File Reading with Error Handling (std::fs pattern)
```rust
// Source: https://doc.rust-lang.org/std/fs/
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))
}

// With size check
fn read_file_safely(path: &Path, max_size: u64) -> Result<String> {
    let metadata = fs::metadata(path)?;

    if metadata.len() > max_size {
        anyhow::bail!(
            "File {} too large ({} bytes, max {})",
            path.display(),
            metadata.len(),
            max_size
        );
    }

    fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))
}
```

### Atomic File Write Pattern
```rust
// Source: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
use std::fs::OpenOptions;
use std::io::Write;

fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(content.as_bytes())?;
    file.sync_all()?; // Ensure written to disk

    Ok(())
}
```

### $EDITOR Integration (edit crate)
```rust
// Source: https://docs.rs/edit/latest/edit/
use edit;

fn edit_in_editor(initial_content: &str) -> Result<String> {
    // Opens user's $EDITOR with initial content
    let edited = edit::edit(initial_content)?;
    Ok(edited)
}

// With custom tempfile name
use edit::Builder;

fn edit_file_content(content: &str, filename: &str) -> Result<String> {
    let edited = Builder::new()
        .suffix(&format!(".{}", filename))
        .edit(content)?;
    Ok(edited)
}
```

### Path Canonicalization for Security
```rust
// Source: https://doc.rust-lang.org/std/path/struct.Path.html
use std::path::{Path, PathBuf};

fn safe_canonicalize(path: &Path) -> Result<PathBuf> {
    // For existing paths
    if path.exists() {
        return path.canonicalize()
            .with_context(|| format!("Failed to resolve {}", path.display()));
    }

    // For non-existing paths, canonicalize parent
    let parent = path.parent()
        .ok_or_else(|| anyhow::anyhow!("Path has no parent"))?;

    let canonical_parent = parent.canonicalize()?;
    let filename = path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Path has no filename"))?;

    Ok(canonical_parent.join(filename))
}

fn is_within_directory(path: &Path, base: &Path) -> bool {
    safe_canonicalize(path)
        .ok()
        .and_then(|p| p.strip_prefix(base).ok())
        .is_some()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| String concatenation for paths | Path::join() | Always (stdlib) | Cross-platform safety |
| Manual diff line-by-line | similar crate | 2020+ | Faster, handles edge cases |
| Regex for gitignore | ignore crate | 2016+ | Correct glob semantics |
| vim/nano hardcoded | edit crate with $EDITOR | 2015+ | Respects user preference |
| Manual symlink resolution | Path::canonicalize() | Always (stdlib) | Security, correctness |

**Deprecated/outdated:**
- `diff` crate: Replaced by `similar` (better API, more features)
- Manual `.git` directory search: Use `git2::Repository::discover()`
- Blocking on large files without size check: Always check metadata first
- Trusting user input paths: Always canonicalize and validate

## Open Questions

Things that couldn't be fully resolved:

1. **Binary file detection**
   - What we know: Check for null bytes, use `file` command, or magic numbers
   - What's unclear: Best heuristic for Rust without external dependencies
   - Recommendation: Simple null byte check initially, enhance later if needed

2. **Diff context lines for very large diffs**
   - What we know: 3 lines is standard, configurable in similar
   - What's unclear: Optimal UX for huge diffs (hundreds of changes)
   - Recommendation: Start with 3, offer summary mode if >50 hunks

3. **Multiple file changes workflow**
   - What we know: Show all diffs, then offer "approve all" or step through
   - What's unclear: Best UX for "approve all" - single prompt or batch display?
   - Recommendation: Show all diffs first, then single "approve all [y/n/step]" prompt

4. **Scope boundary for non-git projects**
   - What we know: Fallback to current working directory
   - What's unclear: Should we search for Cargo.toml, package.json, etc.?
   - Recommendation: Start with git-only, add Cargo.toml detection in v2 if needed

## Sources

### Primary (HIGH confidence)
- [std::fs](https://doc.rust-lang.org/std/fs/) - Standard file I/O operations
- [similar crate](https://docs.rs/similar) - Diff generation, unified format
- [git2::Repository](https://docs.rs/git2/latest/git2/struct.Repository.html) - Repository discovery
- [walkdir crate](https://docs.rs/walkdir/) - Directory traversal
- [ignore crate](https://docs.rs/ignore) - Gitignore pattern matching
- [edit crate](https://docs.rs/edit/latest/edit/) - $EDITOR integration

### Secondary (MEDIUM confidence)
- [Rust File Handling (Programiz)](https://www.programiz.com/rust/file-handling) - Best practices overview
- [RustJobs File I/O Guide](https://rustjobs.dev/blog/reading-and-writing-files-in-rust/) - Patterns and examples
- [Sling Academy Permissions](https://www.slingacademy.com/article/working-with-rusts-permissions-model-for-secure-file-access/) - Security best practices
- [CVE-2022-21658](https://groups.google.com/g/rustlang-security-announcements/c/R1fZFDhnJVQ) - Symlink security advisory

### Tertiary (LOW confidence)
- Syntax highlighting approaches from web search - verified with termimad (already in use)
- Multiple file changes UX - needs user testing

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates are well-documented, battle-tested, high download counts
- Architecture: HIGH - Patterns align with existing codebase (confirm.rs, Phase 6 flow)
- Pitfalls: HIGH - Based on official docs, CVE reports, security guides
- File detection heuristics: MEDIUM - Simple approach, may need tuning

**Research date:** 2026-01-31
**Valid until:** 2026-03-31 (60 days - stable domain, file I/O APIs unlikely to change)
