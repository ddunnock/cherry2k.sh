# Phase 3: Storage and Session Continuity - Context

**Gathered:** 2026-01-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable conversation context that persists across CLI invocations. Users can have multi-turn conversations that survive terminal closure and resume previous sessions. Context window management prevents token overflow.

**In scope:**
- SQLite storage for conversation history
- Session creation, identification, and resume
- Context window management with summarization
- Basic session lifecycle commands (resume, new, clear, list)

**Out of scope:**
- Cross-device sync
- Export/import
- Session sharing

</domain>

<decisions>
## Implementation Decisions

### Session identity
- **Hybrid creation**: Auto-continue current session on `cherry2k chat`, but `cherry2k new` forces fresh session
- **Auto-generated IDs**: Timestamp-based (e.g., 2026-01-30-1423) — no user input needed
- **Per-directory scope**: Each working directory has its own session history
- **4-hour idle timeout**: Auto-create new session if last message was >4 hours ago

### Context window management
- **Summarize oldest**: When history exceeds token limit, AI summarizes older messages into compact context block
- **Subtle indicator**: Show "(context summarized)" note when summarization occurs
- **16K token budget**: Standard allocation for conversation history
- **Separate system budget**: System messages don't count against the 16K conversation limit

### Resume experience
- **`cherry2k resume`** (no args): Automatically continue most recent session in current directory
- **`cherry2k resume <id>`**: Resume specific session by timestamp ID
- **`cherry2k resume --list`**: Simple table showing ID, timestamp, first message preview
- **Silent resume**: No history display on resume — user knows what they were doing

### Data lifecycle
- **30-day expiration**: Auto-delete sessions older than 30 days
- **Soft storage limit**: Warn at 100MB, suggest cleanup (don't auto-delete)
- **`cherry2k clear`**: Delete all sessions with y/n confirmation
- **XDG data dir**: Store database at ~/.local/share/cherry2k/sessions.db

### Claude's Discretion
- SQLite schema design
- Exact summarization prompt
- Token counting implementation
- Migration strategy

</decisions>

<specifics>
## Specific Ideas

No specific product references — open to standard approaches for CLI session management.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-storage-and-session-continuity*
*Context gathered: 2026-01-30*
