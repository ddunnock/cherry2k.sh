# Domain Pitfalls: Terminal AI Assistants

**Project:** Cherry2K - Zsh Terminal AI Assistant
**Researched:** 2026-01-29
**Overall Confidence:** HIGH

---

## Executive Summary

Building a terminal AI assistant that integrates deeply with zsh involves several well-documented pitfalls. This research consolidates lessons from production incidents, security research, and open-source project issues (Claude Code, Gemini CLI, Warp, zsh_codex, powerlevel10k) into actionable guidance.

**Critical finding:** The most dangerous pitfalls are NOT the obvious ones. Teams consistently underestimate:
1. Security implications of command execution
2. ZLE widget state management complexity
3. Async/SQLite interaction foot-guns

---

## Critical Pitfalls

Mistakes that cause security vulnerabilities, data loss, or fundamental architecture problems requiring major rework.

---

### Pitfall 1: Auto-Execute Without Sandboxing

**What goes wrong:** AI-suggested commands execute automatically without proper sandboxing. Malicious file content, compromised conversation history, or crafted input causes destructive operations (`rm -rf`, credential exfiltration, cloud resource deletion).

**Why it happens:** Teams prioritize UX ("fewer prompts!") over security. The assumption is "AI won't suggest dangerous things" — but LLMs can be manipulated via indirect prompt injection embedded in files they read.

**Consequences:**
- Credential theft (`.env`, SSH keys, AWS credentials leaked via "helpful" command output)
- File system destruction (documented incidents of 46+ files corrupted despite safety warnings)
- Cloud resource deletion (2025 incidents of AI tools deleting cloud resources with developer privileges)

**Prevention:**
1. **Default to ask-mode.** User must explicitly opt into auto-execute.
2. **Implement OS-level sandboxing** (macOS seatbelt, Linux bubblewrap) for command execution — not just application-level checks.
3. **Network isolation** — commands cannot phone home to attacker servers.
4. **Dangerous command detection** — maintain a blocklist of patterns (`rm -rf`, `curl | sh`, `chmod 777`, credential-adjacent commands).
5. **Audit trail** — log every command executed, even in auto-mode.

**Detection (warning signs):**
- No permission system in design docs
- "We'll add security later" in planning discussions
- Auto-execute enabled by default in config schema

**Relevant phase:** Phase 1 (Core Safety) — this must be foundational, not bolted on.

