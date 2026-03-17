# ADR Resolution Log

Tracks which research studies unblocked which assumption ADRs.
When a study concludes, update this log and the corresponding ADR file.

---

## Open Assumption ADRs

(none — all resolved 2026-03-14)

---

## How to Close an ADR

1. Study reaches `concluded` status
2. `findings/conclusion.md` contains a clear recommendation
3. Open the ADR file at `_planning/adr/[NNN]-[slug].md`
4. Change `Status: assumption` → `Status: accepted`
5. Update the Decision and Rationale sections with evidence from findings
6. Add a row to the "Closed ADRs" table below
7. Update the gap in `_meta/gaps/CONTEXT.md` to `closed`

---

## Closed ADRs

| ADR | Closed By | Date | Summary of Evidence |
|-----|-----------|------|-------------------|
| ADR-004 | reasoning | 2026-03-14 | k=5 matches 5D vector size, bounds reads at 6/hop, configurable per-file — no empirical study needed |
| ADR-005 | reasoning | 2026-03-14 | On-read wins: ctx must be fresh when AI navigates neighbors; batch = stale navigation |
| ADR-007 | policy | 2026-03-14 | Ticker is append-only, file numbers are permanent refs — deletion breaks log integrity |
| ADR-008 | reasoning | 2026-03-14 | tool_use for initial build; MCP is explicit named future migration path (simpler first) |

---

## ADR-007 — Policy Decision (no study required)

ADR-007 (no file deletion) does not require empirical research.
It is a data integrity decision: ticker.log contains permanent references to file numbers.
Deleting a file would break those references.

**Resolution:** Change ADR-007 status to `accepted` now.
Rationale: the ticker is append-only. Any file number ever appended to the ticker
must remain readable forever. Deletion = broken references. Deprecation = safe.

Action: Update `_planning/adr/007-no-deletion.md` status to `accepted`.
