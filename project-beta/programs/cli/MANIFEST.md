# MANIFEST — programs/cli

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-beta-programs-cli |
| `type` | program |
| `depth` | 3 |
| `parent` | project-beta/programs/ |
| `status` | scaffold |

## What I Am
[One sentence: what this CLI tool does and who runs it.]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CLAUDE.md | file | Map of this program's internal structure |
| CONTEXT.md | file | Task routing within this program |
| src/ | folder | Source code |
| tests/ | folder | Test suites |
| docs/ | folder | Usage documentation, command reference |

## What I Need From Parent
- Architecture decisions from `../../_planning/adr/`

## What I Return To Parent
- Built CLI binary / executable
- No cross-program dependencies (single-program project)

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Adding a command | `CONTEXT.md` |
| Architecture question | `../../_planning/adr/` |

## Gap Status
See `../../_meta/gaps/CONTEXT.md`
