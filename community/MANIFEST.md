# MANIFEST — community/

## Envelope
| Field | Value |
|-------|-------|
| `id` | community |
| `type` | workspace |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
The distribution workspace. Content from production/ gets repurposed here
into newsletters, social posts, and event materials.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Task router for community/distribution tasks |
| docs/ | folder | Platform specs, calendar rules |
| content/ | folder | Output by type: newsletters, social, events, templates |

## What I Need From Parent
- Deliverables from `production/` as source material
- Voice guide from `writing-room/docs/voice.md` for tone consistency

## What I Return To Parent
Published or scheduled community content.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Write a newsletter | CONTEXT.md |
| Create social posts | CONTEXT.md |
| Plan an event | CONTEXT.md |
| Check platform specs | docs/platforms.md |
| Check publishing calendar | docs/calendar-rules.md |

## Cross-Workspace Dependencies
- `writing-room/docs/voice.md` — loaded for tone on all outputs (gap-002: writing-room unaware)
- `production/` — source deliverables (gap-001: handoff undocumented on production side)
