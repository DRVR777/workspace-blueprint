# MANIFEST — knowledge-graph/programs/indexer/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-indexer |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/knowledge-graph/programs/ |
| `status` | specced |

## What I Am
Computes 5D semantic vectors for Data/ files using heuristic rules.
Finds k nearest neighbors per file. Updates file metadata in place.

## External Dependencies
| Depends On | What | Contract |
|------------|------|----------|
| data-store | Data/ files to index | shared/contracts/file-record.md |

## What I Produce
- Updated `vector` field in each file's metadata
- Updated `neighbors` field in each file's metadata
- Updated `last_indexed` field in each file's metadata
