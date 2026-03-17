# MANIFEST — workspace-builder/programs/auditor

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-builder-auditor |
| `type` | program |
| `depth` | 4 |
| `parent` | workspace-builder/programs/ |
| `status` | active |

## What I Am
Audits the workspace against PRD requirements. Reads the roadmap,
checks what exists, generates gap entries for what doesn't.

## External Dependencies
| Depends On | What | Location |
|------------|------|----------|
| roadmap.md | PRD requirement list with status | ../../_planning/roadmap.md |
| Workspace file tree | Current state | All folders from {root} down |

## What I Produce
Gap entries appended to `../../_meta/gaps/pending.txt`
Updated roadmap status rows

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Run an audit | CONTEXT.md |
| See open gaps from last audit | ../../_meta/gaps/CONTEXT.md |
