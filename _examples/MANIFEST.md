# MANIFEST — _examples/

## Envelope
| Field | Value |
|-------|-------|
| `id` | examples |
| `type` | reference |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
Teaching examples. Not part of the live workflow.
Contains filled-in examples of the workspace architecture patterns so
new users can see concrete implementations before adapting to their domain.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| 01-how-to-adapt-this.md | file | How to replace the Acme domain with your own |
| 02-skill-integration-patterns.md | file | How skills and MCPs wire into CONTEXT.md routing |
| 03-context-md-anatomy.md | file | Anatomy of a well-formed CONTEXT.md |
| 04-common-mistakes.md | file | Patterns that look right but break the architecture |

## What I Need From Parent
Nothing — read-only reference.

## What I Return To Parent
Nothing — reference only. Never modify these from live workflow sessions.

## Routing Rules
Read these files when learning or teaching the system.
Do not load them during active task sessions — they are not operational context.
