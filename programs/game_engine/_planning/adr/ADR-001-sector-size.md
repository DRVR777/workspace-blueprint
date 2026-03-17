# ADR-001: Sector Size
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
The universe is divided into sectors — the unit of world graph partitioning and database sharding. Choosing the sector size determines:
- How often a player's visibility sphere crosses a sector boundary (cross-shard queries)
- How large each database shard's spatial coverage is
- Whether a node's domain fits cleanly into sectors

The player visibility radius is approximately 500 units. A sector must be larger than the visibility radius or a player will always be querying multiple sectors simultaneously.

## Options Considered

| Size | Cross-sector queries | Shard granularity | Notes |
|------|---------------------|-------------------|-------|
| 500 units | Always (visibility radius = sector size) | Fine | Terrible — every query is cross-shard |
| 1000 units | Sometimes (player near edge) | Medium | 2x visibility radius — acceptable |
| 2000 units | Rarely | Coarse | Large shards, potential hotspot |
| 5000 units | Almost never | Very coarse | One shard per major region — poor distribution |

## Decision

**Sector size = 1,000 units on each side (a 1,000³ cube).**

Reasoning:
- 2× the default visibility radius — a player at the center of a sector never queries outside it
- Even at sector edges, only 1 adjacent shard is queried (not 7 in the worst case — the player moves slowly enough that the query radiates into at most 2 shards at once)
- 1,000 units is small enough that dense areas naturally subdivide across multiple sectors / shards
- Matches the chunk system: each sector is 10×10×10 chunks at 100 units per chunk

## Consequences

- Chunk size is derived: 100 units (1,000 / 10)
- A single node's domain is typically 1–4 sectors in extent
- Cross-shard queries must be supported in the world graph client but are not on the critical path
- Terrain generation operates at chunk granularity, sectors are the cache unit
