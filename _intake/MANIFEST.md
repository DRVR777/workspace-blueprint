# MANIFEST — _intake

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake |
| `type` | intake |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
The inbound queue for PRD documents. Drop a PRD here and the prd-intake
agent reads it, scaffolds the project, and archives the source document.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| queue/ | folder | PRDs waiting to be processed — drop files here |
| processed/ | folder | PRDs that have been processed — one file per project, renamed with project slug and date |

## Lifecycle of a PRD

```
_intake/queue/[prd-file]
    ↓ (run _meta/prd-intake.md)
[PROJECT_NAME]/ scaffolded
    ↓
_intake/processed/[PROJECT_NAME]-prd-[ISO-date].[ext]
```

## Rules
- Never delete from processed/ — it is the permanent record of what initiated each scaffold
- queue/ should be empty between runs — one PRD per run
- If multiple PRDs are in queue/, process them one at a time

## Routing Rules
| Condition | Go To |
|-----------|-------|
| You have a PRD and want to scaffold a project | Read `_meta/prd-intake.md`, then drop PRD in queue/ |
| You want to see what initiated a project's scaffold | `processed/[PROJECT_NAME]-prd-[date].[ext]` |
