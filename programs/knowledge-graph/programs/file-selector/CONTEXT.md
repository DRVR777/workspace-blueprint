# programs/file-selector — Task Router

## What This Program Is
The AI's navigation tool. Reads files. Logs every read to ticker.log.
Can retrieve by file number or by 5D proximity query.
Implemented as a Claude tool (tool_use) — see ADR-008.

---

## Before Writing Any Code
Read ADR-008 (tool_use implementation).
Read shared/contracts/ticker-entry.md — this is what file-selector produces on every read.
Read shared/contracts/file-record.md — this is what file-selector returns.
data-store must exist before file-selector can be tested.

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Implement read-by-number | MANIFEST.md, shared/contracts/file-record.md | indexer, context-builder |
| Implement ticker write | shared/contracts/ticker-entry.md | vector specs |
| Implement proximity query | _planning/5d-vector-spec.md §Distance | data-store src/ |
| Define Claude tool schema | _planning/adr/008-file-selector-as-claude-tool.md | everything else |

---

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Read-by-number done | Call tool with "0001", show return value | approve / fix format |
| Ticker write done | Show ticker.log entry after read | approve |
| Tool schema done | Full JSON schema | approve / revise |

## Audit
Before committing to output/:
- [ ] Reading file0001.md produces correct return shape (matches file-record.md contract)
- [ ] ticker.log has exactly one new entry per read (no duplicates, no missing)
- [ ] access_count in file metadata increments on each read
- [ ] Proximity query returns k files sorted by ascending distance
- [ ] Reading a non-existent file returns structured error (not exception)

---

## Inputs
- `file_number` (string, 4-digit) OR `query_vector` (5 floats)
- `session_id` (string)
- `reason` (string)

## Outputs
- File content object (matches file-record.md contract)
- Side effect: new line in Data/ticker.log
