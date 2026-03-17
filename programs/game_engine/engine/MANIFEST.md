---
name: engine
parent: game_engine
type: subsystem
status: scaffold
layer: 2-4 (platform abstractions + engine primitives + world systems)
---

# engine — Client-Side Rendering and Local State

**What this subsystem owns**:
- The rendering loop (frame production at target frame rate)
- The local world state (client-side snapshot of visible server state)
- The LOD system (tier assignment, blend factors, transition smoothing)
- The asset cache (disk + memory cache of downloaded geometry)
- Local physics prediction (velocity integration, terrain collision)
- Server reconciliation (correcting prediction errors from authoritative updates)
- The visibility system (frustum culling, occlusion, distance cutoff)
- The asset streaming consumer (receives asset chunks, assembles, stores)

**What this subsystem does NOT own**:
- The WebSocket connection itself (network layer)
- Player input handling (input layer above this)
- UI / HUD (interface layer above this)
- The authoritative world state (that is `world/`)
- Asset storage on the server (that is asset-store infrastructure)

**Layers within engine/**:

```
engine/
├── MANIFEST.md              (this file)
├── CONTEXT.md
├── _planning/
│   ├── adr/                 (engine-specific ADRs)
│   └── roadmap.md
└── programs/                (engine sub-programs)
    -- renderer/             (frame production, scene graph, draw calls)
    -- lod/                  (LOD tier system, impostor system, transition blending)
    -- asset-pipeline/       (cache management, streaming consumer, decompression)
    -- local-simulation/     (client-side prediction, reconciliation)
    -- visibility/           (frustum culling, occlusion, distance management)
```

**Key contracts this subsystem consumes**:
- `world-state-contract.md` — reads world state (positions, properties)
- `asset-store-contract.md` — requests and receives geometry
- `lod-system-contract.md` — publishes LOD API for the renderer

**PRD sections that define this subsystem**:
- Part V: Local Engine
- Part VI: LOD System
- Part VII: Asset Pipeline
- Part XVI: Performance Contracts (client section)
