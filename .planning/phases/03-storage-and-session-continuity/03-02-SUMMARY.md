---
phase: 03-storage-and-session-continuity
plan: 02
subsystem: repository
tags: [sqlite, session, message, crud, async]

# Dependency graph
requires:
  - phase: 03-01
    provides: Database connection, schema with sessions/messages tables
provides:
  - Session repository with CRUD operations and 4-hour idle continuation
  - Message repository with transaction-safe saves and summarization support
  - 30-day session cleanup for retention policy
affects: [03-03, context-window, cli-integration]

# Tech tracking
tech-stack:
  added: []
  patterns: [repository pattern, timestamp-based IDs, atomic transactions]

key-files:
  created:
    - crates/storage/src/session.rs
    - crates/storage/src/message.rs
  modified:
    - crates/storage/src/lib.rs

key-decisions:
  - "Timestamp-based session IDs (YYYY-MM-DD-HHMM-SSS) for human readability and uniqueness"
  - "4-hour idle threshold for session continuation (matches typical work session)"
  - "Atomic transaction for save_message (message insert + session timestamp update)"
  - "Role stored as lowercase string, parsed on retrieval"

patterns-established:
  - "Repository functions take &Database and use db.call() for async operations"
  - "Session/Message structs separate from database row mapping"
  - "OptionalExtension trait for query_row().optional() pattern"

# Metrics
duration: 4min
completed: 2026-01-30
---

# Phase 03 Plan 02: Session and Message Repository Summary

**Repository layer for sessions and messages with async-safe database access and 4-hour idle continuation**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-30T23:26:30Z
- **Completed:** 2026-01-30T23:30:58Z
- **Tasks:** 2
- **Files created:** 2
- **Tests added:** 39 (21 session + 18 message)

## Accomplishments

- Session repository with full CRUD operations and per-directory scoping
- Automatic session continuation if last message within 4 hours
- 30-day retention policy with cleanup_old_sessions()
- Message repository with atomic saves (transaction for message + timestamp update)
- Summary message support for context window management (is_summary flag)
- Token count tracking for future context window optimization
- 39 comprehensive tests covering all repository operations

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement session repository** - `27c8dac` (feat)
2. **Task 2: Implement message repository** - `9471a97` (feat)

## Files Created/Modified

- `crates/storage/src/session.rs` - Session and SessionInfo structs, 8 repository functions
- `crates/storage/src/message.rs` - StoredMessage struct, 6 repository functions
- `crates/storage/src/lib.rs` - Module declarations and re-exports

## API Overview

### Session Functions

| Function | Purpose |
|----------|---------|
| `generate_session_id()` | Creates timestamp-based ID (YYYY-MM-DD-HHMM-SSS) |
| `create_session()` | Creates new session for working directory |
| `get_or_create_session()` | Returns active session or creates new (4-hour threshold) |
| `get_session()` | Retrieves session by ID |
| `list_sessions()` | Lists sessions for directory with first message preview |
| `update_session_timestamp()` | Updates last_message_at to now |
| `delete_session()` | Deletes session (cascades to messages) |
| `cleanup_old_sessions()` | Deletes sessions older than 30 days |

### Message Functions

| Function | Purpose |
|----------|---------|
| `save_message()` | Saves message with atomic session timestamp update |
| `save_summary()` | Saves summary message (is_summary=true, role=system) |
| `get_messages()` | Retrieves all messages for session (oldest first) |
| `get_messages_since()` | Retrieves messages after given timestamp |
| `count_messages()` | Returns message count for session |
| `delete_messages_before()` | Deletes old messages (for post-summarization cleanup) |

## Decisions Made

1. **Timestamp-based session IDs**: Format "YYYY-MM-DD-HHMM-SSS" provides human-readable, sortable, and unique identifiers. Milliseconds prevent collisions.

2. **4-hour idle threshold**: Sessions auto-continue if last message within 4 hours - matches typical work session length without creating too many sessions.

3. **Atomic save_message transaction**: Both message insert and session timestamp update happen in single transaction - prevents inconsistent state if either fails.

4. **Role string storage**: Store Role enum as lowercase string ("user", "assistant", "system") - matches JSON API formats and is human-readable in database.

## Deviations from Plan

None - plan executed exactly as written.

## Test Coverage

- **Session tests:** 21 tests covering ID generation, CRUD operations, 4-hour threshold, 30-day cleanup
- **Message tests:** 18 tests covering saves, retrieval, transaction atomicity, role parsing

## Next Phase Readiness

- Repository layer complete and tested
- Ready for Plan 03-03: Session Management Service
- Session/message CRUD operations available for CLI integration
- Token count field ready for context window management

---
*Phase: 03-storage-and-session-continuity*
*Completed: 2026-01-30*
