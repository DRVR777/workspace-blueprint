# Architecture Decision Records

*Each file in this directory records one architectural decision.*
*Open ADRs are blocking. Accepted ADRs are final. Superseded ADRs are archived.*

## Status Summary

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-001 | Sector size = 1,000 units | **ACCEPTED** 2026-03-14 |
| ADR-002 | R-tree (world graph DB) + octree (node memory) | **ACCEPTED** 2026-03-14 |
| ADR-003 | Semi-implicit Euler integration | **ACCEPTED** 2026-03-14 |
| ADR-004 | Collision: use proven library behind contract | **ACCEPTED** 2026-03-14 |
| ADR-005 | BC7 (desktop) + ASTC (mobile), format per client caps | **ACCEPTED** 2026-03-14 |
| ADR-006 | Flatbuffers (game state) + Protobuf (control) | **ACCEPTED** 2026-03-14 |
| ADR-007 | Edge gateway architecture | OPEN — blocks Phase 2 |
| ADR-008 | Ticker log storage engine | OPEN — blocks Phase 3 |
| ADR-009 | World graph replication strategy | OPEN — blocks Phase 2 |
| ADR-010 | In-world computer language | OPEN — blocks Phase 3 |
| ADR-011 | Geographic region boundary strategy | OPEN — blocks Phase 4 |
| ADR-012 | Asset compression algorithm | OPEN — blocks Phase 1 |
| ADR-013 | Behavior AI evaluation order | OPEN — blocks Phase 3 |
| ADR-014 | World seed: domain-warped fractal Simplex noise | **ACCEPTED** 2026-03-14 |
| ADR-015 | Tech stack: TS/R3F client, Rust server, Rapier shared physics, 5-layer arch | **ACCEPTED** 2026-03-18 |

## ADR Format

Each ADR file uses this structure:
```
# ADR-NNN: [Decision Name]
Status: open | accepted | superseded
Supersedes: ADR-NNN (if applicable)

## Context
Why does this decision need to be made?

## Options
What are the candidates?

## Decision
What did we decide? (empty when status=open)

## Consequences
What does this decision imply?
```
