# ADR-015: Technology Stack — Language and Framework Selection
Status: accepted
Date: 2026-03-18
Blocking: Phase 0

## Context

NEXUS is the custom engine powering Dreamworld — a 3D spatial internet where websites are worlds, any surface can be an HTML screen, in-game computers map to real VPS containers, and iframes embed external web apps inside the 3D world. The technology stack must support:

1. **Browser-native client** — DOM access for HTML overlays, iframes, React components on 3D surfaces, postMessage bridging, WebSocket
2. **High-performance server** — 50+ ticks/s, thousands of entity updates/s, per-world custom physics, dynamic world process spawn/kill
3. **Screen-object architecture** — raycasting UV→pixel coordinate hit-testing against HTML UI rendered on 3D plane geometries
4. **Shared physics** — same physics library running on both server (authoritative) and client (prediction/reconciliation)

The ELEV8 post-mortem showed that choosing tools before understanding requirements leads to fighting the engine. This ADR locks the stack after requirements are fully specced.

### Why not Bevy (or any existing game engine)?

Bevy, Godot, Unity, and Unreal were evaluated. All fail on the same requirement: **the screen-object architecture**.

Dreamworld's core mechanic is that any 3D surface becomes a live HTML screen — rendering React components, embedding iframes of external web apps, running a full computer OS as an HTML overlay with postMessage bridging to the game engine. This requires:

- Direct DOM access (createElement, iframe, postMessage)
- CSS layout and scroll on 3D surfaces
- React component rendering onto canvas textures
- UV→pixel coordinate raycasting for 3D surface interaction

These are **browser platform features**. No native game engine provides them. Bevy compiles to WASM but has no DOM access — it draws to a canvas element and that's it. Wrapping Bevy in a web shell and bridging to the DOM would be more complex than building the renderer directly in the platform that already has these features.

Additionally:
- Bevy's renderer is designed for native graphics pipelines, not Three.js/WebGPU browser contexts
- Bevy's asset pipeline assumes local filesystem, not CDN/HTTP delivery
- Bevy's window management is irrelevant in a browser tab
- Bevy's ECS is excellent — but available as a standalone crate without the engine

## Decision

### Client: TypeScript + React Three Fiber

| Component | Technology | Version (current) |
|-----------|-----------|-------------------|
| Language | TypeScript | 5.6+ |
| UI framework | React | 18.3+ |
| 3D renderer | React Three Fiber (@react-three/fiber) | 8.17+ |
| 3D utilities | @react-three/drei (dev/tooling only) | 9.x |
| Bundler | Vite | 5.4+ |
| Graphics backend | Three.js (abstracted by R3F) | 0.170+ |
| Future graphics | WebGPU (behind GfxContext abstraction) | When stable |
| Audio | Web Audio API | Native browser |
| Networking | WebSocket | Native browser |
| Serialization (decode) | flatbuffers (JS/TS) | — |
| Serialization (control) | protobuf.js / ts-proto | — |

**React Three Fiber, not raw Three.js.** R3F provides:
- Declarative scene graph as React components (`<mesh>`, `<group>`)
- React reconciler manages Three.js object lifecycle — no manual dispose/cleanup
- Hooks (`useFrame`, `useThree`) integrate the render loop with React state
- The screen-object system (HTML overlays on 3D surfaces) is naturally expressed as React components wrapping both 3D geometry and DOM elements

**Drei: dev/tooling only.** Drei helpers (OrbitControls, Stats, Html) are useful for development and debugging. Production rendering uses custom R3F components backed by the GfxContext abstraction. Drei must never appear in the hot render path — its components are generic, hide allocations, and can't be tuned to Dreamworld's specific update lifecycle.

### Server: Rust

| Component | Technology | Crate |
|-----------|-----------|-------|
| Language | Rust | stable (1.75+) |
| Async runtime | Tokio | tokio |
| WebSocket | tokio-tungstenite | tokio-tungstenite |
| Entity system | Standalone ECS | bevy_ecs (standalone) or hecs |
| Physics | Rapier 3D | rapier3d |
| Serialization (game state) | Flatbuffers | flatbuffers |
| Serialization (control) | Protocol Buffers | prost |
| Spatial index (memory) | Octree (custom) | — |
| Spatial index (persistent) | R-tree via PostGIS | sqlx + PostGIS |
| Noise generation | FastNoiseLite | fastnoise-lite |

**Rust, not Node.js/Python.** The Dreamworld PRD specifies Node.js gateway + Python agent service. This ADR supersedes the server language choice for the **world simulation** layer only:

- **World processes (tick loop, physics, spatial)**: Rust — 50 ticks/s with collision detection, spatial queries, and entity updates for hundreds of objects requires zero-GC, predictable latency
- **Agent service**: Python — stays Python (LLM API calls, knowledge graph operations are I/O-bound, not CPU-bound)
- **Gateway**: To be decided in ADR-007 (edge gateway architecture) — Rust or Node.js both viable for packet routing

