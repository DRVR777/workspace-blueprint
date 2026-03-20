# game_engine — Build Roadmap

*Status: BUILDING. All specs complete. Rust server compiles (19 tests pass). R3F client renders at 60+ FPS. Wiring WebSocket next.*

---

## Gate: Before Any Phase Begins — ALL PASSED

- [x] ADR-001 through ADR-006, ADR-014, ADR-015 — all accepted
- [x] shared/schemas/ — 18+ schema files written (Flatbuffers + Protobuf)
- [x] 8 shared contracts finalized (world-state, simulation, world-graph, node-registry, lod, session, ticker-log, asset-store)
- [x] All Phase 0 sub-programs specced (simulation, visibility added 2026-03-19)
- [x] Tech stack locked: TypeScript/R3F client, Rust server, Rapier physics (ADR-015)

---

## Phase 0: Foundation
**Goal**: One node, one player, empty world, 50 ticks/sec, 60+ FPS client

**Tech note**: Server is Rust (world/crates/). Client is TypeScript/R3F (engine/programs/renderer/).
Reference code: ELEV8-source (R3F), personalWebsite (raw Three.js).

Build order within Phase 0:
1. ~~`world/crates/nexus-core`~~ — ✅ DONE: Vec3, Quat, AABB, PhysicsBody, config, constants
2. ~~`world/crates/nexus-spatial`~~ — ✅ DONE: octree insert/remove/move/query (6 tests pass)
3. ~~`world/crates/nexus-simulation`~~ — ✅ DONE: 5-stage run_tick with Rapier (compiles)
4. ~~`world/crates/nexus-node`~~ — ✅ DONE: tokio entry point, 50Hz tick loop, WebSocket echo server
5. ~~`engine/programs/renderer/`~~ — ✅ DONE: R3F renders terrain + 50 instanced entities at 60+ FPS
6. **WebSocket bridge** — ← NEXT: connect R3F client to Rust server, HANDSHAKE protocol
7. Character controller — port from ELEV8-source AvatarController.tsx
8. Integration: client connects to node, position syncs, frame renders

**Phase 0 done when**: One player, moving, server-authoritative, client-predicted, 50 ticks server, 60+ FPS client.

---

## Phase 1: World Content
**Goal**: Real terrain, real objects, multiple players, persistence

Build order within Phase 1:
1. `world/programs/world-generator/` — procedural terrain generation
2. `engine/programs/asset-pipeline/` — cache, streaming consumer
3. `engine/programs/lod/` — LOD tier system
4. `engine/programs/visibility/` — frustum culling
5. World graph persistence layer (objects survive restart)
6. Multi-player position sync
7. Object interaction (pickup)

**Phase 1 done when**: Two players see each other in a real landscape. Objects persist.

---

## Phase 2: Multi-Node
**Goal**: Node boundaries, handoffs, world graph sharding

Build order:
1. Orchestration controller (domain map, health monitoring)
2. Domain boundary detection in simulation
3. Handoff protocol between nodes
4. World graph second shard
5. Edge gateway (player routing)

---

## Phase 3: Full World Systems
**Goal**: Building, economy, agents, machines

Build order:
1. Building system
2. Resource gathering + crafting
3. Inventory
4. AI agent behavior trees
5. Machine objects
6. Ticker log (full audit)
7. Computer objects (stub)

---

## Phase 4: Scale
**Goal**: Thousands of concurrent players, geographic distribution, auto-scaling

Build order:
1. Load-based node splitting
2. Geographic regions (2 minimum)
3. Cross-region entity handoff
4. Auto-scaling orchestration
5. Performance optimization pass
6. Observability stack

---

## Unscheduled (Post-Phase 4)
- In-world computer full implementation (ADR-010 must resolve first)
- Social systems (guilds, friends)
- Telegram interface (Dreamworld requirement, out of engine scope)
- Token economy (game logic layer, above engine)
