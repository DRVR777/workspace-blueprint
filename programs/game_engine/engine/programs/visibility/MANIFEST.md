---
name: visibility
parent: engine
type: sub-program
status: active
phase: 0 (stub) → 1 (full)
layer: 3 (engine primitive)
depends-on: none (pure computation over inputs)
---

# visibility — Frustum Culling, Occlusion, and Distance Management

**What this sub-program is**: The gate between the world state and the renderer. Every frame, it takes the set of nearby objects/entities (already scoped by the server's interest management) and the current camera, and returns only the subset that is actually visible — inside the camera frustum, within render distance, and not fully occluded. The renderer draws exactly what visibility returns. Nothing more, nothing less.

**What it owns**:
- Frustum construction from camera view/projection matrices
- Frustum-vs-bounding-volume tests (sphere, AABB)
- Distance-based cutoff (render distance, per-object-type overrides)
- LOD tier assignment delegation (calls lod-system-contract, returns tier with each result)
- Terrain chunk frustum testing
- Front-to-back sorting (for optimal GPU early-Z rejection)
- Occlusion culling (Phase 2+: terrain-based and BVH-based)

**What it does NOT own**:
- LOD tier threshold math (that is lod/ — visibility calls `assign_tier`)
- The camera itself (renderer owns camera; visibility receives it as input)
- The nearby object set (server's interest management + client world state provide this)
- Rendering (renderer draws what visibility returns)
- Asset loading (asset-pipeline handles geometry fetching)

---

## Public Interface

This is the contract the renderer calls every frame.

### Primary: Object culling

```
cull(
  nearby_objects: list of object_record,
  camera: CameraState
) → list of CulledObject
```

- Receives the client's current nearby_objects (pre-filtered by server visibility radius)
- Returns only those inside the frustum AND within render distance
- Sorted front-to-back by distance from camera
- Each result includes the LOD tier (from lod-system-contract)

### Primary: Entity culling

```
cull_entities(
  nearby_entities: list of entity_record,
  camera: CameraState
) → list of CulledEntity
```

- Same logic as `cull` but for entities (players, NPCs)
- Entities have larger effective visibility distance than objects (players should never pop in suddenly)

### Primary: Terrain chunk culling

```
cull_terrain(
  loaded_chunks: list of terrain_chunk,
  camera: CameraState
) → list of CulledChunk
```

- Tests each terrain chunk's AABB against the frustum
- Returns visible chunks with distance and LOD tier
- Terrain LOD is distance-based: near chunks get full heightmap, far chunks get simplified

### Configuration

```
set_render_distance(distance: float32) → void
  -- Global maximum render distance (default: 5,000 units, matches LOD invisible threshold)
  -- Objects beyond this are never returned regardless of frustum

set_entity_render_distance(distance: float32) → void
  -- Override for entities (default: 1.5x global render distance)
  -- Players and NPCs visible further than static objects

set_quality_mode(mode: QualityMode) → void
  -- LOW:    render_distance * 0.5, skip occlusion
  -- MEDIUM: render_distance * 0.75
  -- HIGH:   render_distance * 1.0, full occlusion
  -- ULTRA:  render_distance * 1.5, full occlusion + extended entity distance
```

---

## Data Shapes

### CameraState (input — constructed by renderer from R3F camera)

```
CAMERA_STATE:
  position:        Vec3f64      -- camera world position
  forward:         Vec3f32      -- camera forward direction (unit vector)
  up:              Vec3f32      -- camera up direction (unit vector)
  right:           Vec3f32      -- camera right direction (unit vector)
  fov_y:           float32      -- vertical field of view in radians
  aspect_ratio:    float32      -- viewport width / height
  near_clip:       float32      -- near clipping plane distance
  far_clip:        float32      -- far clipping plane distance
  view_matrix:     Mat4f32      -- precomputed view matrix
  projection_matrix: Mat4f32    -- precomputed projection matrix
```

### CulledObject (output)

```
CULLED_OBJECT:
  object_id:       uint64
  type_id:         uint32
  position:        Vec3f64
  orientation:     Quat32
  distance:        float32       -- distance from camera (used for sorting + LOD)
  lod_tier:        uint8         -- 0-4 from lod-system-contract, or 5=INVISIBLE (filtered out)
  lod_blend:       lod_blend_result  -- from assign_tier_with_blend (Phase 1+)
  in_transition:   bool          -- true if LOD blending is active (render both tiers)
```

### CulledEntity (output)

```
CULLED_ENTITY:
  entity_id:       uint64
  player_id:       uint64        -- 0 for NPCs
  position:        Vec3f64
  orientation:     Quat32
  velocity:        Vec3f32
  distance:        float32
  lod_tier:        uint8
  is_local_player: bool          -- true for the player's own entity (always visible, special render)
```

### CulledChunk (output)

```
CULLED_CHUNK:
  sector_coords:   Vec3i32
  chunk_coords:    Vec3i32
  distance:        float32       -- distance from camera to chunk center
  lod_tier:        uint8         -- terrain LOD (0 = full heightmap, 1+ = simplified)
```

---

## Frustum Construction and Testing

### Building the frustum

The frustum is 6 planes extracted from the combined view-projection matrix. This is the standard Griess-Hartmann method — extract directly from the matrix rows, no need to compute plane intersections.

```
extract_frustum_planes(vp_matrix: Mat4f32) → Frustum

  -- vp = projection * view
  -- Each plane is Ax + By + Cz + D >= 0 (inside)
  row0 = vp_matrix.row(0)
  row1 = vp_matrix.row(1)
  row2 = vp_matrix.row(2)
  row3 = vp_matrix.row(3)

  planes[LEFT]   = normalize_plane(row3 + row0)
  planes[RIGHT]  = normalize_plane(row3 - row0)
  planes[BOTTOM] = normalize_plane(row3 + row1)
  planes[TOP]    = normalize_plane(row3 - row1)
  planes[NEAR]   = normalize_plane(row3 + row2)
  planes[FAR]    = normalize_plane(row3 - row2)

  RETURN Frustum { planes }
```

### Sphere-vs-frustum test

```
sphere_in_frustum(center: Vec3f32, radius: float32, frustum: Frustum) → CullResult

  FOR EACH plane in frustum.planes:
    signed_distance = dot(plane.normal, center) + plane.d
    IF signed_distance < -radius:
      RETURN OUTSIDE          -- fully outside this plane → invisible
    IF signed_distance < radius:
      result = INTERSECTING   -- partially inside → needs further testing or assume visible

  RETURN INSIDE               -- fully inside all planes → definitely visible

  -- CullResult: OUTSIDE | INTERSECTING | INSIDE
```

### AABB-vs-frustum test (for terrain chunks and objects with AABB)

```
aabb_in_frustum(min: Vec3f32, max: Vec3f32, frustum: Frustum) → CullResult

  FOR EACH plane in frustum.planes:
    -- Find the "positive vertex" (corner most aligned with plane normal)
    p_vertex = Vec3(
      plane.normal.x >= 0 ? max.x : min.x,
      plane.normal.y >= 0 ? max.y : min.y,
      plane.normal.z >= 0 ? max.z : min.z,
    )
    IF dot(plane.normal, p_vertex) + plane.d < 0:
      RETURN OUTSIDE

    -- Find the "negative vertex" (corner least aligned with plane normal)
    n_vertex = Vec3(
      plane.normal.x >= 0 ? min.x : max.x,
      plane.normal.y >= 0 ? min.y : max.y,
      plane.normal.z >= 0 ? min.z : max.z,
    )
    IF dot(plane.normal, n_vertex) + plane.d < 0:
      result = INTERSECTING

  RETURN INSIDE
```

---

## The Cull Pipeline

```
cull(nearby_objects, camera) → list of CulledObject

  frustum = extract_frustum_planes(camera.projection_matrix * camera.view_matrix)
  render_dist = current_render_distance
  results = []

  FOR EACH obj in nearby_objects:

    -- [Gate 1] Distance check (cheapest — scalar comparison)
    dist = distance(camera.position, obj.position)
    IF dist > render_dist:
      CONTINUE

    -- [Gate 2] LOD tier check (determines if object is worth rendering)
    lod_result = lod_system.assign_tier_with_blend(obj.type_id, dist)
    IF lod_result.current_tier == INVISIBLE:
      CONTINUE

    -- [Gate 3] Frustum test (sphere test using bounding box radius)
    bounding_radius = max_extent(obj.bounding_box) * 0.5
    cull_result = sphere_in_frustum(obj.position, bounding_radius, frustum)
    IF cull_result == OUTSIDE:
      CONTINUE

    -- [Gate 4] Occlusion test (Phase 2+ — skip for now)
    -- IF is_occluded(obj.position, bounding_radius, camera):
    --   CONTINUE

    -- Passed all gates — add to results
    results.push(CulledObject {
      object_id:    obj.id,
      type_id:      obj.type_id,
      position:     obj.position,
      orientation:  obj.orientation,
      distance:     dist,
      lod_tier:     lod_result.current_tier,
      lod_blend:    lod_result,
      in_transition: lod_result.needs_blend,
    })

  -- Sort front-to-back by distance (enables GPU early-Z rejection)
  results.sort_by(|a, b| a.distance.compare(b.distance))

  RETURN results
```

### Gate ordering rationale

Gates are ordered cheapest-to-most-expensive so that the maximum number of objects are eliminated before reaching expensive tests:

1. **Distance** — single `distance()` call, scalar comparison. Eliminates everything beyond render distance.
2. **LOD tier** — lookup + distance comparison. Eliminates objects that would be INVISIBLE at this distance.
3. **Frustum** — 6 dot products per plane. Eliminates everything outside the camera view.
4. **Occlusion** — raycast or query (expensive). Only runs on objects that passed all previous gates.

### Entity culling

Same pipeline as `cull()` but with these differences:

```
cull_entities(nearby_entities, camera) → list of CulledEntity

  -- Same frustum, but:
  -- 1. Use entity_render_distance (1.5x global) instead of render_distance
  -- 2. Local player is ALWAYS included (skip all gates)
  -- 3. Entities skip LOD tier check (entities don't LOD in Phase 0-1)
  -- 4. Entities use a larger bounding radius (capsule height, not tight AABB)

  FOR EACH entity in nearby_entities:
    IF entity.player_id == local_player_id:
      -- Always visible — skip culling
      results.push(CulledEntity { ..., is_local_player: true })
      CONTINUE

    dist = distance(camera.position, entity.position)
    IF dist > entity_render_distance:
      CONTINUE

    cull_result = sphere_in_frustum(entity.position, ENTITY_BOUNDING_RADIUS, frustum)
    IF cull_result == OUTSIDE:
      CONTINUE

    results.push(CulledEntity {
      entity_id:     entity.id,
      player_id:     entity.player_id,
      position:      entity.position,
      orientation:   entity.orientation,
      velocity:      entity.velocity,
      distance:      dist,
      lod_tier:      0,  -- Phase 0: always full detail
      is_local_player: false,
    })

  results.sort_by(|a, b| a.distance.compare(b.distance))
  RETURN results
```

### Terrain chunk culling

```
cull_terrain(loaded_chunks, camera) → list of CulledChunk

  frustum = extract_frustum_planes(camera.projection_matrix * camera.view_matrix)
  results = []

  FOR EACH chunk in loaded_chunks:
    chunk_center = chunk_world_center(chunk.sector_coords, chunk.chunk_coords)
    dist = distance(camera.position, chunk_center)

    -- Distance gate
    IF dist > render_distance * TERRAIN_DISTANCE_MULTIPLIER:
      CONTINUE

    -- AABB frustum test
    chunk_aabb = chunk_world_aabb(chunk)
    IF aabb_in_frustum(chunk_aabb.min, chunk_aabb.max, frustum) == OUTSIDE:
      CONTINUE

    -- Terrain LOD (simple distance-based)
    terrain_lod = compute_terrain_lod(dist)

    results.push(CulledChunk {
      sector_coords: chunk.sector_coords,
      chunk_coords:  chunk.chunk_coords,
      distance:      dist,
      lod_tier:      terrain_lod,
    })

  results.sort_by(|a, b| a.distance.compare(b.distance))
  RETURN results
```

---

## Server-Side Interest Management (Relationship)

The visibility system operates on the **client** side. The **server** performs a coarser filter called interest management:

```
SERVER (node-manager, broadcast Phase D):
  FOR EACH client C:
    relevant_changes = filter by distance(change.position, C.player.position) <= C.visibility_radius

CLIENT (visibility system, every frame):
  FOR EACH object in nearby_objects (already filtered by server):
    apply distance gate, LOD gate, frustum gate, occlusion gate
```

The server's `visibility_radius` (default 500 units per PRD §11.3) is **always larger** than the client's `render_distance` (default 5,000 units matching LOD invisible threshold). This means the server sends more than the client renders — the client's frustum cull removes the excess. This is intentional:

- Server broadcasts a wider radius so objects entering the view don't pop in
- Client frustum-culls the excess so the GPU only renders what's on screen
- The gap between server radius and render distance provides a "warm zone" where objects are tracked but not drawn, enabling instant visibility when the camera turns

### Dynamic visibility radius (server-side, per PRD §11.3)

```
compute_visibility_radius(player, node_load) → float32

  base = 500.0                    -- units (= half the sector size, ADR-001)
  load_scale = 1.0 - (node_load * 0.3)   -- at 100% load, radius shrinks to 70%
  speed_bonus = clamp(magnitude(player.velocity) / 50.0, 0.0, 0.5)  -- fast → see further

  RETURN base * max(load_scale + speed_bonus, 0.5)
  -- Floor at 50% of base (250 units) even under extreme load
```

This is computed by the **node-manager**, not by the visibility sub-program. Documented here for architectural context only.

---

## Occlusion Culling (Phase 2+)

Phase 0 and 1 do not perform occlusion culling. Phase 2 adds two occlusion strategies:

### Strategy 1: Terrain height occlusion (cheap)

```
is_terrain_occluded(obj_position, camera_position, terrain_heightmap) → bool

  -- Cast ray from camera to object along XZ plane
  -- Sample terrain height at intervals along the ray
  -- If any terrain sample is higher than the ray at that point, object is occluded
  -- Fast: O(ray_length / sample_spacing) heightmap lookups, no geometry

  ray_dir = normalize(obj_position.xz - camera_position.xz)
  ray_length = distance_2d(camera_position.xz, obj_position.xz)
  sample_count = min(ray_length / OCCLUSION_SAMPLE_SPACING, MAX_OCCLUSION_SAMPLES)

  FOR i in 0..sample_count:
    t = i / sample_count
    sample_xz = lerp(camera_position.xz, obj_position.xz, t)
    ray_height = lerp(camera_position.y, obj_position.y, t)
    terrain_height = heightmap.sample(sample_xz)

    IF terrain_height > ray_height + OCCLUSION_MARGIN:
      RETURN true  -- terrain blocks line of sight

  RETURN false
```

### Strategy 2: BVH occlusion queries (expensive, large objects only)

Deferred to Phase 2. Uses GPU occlusion queries or a software rasterizer for the depth buffer. Only worth running for objects with bounding volume > MIN_OCCLUSION_TEST_SIZE.

---

## Constants

```
-- Render distances
DEFAULT_RENDER_DISTANCE       = 5000.0    -- units (matches LOD invisible threshold)
DEFAULT_ENTITY_RENDER_DISTANCE = 7500.0   -- 1.5x object render distance
TERRAIN_DISTANCE_MULTIPLIER   = 1.2       -- terrain visible 20% further than objects
ENTITY_BOUNDING_RADIUS        = 1.5       -- capsule approximation for entities

-- Terrain LOD thresholds
TERRAIN_LOD_0_DIST            = 200.0     -- full detail
TERRAIN_LOD_1_DIST            = 800.0     -- half resolution
TERRAIN_LOD_2_DIST            = 2000.0    -- quarter resolution
TERRAIN_LOD_3_DIST            = 4000.0    -- eighth resolution (flat mesh)

-- Occlusion (Phase 2+)
OCCLUSION_SAMPLE_SPACING      = 10.0      -- units between terrain height samples
MAX_OCCLUSION_SAMPLES         = 50        -- cap ray marching samples
OCCLUSION_MARGIN              = 2.0       -- units above terrain to account for imprecision
MIN_OCCLUSION_TEST_SIZE       = 5.0       -- units — only test large objects for BVH occlusion

-- Quality presets
QUALITY_LOW_MULTIPLIER        = 0.5
QUALITY_MEDIUM_MULTIPLIER     = 0.75
QUALITY_HIGH_MULTIPLIER       = 1.0
QUALITY_ULTRA_MULTIPLIER      = 1.5
```

---

## R3F Integration

This is a TypeScript sub-program running client-side. It integrates with React Three Fiber:

### Implementation shape

```typescript
// The visibility system is a plain TypeScript module, NOT a React component.
// It is called by the renderer's useFrame() hook every frame.
// It does not manage React state — it returns arrays that the renderer consumes.

interface VisibilitySystem {
  cull(nearbyObjects: ObjectRecord[], camera: CameraState): CulledObject[];
  cullEntities(nearbyEntities: EntityRecord[], camera: CameraState): CulledEntity[];
  cullTerrain(loadedChunks: TerrainChunk[], camera: CameraState): CulledChunk[];
  setRenderDistance(distance: number): void;
  setEntityRenderDistance(distance: number): void;
  setQualityMode(mode: QualityMode): void;
}
```

### How the renderer calls it (R3F)

```typescript
// Inside the renderer's main component:
useFrame(({ camera }) => {
  const cameraState = extractCameraState(camera);  // R3F camera → CameraState

  const visibleObjects = visibilitySystem.cull(worldState.nearbyObjects, cameraState);
  const visibleEntities = visibilitySystem.cullEntities(worldState.nearbyEntities, cameraState);
  const visibleChunks = visibilitySystem.cullTerrain(worldState.loadedChunks, cameraState);

  // Pass results to instanced mesh renderers, entity renderers, terrain renderers
  updateScene(visibleObjects, visibleEntities, visibleChunks);
});
```

### Why NOT a React component

Visibility runs every frame (60+ times per second). React's reconciliation is too slow for this. The visibility system is a pure function module — no hooks, no state, no re-renders. The renderer calls it imperatively inside `useFrame` and directly mutates Three.js objects (instance matrices, visibility flags) without going through React.

---

## Phase 0 Scope (Stub)

Phase 0 has very few objects (1 player, flat terrain). The visibility system ships as a **pass-through stub**:

```
Phase 0 cull():
  -- No frustum culling needed (< 10 objects)
  -- Return all objects with distance computed and LOD tier 0
  -- Sorted front-to-back

Phase 0 cull_entities():
  -- Return all entities
  -- Local player marked as is_local_player

Phase 0 cull_terrain():
  -- Return all loaded chunks (Phase 0 has 1 chunk)
```

The stub implements the full interface so the renderer doesn't need to change when real culling is added in Phase 1.

---

## Phase 1 Scope (Full)

- Frustum construction from R3F camera matrices
- Sphere-vs-frustum test for all objects
- AABB-vs-frustum test for terrain chunks
- Distance-based culling with configurable render distance
- LOD tier assignment via lod-system-contract
- LOD blend results passed through to renderer
- Front-to-back sorting
- Entity-specific render distance (1.5x)
- Quality mode presets (LOW/MEDIUM/HIGH/ULTRA)
- Terrain LOD (4 distance tiers)

---

## Performance Budget

Visibility runs every frame inside the renderer's 16.6ms budget (60 FPS). Target:

```
Phase 0 (stub):   < 0.1ms   -- trivial pass-through
Phase 1 (full):   < 1.0ms   -- frustum + distance + LOD for up to 2,000 objects
Phase 2 (occl.):  < 2.0ms   -- adds terrain occlusion for up to 5,000 objects
```

Breakdown at Phase 1 capacity (2,000 nearby objects):
- Distance computation: 2,000 × Vec3 distance = ~0.05ms
- LOD tier lookup: 2,000 × table lookup = ~0.02ms
- Frustum test: ~500 remaining × 6 dot products = ~0.1ms
- Sort: ~300 visible × front-to-back sort = ~0.05ms
- Total: ~0.3ms (well within 1.0ms budget)

---

## Testing Requirements

1. **Frustum extraction test**: Known view-projection matrix → verify 6 planes match expected normals/distances
2. **Sphere-in-frustum test**: Sphere at known positions → verify INSIDE/OUTSIDE/INTERSECTING results
3. **AABB-in-frustum test**: Box at known positions → verify correct cull results
4. **Distance gate test**: Object at render_distance + 1 → not returned. Object at render_distance - 1 → returned.
5. **LOD integration test**: Object at each LOD threshold distance → correct tier assigned in CulledObject
6. **Sort test**: 100 objects at random distances → output sorted ascending by distance
7. **Entity always-visible test**: Local player entity → always returned regardless of frustum
8. **Quality mode test**: Set quality LOW → render distance reduced to 50%. Objects beyond new distance not returned.
9. **Performance test**: 2,000 objects, random positions → cull completes in < 1.0ms
