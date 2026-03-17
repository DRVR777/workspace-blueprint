# MANIFEST — writing-room/

## Envelope
| Field | Value |
|-------|-------|
| `id` | writing-room |
| `type` | workspace |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
The writing workspace. Ideas and topics enter; polished drafts exit.
Voice, research, and editing all happen here before content moves to production/.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Task router for writing tasks |
| docs/ | folder | Stable reference: voice.md, style-guide.md, audience.md |
| drafts/ | folder | Work in progress |
| final/ | folder | Completed drafts ready for production handoff |

## What I Need From Parent
Nothing — self-contained writing scope.

## What I Return To Parent
Final drafts copied to `production/workflows/01-briefs/` for pipeline entry.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Write or edit content | CONTEXT.md |
| Check voice rules | docs/voice.md |
| Check style | docs/style-guide.md |
| Check audience | docs/audience.md |

## Cross-Workspace Note
`docs/voice.md` is consumed by `community/` — do not treat it as writing-room-internal only.
Any change to voice.md affects community outputs. See gap-002 in root `_meta/gaps/`.
