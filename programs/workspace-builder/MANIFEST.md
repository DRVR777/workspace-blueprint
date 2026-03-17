# MANIFEST — workspace-builder

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-builder |
| `type` | project |
| `depth` | 2 |
| `parent` | programs/ |
| `version` | 0.1.0 |
| `status` | active |
| `created` | 2026-03-13T00:00:00Z |

## What I Am
The self-improvement project. It reads the workspace's source PRDs (MWP v1, MWP v2, ALWS),
tracks what has been implemented, generates gaps for what's missing, and executes build tasks
to close those gaps. The workspace builds itself through this project.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| CLAUDE.md | file | active | Depth-1 map of programs |
| CONTEXT.md | file | active | Task router |
| _planning/ | folder | active | PRD requirements, ADRs, build roadmap |
| _meta/ | folder | active | Project-internal gaps |
| programs/ | folder | active | Build programs |
| programs/prd-tracker/ | folder | active | Extracts and tracks PRD requirements |
| programs/auditor/ | folder | active | Audits current workspace state vs requirements |

## What I Need From Parent
- Access to all workspace files (for auditing)
- Root `_meta/` tools (runner, spec-review, gap-detection)

## What I Return To Parent
- Gap objects representing unimplemented PRD requirements
- Fix tasks executed against the workspace
- An ever-more-complete workspace

## Routing Rules
| Condition | Go To |
|-----------|-------|
| See what the PRDs require | _planning/prd-source.md |
| Check implementation status | _planning/roadmap.md |
| Run an audit pass | programs/auditor/CONTEXT.md |
| Track specific requirements | programs/prd-tracker/CONTEXT.md |
| Log a gap | _meta/gaps/pending.txt |

## Gap Status
This project is the gap-generation engine. It generates gaps for the workspace, not for itself.
Internal gaps: _meta/gaps/CONTEXT.md
