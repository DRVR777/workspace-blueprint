# MANIFEST — project-alpha

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha |
| `type` | project |
| `depth` | 2 |
| `parent` | programs/ |
| `version` | 0.1.0 |
| `status` | scaffold |

## What I Am
[Replace with your project name and one sentence — what problem it solves and what it produces.]

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| CLAUDE.md | file | active | Depth-1 map of this project's programs |
| CONTEXT.md | file | active | Task router |
| _planning/ | folder | active | Architecture decisions before code |
| _meta/ | folder | active | Project-internal gap registry |
| shared/ | folder | active | Contracts and types between programs |
| programs/ | folder | active | All runnable/deployable components |
| programs/api/ | folder | scaffold | REST API — authentication, data, business logic |
| programs/frontend/ | folder | scaffold | Web client — user interface and interactions |

## What I Need From Parent
Nothing — self-contained. Cross-project deps in {root}/_meta/contracts/.

## What I Return To Parent
[The project's output: an API, a CLI, a deployed service, etc.]

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Make an architectural decision | _planning/CONTEXT.md |
| Work inside a specific program | programs/[name]/CONTEXT.md |
| Define or update a contract | shared/MANIFEST.md |
| Log a gap | _meta/gaps/pending.txt |
| Orient with no prior context | This MANIFEST, then CLAUDE.md |

## Gap Status
See _meta/gaps/CONTEXT.md — check for blocking gaps before building.
