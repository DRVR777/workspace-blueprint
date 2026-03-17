# MANIFEST — project-alpha/shared

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-shared |
| `type` | contracts |
| `depth` | 2 |
| `parent` | project-alpha/ |
| `status` | active |

## What I Am
The source of truth for every boundary between programs inside project-alpha.
If two programs need to agree on a data shape, an API response, or an event format
— the definition lives here, not inside either program.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| `contracts/` | folder | API schemas, event shapes, response formats — one file per boundary |
| `types/` | folder | Shared type definitions referenced by multiple programs |

## The Rule
If a program's code defines a type or shape that another program consumes —
it belongs here instead. Programs may import from `shared/`. They do not
import from each other.

## What I Need From Parent
Architectural decisions from `_planning/adr/` that define what contracts exist.

## What I Return To Parent
Contracts that programs build against. When a contract changes, all programs
that depend on it must be updated. This is tracked as a `missing_bridge` gap
if any consumer is not updated.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Defining a new API boundary | Create `contracts/[name].json` or `contracts/[name].md` |
| Defining a shared type | Create `types/[name].ts` or `types/[name].json` |
| Checking what contracts exist | Read file list in `contracts/` |
