---
name: world-state-contract
status: accepted
version: 0.2
published_by: world/
consumed_by: engine/, agents, network-layer
---

# World State Contract

The world-state interface is the read/write surface for the persistent object graph within a node's domain. Game state lives here. The simulation layer mutates it each tick. Clients observe it via subscriptions.

---

## What This Contract Provides

### READ operations (no side effects)

`get_object(id: uint64)` → object_record | NOT_FOUND

`query_radius(center: Vec3f64, radius: float64)` → list of object_record
  - Returns all objects whose position is within `radius` of `center`
  - Results sorted by distance ascending

`query_box(min: Vec3f64, max: Vec3f64)` → list of object_record
  - Returns all objects whose position is within the axis-aligned bounding box

`get_entity(id: uint64)` → entity_record | NOT_FOUND
  - Returns the extended entity record (superset of object_record) for living entities

`get_terrain(sector_coords: Vec3i32, chunk_coords: Vec3i32)` → terrain_chunk | NOT_FOUND

`get_neighbors(id: uint64)` → list of neighbor_entry

### SUBSCRIBE operations (event streams)

`subscribe_to_domain(domain_id: uint64)` → stream of state_change_event
`subscribe_to_object(id: uint64)` → stream of state_change_event
`subscribe_to_radius(center: Vec3f64, radius: float64)` → stream of state_change_event

### WRITE operations

`submit_change_request(request: change_request)` → sequence_number | change_error
  - Requests go through the simulation layer — they do NOT apply immediately
  - Applied in the next tick if valid
  - Returns the sequence_number on acknowledgment (matches PLAYER_ACTION.sequence_number)

---

## Data Shapes

### object_record (source: PRD §9.2)

```
OBJECT_RECORD:

  IDENTITY:
    id:            uint64    — globally unique, never reused
    type_id:       uint32    — references object type definition in type registry
    created_at:    uint64    — Unix timestamp ms
    created_by:    uint64    — player_id, node_id, or 0 = world_seed
    version:       uint64    — increments on every state change

  SPATIAL:
    position:      Vec3f64   — (float64, float64, float64) — high precision for large world
    orientation:   Quat32    — unit quaternion (float32 x4)
    bounding_box:  AABB64    — (min: Vec3f64, max: Vec3f64)
    domain_id:     uint64    — currently owning node's domain identifier
    sector_coords: Vec3i32   — (int32, int32, int32)
    chunk_coords:  Vec3i32   — (int32, int32, int32)

  STATE:
    properties:    map[string → typed_value]
      — typed_value is one of: bool | int64 | float64 | string | uint64 (object ref)
    state_enum:    uint8     — 0=active, 1=damaged, 2=destroyed, 3=dormant, 255=unknown

  GRAPH:
    neighbors:     list of neighbor_entry
    cluster_id:    uint32
    semantic_vector: [float32 x5]   — 5D embedding (Appendix B)

  ASSETS:
    asset_version: uint32

  BEHAVIOR:
    embedded_instructions: bytes    — format defined by agent system (Part XII)

  HISTORY:
    ticker_log_pointer:  uint64
    last_event_at:       uint64    — Unix timestamp ms
    state_change_count:  uint64
```

### entity_record (superset of object_record — players and NPCs)

```
ENTITY_RECORD extends OBJECT_RECORD with:

  PHYSICS:
    body_category:      uint8     — 0=dynamic, 1=static, 2=kinematic
    velocity:           Vec3f32
    angular_velocity:   Vec3f32
    mass:               float32
    moment_of_inertia:  float32

  AI:
    active_behavior:    string    — name of currently executing behavior definition
    behavior_stack:     list of string
    nav_target:         Vec3f64 | null

  PLAYER (populated only for player-controlled entities):
    player_id:          uint64
    display_name:       string
    auth_level:         uint8     — 0=player, 1=moderator, 2=admin
```

### neighbor_entry

```
NEIGHBOR_ENTRY:
  object_id:         uint64
  relationship_type: uint8    — 0=spatial_proximity, 1=ownership, 2=construction, 3=semantic
  weight:            float32
```

### terrain_chunk

```
TERRAIN_CHUNK:
  sector_coords:  Vec3i32
  chunk_coords:   Vec3i32
  heightmap:      [float32]   — (CHUNK_SIZE+1)^2 values, column-major
  surface_types:  [uint8]     — CHUNK_SIZE^2 values (material/biome per cell)
  version:        uint32
```

### change_request

```
CHANGE_REQUEST:
  source:          uint64   — player_id, node_id, or 0 for system
  type:            uint8    — MOVE=0, PROPERTY_CHANGE=1, CREATE=2, DESTROY=3, INTERACT=4
  object_id:       uint64   — target (unused for CREATE)
  sequence_number: uint32
  requires_ack:    bool
  payload:         bytes    — type-specific, see action type registry
```

### state_change_event

```
STATE_CHANGE_EVENT:
  sequence:        uint64   — monotonic, assigned by ticker log
  timestamp_ms:    uint64
  object_id:       uint64
  change_type:     uint8    — matches ObjectChangeType in object_state_change.fbs
  old_state_hash:  uint32   — CRC32 of pre-change state (conflict detection)
  payload:         bytes    — mirrors ObjectStateChange payload
```

### change_error

```
CHANGE_ERROR:
  code:    uint8
  message: string

Error codes:
  0x01 NOT_FOUND         — object does not exist
  0x02 PERMISSION_DENIED — source lacks permission
  0x03 INVALID_STATE     — change invalid for current object state
  0x04 PHYSICS_VIOLATION — change would violate physics constraints
  0x05 DOMAIN_MISMATCH   — object is not in this node's domain
```

---

## domain_id

A `domain_id` equals the `node_id` of the node that owns a spatial region. When a node hands off a sector, the `domain_id` of all objects in that sector changes. The node registry maps domain_ids to node addresses.

---

## What This Contract Does NOT Provide

- Direct field writes — use `submit_change_request`
- Ticker log access — see `ticker-log-contract.md`
- Asset geometry — see `asset-store-contract.md`
- Authentication — see `player-session-contract.md`
