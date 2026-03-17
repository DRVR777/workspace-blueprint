# MANIFEST — workspace-builder/_meta/

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-builder-meta |
| `type` | meta |
| `depth` | 3 |
| `parent` | programs/workspace-builder/ |
| `status` | active |

## What I Am
The workspace-builder's internal gap registry.
Note: this project generates gaps FOR the workspace. This registry tracks
gaps IN the builder itself — a separate concern.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| gaps/ | folder | Internal gap registry for the builder project |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Log a builder-internal inference | gaps/pending.txt |
| Find builder-internal gaps | gaps/CONTEXT.md |
| Log a workspace-level gap found during audit | {root}/_meta/gaps/pending.txt |
