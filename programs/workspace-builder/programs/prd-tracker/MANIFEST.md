# MANIFEST — workspace-builder/programs/prd-tracker

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-builder-prd-tracker |
| `type` | program |
| `depth` | 4 |
| `parent` | workspace-builder/programs/ |
| `status` | active |

## What I Am
Reads source PRDs and extracts structured requirements into the roadmap.

## External Dependencies
| Depends On | What | Location |
|------------|------|----------|
| prd-source.md | MWP v1, v2, and ALWS PRD text | ../../_planning/prd-source.md |

## What I Produce
Updated `../../_planning/roadmap.md` with requirements and implementation status.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Extract or reconcile requirements | CONTEXT.md |
| See current requirements | ../../_planning/roadmap.md |
