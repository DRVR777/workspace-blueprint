# ADR-002: World Graph Spatial Index Type
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
The world graph needs a spatial index so that queries like "find all objects within radius R of position P" are answered in logarithmic time, not linear time. The choice of index type affects:
- Query performance for the most common operation (radius search)
- Write performance (insert/delete/move)
- Implementation complexity
- Database support (can we use an existing index, or must we build one?)

Two indexes are needed at different levels:
1. **World graph database index** — persisted, across all objects globally
2. **Node in-memory index** — volatile, only objects in the node's domain

These can be different types because their constraints differ.

## Options Considered

| Index Type | Radius query | Insert/Delete | Persistence | Complexity |
|------------|-------------|---------------|-------------|------------|
| R-tree | O(log N + K) | O(log N) | Native in spatial DBs | Low (use DB support) |
| Octree | O(log N + K) | O(log N) amortized | Must serialize | Medium |
| Spatial hash grid | O(1) amortized | O(1) | Simple to serialize | Low |
| k-d tree | O(√N + K) | O(N) for rebalance | Must serialize | Medium |
| Hilbert curve | O(log N + K) | O(log N) | Excellent DB support | High |

**Why not k-d tree**: Rebalancing on delete is O(N) — unacceptable for a live game world with constant inserts/deletes.

**Why not spatial hash**: Degenerates for large radius queries — O(R³ / cell_size³) cells to inspect. With visibility radius 500 and cell size 100, that's 1,000 cells per query. Also requires tuning cell size globally.

## Decision

**Two different indexes for two different purposes:**

**World graph database index → R-tree**
Reason: R-trees are natively supported by every major spatial database (PostGIS, SQLite/SpatiaLite, MongoDB with 2dsphere). We get the index for free with zero implementation cost. R-trees have excellent range query and radius query performance. The industry has 40 years of R-tree implementations behind it.

**Node in-memory index → Octree**
Reason: For in-memory simulation with dynamic subdivision (node split/merge), an octree maps naturally to the domain hierarchy already in use. Octrees are cache-friendly (spatial locality), simple to implement, and match the sector/chunk structure. Re-indexing after a node split is O(N) — acceptable since splits are infrequent.

## Consequences

- The world graph client must wrap R-tree queries in its interface (callers never know it's an R-tree)
- Node simulation uses the octree specified in PRD Part III.2
- Octree leaf parameters (MAX_OBJECTS_PER_LEAF, MIN_OBJECTS_PER_LEAF) are tuning knobs — start at 32 and 8
- Cross-node queries use the R-tree through the world graph client, not direct octree access
