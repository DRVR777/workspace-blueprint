---
name: renderer
parent: engine
type: sub-program
status: active
phase: 0
layer: 2-3 (platform abstraction + engine primitive)
---

# renderer — Frame Production

**What this sub-program is**: The loop that turns the client's world state into pixels. It runs as fast as the hardware allows (targeting 60+ FPS) and is entirely independent of the server tick rate. The renderer is a pure consumer — it reads the client world state and draws it; it never modifies world state.

**What it owns**:
- The main render loop (runs once per frame)
- The scene graph (the hierarchy of what needs to be drawn this frame)
- Draw call batching (combine many similar objects into fewer GPU operations)
- Lighting computation (directional sun, ambient, point lights from world objects)
- Atmosphere and fog (distance fog that hides the LOD transition zone)
- Post-processing effects (applied after the scene is rendered — bloom, color grading, anti-aliasing)
- The camera (position, orientation, field of view, near/far clip planes)
- Terrain rendering (special-cased: terrain has different rendering needs than objects)

**What it does NOT own**:
- LOD tier selection (that is lod/)
- Asset loading from disk or network (that is asset-pipeline/)
- Visibility culling (that is visibility/ — it hands renderer a pre-culled list)
- World state (it reads from engine's local world state snapshot, never writes to it)
- Physics (that is local-simulation/)

**The render loop**:

```
EACH FRAME:

  frame_start = now()

  [Step 1] Snapshot client world state
    state = client_world_state.snapshot()
    -- Shallow copy of positions and states — the world state can be updated
    -- by the network thread concurrently; we snapshot to avoid mid-frame inconsistency

  [Step 2] Update camera
    camera.position = state.player.position + camera_offset
    camera.orientation = state.player.look_orientation
    camera.update_matrices()  -- compute view and projection matrices

  [Step 3] Get visible object list from visibility/
    visible_objects = visibility_system.cull(state.nearby_objects, camera)
    -- Returns list of (object_id, distance, lod_tier) sorted front-to-back

  [Step 4] Get visible entity list from visibility/
    visible_entities = visibility_system.cull_entities(state.nearby_entities, camera)

  [Step 5] Build draw batches
    draw_batches = {}  -- keyed by (object_type_id, lod_tier, material)
    FOR EACH obj in visible_objects:
      geometry = asset_cache.get(obj.type_id, obj.lod_tier)
      IF geometry == PENDING:
        -- geometry not yet cached; use best available tier
        geometry = asset_cache.get_best_available(obj.type_id)
      IF geometry == null:
        SKIP  -- nothing to draw yet
      batch_key = (obj.type_id, obj.lod_tier, geometry.material_id)
      draw_batches[batch_key].ADD(transform_matrix(obj.position, obj.orientation))

  [Step 6] Render terrain
    FOR EACH loaded chunk in state.terrain:
      IF chunk_in_frustum(chunk, camera):
        render_terrain_chunk(chunk)

  [Step 7] Submit draw batches to GPU
    FOR EACH batch in draw_batches:
      bind_shader(batch.material.shader)
      upload_instance_transforms(batch.transforms)  -- instanced rendering
      draw_instanced(batch.geometry, batch.transforms.count)

  [Step 8] Render entities (separate pass — entities may need skinning/animation)
    FOR EACH entity in visible_entities:
      render_entity(entity, state)

  [Step 9] Post-processing
    apply_atmosphere_fog(depth_buffer, camera.far_clip)
    apply_post_effects(color_buffer)  -- bloom, anti-aliasing, color grade

  [Step 10] Present
    swap_buffers()

  frame_duration = now() - frame_start
  metrics.record_frame_time(frame_duration)
```

**Key rendering technique: GPU Instancing**
Objects of the same type at the same LOD tier are rendered in a single draw call by uploading all their transform matrices to the GPU as an instance buffer. This means 10,000 trees of the same type cost approximately the same as 1 tree. This is how the engine can render dense worlds at high frame rates.

**Phase 0 scope (minimal)**:
- Render a flat terrain quad at 60 FPS
- Render player avatar (simple capsule shape)
- Render other player avatars as position-synced capsules
- No LOD (single geometry tier for everything)
- No post-processing
- Simple directional lighting only (sun + ambient)
- No atmosphere/fog

**Phase 1 additions**:
- Instanced rendering of world objects
- LOD blending
- Atmospheric fog (hides LOD pop at distance)
- Impostor rendering (billboarded sprites for far objects)

**Performance target**:
- Phase 0: 60 FPS with 1 player and flat terrain on target hardware
- Phase 1: 60 FPS with 100 objects in view at mixed LOD tiers
- Phase 3: 60 FPS with 1,000 objects in view, 50 other players visible
