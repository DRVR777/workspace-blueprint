# NEXUS Rust Server — Crate Guide

## Architecture

4 crates in a Cargo workspace, matching the 5-layer architecture (ADR-015).
Each layer only depends on the layer below it.

```
nexus-node          ← Layer 4+5: Entry point, WebSocket, tick loop, broadcasting
  └─ nexus-simulation  ← Layer 3: Physics (Rapier), game rules, tick pipeline
       └─ nexus-spatial    ← Layer 2: Octree spatial index, proximity queries
            └─ nexus-core      ← Layer 1: Vec3, Quat, PhysicsBody, config (zero deps)
```

## Crate details

### nexus-core (Layer 1)
Zero external dependencies. Pure types and math.
- `math.rs` — Vec3f32, Vec3f64, Quat32, Aabb64 with full operator overloads
- `types.rs` — PhysicsBody, ChangeRequest, TickResult, CollisionData
- `config.rs` — WorldPhysicsConfig with 4 gravity modes (directional, spherical, zero, zone-based)
- `constants.rs` — Every magic number from the specs

### nexus-spatial (Layer 2)
Depends: nexus-core
- `octree.rs` — Full octree: insert, remove, move, query_radius, query_box
- Subdivision at 32 objects per leaf, max depth 16
- AABB-sphere intersection for efficient radius queries

### nexus-simulation (Layer 3)
Depends: nexus-core, nexus-spatial, rapier3d, hecs
- `lib.rs` — `run_tick()` orchestrates the 5-stage pipeline
- `validate.rs` — Stage 1: input validation, force clamping, spawn limits
- `actions.rs` — Stage 2: MOVE→force, CREATE→spawn, DESTROY→remove
- `physics.rs` — Stage 3: Rapier integration (build world, step, read back, collision events)
- `rules.rs` — Stage 4: domain boundary detection (AI/triggers stubbed for Phase 0)
- `diff.rs` — Stage 5: compare before/after snapshots, emit state changes

### nexus-node (Layer 4+5)
Depends: all + tokio, tokio-tungstenite, flatbuffers, prost
- `main.rs` — Entry point: init world state, spawn tick loop + WebSocket server
- `tick_loop.rs` — 50Hz tick loop with metrics, load monitoring
- `server.rs` — WebSocket accept, connection handling

## Build

```bash
cargo build          # debug
cargo build --release # production
cargo test           # run all 19 tests
cargo run            # start server on port 9001
```

## Spec references

Each crate implements a spec from `world/programs/` or `shared/contracts/`:
- nexus-core → constants from all contracts
- nexus-spatial → `world/programs/spatial/MANIFEST.md`
- nexus-simulation → `world/programs/simulation/MANIFEST.md` + `shared/contracts/simulation-contract.md`
- nexus-node → `world/programs/node-manager/MANIFEST.md`
