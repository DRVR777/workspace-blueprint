# MANIFEST — knowledge-graph/shared/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-shared |
| `type` | contracts |
| `depth` | 3 |
| `parent` | programs/knowledge-graph/ |
| `status` | active |

## What I Am
Hard boundaries between programs. Programs never import from each other directly.
All cross-program data shapes are defined here.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| contracts/file-record.md | file | defined | Shape of a Data/ file as a structured object |
| contracts/ticker-entry.md | file | defined | Shape of one line in ticker.log |
| contracts/context-file.md | file | defined | Shape of ctx-NNNN.md output |

## Contract Dependencies

| Producer | Contract | Consumers |
|----------|----------|-----------|
| data-store | file-record | file-selector, indexer, context-builder |
| file-selector | ticker-entry | context-builder |
| context-builder | context-file | AI (direct read) |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Find a file/data shape | contracts/file-record.md |
| Find a ticker entry shape | contracts/ticker-entry.md |
| Find a context file shape | contracts/context-file.md |
