---
name: simulation-contract
status: accepted
version: 0.2
published_by: world/simulation/
consumed_by: world/node-manager/ (tick orchestrator), world/world-generator/ (initial state)
---

# Simulation Contract

The simulation layer computes one tick of the world: applies forces, detects and resolves collisions, runs entity AI, processes player actions, and advances state machines. It is a pure function — same inputs always produce same outputs.

---

## What This Contract Provides

### Primary interface

`run_tick(snapshot: world_state_snapshot, inputs: list of change_request, dt: float32)`
  → `tick_result`
  - `snapshot`: all physics bodies and entities in the node's domain at tick start
  - `inputs`: all player actions and system requests queued since the last tick
  - `dt`: elapsed seconds — should equal TARGET_TICK_DURATION (0.02s = 50Hz) but may vary
  - Returns the complete result; the node-manager writes changes to the world graph
  - **Deterministic**: same inputs → same outputs, always

### Single-body utilities (used for replay and debugging)

`integrate_body(body: physics_body, dt: float32)` → physics_body
  - One physics step for a single dynamic body (Category A only)

`detect_collisions(bodies: list of physics_body)` → list of collision_pair
  - Broad phase (sweep-and-prune) + narrow phase (GJK/SAT)

`resolve_collision(body_a: physics_body, body_b: physics_body, collision: collision_data)`
  → `(impulse_a: Vec3f32, impulse_b: Vec3f32)`

---

## Data Shapes

### physics_body

```
PHYSICS_BODY:
  object_id:         uint64
  category:          uint8     — 0=dynamic, 1=static, 2=kinematic (source: PRD §10.2)

  -- Present for all categories:
  position:          Vec3f64
  orientation:       Quat32    — unit quaternion
  collision_shape:   uint8     — 0=sphere, 1=box, 2=convex_hull
  shape_params:      bytes     — sphere: [radius: float32]
                               — box: [half_extents: Vec3f32]
                               — convex_hull: [vertex_count: uint16, vertices: Vec3f32[]]

  -- Dynamic (category=0) only:
  velocity:          Vec3f32
  angular_velocity:  Vec3f32
  mass:              float32
  moment_of_inertia: float32
  applied_force:     Vec3f32   — reset to zero after each integrate_body call
  applied_torque:    Vec3f32   — reset to zero after each integrate_body call

  -- Kinematic (category=2) only:
  scripted_velocity: Vec3f32   — set by AI or scripted motion; not affected by forces
```

### world_state_snapshot

```
WORLD_STATE_SNAPSHOT:
  tick_number:   uint64
  timestamp_ms:  uint64
  domain_id:     uint64
  bodies:        list of physics_body      — all objects with a collision shape
  entities:      list of entity_record     — all living entities (from world-state-contract)
  terrain:       list of terrain_chunk     — static collision geometry for this domain
```

### collision_pair

```
COLLISION_PAIR:
  body_a_id:       uint64
  body_b_id:       uint64
  collision_data:  collision_data
```

### collision_data

```
COLLISION_DATA:
  contact_point:  Vec3f32   — world-space point of contact
  contact_normal: Vec3f32   — unit normal pointing from B toward A
  penetration_depth: float32
```

### tick_result

```
TICK_RESULT:
  next_tick_number:    uint64
  state_changes:       list of state_change_event   — what changed (write to world graph)
  events:              list of simulation_event      — triggers for game logic
  rejected_requests:   list of rejected_request      — inputs that failed validation
```

### simulation_event

```
SIMULATION_EVENT:
  type:       uint16    — see simulation event registry below
  object_id:  uint64    — primary object involved
  other_id:   uint64    — secondary object (if applicable), 0 = none
  payload:    bytes     — event-type-specific

Simulation event types (Phase 0 subset):
  0x0001  COLLISION          — two objects collided; payload: collision_data
  0x0002  THRESHOLD_CROSSED  — object crossed a domain boundary
  0x0003  STATE_EXHAUSTED    — a timed state machine reached its end state
  0x0004  AI_ACTION          — entity AI emitted an action; payload: action descriptor
  0x0005  PLAYER_ACTION_RESULT — a change_request was processed; payload: result + sequence_number
```

### rejected_request

```
REJECTED_REQUEST:
  original_sequence_number: uint32
  reason_code:              uint8    — matches change_error codes in world-state-contract
```

---

## Constants

```
TARGET_TICK_DURATION  = 0.02s     (50 ticks/second — source: PRD §16)
GRAVITY               = (0, -9.8, 0)  (m/s² — Y-up coordinate system)
DAMPING_COEFFICIENT   = 0.01      (linear velocity damping per second)
ANGULAR_DAMPING       = 0.05      (angular velocity damping per second)
```

---

## Invariants

- `run_tick` is deterministic: same inputs → same outputs
- `run_tick` does not write to the world graph — it returns `tick_result`; the node-manager writes
- `run_tick` never produces a world state where dynamic bodies overlap (collision-free invariant)
- `integrate_body` only accepts category=0 (dynamic) bodies — throws if called on static/kinematic
- All physics is computed in world-space, Y-up coordinate system

---

## What This Contract Does NOT Provide

- World graph persistence — node-manager handles writes from `tick_result.state_changes`
- Ticker log writes — node-manager handles those from `tick_result.state_changes`
- Rendering or client-side prediction — those are engine/ concerns
- Procedural terrain generation — see world/world-generator/
