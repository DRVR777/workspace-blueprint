# MANIFEST — programs/knowledge-graph/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph |
| `type` | project |
| `depth` | 2 |
| `parent` | programs/ |
| `status` | specced |
| `created` | 2026-03-13 |

## What I Am
A Cognitive Document System (CDS) — a self-navigating knowledge graph where documents
are active participants. Each document knows its 5D position in semantic space, knows
its neighbors, and carries an embedded prompt that fires when the AI reads it.

The AI navigates via tool-use (file-selector), not passive retrieval. Navigation is
logged as an attention trace (global ticker). Documents self-describe their position
relative to their neighbors on first read.

This is NOT a standard RAG system. See `_planning/rag-vs-cds.md`.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| _planning/ | folder | active | Full spec, ADRs, architecture, implementation order |
| _meta/ | folder | active | Project-internal gap registry |
| programs/ | folder | active | Four sub-programs: data-store, file-selector, indexer, context-builder |
| shared/ | folder | active | Contracts between programs |
| research/ | folder | active | 10 empirical studies — validates ADRs before building |

## What I Need From Parent
Nothing — self-contained. The workspace's `_meta/` provides gap detection and runner.

## What I Return To Parent
A navigable knowledge graph. Any agent in this workspace can call `file-selector`
as a tool to read documents, navigate by 5D proximity, and leave an attention trace.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Understand what this project is | _planning/prd-source.md |
| Understand system architecture | _planning/architecture.md |
| See why this is not RAG | _planning/rag-vs-cds.md |
| Work on a specific program | programs/[name]/CONTEXT.md |
| Check implementation order | _planning/roadmap.md |
| Run a research study | research/CONTEXT.md |
| Check which ADRs are unblocked | research/adr-resolution-log.md |
| Find a contract | shared/MANIFEST.md |
