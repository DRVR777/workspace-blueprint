# ADR-006: Global Ticker — Append-Only Text File

Status: accepted
Date: 2026-03-13
Source: User specified "adding to the global ticker" — file format inferred

## Decision
The global ticker is `Data/ticker.log` — a plain text file with one entry per line,
append-only. No database. No server. No locking protocol for initial implementation.

## Rationale
- Consistent with workspace principle: folder structure IS the infrastructure
- Plain text means any agent, script, or human can read/write without tooling
- Append-only makes it a natural log — no data is ever lost
- Atomic appends are safe for sequential use (one agent at a time)
- The file is part of the Data/ folder — same location as what it describes

## Format
```
[ISO-8601] | [file_number] | [session_id] | [reason]
```

## Consequences
- No concurrent write safety — if two agents write simultaneously, entries may interleave
  (acceptable for initial implementation; upgrade to SQLite if concurrency is needed)
- ticker.log grows indefinitely — add rotation (ticker-YYYYMM.log) if it becomes large
- The full navigation history is preserved — this is a feature, not a bug
- graph_builder.py can materialize ticker.log into graph.json at any time

## Upgrade path
If concurrent access is needed: swap ticker.log for a SQLite database.
The ticker-entry.md contract shape stays identical — only the write/read mechanism changes.