**Why Rust over C++:**
- Cargo vs CMake — Cargo is a dramatically better build system
- Memory safety without GC — no tick jitter from garbage collection, no use-after-free CVEs
- `bevy_ecs` and `rapier3d` are Rust-native with excellent APIs
- Tokio async ecosystem is mature for WebSocket servers
- Rapier compiles to WASM — same physics library on client and server (shared physics requirement)

**Standalone ECS, not Bevy engine.** `bevy_ecs` is published as an independent crate. It provides the ECS pattern (Components, Systems, Queries, Resources, Change Detection) without Bevy's renderer, asset pipeline, or window management. Alternatively, `hecs` is lighter-weight if full Bevy ECS features aren't needed. Decision between the two deferred to implementation start.

### Shared: Rapier for physics (both sides)

Rapier is the physics library for both server and client:
- **Server**: `rapier3d` Rust crate — native performance for authoritative physics
- **Client**: `@dimforge/rapier3d-compat` WASM package — same library, same determinism, runs in browser for client-side prediction

This satisfies ADR-004 (proven collision library behind contract) and ensures server/client physics match exactly — critical for prediction/reconciliation.

## 5-Layer Architecture

Both client and server follow a strict 5-layer architecture. Each layer only depends on the layer directly below it. Interfaces between layers are explicit contracts.

### Server Layers

```
┌─────────────────────────────────────────────┐
│  Layer 5: Orchestration                     │
│  World process lifecycle, health, scaling   │
│  spawn/kill per world, load from DB on      │
│  player enter, flush to DB on player exit   │
├─────────────────────────────────────────────┤
│  Layer 4: Protocol                          │
│  WebSocket accept, wire framing, Flatbuf/   │
│  Protobuf codec, Packet header routing,     │
│  interest management (who gets what)        │
├─────────────────────────────────────────────┤
│  Layer 3: Simulation                        │
│  Tick loop, Rapier physics step, collision  │
│  resolution, entity behavior, game rules,   │
│  per-world custom physics params            │
├─────────────────────────────────────────────┤
│  Layer 2: Spatial                           │
│  Octree (in-memory), R-tree queries (DB),   │
│  sector management, proximity queries,      │
│  node boundary detection                    │
├─────────────────────────────────────────────┤
│  Layer 1: Core                              │
│  Vec3, Quaternion, EntityId, Timestamp,     │
│  arena allocators, ECS world, config types  │
└─────────────────────────────────────────────┘
```

### Client Layers

```
┌─────────────────────────────────────────────┐
│  Layer 5: Experience                        │
│  Computer OS (React), HUD, menus, input     │
│  mapping, screen-object interaction,        │
│  iframe embedding, agent chat UI            │
├─────────────────────────────────────────────┤
│  Layer 4: Scene Management                  │
│  Entity lifecycle (R3F components), LOD     │
│  transitions, frustum culling, asset cache, │
│  character controller, camera spine         │
├─────────────────────────────────────────────┤
│  Layer 3: GfxContext                        │
│  Render abstraction (WebGL today, WebGPU    │
│  tomorrow), shader management, texture      │
│  upload, draw call batching                 │
├─────────────────────────────────────────────┤
│  Layer 2: Protocol                          │
│  WebSocket connection, wire framing,        │
│  Flatbuf/Protobuf codec, client-side        │
│  prediction, server reconciliation          │
├─────────────────────────────────────────────┤
│  Layer 1: Core                              │
│  Shared types, math utils (Vec3, Quat),     │
│  time sync, Rapier WASM physics, config     │
└─────────────────────────────────────────────┘
```

### Layer Rules

1. **Downward only** — Layer N imports from Layer N-1. Never upward, never skip layers.
2. **Interface contracts** — Each layer boundary has an explicit interface (trait in Rust, TypeScript interface on client). Implementations are swappable.
3. **No cross-layer data types** — Data crossing a boundary is transformed at the boundary. Layer 3 doesn't know Layer 5's types.
4. **Testing per layer** — Each layer is testable in isolation with mocked lower layers.
5. **One responsibility** — If a module touches two layers, split it.

## Consequences

- `world/programs/node-manager/` and `world/programs/spatial/` will be rewritten from Python to Rust
- Phase 0 Python prototype code serves as reference but is superseded
- `engine/programs/renderer/` remains TypeScript/R3F — no change
- Rapier WASM is added to client dependencies (`@dimforge/rapier3d-compat`)
- `rapier3d` is added to server Cargo dependencies
- `bevy_ecs` or `hecs` is added to server Cargo dependencies (final choice at implementation start)
- Agent service and knowledge service remain Python (separate processes, I/O-bound work)
- Gateway language decision deferred to ADR-007
- drei restricted to devDependencies — lint rule to prevent import in production bundles
