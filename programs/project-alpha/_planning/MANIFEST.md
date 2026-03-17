# MANIFEST — project-alpha/_planning/

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-planning |
| `type` | planning |
| `depth` | 3 |
| `parent` | programs/project-alpha/ |
| `status` | active |

## What I Am
Where decisions get made before code gets written.
Contains ADRs, system design, and the build roadmap.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Task router + ADR template |
| adr/ | folder | Architecture Decision Records |
| roadmap.md | file | Build order and program status |
| system-design/ | folder | Data flows and service boundaries (create when needed) |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Make a decision | CONTEXT.md |
| Check build order | roadmap.md |
| Read existing decisions | adr/ |
