# ADR-004: k-Nearest Neighbors — Default k=5

Status: accepted — 2026-03-14
Date: 2026-03-13

## Decision
Default k=5 neighbors per file. Configurable per-file via `k: N` in metadata header.

## Context
The user specified that each file has a neighbor list but did not specify how many neighbors.
k=5 is a common default in k-NN systems and matches the 5-dimension vector size (convenient, not coincidental).

## Consequences
- Each file lists at most 5 neighbors by default
- This means: reading a file + all its neighbors = at most 6 file reads per hop
- k=5 keeps the embedded prompt's neighbor-reading loop bounded
- Files with fewer than k total files in the Data/ will have fewer neighbors (correct behavior)
- Custom k is supported for hub nodes (k=10+) or leaf nodes (k=2)

## Needs human validation
Change k if you want the AI to explore more broadly (higher k) or stay focused (lower k).
The trade-off: higher k = richer context, more tool calls. Lower k = faster navigation, less context.
