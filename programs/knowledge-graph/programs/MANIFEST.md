# MANIFEST — knowledge-graph/programs/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-programs |
| `type` | programs |
| `depth` | 3 |
| `parent` | programs/knowledge-graph/ |
| `status` | active |

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| data-store/ | folder | specced | Creates and manages Data/ folder and files |
| file-selector/ | folder | specced | AI tool for reading files; logs to ticker |
| indexer/ | folder | specced | Computes 5D vectors and neighbor lists |
| context-builder/ | folder | specced | Runs embedded prompts; writes ctx files |

## Build Order
data-store → file-selector → indexer → context-builder

Each program depends on the ones above it in the build order.
Do not build a program until its dependencies are complete.
