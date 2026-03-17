# MANIFEST — project-beta/_planning/

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-beta-planning |
| `type` | planning |
| `depth` | 3 |
| `parent` | programs/project-beta/ |
| `status` | active |

## What I Am
Where decisions get made before code gets written.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Task router + ADR template |
| adr/ | folder | Architecture Decision Records |
| roadmap.md | file | Build order and program status |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Make a decision | CONTEXT.md |
| Check build order | roadmap.md |
| Read existing decisions | adr/ |
