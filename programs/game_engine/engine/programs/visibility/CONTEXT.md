# visibility — Build Contract

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: cull pipeline, frustum math, data shapes, R3F integration |
| `../../shared/contracts/lod-system-contract.md` | assign_tier, assign_tier_with_blend — called per object |
| `../../shared/contracts/world-state-contract.md` | object_record, entity_record, terrain_chunk shapes |
| `../renderer/MANIFEST.md` | The caller: renderer invokes cull() in useFrame() |
| `../local-simulation/MANIFEST.md` | Provides predicted entity positions consumed by cull_entities |

Do NOT load: world/ files, server-side code, network schemas.

---

## Process

### Phase 0 (Stub)

1. **Create TypeScript module** `src/visibility.ts`:
   - Implement `VisibilitySystem` interface (cull, cullEntities, cullTerrain, setRenderDistance, setEntityRenderDistance, setQualityMode)
   - `cull()`: compute distance for each object, assign LOD tier 0, sort front-to-back, return all
   - `cullEntities()`: return all entities with distance computed, mark local player
   - `cullTerrain()`: return all loaded chunks
   - Export a singleton instance

2. **Create type definitions** `src/types.ts`:
   - CameraState, CulledObject, CulledEntity, CulledChunk, QualityMode
   - Ensure shapes match MANIFEST.md data shapes exactly

3. **Wire into renderer**:
   - Import visibility system in renderer's main component
   - Call `cull()` / `cullEntities()` / `cullTerrain()` inside `useFrame()` hook
   - Pass results to existing rendering logic

4. **Write tests** (tests 4, 6, 7 from MANIFEST.md — the ones that apply to the stub):
   - Distance gate, sort order, entity always-visible

5. **Write `output/phase0-complete.md`**: stub passing, renderer wired, all Phase 0 tests passing.

### Phase 1 (Full)

6. **Implement frustum extraction**:
   - `extractFrustumPlanes(vpMatrix: Mat4)` → 6 normalized planes
   - Test with known matrices

7. **Implement frustum tests**:
   - `sphereInFrustum(center, radius, frustum)` → OUTSIDE | INTERSECTING | INSIDE
   - `aabbInFrustum(min, max, frustum)` → OUTSIDE | INTERSECTING | INSIDE

8. **Implement full cull pipeline** (4-gate: distance → LOD → frustum → occlusion stub):
   - Wire LOD system contract calls (assign_tier_with_blend)
   - Populate lod_blend and in_transition fields in CulledObject
   - Apply quality mode multiplier to render distance

9. **Implement entity-specific culling**:
   - Extended render distance (1.5x)
   - Larger bounding radius (capsule approximation)
   - Skip LOD for Phase 1

10. **Implement terrain chunk culling**:
    - AABB frustum test per chunk
    - Terrain LOD tiers (4 distance bands)

11. **Write all remaining tests** (tests 1-3, 5, 8, 9 from MANIFEST.md):
    - Frustum extraction, sphere test, AABB test, LOD integration, quality mode, performance

12. **Write `output/phase1-complete.md`**: full frustum culling, LOD integration, performance verified.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 3 | Stub wired into renderer — no visual change, but cull() called every frame | approve / revise interface |
| Step 8 | Full frustum culling — objects behind camera not rendered | approve → entities + terrain / revise |
| Step 11 | All 9 tests passing, performance within budget | approve → write output / fix failures |

---

## Audit

Before writing to `output/` (Phase 0):
- [ ] VisibilitySystem interface matches MANIFEST.md exactly
- [ ] Stub returns all objects with correct distance values
- [ ] Front-to-back sort is correct (ascending distance)
- [ ] Local player entity always returned with is_local_player = true
- [ ] Renderer calls cull() inside useFrame(), not in React render cycle
- [ ] No React state or hooks inside visibility module (pure TypeScript)

Before writing to `output/` (Phase 1):
- [ ] Frustum planes extracted correctly from view-projection matrix
- [ ] Objects fully outside frustum are not returned
- [ ] Objects at render_distance + 1 are not returned
- [ ] LOD tier matches lod-system-contract for each distance
- [ ] Quality mode multiplier correctly scales render distance
- [ ] Entity render distance is 1.5x object render distance
- [ ] Terrain chunks use AABB test, not sphere test
- [ ] All 9 tests pass
- [ ] 2,000 objects culled in < 1.0ms

---

## Outputs

| Output | Location |
|--------|----------|
| Visibility module | `src/visibility.ts` |
| Type definitions | `src/types.ts` |
| Tests | `src/__tests__/` |
| Phase 0 completion summary | `output/phase0-complete.md` |
| Phase 1 completion summary | `output/phase1-complete.md` |
