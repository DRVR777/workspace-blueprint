# game_engine — Task Router

**Project**: NEXUS — custom spatial computing substrate for Dreamworld

## Quick Start

| You want to... | Go to |
|----------------|-------|
| Understand the full requirements | `PRD.md` |
| See the tech stack | `_planning/adr/ADR-015-technology-stack.md` |
| See all architecture decisions | `_planning/adr/README.md` (8 accepted, 7 open) |
| See the build roadmap | `_planning/roadmap.md` |
| Work on server code (Rust) | `world/crates/` — 4 crates: nexus-core, nexus-spatial, nexus-simulation, nexus-node |
| Work on client code (TypeScript/R3F) | `engine/programs/renderer/` |
| Read subsystem contracts | `shared/contracts/` — 8 contracts |
| Read network schemas | `shared/schemas/` — 18+ schemas (Flatbuffers + Protobuf) |
| Find open gaps | `_meta/gaps/pending.txt` |
| See reference 3D code (R3F) | `../ELEV8-source/components/scene/` |
| See reference 3D code (raw Three.js) | `../personalWebsite/src/` |
| Read the campaign tracker | `../../campaigns/game_engine/clarification.md` |

## Architecture (5-Layer, from ADR-015)

```
SERVER (Rust)                          CLIENT (TypeScript/R3F)
┌─────────────────────┐                ┌─────────────────────┐
│ L5: Orchestration   │                │ L5: Experience      │
│ L4: Protocol        │◄──WebSocket──►│ L4: Scene Mgmt      │
│ L3: Simulation      │                │ L3: GfxContext       │
│ L2: Spatial         │                │ L2: Protocol         │
│ L1: Core            │                │ L1: Core             │
└─────────────────────┘                └─────────────────────┘
```

## Current State (2026-03-19)

- All Phase 0 specs complete (simulation, visibility, world-graph)
- Rust server compiles: 4 crates, 19 tests passing, Rapier physics integrated
- R3F client runs at 60+ FPS with instanced rendering
- **Next**: wire WebSocket connection between server and client → first playable moment
- Server deploying to VPS for multiplayer testing

## Contracts (8 accepted)

| Contract | Defines |
|----------|---------|
| `simulation-contract` | run_tick, physics_body, tick_result |
| `world-state-contract` | object_record, entity_record, change_request |
| `world-graph-contract` | world_record, portals, subworlds, constellation |
| `node-registry-contract` | domain routing, node_descriptor |
| `lod-system-contract` | LOD tier assignment, blend results |
| `player-session-contract` | auth, session management |
| `ticker-log-contract` | audit log of state changes |
| `asset-store-contract` | asset caching, streaming |

## Load Rules
- Always load `MANIFEST.md` first
- Load `PRD.md` only when you need deep requirements detail
- Load a subsystem's MANIFEST before working in that subsystem
- Never load more than one subsystem's internals at once unless explicitly crossing boundaries
