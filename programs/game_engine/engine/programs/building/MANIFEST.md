---
name: building
parent: engine
type: sub-program
status: active
phase: 1
layer: 5 (experience)
depends-on: engine/renderer/, engine/visibility/, world/simulation/ (via network)
---

# building — Structural Construction System

**What this sub-program is**: The building system lets players construct custom structures — houses, towers, bridges, caves, anything — by placing, rotating, and connecting structural primitives on a fine grid. Walls can be angled. Floors can be multi-level. Ramps connect heights. Every placed piece becomes a static physics body in the server's Rapier world, visible and collidable for all players.

**What it owns**:
- Building primitives: wall, floor, ramp, stair, roof, door frame, window frame, column, wedge
- Placement mode: enter/exit build mode, ghost preview, grid snapping, rotation
- Grid system: 0.3m snap grid (≈1 foot), free rotation on Y axis, 15° snap on tilt
- Material system: per-piece material assignment (wood, stone, metal, glass, concrete)
- Structural validation: pieces must connect to something (no floating walls), max height limit
- Network integration: CREATE action → server validates → spawns static body → broadcasts
- Build mode UI: piece selector, rotation controls, material picker, undo/redo

**What it does NOT own**:
- Physics simulation (that is world/simulation/ — building just sends CREATE actions)
- Rendering of placed pieces (renderer handles static meshes once created)
- Object persistence (server writes to world graph)
- Terrain modification (separate system, Phase 2+)

---

## Building Primitives

Every primitive is a parametric shape defined by dimensions + rotation. No custom meshes in Phase 1 — everything is built from boxes, wedges, and cylinders.

### Primitive catalog

```
PRIMITIVE TYPES:

  WALL
    shape:      box
    default:    3.0m wide × 3.0m tall × 0.2m thick
    min:        0.3m × 0.3m × 0.1m
    max:        12.0m × 6.0m × 0.5m
    snap:       bottom edge snaps to floor top, side edges snap to adjacent walls
    rotation:   free Y rotation, can be angled for non-90° rooms

  FLOOR
    shape:      box
    default:    3.0m × 0.2m × 3.0m
    min:        0.3m × 0.1m × 0.3m
    max:        12.0m × 0.5m × 12.0m
    snap:       top surface at wall height intervals (0, 3.0m, 6.0m, ...)
    rotation:   free Y rotation

  RAMP
    shape:      wedge (triangular prism)
    default:    3.0m long × 3.0m rise × 1.5m wide
    min:        0.3m × 0.3m × 0.3m
    max:        12.0m × 6.0m × 6.0m
    snap:       bottom edge snaps to lower floor, top edge snaps to upper floor
    rotation:   free Y rotation, slope angle determined by rise/run ratio
    physics:    box collider tilted to match slope (Rapier handles angled statics)

  STAIR
    shape:      stepped wedge (visual) + ramp collider (physics)
    default:    3.0m long × 3.0m rise × 1.2m wide, 10 steps
    step_count: ceil(rise / 0.3)  — one step per grid unit
    rotation:   free Y rotation
    physics:    single angled box collider (same as ramp — player walks up smoothly)

  ROOF
    shape:      wedge (same geometry as ramp)
    default:    6.0m span × 2.0m peak × 0.15m thick
    variants:   flat (box), angled (wedge), peaked (two wedges mirrored)
    snap:       sits on top of walls
    rotation:   free Y rotation

  DOOR_FRAME
    shape:      wall with rectangular cutout
    default:    3.0m wide × 3.0m tall × 0.2m thick, opening 1.0m × 2.4m centered
    snap:       same as wall
    rotation:   same as wall
    note:       Phase 2 adds actual door objects (hinge joint, open/close)

  WINDOW_FRAME
    shape:      wall with rectangular cutout (higher than door)
    default:    3.0m wide × 3.0m tall × 0.2m thick, opening 1.2m × 1.2m at 1.0m height
    snap:       same as wall
    rotation:   same as wall
    material:   cutout can have glass material (transparent)

  COLUMN
    shape:      cylinder
    default:    0.3m radius × 3.0m tall
    min:        0.1m × 0.3m
    max:        1.0m × 12.0m
    snap:       bottom to floor, top to ceiling/floor above
    rotation:   none (cylindrical symmetry)

  WEDGE
    shape:      triangular prism (general purpose angled piece)
    default:    1.0m × 1.0m × 1.0m
    use:        fill gaps, create angled transitions, decorative corners
    rotation:   free on all axes
```

