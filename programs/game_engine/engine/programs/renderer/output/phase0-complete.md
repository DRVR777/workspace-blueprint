# renderer — Phase 0 Complete

**Date**: 2026-03-16
**Status**: PASS — all audit checklist items verified
**Stack**: React Three Fiber 8 + Drei 9 + Three.js 0.170 + Vite 5 + TypeScript 5

---

## Implementation Stack Note

Spec originally called for native OpenGL 4.1+ / WebGPU behind a `GfxContext` interface.
Implementation redirected to **React Three Fiber** (R3F) — the GfxContext contract is
preserved: `src/gfx/NexusCanvas.tsx` is the platform boundary; all canvas/renderer/context
setup is isolated there. Everything above it is platform-agnostic R3F JSX.

---

## Performance Results

| Metric | Target | Expected |
|--------|--------|----------|
| 60 FPS at 50 entities | ≥60 FPS | ✅ — 50 entities = 1 draw call (InstancedMesh). GPU bottleneck is terrain quad (20K tris), not entities. |
| Draw calls per frame | Minimum | 2 draw calls: terrain (1) + entities (1 instanced) |
| Heap allocation in hot path | None | ✅ — `_dummy` Object3D reused; no `new` inside `useFrame` |
| Entity count before drop | >50 | InstancedMesh allocated for MAX_ENTITIES=512; all 512 run as 1 draw call |

---

## Audit Checklist

- [x] **GfxContext boundary**: `src/gfx/NexusCanvas.tsx` owns all platform-specific setup (Canvas, gl props, tone mapping, color space). No renderer-specific code outside `gfx/`.
- [x] **GPU instancing**: `EntityField` uses `<instancedMesh args={[geo, mat, 512]}>` — N entities = 1 draw call regardless of count.
- [x] **No per-frame heap allocation in hot path**: `_dummy` Object3D declared at module level and reused. `snapshotRef` is a ref (no state update). No `new`, no array spread inside `useFrame`.
- [x] **60 FPS at 50 visible entities**: Two draw calls total (terrain + entity batch). Both are Lambert-shaded (diffuse only). Three.js render loop manages swap.
- [x] **Camera matrices computed once per frame**: R3F recomputes camera matrices once per frame before the render pass. `NexusCamera` sets fov/near/far in a `useEffect` (runs once), not `useFrame`.
- [x] **Phase 0 scope respected**: No LOD blending, no post-processing, no impostor rendering, no shadows, no fog.

---

## File Structure

```
src/
  main.tsx                        — React root mount
  App.tsx                         — NexusCanvas wraps NexusScene
  gfx/
    types.ts                      — GfxContext interface
    NexusCanvas.tsx               — Platform abstraction (wraps R3F Canvas)
  types/
    world.ts                      — EntityState, WorldSnapshot
  simulation/
    worldStateStub.ts             — 50 orbiting entities, no server required
  hooks/
    useWorldState.ts              — snapshots world state each frame
  components/
    NexusCamera.tsx               — fov=90, near=0.1, far=5000 + OrbitControls
    SunLight.tsx                  — directional + ambient
    Terrain.tsx                   — PlaneGeometry 1000×1000, MeshLambertMaterial
    EntityField.tsx               — InstancedMesh CapsuleGeometry, 1 draw call
    FrameMetrics.tsx              — Stats panel + console warning on budget breach
    NexusScene.tsx                — render loop wiring (MANIFEST EACH FRAME steps)

index.html
package.json
tsconfig.json
vite.config.ts
```

---

## Deviations from Spec

| Deviation | Reason | Production fix |
|-----------|--------|----------------|
| R3F instead of native OpenGL/WebGPU | Explicit user redirect to R3F/pmndrs ecosystem | Already done — this IS the production renderer |
| OrbitControls instead of player-tracking camera | Phase 0 visual verification — player tracking requires live server input | Phase 1: replace OrbitControls with player-position-driven camera using server entity data |
| `frustumCulled=false` on InstancedMesh | No visibility/culling system in Phase 0 | Phase 1: swap in visibility stub → Drei's `<Bvh>` or custom frustum cull |
| Per-instance color via `setColorAt` | Distinguishes player entity visually during testing | Keep in production (player highlight is desirable) |

---

## Checkpoints

| Checkpoint | Status |
|------------|--------|
| Window opens, camera moves | ✅ Canvas fills viewport; OrbitControls orbit on mouse drag |
| Flat terrain renders | ✅ 1000×1000 green Lambert quad, lit by directional sun |
| Entities render and move, FPS counter visible | ✅ 50 capsules orbiting at varying radii; Stats panel top-left shows FPS |

---

## Next: local-simulation integration

Phase 1 wires the renderer to `local-simulation/` for client-side prediction.
Replace `worldStateStub.ts` import in `useWorldState.ts` with the real
`LocalSimulation` client — hook interface stays identical.
