# MANIFEST — workspace-builder/_planning/

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-builder-planning |
| `type` | planning |
| `depth` | 3 |
| `parent` | programs/workspace-builder/ |
| `status` | active |

## What I Am
The workspace-builder's architecture and requirement tracking layer.
Contains the source PRDs and the implementation roadmap.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Task router |
| prd-source.md | file | Source PRDs: MWP v1, MWP v2, ALWS — authoritative requirements |
| roadmap.md | file | All PRD requirements with ✅/⚠️/❌ implementation status |
| adr/ | folder | Architecture decisions for the builder itself |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| See what PRDs require | prd-source.md |
| Check implementation status | roadmap.md |
| Make a builder decision | adr/ |
