# game_engine — Build Roadmap

*Status: specced → ready to build. All spec reviews passed 2026-03-14. Begin Phase 0 implementation with node-manager.*

---

## Gate: Before Any Phase Begins

All of these must be true before Phase 0 starts:

- [x] ADR-001 resolved: sector size = 1,000 units
- [x] ADR-002 resolved: R-tree (DB) + octree (memory)
- [x] ADR-003 resolved: semi-implicit Euler
- [x] ADR-004 resolved: proven collision library behind contract
- [x] ADR-005 resolved: BC7 + ASTC dual format
- [x] ADR-006 resolved: Flatbuffers + Protobuf
- [x] ADR-014 resolved: domain-warped fractal Simplex noise
- [x] shared/schemas/ — all 16 schema files written: 10 .fbs + 6 .proto (GAP-011 closed 2026-03-14)
- [x] Shared contracts finalized: world-state ✅ simulation ✅ node-registry ✅ lod-system ✅ player-session ✅ ticker-log ✅ (GAP-002 closed 2026-03-14)
- [x] Spec review passes for `world/programs/spatial/`  ← PASS 2026-03-14
- [x] Spec review passes for `world/programs/node-manager/`  ← PASS 2026-03-14
- [x] Spec review passes for `engine/programs/renderer/`  ← PASS 2026-03-14
- [x] Spec review passes for `engine/programs/local-simulation/`  ← PASS 2026-03-14

---

## Phase 0: Foundation
**Goal**: One node, one player, empty world, 50 ticks/sec, 60 FPS client

Build order within Phase 0:
1. `world/programs/node-manager/` — node lifecycle, tick loop skeleton
2. `world/programs/simulation/` — physics integration (dynamic bodies only)
3. `world/programs/spatial/` — in-memory octree
4. `engine/programs/renderer/` — renders a flat plane at 60 FPS
5. `engine/programs/local-simulation/` — client-side prediction (position only)
6. Network protocol — handshake, position update, simple action (Part VIII)
7. Integration: client connects to node, position syncs, frame renders

**Phase 0 done when**: One player, moving, server-authoritative, client-predicted, 50 ticks server, 60 FPS client.

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
