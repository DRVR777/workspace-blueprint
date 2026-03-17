# MANIFEST — programs/frontend

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-programs-frontend |
| `type` | program |
| `depth` | 4 |
| `parent` | programs/project-alpha/programs/ |
| `status` | scaffold |

## What I Am
[One sentence: what this program renders and who uses it.]

## External Dependencies
| Depends On | What | Contract Location |
|------------|------|-------------------|
| programs/api | API response shapes | `../../shared/contracts/` |

## What I Produce
Rendered interface that fulfills the project's user-facing requirements.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Build a new feature | CONTEXT.md |
| Check API response shapes | `../../shared/contracts/` |
| Architecture question | `../../_planning/CONTEXT.md` |

## Gap Status
See `../../_meta/gaps/CONTEXT.md` — check for blocking gaps before building.
Note: programs/api/ must be at `specced` before this program builds.
