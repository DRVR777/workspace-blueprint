# MANIFEST — knowledge-graph/programs/file-selector/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-file-selector |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/knowledge-graph/programs/ |
| `status` | specced |

## What I Am
The AI's navigation tool. A Claude tool (tool_use) that reads files from Data/
and logs every read to ticker.log. The only way the AI accesses documents.

## External Dependencies
| Depends On | What | Contract |
|------------|------|----------|
| data-store | Data/ folder must exist and contain files | shared/contracts/file-record.md |

## What I Produce
- File content as structured object (file-record.md contract)
- New line in Data/ticker.log (ticker-entry.md contract)
- Updated access_count in file metadata
