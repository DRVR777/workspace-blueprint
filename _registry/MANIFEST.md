# MANIFEST — _registry/

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-registry |
| `type` | registry |
| `depth` | 1 |
| `parent` | workspace-blueprint/ (root) |
| `status` | active |

## What I Am
Layer 1 of the ALWS system. Maps task types to workspaces. When a new task type
arrives that has no registered workspace, this layer triggers the workspace-builder
pipeline to scaffold one. Tracks average quality per workspace type over time.

This folder is read by intake-pipeline stage-02-classify (type detection) and
stage-03-route (workspace lookup). It is written by stage-05-update (quality tracking)
and workspace-builder (new workspace registration).

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| REGISTRY.md | file | Live mapping: task type → workspace path, quality, run count |
| CONTEXT.md | file | How to read, update, and extend the registry |
| quality-schema.json | file | Schema for the per-type quality tracking entries in REGISTRY.md |

## What I Need From Parent
Nothing. This folder reads from the workspace and writes back to it.

## What I Give To Children
Nothing. No children.

## What I Return To Parent
- Workspace lookup results (consumed by intake-pipeline stage-03-route)
- New workspace triggers (consumed by workspace-builder when type is unknown)

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Classify a task type and find the right workspace | REGISTRY.md |
| Add or update a workspace type registration | CONTEXT.md |
| Understand the quality tracking structure | quality-schema.json |

## Layer 0 Test
This folder routes tasks to workspaces. It contains no domain knowledge.
A legal task, a protein folding task, and a jazz composition task are all equal here —
just different `task_type` strings pointing to different workspace paths.
