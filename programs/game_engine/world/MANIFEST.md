---
name: world
parent: game_engine
type: subsystem
status: scaffold
layer: 3-4 (world systems + engine primitives)
---

# world — Server-Side Spatial Substrate

**What this subsystem owns**:
- The octree and all spatial partitioning logic
- The domain system (what node owns what region)
- The world graph client (reading and writing object state)
- World generation (procedural terrain, sector initialization)
- The simulation tick (physics, entity AI, interaction resolution)
- Node lifecycle (starting, active, draining, splitting, merging)
- Inter-node communication (handoffs, boundary events)

**What this subsystem does NOT own**:
- Rendering (that is `engine/`)
- Asset geometry (that is `engine/asset-pipeline`)
- Player WebSocket connections (that is the network layer above world)
- The world graph database itself (that is infrastructure, not this subsystem)

**Layers within world/**:

```
world/
├── MANIFEST.md              (this file)
├── CONTEXT.md
├── _planning/
│   ├── adr/                 (world-specific ADRs)
│   └── roadmap.md           (world build order)
└── programs/                (world sub-programs, added as PRD sections mature)
    -- spatial/              (octree, spatial index, sector/chunk management)
    -- simulation/           (physics, collision, entity AI, tick runner)
    -- node-manager/         (node lifecycle, handoffs, domain map)
    -- world-generator/      (procedural terrain, sector initialization)
```

**Key contracts this subsystem publishes**:
- `world-state-contract.md` — other subsystems read world state through this
- `simulation-contract.md` — physics and tick results
- `node-registry-contract.md` — domain map queries

**PRD sections that define this subsystem**:
- Part III: World Architecture
- Part IV: Node System
- Part X: Simulation System
- Part XIV: Platform and Orchestration (shared with platform subsystem)
