# MANIFEST — project-beta [DEPRECATED — use programs/project-beta/]

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-beta |
| `type` | project |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | scaffold |

## What I Am
[One sentence: what this project is and what it produces.]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CLAUDE.md | file | Always-loaded map of this project's internals |
| CONTEXT.md | file | Routes tasks to the right program or planning layer |
| MANIFEST.md | file | This file |
| _meta/ | folder | Project-internal gap registry |
| _planning/ | folder | Architecture decisions, system design, roadmap |
| shared/ | folder | Contracts and types shared between this project's programs |
| programs/ | folder | The actual programs (services, apps, tools) |

## What I Need From Parent
- Root `_meta/` for cross-project gap escalation
- Root `CLAUDE.md` for workspace-level naming rules

## What I Give To Children
- Architectural decisions from `_planning/adr/`
- Shared contracts from `shared/contracts/`

## What I Return To Parent
- Deliverables documented in `_planning/roadmap.md`
- Cross-project dependencies escalated to root `_meta/gaps/`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Planning or architecture work | `_planning/CONTEXT.md` |
| Building a specific program | `programs/[name]/CONTEXT.md` |
| Checking shared contracts | `shared/` |
| Logging a project-internal gap | `_meta/gaps/pending.txt` |
| Logging a cross-project gap | root `_meta/gaps/pending.txt` |

## Gap Status
See `_meta/gaps/CONTEXT.md`