**Sources:**
- [Gemini CLI vulnerability - silent command execution](https://www.bleepingcomputer.com/news/security/flaw-in-gemini-cli-ai-coding-assistant-allowed-stealthy-code-execution/)
- [Claude Code sandboxing documentation](https://code.claude.com/docs/en/sandboxing)
- [Anthropic's sandboxing engineering blog](https://www.anthropic.com/engineering/claude-code-sandboxing)
- [VS Code AI agent file corruption incident](https://github.com/microsoft/vscode/issues/256975)

---

### Pitfall 2: Silent Failures from AI

**What goes wrong:** AI generates code/commands that appear to work but fail silently. No syntax errors, no crashes — just incorrect behavior. Commands run but don't do what user expected.

**Why it happens:** Recent LLMs (2025-2026) increasingly produce "surface-valid" outputs that pass basic checks but fail functionally. This is worse than obvious crashes because users trust the output.

**Consequences:**
- User executes commands thinking they're correct
- Data corruption from "working" but wrong file operations
- Security vulnerabilities from incomplete safety checks removed by AI
- Hours of debugging seemingly correct code

**Prevention:**
1. **Show command effects before execution.** "This will: [list effects]"
2. **Dry-run mode for file operations.** Show diff, don't apply until confirmed.
3. **Validate AI output** — for commands, check syntax; for file writes, verify structure.
4. **Never trust AI claims** about what a command does; parse and verify independently.

**Detection:**
- AI describes command effects without verification logic
- File operations don't preview changes
- No validation between AI response and execution

**Relevant phase:** Phase 2 (Command Execution) and Phase 3 (File Operations)

**Sources:**
- [IEEE Spectrum: AI coding assistants failing in insidious ways](https://spectrum.ieee.org/ai-coding-degrades)

---

### Pitfall 3: ZLE Widget State Corruption

**What goes wrong:** ZLE widget calls `zle` commands when ZLE is not active (in subshells, async callbacks, trap handlers), causing "widgets can only be called when ZLE is active" errors and corrupted terminal state.

**Why it happens:** ZLE is stateful. Custom widgets that spawn subprocesses, use async callbacks, or handle signals don't realize ZLE context doesn't propagate.

**Consequences:**
- Widget silently fails, user input lost
- Terminal requires reset (`reset` or new session)
- Race conditions between async Rust code and ZLE state
- CUTBUFFER corruption (previous command leaks into new invocations)

**Prevention:**
1. **Check ZLE active state** before any `zle` call: `zle && zle <widget>`
2. **Never call ZLE from subshells** — communicate via temp files or environment variables instead.
3. **Handle async carefully** — Rust process should communicate results to zsh synchronously, not via callbacks.
4. **Reset CUTBUFFER** at widget start to prevent leakage.
5. **Use `.widget-name`** prefix when explicitly calling built-in widgets to avoid override conflicts.
6. **Test widget registration** — ensure `zle -N` is called before `bindkey`.

**Detection:**
- "widgets can only be called when ZLE is active" errors in logs
- Terminal state corruption after using widget
- Inconsistent behavior between interactive and scripted use

**Relevant phase:** Phase 1 (Terminal Integration)

**Sources:**
- [zsh mailing list: widgets only active in ZLE](https://www.zsh.org/mla/users/2006/msg01266.html)
- [powerlevel10k async ZLE issues](https://github.com/romkatv/powerlevel10k/issues/631)
- [zsh-autosuggestions unhandled widget](https://github.com/zsh-users/zsh-autosuggestions/issues/787)
- [zsh ZLE documentation](https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html)
- [sgeb.io: ZLE custom widgets guide](https://sgeb.io/posts/zsh-zle-custom-widgets/)

---

### Pitfall 4: Blocking Async Runtime with Sync Code

**What goes wrong:** rusqlite or other synchronous code runs on tokio worker threads, blocking the async runtime. Under load, all worker threads starve and the application hangs.

**Why it happens:** rusqlite's `Connection` is not `Sync`. Developers wrap sync calls in async functions thinking it "works" — tests pass, but production load causes thread starvation.

**Consequences:**
- Application freezes under concurrent requests
- Streaming responses stop mid-stream
- Timeout errors that are hard to diagnose
- Emergency rollbacks after deployment

**Prevention:**
1. **Use `tokio-rusqlite`** — spawns dedicated thread per connection, communicates via channels.
2. **Never put sync I/O in async context** without `spawn_blocking` or dedicated thread.
3. **Validate with load testing** — "async Rust compiles foot-guns that only fire under load."
4. **Monitor tokio runtime** — use tokio-console to detect blocked workers.

**Detection:**
- Application hangs under concurrent use
- Streaming stops mid-response
- `tokio::spawn` with rusqlite in call chain

**Relevant phase:** Phase 2 (Storage Layer) and Phase 4 (Multi-Provider Streaming)

**Sources:**
- [Qovery: Common mistakes with Rust async](https://www.qovery.com/blog/common-mistakes-with-rust-async)
- [Medium: 7 async Rust mistakes in production](https://ritik-chopra28.medium.com/async-rust-in-production-the-7-mistakes-that-cost-us-2-weeks-of-debugging-63699587a878)
- [tokio-rusqlite documentation](https://docs.rs/tokio-rusqlite/latest/tokio_rusqlite/)
- [rusqlite tokio::spawn issue](https://github.com/rusqlite/rusqlite/issues/1013)

---

### Pitfall 5: Provider Abstraction Leakage

**What goes wrong:** Provider abstraction assumes all APIs work the same way. When they don't (different message formats, streaming protocols, error codes, rate limiting), the abstraction leaks and requires provider-specific code paths scattered throughout.

**Why it happens:** Teams design abstraction based on one provider (usually OpenAI), then discover Anthropic has different message sequencing, Ollama has different streaming, and the "unified" interface can't express real differences.

**Consequences:**
- Provider-specific `if` statements throughout codebase
- Features that work on one provider, break on others
- Brittle code that breaks when any provider updates API
- Inability to use provider-specific features

**Prevention:**
1. **Design abstraction from multiple providers simultaneously** — read OpenAI, Anthropic, AND Ollama docs before defining trait.
2. **Accept that abstraction will be leaky** — design for it with provider-specific extension points.
3. **Normalize at boundary, not throughout** — transform to internal format at provider layer, not in business logic.
4. **Test against all providers** — not just mocks of your abstraction.
5. **Handle message sequence requirements** — some providers reject deviation from System -> User -> Assistant -> Tool -> Assistant.

**Detection:**
- Provider-specific conditionals outside provider module
- Features that work on OpenAI but fail on Anthropic
- "TODO: handle for other providers" comments

**Relevant phase:** Phase 4 (Provider Abstraction) — design before implementation

**Sources:**
- [Kilo Code provider architecture](https://deepwiki.com/Kilo-Org/kilocode/2.6-model-context-protocol-integration)
- [Vercel AI SDK provider foundations](https://ai-sdk.dev/docs/foundations/providers-and-models)
- [supermemory: API interoperability challenges](https://supermemory.ai/blog/we-solved-ai-api-interoperability/)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or degraded user experience.

---

### Pitfall 6: SQLite Busy/Lock Errors

**What goes wrong:** "Database is locked" errors appear under concurrent access. Users lose conversation history or experience hangs.

**Why it happens:** SQLite uses global write lock. Without busy timeout, concurrent writes fail immediately. Read-to-write transaction upgrades fail even with busy timeout.

**Consequences:**
- Lost conversation data
- User-visible errors
- Data corruption if not handled atomically

**Prevention:**
1. **Always set `PRAGMA busy_timeout=5000`** (or higher) on every connection.
2. **Use `BEGIN IMMEDIATE`** for write transactions to acquire lock upfront.
3. **Keep transactions short** — one statement = one transaction where possible.
4. **Avoid read-then-write patterns** — they can't upgrade locks gracefully.
5. **Implement application-level retry** — SQLite returns SQLITE_BUSY to avoid deadlock; retry is caller's responsibility.
6. **Single writer pattern** — route all writes through single connection if concurrent access needed.

**Detection:**
- "database is locked" errors in logs
- Intermittent data loss
- Operations that work in testing but fail in production

**Relevant phase:** Phase 2 (Storage Layer)

**Sources:**
- [SQLite locking v3 documentation](https://sqlite.org/lockingv3.html)
- [SQLite concurrent writes and locks](https://tenthousandmeters.com/blog/sqlite-concurrent-writes-and-database-is-locked-errors/)
- [SQLite busy handler](https://sqlite.org/c3ref/busy_handler.html)

---

### Pitfall 7: Streaming Response Interruption

**What goes wrong:** Network interruption, timeout, or error mid-stream leaves user with partial response. No recovery mechanism, user must re-ask.

**Why it happens:** Streaming responses assume stable connections. Teams handle stream start but not stream interruption. Partial responses are discarded.

**Consequences:**
- User loses context of long responses
- Repeated API costs for re-requesting
- Poor UX for slow or interrupted connections

**Prevention:**
1. **Persist partial responses** — save as they stream, not only on completion.
2. **Implement stream resumption** where provider supports it (or fake it with conversation history).
3. **Show clear error state** — "Response interrupted, showing partial result" not silent failure.
4. **Detect stalled streams** — timeout if no data for N seconds.
5. **Handle mid-stream errors gracefully** — show what was received, offer retry.

**Detection:**
- No partial response visible after interruption
- Stream processing assumes happy path only
- No timeout handling for slow streams

**Relevant phase:** Phase 4 (Streaming)

**Sources:**
- [AI SDK: Resume streams](https://ai-sdk.dev/docs/ai-sdk-ui/chatbot-resume-streams)
- [OpenAI streaming guide](https://platform.openai.com/docs/guides/streaming-responses)
- [Perplexity streaming responses](https://docs.perplexity.ai/guides/streaming-responses)

---

### Pitfall 8: File Operation Scope Creep

**What goes wrong:** AI reads/writes files outside intended scope. User asks about one file, AI reads their entire home directory. AI writes to system files or other projects.

**Why it happens:** No clear boundary on what AI can access. "Current directory" expands to parent directories. Symbolic links escape restrictions.

**Consequences:**
- Privacy violations (reading unrelated files)
- Cross-project contamination
- Accidental modification of system files
- Security boundary violations

**Prevention:**
1. **Explicit scope at session start** — AI operates within CWD and subdirectories only.
2. **Resolve symlinks before checking boundaries** — don't let symlinks escape scope.
3. **Blocklist sensitive paths** — `.git/`, `..`, absolute paths outside scope, dotfiles by default.
4. **Log all file access** — audit trail for debugging and security review.
5. **Confirm directory traversal** — if AI wants to read `../`, require explicit approval.

**Detection:**
- No path validation in file operation code
- Symlinks not resolved before boundary check
- No logging of file access

**Relevant phase:** Phase 3 (File Operations)

---

### Pitfall 9: Conversation Context Explosion

**What goes wrong:** Conversation history grows unbounded. Long sessions hit context limits, causing truncation or API failures. Each message includes full history, making requests slow and expensive.

**Why it happens:** Simple implementation appends all messages. No summarization, no sliding window, no context management.

**Consequences:**
- API failures on long sessions
- Exponentially increasing costs per message
- Slow response times
- Important context truncated unpredictably

**Prevention:**
1. **Implement sliding window** — keep last N messages, summarize earlier context.
2. **Track token count** — know when approaching limit, proactively summarize.
3. **Separate conversation storage from context window** — store all in SQLite, send subset to API.
4. **System message budget** — reserve tokens for system prompt, adjust history accordingly.
5. **User visibility** — show when context is being truncated.

**Detection:**
- No token counting in conversation handling
- All stored messages sent to API
- API errors mentioning context length

**Relevant phase:** Phase 2 (Conversation Management)

---

### Pitfall 10: In-Memory SQLite Pool Confusion

**What goes wrong:** Team creates connection pool with in-memory SQLite. Each connection has separate, isolated database. Data appears lost randomly.

**Why it happens:** In-memory SQLite databases are per-connection. Pool rotates connections. Data written to connection A is invisible to connection B.

**Consequences:**
- Conversations vanish unpredictably
- Test data missing in assertions
- Bizarre "works sometimes" bugs

**Prevention:**
1. **Never pool in-memory SQLite** — use single connection or file-based database.
2. **For testing, use temp file** — not `:memory:`.
3. **If you must share in-memory, use `file::memory:?cache=shared`** — but understand implications.
4. **Document this constraint** prominently in storage layer.

**Detection:**
- `Connection::open_in_memory` with pooling code nearby
- Data that "disappears" between operations
- Tests that pass individually but fail together

**Relevant phase:** Phase 2 (Storage Layer)

**Sources:**
- [bb8-rusqlite: in-memory warning](https://lib.rs/crates/bb8-rusqlite)
- [tokio-rusqlite: in-memory limitations](https://docs.rs/tokio-rusqlite/latest/tokio_rusqlite/)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major rework.

---

### Pitfall 11: CUTBUFFER Persistence

**What goes wrong:** ZLE CUTBUFFER persists across widget invocations. User invokes widget on empty line, gets previous widget's output inserted.

**Why it happens:** ZLE doesn't reset CUTBUFFER between invocations. Widget modifies CUTBUFFER but doesn't reset it first.

**Prevention:** Reset CUTBUFFER as first operation in widget.

**Relevant phase:** Phase 1 (Terminal Integration)

---

### Pitfall 12: Tokio Multi-Threaded Requirement

**What goes wrong:** Using `block_in_place` with single-threaded runtime panics.

**Why it happens:** tokio-rusqlite and similar wrappers require multi-threaded runtime for `block_in_place`.

**Prevention:** Use `#[tokio::main]` not `#[tokio::main(flavor = "current_thread")]`.

**Relevant phase:** Phase 2 (Runtime Configuration)

---

### Pitfall 13: Nested Result Types

**What goes wrong:** `tokio-rusqlite::Connection::call` returns `Result<Result<T>>`. Code handles outer Result, ignores inner.

**Why it happens:** Outer Result is channel error, inner is SQLite error. Easy to `.unwrap()` the outer and miss inner.

**Prevention:** Always flatten or handle both Result layers explicitly.

**Relevant phase:** Phase 2 (Storage Layer)

---

### Pitfall 14: Stream Backpressure Neglect

**What goes wrong:** Fast AI streaming output without backpressure overwhelms terminal rendering or memory.

**Why it happens:** Streams consumed as fast as produced without `.buffered()` or throttling.

**Prevention:** Use `.buffered(n)` or terminal rendering throttle for display.

**Relevant phase:** Phase 4 (Streaming Display)

---

### Pitfall 15: Missing API Key Validation

**What goes wrong:** Invalid API key detected only after first API call. User configures provider, tries to use it, gets cryptic auth error.

**Why it happens:** Config validation checks key format but not actual validity.

**Prevention:** Implement `validate_config()` that makes lightweight API call to verify credentials.

**Relevant phase:** Phase 4 (Provider Configuration)

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation | Phase |
|-------------|----------------|------------|-------|
| ZLE Widget | State corruption (#3) | Check ZLE active, no subshell calls | 1 |
| Command Execution | Auto-execute without sandboxing (#1) | OS-level sandbox, default to ask | 1 |
| SQLite Storage | Async blocking (#4), Busy locks (#6) | tokio-rusqlite, busy_timeout | 2 |
| Conversation | Context explosion (#9) | Token tracking, sliding window | 2 |
| File Operations | Scope creep (#8), Silent failure (#2) | Path validation, diff preview | 3 |
| Provider Abstraction | Leaky abstraction (#5) | Multi-provider design upfront | 4 |
| Streaming | Interruption (#7), Backpressure (#14) | Partial persist, buffering | 4 |

---

## Confidence Assessment

| Pitfall Category | Confidence | Reasoning |
|------------------|------------|-----------|
| Security (1, 2) | HIGH | Documented real-world incidents, security research |
| ZLE (3, 11) | HIGH | Official zsh documentation, multiple issue trackers |
| Async/SQLite (4, 6, 10, 12, 13) | HIGH | tokio-rusqlite docs, rusqlite issues |
| Provider abstraction (5) | MEDIUM | Industry patterns, but Cherry2K-specific design unknown |
| Streaming (7, 14) | MEDIUM | AI SDK docs, general streaming patterns |
| File operations (8) | MEDIUM | Security best practices, not Cherry2K-specific research |
| Context (9) | MEDIUM | General LLM patterns, provider-specific limits vary |

---

## Summary for Roadmap

**Must address in Phase 1 (foundational):**
- Sandboxing architecture for command execution
- ZLE widget state management

**Must address in Phase 2 (before production):**
- tokio-rusqlite for async SQLite
- Busy timeout and single-writer pattern
- Context window management

**Must address before multi-provider (Phase 4):**
- Design abstraction with all three providers in mind
- Streaming interruption handling

**Can iterate on (lower risk):**
- Stream backpressure
- API key pre-validation
- CUTBUFFER management

---

*Research complete. Recommend deep-dive research flag for Phase 4 (Provider Abstraction) to verify actual API differences between OpenAI, Anthropic, and Ollama before trait design.*