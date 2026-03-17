# MANIFEST — programs/api

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-programs-api |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/project-alpha/programs/ |
| `status` | scaffold |

## What I Am
[One sentence: what this program does and what it exposes.]

## External Dependencies
| Depends On | What | Contract Location |
|------------|------|-------------------|
| none | — | — |

## What I Produce
Implemented contracts (the code that fulfills `../../shared/contracts/`)

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Write a new feature | CONTEXT.md → load relevant contract from `../../shared/contracts/` |
| Architecture question | `../../_planning/CONTEXT.md` |
| Define a new inter-program interface | `../../shared/contracts/` |

## Gap Status
See `../../_meta/gaps/CONTEXT.md` — check for blocking gaps before building.
