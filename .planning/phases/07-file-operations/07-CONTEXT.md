# Phase 7: File Operations - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable AI to read, write, and edit files with user approval. AI can detect file references from conversation context, propose changes with diff preview, and execute writes only after explicit confirmation. All operations respect project scope boundaries.

</domain>

<decisions>
## Implementation Decisions

### File Reading Triggers
- Smart detection: AI infers file references from context (e.g., "fix the bug in main" → reads main.rs)
- Auto-expand imports: Read files imported/used by the target file for fuller context
- Brief notice: Show "Reading: file.rs" before response (not silent, not verbose)
- Smart excerpt for large files: Read relevant sections based on query rather than truncating

### Diff Preview Format
- Unified diff: Standard git-style with +/- lines (familiar to developers)
- 3 context lines: Git default, compact display
- Retro theme colors: Match the green terminal aesthetic from Phase 4 (not standard red/green)
- New file preview: Show file purpose + first 20 lines, 'y' to see full content

### Approval Flow
- All-or-nothing: Single [y/n] for entire changeset (not per-file)
- Edit option: [y/n/e] where 'e' opens proposed changes in $EDITOR
- Brief confirmation: "Written: file.rs" one-liner on success
- Offer retry on failure: "Write failed: [reason]. Retry with sudo? [y/n]"

### Directory Scope Rules
- Git root as boundary: Use .git directory to determine project scope
- Symlinks: Follow only if target is within project root
- Configurable blocklist: Default blocked patterns (.env, .ssh/*) with config override
- Out-of-scope handling: Warn and confirm ("This is outside project. Proceed anyway? [y/n]")

### Claude's Discretion
- Exact file detection heuristics
- Smart excerpt algorithm for large files
- Retry strategies for failed writes
- Default blocklist patterns

</decisions>

<specifics>
## Specific Ideas

- Match Phase 4's retro green terminal aesthetic for diff colors
- Edit option [e] mirrors the command confirmation flow from Phase 6
- Scope detection aligns with how developers think about project boundaries (git root)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-file-operations*
*Context gathered: 2026-01-31*
