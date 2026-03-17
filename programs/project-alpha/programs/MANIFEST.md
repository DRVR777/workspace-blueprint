# MANIFEST — project-alpha/programs/

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-programs |
| `type` | programs-container |
| `depth` | 3 |
| `parent` | programs/project-alpha/ |
| `status` | active |

## What I Am
Container for all runnable/deployable components of project-alpha.
Each subfolder is a program with its own MANIFEST, CONTEXT, src/, and tests/.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| api/ | folder | scaffold | REST API — authentication, data, business logic |
| frontend/ | folder | scaffold | Web client — user interface and interactions |

## Build Order
api/ builds first (no dependencies).
frontend/ builds second (consumes api/ contracts).
See `../‌_planning/roadmap.md` for the full dependency graph.

## Rules
Programs in this folder never import from each other.
All shared shapes go through `../shared/contracts/`.