### Primitive data shape (sent to server as CREATE payload)

```
BUILDING_PIECE:
  piece_type:     uint8       — 0=wall, 1=floor, 2=ramp, 3=stair, 4=roof,
                                5=door_frame, 6=window_frame, 7=column, 8=wedge
  position:       Vec3f64     — world-space position (center of piece)
  orientation:    Quat32      — rotation (free Y, snapped tilt)
  dimensions:     Vec3f32     — width, height, depth (meters)
  material_id:    uint16      — references material registry
  color:          uint32      — RGBA override (0 = use material default)
  snap_group:     uint64      — ID linking connected pieces (for structural validation)
  owner_id:       uint64      — player who placed this
```

---

## Grid System

### The 0.3m grid

```
GRID_SNAP_SIZE     = 0.3       -- meters (≈1 foot / 12 inches)
ROTATION_SNAP_Y    = 15.0      -- degrees (24 positions around Y axis)
ROTATION_SNAP_TILT = 15.0      -- degrees (for ramp angles)
ROTATION_FREE      = true      -- hold Alt to disable rotation snap
```

**Why 0.3m**: Small enough for detailed interiors (furniture fits naturally, doorways are 3-4 grid units wide, walls are 10 grid units tall). Large enough that the grid doesn't feel like pixel art. A standard room is 10×10 grid units (3m×3m) which feels right for a small bedroom.

### Grid visualization

```
BUILD MODE GRID:
  -- Rendered as a translucent plane below the cursor
  -- Shows a 10×10 grid section (3m×3m) centered on cursor
  -- Grid lines: thin white, 20% opacity
  -- Snap points: small dots at intersections
  -- Color: green when placement is valid, red when invalid
  -- Fades out 5m from cursor (not visible everywhere)
```

### Snap behavior

```
SNAP RULES:

  Position snap:
    piece.position.x = round(cursor.x / GRID_SNAP_SIZE) * GRID_SNAP_SIZE
    piece.position.z = round(cursor.z / GRID_SNAP_SIZE) * GRID_SNAP_SIZE
    piece.position.y depends on piece type:
      FLOOR:  snaps to height intervals (0, 3.0, 6.0, ...) or on top of existing floor
      WALL:   bottom edge at floor height
      RAMP:   bottom edge at lower floor, top edge computed from dimensions
      COLUMN: bottom at floor height
      ROOF:   bottom at wall top height

  Rotation snap (Y axis):
    piece.yaw = round(cursor_yaw / ROTATION_SNAP_Y) * ROTATION_SNAP_Y
    Hold Alt: free rotation (any angle)

  Edge-to-edge snap:
    When placing a wall near another wall's edge:
      if gap < GRID_SNAP_SIZE:
        snap new wall's edge to existing wall's edge
        auto-align orientation to match or be perpendicular

  Floor-to-wall snap:
    When placing a floor near wall tops:
      if height difference < GRID_SNAP_SIZE:
        snap floor bottom to wall top
```

---

## Placement Mode

### Entering and exiting build mode

```
TOGGLE BUILD MODE:
  Key: B
  On enter:
    -- Unlock pointer (exit first-person look)
    -- Show build mode UI (piece selector, bottom of screen)
    -- Show grid around cursor
    -- Camera switches to slightly elevated third-person (see what you're building)
    -- Movement still works (WASD) but camera pulls back

  On exit:
    -- Remove ghost preview
    -- Hide build UI and grid
    -- Return to first-person pointer lock
```

### Placement flow

