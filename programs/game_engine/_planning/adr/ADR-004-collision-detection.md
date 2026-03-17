# ADR-004: Collision Detection — Library vs Custom
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
The ELEV8 post-mortem identified "physics reinvention" as a fatal mistake. However, ELEV8 reinvented a *complete force-directed graph physics system* — a much larger scope than collision detection alone. The question here is narrower: should we implement collision detection algorithms ourselves (GJK, SAT, BVH), or use a proven library?

Key considerations:
- Collision detection is well-understood mathematically but has many edge cases (parallel faces, degenerate shapes, numerical precision)
- A physics library typically bundles integration + collision + constraints — we only need collision detection, because integration is specified separately in ADR-003
- The collision detection interface must be replaceable (see modularity spec, Part XV)
- We must be able to run collision detection on the server (node simulation) AND the client (local prediction)
- The client runs in a browser environment — not all native physics libraries can run there

## Options Considered

**Option A: Implement GJK + SAT + BVH from scratch**
- Full control over algorithms
- No dependency overhead
- High implementation risk: GJK has corner cases that take weeks to debug
- Time cost: 2-4 weeks minimum for a production-quality implementation
- Risk: Exactly the ELEV8 mistake — not in spirit (we specified it first) but in execution cost

**Option B: Use a full physics library (handles integration + collision)**
- Eliminates the integration we already decided in ADR-003
- Tight coupling to one library's model — hard to swap
- Usually overkill and opinionated about the whole physics pipeline

**Option C: Use a collision-only library / algorithm collection**
- Gets proven collision detection without coupling to a full physics pipeline
- Can run on server and (with appropriate wrapper) in browser
- The interface wraps the library — the library is swappable

**Option D: Use the physics engine embedded in the rendering technology (if chosen)**
- Cannon.js, Rapier, etc. are available for browser 3D
- Risk: ties physics to rendering technology — violates layer separation

## Decision

**Option C: Use a proven collision detection library, wrapped behind the simulation-contract interface.**

Specifically:
- The collision detection implementation is hidden behind the `simulation-contract`
- Internally, it uses a proven library (exact library chosen at implementation time based on server/client platform requirements)
- The library must be capable of: sphere-sphere, box-box, sphere-box, convex-convex tests
- It must support BVH (Bounding Volume Hierarchy) for broad-phase acceleration
- It must be deterministic (same inputs → same outputs)

**What is NOT decided here**: the specific library. That is an implementation decision made when Phase 0 begins, based on what runs correctly on both server and browser with the required shape types.

## Anti-pattern Guard

This is explicitly NOT the ELEV8 mistake because:
1. The algorithm is specified (what it must do)
2. The interface is defined (simulation-contract)
3. The implementation is delegated to a proven library
4. The library is swappable through the interface

ELEV8's mistake was building the physics engine *from scratch with no specification*. This ADR specifies the behavior completely and selects a proven implementation.

## Consequences

- Collision detection is tested independently of the rest of the simulation
- Swapping the collision library requires only changing the code behind `simulation-contract` — no other subsystem is affected
- Sub-stepping (running collision detection multiple times per tick for fast objects) is supported if the library supports continuous collision detection (CCD)
- The broad-phase BVH is rebuilt from the node's octree each tick — not separately maintained
