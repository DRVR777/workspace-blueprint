# ADR-005: Context-Builder Trigger — On-Read (not Batch)

Status: accepted — 2026-03-14
Date: 2026-03-13

## Decision
context-builder runs immediately after each file read (triggered by ticker.log entry),
not as a periodic batch sweep. The embedded prompt fires on read.

## Context
Two options were considered:
- **On-read**: context-builder watches ticker.log; when a new entry appears, it runs
  the embedded prompt for that file. Context file is up-to-date within seconds of reading.
- **Batch**: context-builder runs periodically (e.g., every 10 minutes or at session end).
  Context files may be stale during a session.

## Rationale
On-read chosen because:
- The AI reading a file immediately benefits from an up-to-date context file
- If the AI reads file0042, then reads its neighbors, it can immediately read ctx-0042.md
  to get a synthesized view — this is only useful if ctx-0042.md is written immediately
- Batch mode would require the AI to wait or re-read — breaks the navigation flow

## Consequences
- context-builder must implement ticker.log watching (polling the file for new lines)
- Each session may trigger many context-builder runs — performance concern at scale
- Mitigation: skip if ctx file is < N minutes old (configurable staleness threshold, default 5 min)
- If context-builder is unavailable, the system degrades gracefully (navigation still works, ctx files are stale)

## Needs human validation
If the system feels slow due to context-builder blocking navigation, switch to batch mode.