```
PLACEMENT FLOW:

  [1] Player presses B → enters build mode
  [2] Select piece type from UI bar (1-9 keys or click)
  [3] Mouse moves → ghost preview follows cursor on grid
      -- Ghost is translucent version of the piece
      -- Snaps to grid position
      -- Rotates with Q/E keys (Y axis) or R key (tilt for ramps)
      -- Green = valid placement, Red = invalid (overlapping, floating, etc.)

  [4] Left click → place piece
      -- Client sends CREATE action to server:
          ChangeRequest {
            source: player_id,
            change_type: CREATE,
            object_id: generated_id,
            payload: serialize(BUILDING_PIECE)
          }
      -- Client immediately shows the piece (optimistic placement)
      -- Server validates:
          - No overlap with existing static bodies
          - Connected to ground or another piece (no floating)
          - Within build radius of player (MAX_BUILD_DISTANCE = 10m)
          - Player has build permission in this world
      -- Server broadcasts to all clients if valid
      -- If server rejects: client removes the optimistic piece, shows error

  [5] Right click → cancel / remove ghost
  [6] Middle click on existing piece → select its type + material (eyedropper)

  [7] Delete key on existing piece → sends DESTROY action
      -- Only owner can destroy (or world admin)
```

### Rotation controls

```
ROTATION:
  Q         → rotate -15° on Y axis (counter-clockwise from above)
  E         → rotate +15° on Y axis (clockwise from above)
  R         → cycle tilt angle (0°, 15°, 30°, 45°, 60°, 75°, 90°) — for ramps/roofs
  Alt+drag  → free rotation (any angle, no snap)
  Shift+Q/E → rotate 90° (quick right-angle snap)
```

---

## Material System

```
MATERIAL REGISTRY:

  id    name            color_default    roughness    metalness    transparent
  0     wood_planks     #8B6914          0.8          0.0          false
  1     wood_dark       #4A3728          0.7          0.0          false
  2     stone_brick     #888888          0.9          0.0          false
  3     stone_smooth    #AAAAAA          0.6          0.0          false
  4     concrete        #C0C0C0          0.95         0.0          false
  5     metal_steel     #B0B0B0          0.3          0.8          false
  6     metal_rust      #8B4513          0.7          0.5          false
  7     glass           #CCE5FF          0.05         0.1          true
  8     glass_tinted    #334455          0.05         0.1          true
  9     plaster_white   #F5F5F0          0.9          0.0          false
  10    plaster_color   #E8D4B8          0.85         0.0          false
  11    tile_white      #FFFFFF          0.4          0.0          false
  12    tile_dark       #333333          0.3          0.0          false
  13    brick_red       #8B3A2A          0.85         0.0          false
  14    thatch          #C4A35A          0.95         0.0          false
  15    ice             #E0F0FF          0.1          0.0          true

  -- Phase 2: custom materials from agent-generated textures
```

### Material application

```
MATERIAL CONTROLS (in build mode):
  M           → open material picker
  Scroll      → cycle materials on selected piece type
  Click piece → apply current material to existing piece (PROPERTY_CHANGE action)
```

---

## Structural Validation

Placed pieces must be structurally sound. No floating walls, no impossible structures.

```
VALIDATION RULES:

  [1] GROUND CONNECTION:
      Every piece must trace a path of connected pieces down to the ground (y=0).
      "Connected" means two pieces share an edge within GRID_SNAP_SIZE tolerance.
      Exception: pieces in creative mode (world owner, no validation)

  [2] NO OVERLAP:
      New piece's bounding box must not overlap any existing static body.
      Tolerance: GRID_SNAP_SIZE * 0.5 (allows touching but not intersecting)

  [3] BUILD DISTANCE:
      Player must be within MAX_BUILD_DISTANCE (10m) of placement point.

  [4] BUILD PERMISSION:
      Player must have build permission in the current world.
      World owner always has permission.
      Other players: world config flag `allow_guest_building: bool`

  [5] PIECE LIMIT:
      Max pieces per world: MAX_BUILDING_PIECES = 5000 (Phase 1)
      Max pieces per player per world: MAX_PLAYER_PIECES = 1000

  [6] HEIGHT LIMIT:
      Max build height: MAX_BUILD_HEIGHT = 100m (333 grid units)
```

### Structural integrity (Phase 2+)

Phase 1 does simple connectivity validation (is it connected to ground?). Phase 2 adds weight-based structural integrity:

```
PHASE 2 — STRUCTURAL INTEGRITY:
  Each material has a weight and a load capacity.
  Weight propagates downward through connected pieces.
  If a column or wall's load exceeds its capacity, it breaks.
  Breaking cascades: unsupported pieces above also break.
  This creates emergent physics: thin columns can't hold massive floors.
```

---

## Undo/Redo

