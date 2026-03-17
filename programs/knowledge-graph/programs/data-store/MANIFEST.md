# MANIFEST — knowledge-graph/programs/data-store/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-data-store |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/knowledge-graph/programs/ |
| `status` | specced |

## What I Am
Manages the Data/ folder. Creates files with correct naming, format, and embedded prompts.
Single source of truth for file creation — nothing else creates files in Data/.

## External Dependencies
| Depends On | What | Contract |
|------------|------|----------|
| (none) | — | — |

## What I Produce
- `Data/file[NNNN].md` — new documents with correct format
- `Data/index.md` — directory of all files with one-line summaries
- `Data/ticker.log` — empty log, bootstrapped on first run
