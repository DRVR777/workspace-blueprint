# MANIFEST — programs/frontend

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-programs-frontend |
| `type` | program |
| `depth` | 3 |
| `parent` | project-alpha/programs/ |
| `status` | scaffold |

## What I Am
[One sentence: what this program renders and who uses it.]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CLAUDE.md | file | Map of this program's internal structure |
| CONTEXT.md | file | Task routing within this program |
| src/ | folder | Source code |
| tests/ | folder | Test suites |
| docs/ | folder | Component registry, design reference |

## What I Need From Parent
- Contracts from `../../shared/contracts/` (shapes of API responses consumed)
- Architecture decisions from `../../_planning/adr/`

## What I Return To Parent
- Rendered interface that fulfills the project's user-facing requirements
- Discovered contract gaps → `../../_meta/gaps/pending.txt`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Building a new feature | `CONTEXT.md` |
| Checking API response shapes | `../../shared/contracts/` |
| Architecture question | `../../_planning/adr/` |

## Gap Status
See `../../_meta/gaps/CONTEXT.md`
