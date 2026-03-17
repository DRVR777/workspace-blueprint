# MANIFEST — programs/api

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-programs-api |
| `type` | program |
| `depth` | 3 |
| `parent` | project-alpha/programs/ |
| `status` | scaffold |

## What I Am
[One sentence: what this program does and what it exposes.]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CLAUDE.md | file | Map of this program's internal structure |
| CONTEXT.md | file | Task routing within this program |
| src/ | folder | Source code |
| tests/ | folder | Test suites |
| docs/ | folder | API documentation, local standards |

## What I Need From Parent
- Relevant contracts from `../../shared/contracts/`
- Architecture decisions from `../../_planning/adr/`
- Technical standards from `../../_planning/standards.md` (when it exists)

## What I Return To Parent
- Implemented contracts (the code that fulfills `shared/contracts/`)
- New contract proposals → surface as gap if `shared/contracts/` doesn't have it yet

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Writing a new feature | `CONTEXT.md` for task routing |
| Checking what this program exposes | `docs/` or `../../shared/contracts/` |
| Architecture question | `../../_planning/adr/` |

## Gap Status
See `../../_meta/gaps/CONTEXT.md`