```
UNDO SYSTEM:
  Ctrl+Z    → undo last placement (sends DESTROY for the piece)
  Ctrl+Y    → redo (sends CREATE again)
  Max stack: 50 actions
  Stack clears on exit build mode
  Only affects pieces placed by the current player in this session
```

---

## Network Protocol

Building uses the existing CREATE/DESTROY/PROPERTY_CHANGE actions from the simulation contract. No new message types needed.

```
PLACE PIECE:
  Client → Server: ChangeRequest {
    source: player_id,
    change_type: CREATE (2),
    object_id: client_generated_uid,
    sequence_number: next_seq,
    requires_ack: true,
    payload: serialize(BUILDING_PIECE)
  }
  Server validates → spawns PhysicsBody::new_static() with box/wedge collider
  Server → Client: PLAYER_ACTION_RESULT (ack) + OBJECT_STATE_CHANGE (broadcast)

REMOVE PIECE:
  Client → Server: ChangeRequest {
    source: player_id,
    change_type: DESTROY (3),
    object_id: piece_id,
    sequence_number: next_seq,
    requires_ack: true,
    payload: empty
  }
  Server validates ownership → removes static body
  Server → Client: PLAYER_ACTION_RESULT (ack) + OBJECT_STATE_CHANGE (broadcast)

CHANGE MATERIAL:
  Client → Server: ChangeRequest {
    source: player_id,
    change_type: PROPERTY_CHANGE (1),
    object_id: piece_id,
    payload: { key: "material_id", value: new_material_id }
  }
```

---

## Geometry Generation (Client-Side)

Building piece geometry is generated procedurally on the client from BUILDING_PIECE data. No GLTF models needed for Phase 1.

```
GEOMETRY GENERATION:

  WALL, FLOOR:
    THREE.BoxGeometry(width, height, depth)

  RAMP, ROOF (angled):
    Custom BufferGeometry — triangular prism (wedge):
      6 vertices defining the triangular cross-section, extruded by width
      UV mapping: front/back faces use triangle, sides use quads

  STAIR:
    Visual: N box geometries stacked (one per step)
    Physics: single angled box (ramp collider) — player walks up smoothly
    step_height = rise / step_count
    step_depth = run / step_count

  DOOR_FRAME, WINDOW_FRAME:
    CSG subtraction: wall box minus opening box
    Or: 3-4 boxes arranged to form the frame (simpler, no CSG library needed)
    Phase 1: use the multi-box approach (top piece + two side pieces + optional bottom)

  COLUMN:
    THREE.CylinderGeometry(radius, radius, height, 8)  — 8 sides (octagonal)

  WEDGE:
    Same geometry as ramp but general-purpose (any orientation)
```

### Wedge geometry helper

```typescript
function createWedgeGeometry(width: number, height: number, depth: number): THREE.BufferGeometry {
  // Triangular prism: flat bottom, angled top surface
  //
  //     4────5
  //    /│   /│
  //   / │  / │      Y
  //  /  1─/──2      │  Z
  // 3──/─0          │ /
  //  │/              │/
  //                  └──── X
  //
  // Bottom face: 0,1,2,3 (flat)
  // Top face: 3,4,5,0 (angled from front-bottom to back-top)
  // Front face: 0,3 (triangle: 0, 3, bottom-edge)
  // Back face: 1,2,4,5 (quad)

  const hw = width / 2
  const hd = depth / 2

  const vertices = new Float32Array([
    // Front triangle (z = -hd)
    -hw, 0, -hd,        // 0: front-bottom-left
     hw, 0, -hd,        // 1: front-bottom-right
    -hw, 0, -hd,        // (degenerate — front face is a line at bottom)

    // Back quad (z = +hd)
    -hw, 0,  hd,        // 2: back-bottom-left
     hw, 0,  hd,        // 3: back-bottom-right
    -hw, height, hd,    // 4: back-top-left
     hw, height, hd,    // 5: back-top-right
  ])

  // ... (full implementation at build time)
  return new THREE.BufferGeometry()
}
```

---

## R3F Component Structure

```
<BuildMode>                          — top-level build mode wrapper
  <BuildModeUI />                    — HTML overlay: piece selector, material picker
  <BuildGrid />                      — translucent grid plane around cursor
  <GhostPreview piece={selected} />  — translucent preview of piece being placed
  <PlacedPieces pieces={placed} />   — renders all placed building pieces
</BuildMode>
```

