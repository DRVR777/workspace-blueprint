# MANIFEST — knowledge-graph/research/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-research |
| `type` | research |
| `depth` | 3 |
| `parent` | programs/knowledge-graph/ |
| `status` | active |
| `created` | 2026-03-14 |

## What I Am
The empirical research layer of the knowledge-graph project.
Before the system is built, we need answers. This folder tracks every open question
from the spec and ADRs as a formal research study: hypothesis → test plan → findings → decision.

Research here directly unblocks assumption ADRs. When a study reaches a conclusion,
the corresponding ADR is updated from `assumption` to `accepted`.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| overview.md | file | active | Master research plan — all 10 studies, priority order, dependencies |
| CONTEXT.md | file | active | Router — which study answers which question |
| adr-resolution-log.md | file | active | Tracks which studies closed which ADRs |
| 01-k-value-optimization/ | folder | active | How many neighbors is optimal? (ADR-004) |
| 02-vector-heuristic-accuracy/ | folder | active | Do heuristic 5D vectors actually capture semantic proximity? |
| 03-neighbor-relevance/ | folder | active | Are k-nearest neighbors in 5D space actually related in content? |
| 04-embedded-prompt-effectiveness/ | folder | active | Do context files improve AI navigation vs no context files? |
| 05-trigger-mode-comparison/ | folder | active | On-read vs batch context-builder: quality and performance (ADR-005) |
| 06-tool-vs-mcp/ | folder | active | Claude tool_use vs MCP server for file-selector (ADR-008) |
| 07-ticker-as-emergent-graph/ | folder | active | Does the navigation trace reveal meaningful graph structure? |
| 08-scalability/ | folder | active | How does the system behave at 10 / 100 / 1000 files? |
| 09-context-file-utility/ | folder | active | How much does the AI actually use ctx files vs raw files? |
| 10-5d-vs-highdim/ | folder | active | Are 5 interpretable dims as useful as 1536-dim embeddings for navigation? |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| See all research questions | overview.md |
| Find the study for a specific ADR | CONTEXT.md |
| Check which ADRs are unblocked | adr-resolution-log.md |
| Run a specific study | [study-folder]/plan.md |
| Log a finding | [study-folder]/findings/ |
