# Workspace Blueprint — Build Status

*Last updated: 2026-03-19*

## Active Projects

### NEXUS Game Engine (`programs/game_engine/`)
**Status**: Phase 0 — server compiles, client renders, wiring WebSocket next

| Component | Status | Location |
|-----------|--------|----------|
| Rust server (4 crates) | Compiles, 19 tests pass | `game_engine/world/crates/` |
| R3F client (renderer) | Runs at 60+ FPS | `game_engine/engine/programs/renderer/` |
| Simulation spec | Complete (5-stage Rapier pipeline) | `game_engine/world/programs/simulation/` |
| Visibility spec | Complete (frustum culling + LOD) | `game_engine/engine/programs/visibility/` |
| World graph contract | Complete (portals, subworlds, constellation) | `game_engine/shared/contracts/world-graph-contract.md` |
| ADRs | 8/15 accepted | `game_engine/_planning/adr/` |
| Contracts | 8 accepted | `game_engine/shared/contracts/` |
| Schemas | 18+ defined | `game_engine/shared/schemas/` |
| WebSocket bridge | **Next up** | — |
| Multiplayer | Not started | — |
| Auth/accounts | Not started | — |

### ORACLE Trading Platform (`programs/oracle/`)
**Status**: Scaffold — 12 blocking gaps to resolve

| Component | Status |
|-----------|--------|
| Signal ingestion (Polymarket) | Scaffolded |
| Whale detector | Scaffolded |
| OSINT fusion | Scaffolded |
| Reasoning engine | Scaffolded |
| Solana executor | Scaffolded |
| Knowledge base | Scaffolded |
| Operator dashboard | Scaffolded |

### Knowledge Graph CDS (`programs/knowledge-graph/`)
**Status**: All 4 programs built, MCP server live globally

## Reference Projects (read-only)

| Project | Purpose |
|---------|---------|
| `programs/ELEV8-source/` | R3F character controller, scene portals, interaction system |
| `programs/personalWebsite/` | Raw Three.js museum walkthrough, asset loading, camera controls |
| `programs/dreamworld/` | PRD + implementation plan — vision source for NEXUS |
| `programs/ELEV8/` | Analysis docs + failure post-mortem |

## Infrastructure

| System | Status |
|--------|--------|
| Agent bus (`_bus/`) | Active — broadcast channel, per-agent inboxes |
| Convention checker | Active — validates workspace patterns |
| Scaffold scripts | Active — `_meta/scripts/scaffold_manifest.py` |
| Graph engine | Available — `_meta/graph-engine/` |
