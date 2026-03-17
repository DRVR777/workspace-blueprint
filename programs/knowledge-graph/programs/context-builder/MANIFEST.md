# MANIFEST — knowledge-graph/programs/context-builder/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-context-builder |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/knowledge-graph/programs/ |
| `status` | specced |

## What I Am
Executes embedded prompts. Watches ticker.log for new reads, reads neighbors
via file-selector, and writes ctx-NNNN.md — the self-description of each document.

## External Dependencies
| Depends On | What | Contract |
|------------|------|----------|
| data-store | Data/ files and their content | shared/contracts/file-record.md |
| file-selector | Neighbor reading during embedded prompt execution | shared/contracts/file-record.md |
| indexer | Populated neighbor lists in file metadata | shared/contracts/file-record.md |

## What I Produce
- `Data/ctx-[NNNN].md` — context file describing each read document
- Appends `context_built` entries to Data/ticker.log
