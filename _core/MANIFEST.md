# MANIFEST — _core/

## Envelope
| Field | Value |
|-------|-------|
| `id` | root-core |
| `type` | conventions |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
Workspace-wide architectural conventions. Single source of truth for all patterns
inherited from MWP v1, MWP v2, and ALWS. All templates and agents reference this
folder rather than rediscovering patterns independently.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONVENTIONS.md | file | Authoritative pattern reference — all 14 architectural patterns |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Look up an architectural pattern | CONVENTIONS.md |
| Add a new pattern | CONVENTIONS.md — append to appropriate section, add to index |
| Find the template to clone | `programs/_template/` |
