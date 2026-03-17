---
name: lod-system-contract
status: accepted
version: 0.1
published_by: engine/lod/
consumed_by: engine/renderer/, engine/asset-pipeline/
---

# LOD System Contract

The LOD system is the arbiter of geometric detail. Given an object and a camera position, it returns exactly what detail level should be rendered and whether a transition blend is needed.

## What This Contract Provides

`assign_tier(object_type_id, distance_from_camera)` → lod_tier (0-4 or INVISIBLE)
  - Pure function — no side effects
  - Deterministic: same inputs always produce same output
  - Uses the object type's configured thresholds scaled by the global LOD quality factor

`assign_tier_with_blend(object_type_id, distance_from_camera)` → lod_blend_result
  - Returns current tier AND a blend factor (0.0 to 1.0) for transition smoothing
  - blend_factor = 0.0 means fully in current tier (no blending)
  - blend_factor = 1.0 means fully transitioned to next tier
  - blend_factor between 0 and 1 means render both tiers at (1-t) and t opacity

`get_thresholds(object_type_id)` → list of 6 float distances [d0, d1, d2, d3, d4, d_invisible]
  - The distance at which tier transitions occur for this object type
  - d0 is always 0 (full detail starts at distance 0)

`set_quality_scale(scale_factor)` → void
  - Global multiplier applied to all thresholds
  - scale_factor 0.5 = everything simplifies at half the normal distance (performance mode)
  - scale_factor 2.0 = everything stays detailed twice as far (quality mode)
  - Default: 1.0

`register_object_type(object_type_id, thresholds)` → void
  - Called by the object type registry when a new type is loaded
  - If called with an already-registered type, updates thresholds (hot-reload support)

## lod_blend_result shape

```
LOD_BLEND_RESULT:
  current_tier: uint8 (0-4 or 5 for INVISIBLE)
  next_tier: uint8 (current_tier + 1, or same as current if in the middle of a tier)
  blend_factor: float32 (0.0 = fully current tier, 1.0 = fully next tier)
  needs_blend: bool (false if blend_factor < BLEND_THRESHOLD, as optimization)
```

## Default Distance Thresholds

These are used for any object type without a custom definition:

| Tier | Transition starts at |
|------|---------------------|
| 0→1 | 50 units |
| 1→2 | 200 units |
| 2→3 | 500 units |
| 3→4 | 1,500 units |
| 4→invisible | 5,000 units |

## What This Contract Does NOT Provide

- It does not trigger asset requests (the renderer or asset pipeline does that when it receives PENDING from the cache)
- It does not know whether geometry is available for a given tier (that is the asset cache's responsibility)
- It does not compute per-frame visibility (that is the visibility system's responsibility)
