# renderer — Build Contract (Phase 0)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: render loop, instancing, Phase 0 scope |
| `../../shared/contracts/world-state-contract.md` | entity_record, terrain_chunk shapes (read-only access) |
| `../../shared/contracts/lod-system-contract.md` | assign_tier interface (Phase 0: stub — always return tier 0) |
| `../../shared/schemas/entity_position_update.fbs` | incoming position update shape |

Do NOT load: world/ files, node-manager, simulation, asset schemas beyond what renderer reads.

---

## Process

1. Set up the render window and graphics context (platform-native: OpenGL 4.1+ or WebGPU). This is the only platform-specific step — abstract it behind a `GfxContext` interface so the rest of the renderer is platform-agnostic.

2. Implement the camera: position, look-orientation, field of view (90°), near clip (0.1), far clip (5000). Compute view and projection matrices each frame. Expose `update(player_position, player_orientation)`.

3. Implement terrain rendering (Phase 0 scope: flat quad):
   - Generate a flat 1000×1000 unit mesh (grid of triangles, 1 unit per cell)
   - Upload once to GPU as a static vertex buffer
   - Render with simple diffuse shader (sun direction + ambient)

4. Implement entity rendering (Phase 0 scope: capsule primitives):
   - Procedurally generate a capsule mesh (cylinder + two hemispheres, ~200 triangles)
   - Upload once as a static mesh
   - Each frame: for each visible entity, compute model matrix from position + orientation
   - Use GPU instancing: upload all entity transform matrices as an instance buffer, render in one draw call
   - Player avatar is rendered the same way (no special treatment in Phase 0)

5. Implement the render loop (MANIFEST.md §"EACH FRAME") for Phase 0:
   - Steps 1–2: snapshot state + update camera
   - Step 3: no visibility system yet — treat all entities in `nearby_entities` as visible
   - Steps 5–8: build entity draw batch + terrain render + instanced entity draw
   - Skip post-processing (Step 9) in Phase 0
   - Step 10: swap buffers

6. Measure frame time. Log to `output/` when consistently above 16.67ms (60 FPS threshold). Phase 0 target: 60 FPS with 50 entities on target hardware.

7. Write `output/phase0-complete.md`: average FPS, entity count tested, GPU draw call count per frame, max entity count before dropping below 60 FPS.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 2 | Window opens, camera moves — screenshot or description of blank scene | approve / fix setup |
| Step 3 | Flat terrain renders — screenshot or description | approve / adjust |
| Step 6 | Entities render and move — FPS counter visible in output | approve → write summary / fix performance |

---

## Audit

Before writing to `output/`:
- [ ] `GfxContext` interface abstracts all platform-specific code — renderer source has no `#ifdef PLATFORM` blocks
- [ ] Entity rendering uses GPU instancing (single draw call for N entities of same type)
- [ ] No per-frame heap allocation inside the render loop hot path (verify with allocator trace or code review)
- [ ] 60 FPS achieved at 50 visible entities on target hardware (logged in output)
- [ ] Camera matrices computed once per frame (not per draw call)
- [ ] Phase 0 scope respected: no LOD blending, no post-processing, no impostor rendering

---

## Outputs

| Output | Location |
|--------|----------|
| Renderer implementation | `src/` |
| GfxContext abstraction | `src/gfx/` |
| Phase 0 performance summary | `output/phase0-complete.md` |
