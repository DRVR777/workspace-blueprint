# Research — Task Router

## Which Study Answers Which Question

| Question | Study | ADR it resolves |
|----------|-------|----------------|
| How many neighbors should each file have? | 01-k-value-optimization/ | ADR-004 |
| Do 5D heuristic vectors actually work? | 02-vector-heuristic-accuracy/ | None (quality) |
| Are the neighbors the AI reads actually relevant? | 03-neighbor-relevance/ | None (quality) |
| Do context files make navigation better? | 04-embedded-prompt-effectiveness/ | None (quality) |
| Should context-builder run on every read or in batches? | 05-trigger-mode-comparison/ | ADR-005 |
| Should file-selector be a Claude tool or MCP server? | 06-tool-vs-mcp/ | ADR-008 |
| Does the ticker become a meaningful graph over time? | 07-ticker-as-emergent-graph/ | None (emergent) |
| How does the system perform at scale? | 08-scalability/ | None (scale) |
| Does the AI actually use context files when they exist? | 09-context-file-utility/ | None (behavior) |
| Is 5D as good as 1536D for this use case? | 10-5d-vs-highdim/ | None (architecture) |

## Routing by Current Work State

| If you are... | Go to |
|---------------|-------|
| About to write any code | 01, 05, 06 first — all three must be concluded |
| Building the indexer | 02-vector-heuristic-accuracy/ — run alongside |
| Building context-builder | 03-neighbor-relevance/, 04-embedded-prompt-effectiveness/ |
| System has been running for a week | 07-ticker-as-emergent-graph/, 09-context-file-utility/ |
| System has 100+ files | 08-scalability/ |
| Considering ML upgrade | 10-5d-vs-highdim/ |

## Status Dashboard

| Study | Status | Conclusion |
|-------|--------|-----------|
| 01 k-value-optimization | designed | pending |
| 02 vector-heuristic-accuracy | designed | pending |
| 03 neighbor-relevance | designed | pending |
| 04 embedded-prompt-effectiveness | designed | pending |
| 05 trigger-mode-comparison | designed | pending |
| 06 tool-vs-mcp | designed | pending |
| 07 ticker-as-emergent-graph | designed | pending |
| 08 scalability | designed | pending |
| 09 context-file-utility | designed | pending |
| 10 5d-vs-highdim | designed | pending |

Update this table as studies progress.
