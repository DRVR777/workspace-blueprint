# MANIFEST — {{PROJECT_NAME}}

## Envelope
| Field | Value |
|-------|-------|
| `id` | {{PROJECT_NAME}} |
| `type` | project |
| `depth` | 2 |
| `parent` | programs/ |
| `version` | 0.1.0 |
| `status` | scaffold |
| `created` | {{CREATED}} |

## What I Am
[Replace with: one sentence — what problem this project solves and what it produces.]

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| CLAUDE.md | file | active | Depth-1 map: program names and one-line purposes |
| CONTEXT.md | file | active | Task router for this project |
| _planning/ | folder | active | Architecture decisions before code |
| _meta/ | folder | active | Project-internal gap registry |
| shared/ | folder | active | Contracts and types between programs |
| programs/ | folder | active | All runnable/deployable components |

## What I Need From Parent
Nothing — self-contained. Cross-project dependencies go in `{root}/_meta/contracts/`.

## What I Return To Parent
[The project's output: an API, a CLI, a deployed service, etc.]

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Make an architectural decision | _planning/CONTEXT.md |
| Work on a specific program | programs/[name]/CONTEXT.md |
| Define or update a contract | shared/MANIFEST.md |
| Log a project-internal gap | _meta/gaps/pending.txt |
| Log a cross-project gap | {root}/_meta/gaps/pending.txt |
| Orient with no prior context | This MANIFEST, then CLAUDE.md |

## Gap Status
Scaffolded from PRD. See _meta/gaps/CONTEXT.md for open gaps.
