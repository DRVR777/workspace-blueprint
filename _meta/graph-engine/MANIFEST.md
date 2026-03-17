# MANIFEST — _meta/graph-engine/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-graph-engine |
| `type` | meta |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
The knowledge graph materializer. Reads distributed edge declarations from file headers
across the workspace and builds a unified `graph.json`. The graph enables traversal,
orphan detection (gap finding by missing edges), and path-finding between any two nodes.

This folder is domain-agnostic. It operates on file paths and edge type strings —
never on the content of the files it indexes.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Agent contract: how to read edges, build graph.json, detect orphans |
| edge-schema.json | file | Schema for edge declarations in file headers |
| graph-schema.json | file | Schema for the materialized graph.json output |

## What I Need From Parent
- Read access to all files in the workspace (scans for edge declarations in headers)
- Write access to `_meta/graph-engine/output/graph.json`

## What I Give To Children
Nothing. No children.

## What I Return To Parent
- `output/graph.json` — full materialized graph
- Orphan report (nodes with no incoming or outgoing edges) → logged to `_meta/gaps/pending.txt`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Build or rebuild the graph | CONTEXT.md |
| Understand edge declaration format | edge-schema.json |
| Understand the graph.json structure | graph-schema.json |
| Read the current graph | output/graph.json (if it exists) |

## Layer 0 Test
This folder reads file paths and edge type labels. It has no knowledge of what those
files contain. Byzantine tax law, protein folding, and jazz harmony all look the same
to a graph materializer.
