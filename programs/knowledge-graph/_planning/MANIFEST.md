# MANIFEST — knowledge-graph/_planning/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-planning |
| `type` | planning |
| `depth` | 3 |
| `parent` | programs/knowledge-graph/ |
| `status` | active |

## What I Am
The full implementation plan for the Cognitive Document System.
All architectural decisions, specs, and build order live here.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| prd-source.md | file | Full system spec from conversation — authoritative requirements |
| rag-vs-cds.md | file | Why this is not a RAG system — conceptual foundation |
| architecture.md | file | System diagram, data flow, component responsibilities |
| file-format-spec.md | file | Exact format for every file in Data/ |
| 5d-vector-spec.md | file | Dimension definitions, computation methods, distance formula |
| roadmap.md | file | Build order, feature list, session plan |
| adr/ | folder | 8 architecture decisions — 3 accepted, 5 assumption |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Understand the system concept | prd-source.md → rag-vs-cds.md |
| Understand data flows | architecture.md |
| Implement file creation | file-format-spec.md |
| Implement the indexer | 5d-vector-spec.md |
| Check what to build next | roadmap.md |
| Make or review a decision | adr/ |