### GhostPreview behavior

```
GHOST PREVIEW:
  -- Follows mouse cursor (raycast onto terrain/existing pieces)
  -- Snaps to 0.3m grid
  -- Rotation follows Q/E input
  -- Material: MeshBasicMaterial, color green (valid) or red (invalid), opacity 0.5
  -- Updates every frame in useFrame() — no React re-renders
  -- When valid: outline glow (emissive)
  -- When invalid: red tint + subtle shake animation
```

---

## Constants

```
-- Grid
GRID_SNAP_SIZE            = 0.3       -- meters (≈1 foot)
ROTATION_SNAP_Y           = 15.0      -- degrees
ROTATION_SNAP_TILT        = 15.0      -- degrees

-- Limits
MAX_BUILD_DISTANCE        = 10.0      -- meters from player
MAX_BUILDING_PIECES       = 5000      -- per world
MAX_PLAYER_PIECES         = 1000      -- per player per world
MAX_BUILD_HEIGHT          = 100.0     -- meters
MAX_PIECE_DIMENSION       = 12.0      -- meters (any single dimension)
MIN_PIECE_DIMENSION       = 0.3       -- meters (one grid unit)

-- Defaults
DEFAULT_WALL_WIDTH        = 3.0       -- meters (10 grid units)
DEFAULT_WALL_HEIGHT       = 3.0       -- meters (10 grid units)
DEFAULT_WALL_THICKNESS    = 0.2       -- meters
DEFAULT_FLOOR_SIZE        = 3.0       -- meters
DEFAULT_FLOOR_THICKNESS   = 0.2       -- meters
DEFAULT_COLUMN_RADIUS     = 0.3       -- meters
DEFAULT_DOOR_WIDTH        = 1.0       -- meters
DEFAULT_DOOR_HEIGHT       = 2.4       -- meters
DEFAULT_WINDOW_WIDTH      = 1.2       -- meters
DEFAULT_WINDOW_HEIGHT     = 1.2       -- meters
DEFAULT_WINDOW_SILL       = 1.0       -- meters above floor

-- Materials
DEFAULT_MATERIAL          = 0         -- wood_planks
MATERIAL_COUNT            = 16        -- Phase 1 material count

-- Undo
MAX_UNDO_STACK            = 50
```

---

## Phase 1 Scope

**Included:**
- All 9 primitive types (wall, floor, ramp, stair, roof, door frame, window frame, column, wedge)
- 0.3m grid snap with free Y rotation
- Ghost preview with valid/invalid coloring
- 16 materials with color, roughness, metalness
- CREATE/DESTROY/PROPERTY_CHANGE network actions
- Simple connectivity validation (connected to ground)
- Undo/redo (Ctrl+Z/Y)
- Build distance and piece count limits
- Procedural geometry (no GLTF models)

**Deferred to Phase 2+:**
- Weight-based structural integrity (collapse mechanics)
- Custom textures from agent-generated content
- Curved pieces (arches, domes)
- Terrain deformation (digging, flattening)
- Blueprint system (save/load/share building designs)
- Copy/paste groups of pieces
- Door/window objects with physics (hinges, open/close)

---

## Testing Requirements

1. **Grid snap test**: Place wall at arbitrary cursor position → position aligns to 0.3m grid
2. **Rotation test**: Press E 24 times → wall completes full 360° rotation
3. **Free rotation test**: Hold Alt + drag → rotation is continuous, not snapped
4. **Ramp test**: Place ramp between two floors → player walks up smoothly (raycasting works)
5. **Overlap test**: Try to place wall inside existing wall → rejected (red ghost)
6. **Connectivity test**: Try to place floating wall → rejected. Place wall connected to floor → accepted.
7. **Material test**: Apply glass material to wall → renders transparent
8. **Network test**: Player A places wall → Player B sees it appear within 1 tick (20ms)
9. **Destroy test**: Delete a wall → other players see it disappear
10. **Undo test**: Ctrl+Z removes last placed piece, Ctrl+Y restores it
11. **Performance test**: 1000 placed pieces → frame rate stays above 60 FPS (instanced rendering)
12. **Door frame test**: Place door frame → player walks through the opening without collision
