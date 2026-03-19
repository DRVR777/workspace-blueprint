---
name: simulation
parent: world
type: sub-program
status: active
phase: 0
layer: 3 (engine primitives)
depends-on: spatial/
---

# simulation — Physics Tick and Game Rule Execution

**What this sub-program is**: The pure-function simulation core. It takes a world snapshot and a list of inputs, advances physics by one tick, resolves collisions, processes player actions, runs entity AI, and returns a `tick_result` describing everything that changed. It has no side effects — it does not write to the world graph, send network messages, or touch any I/O. The node-manager calls it once per tick and applies the results.

**What it owns**:
- The `run_tick` function (the simulation contract's primary interface)
- Physics integration: semi-implicit Euler per ADR-003
- Collision pipeline: broad phase → narrow phase → resolution, using Rapier per ADR-015
- Player action validation and application
- Entity AI tick (behavior tree evaluation, nav mesh queries)
- Per-world physics parameter application (custom gravity, damping, constraints)
- Simulation event emission (COLLISION, THRESHOLD_CROSSED, etc.)
- Determinism guarantee: same inputs → same outputs, always

**What it does NOT own**:
- The spatial index (that is spatial/ — simulation receives query results, doesn't own the octree)
- World graph persistence (node-manager writes tick_result.state_changes)
- Network I/O (node-manager broadcasts results to clients)
- Ticker log writes (node-manager handles those)
- Client-side prediction (that is engine/local-simulation/)

---

## Architecture: 5 Stages of `run_tick`

The tick is a deterministic pipeline of 5 sequential stages. Each stage takes the output of the previous stage. No stage has side effects outside the pipeline.

```
run_tick(snapshot, inputs, dt) → tick_result

  ┌──────────────────────────────────────────────────┐
  │  Stage 1: INPUT VALIDATION                       │
  │  Validate and filter change_requests              │
  │  Reject invalid inputs → rejected_requests        │
  │  Produce: validated_actions[]                     │
  └────────────────────┬─────────────────────────────┘
                       │
  ┌────────────────────▼─────────────────────────────┐
  │  Stage 2: ACTION APPLICATION                     │
  │  Apply validated player actions to snapshot        │
  │  MOVE → set applied_force on body                 │
  │  INTERACT → trigger interaction logic             │
  │  CREATE → spawn new body                          │
  │  DESTROY → mark for removal                       │
  │  Produce: modified_snapshot with forces applied    │
  └────────────────────┬─────────────────────────────┘
                       │
  ┌────────────────────▼─────────────────────────────┐
  │  Stage 3: PHYSICS STEP                           │
  │  3a. Apply world forces (gravity, damping)        │
  │  3b. Integrate all dynamic bodies (Euler)         │
  │  3c. Step Rapier world (collision detect+resolve)  │
  │  3d. Read back positions from Rapier              │
  │  3e. Apply velocity damping                       │
  │  3f. Clear accumulated forces                     │
  │  Produce: post_physics_snapshot                   │
  └────────────────────┬─────────────────────────────┘
                       │
  ┌────────────────────▼─────────────────────────────┐
  │  Stage 4: GAME RULES                             │
  │  4a. Entity AI evaluation (behavior trees)        │
  │  4b. State machine advancement                    │
  │  4c. Trigger evaluation (on_collision, on_rest)   │
  │  4d. Domain boundary check (THRESHOLD_CROSSED)    │
  │  Produce: post_rules_snapshot + simulation_events │
  └────────────────────┬─────────────────────────────┘
                       │
  ┌────────────────────▼─────────────────────────────┐
  │  Stage 5: DIFF                                   │
  │  Compare post_rules_snapshot to original snapshot  │
  │  Emit state_change_events for everything changed   │
  │  Package tick_result                               │
  └──────────────────────────────────────────────────┘
```

---

## Stage 1: Input Validation

```
validate_inputs(snapshot, inputs) → (validated_actions, rejected_requests)

FOR EACH request in inputs:
  -- Check object exists (except CREATE)
  IF request.type != CREATE AND get_object(snapshot, request.object_id) == NOT_FOUND:
    reject(NOT_FOUND)
    CONTINUE

  -- Check source has permission
  IF NOT has_permission(request.source, request.type, request.object_id):
    reject(PERMISSION_DENIED)
    CONTINUE

  -- Check domain ownership
  IF get_object(snapshot, request.object_id).domain_id != snapshot.domain_id:
    reject(DOMAIN_MISMATCH)
    CONTINUE

  -- Type-specific validation
  MATCH request.type:
    MOVE:
      direction = decode_move_payload(request.payload)
      IF magnitude(direction) > MAX_MOVE_FORCE:
        clamp direction to MAX_MOVE_FORCE
      accept(request, clamped_direction)

    PROPERTY_CHANGE:
      IF property_is_readonly(request.payload.key):
        reject(INVALID_STATE)
      ELSE:
        accept(request)

    CREATE:
      IF spawn_count_this_tick >= MAX_SPAWNS_PER_TICK:
        reject(PHYSICS_VIOLATION)
      ELSE:
        accept(request)

    DESTROY:
      IF object_is_indestructible(snapshot, request.object_id):
        reject(INVALID_STATE)
      ELSE:
        accept(request)

    INTERACT:
      IF NOT interaction_is_valid(snapshot, request):
        reject(INVALID_STATE)
      ELSE:
        accept(request)
```

---

## Stage 2: Action Application

```
apply_actions(snapshot, validated_actions) → modified_snapshot

FOR EACH action in validated_actions:
  MATCH action.type:

    MOVE:
      body = get_body(snapshot, action.object_id)
      -- Player movement applies force, not direct position change
      -- This ensures physics consistency (collisions still apply)
      body.applied_force += action.direction * PLAYER_MOVE_FORCE_MULTIPLIER
      emit PLAYER_ACTION_RESULT(action.sequence_number, success)

    CREATE:
      new_body = instantiate_body(action.payload)
      add_body(snapshot, new_body)
      emit PLAYER_ACTION_RESULT(action.sequence_number, success)

    DESTROY:
      mark_for_removal(snapshot, action.object_id)
      emit PLAYER_ACTION_RESULT(action.sequence_number, success)

    INTERACT:
      run_interaction(snapshot, action.object_id, action.payload)
      emit PLAYER_ACTION_RESULT(action.sequence_number, success)

    PROPERTY_CHANGE:
      set_property(snapshot, action.object_id, action.payload.key, action.payload.value)
      emit PLAYER_ACTION_RESULT(action.sequence_number, success)
```

---

## Stage 3: Physics Step (Rapier Integration)

This is where ADR-003 (semi-implicit Euler) and ADR-004/ADR-015 (Rapier) combine.

**Rapier owns collision detection AND constraint solving.** We do NOT implement GJK/SAT/BVH. Rapier's `PhysicsPipeline::step()` handles broad phase, narrow phase, contact resolution, and constraint solving in one call.

**Our responsibility**: translating between our data shapes (simulation-contract types) and Rapier's rigid body/collider API.

```
physics_step(snapshot, dt) → post_physics_snapshot

  -- 3a. Apply world forces
  FOR EACH body in snapshot.bodies WHERE body.category == DYNAMIC:
    -- Gravity (per-world configurable)
    world_gravity = snapshot.world_config.gravity  -- default: (0, -9.8, 0)
    body.applied_force += world_gravity * body.mass

    -- Custom force generators (attraction, orbit, repel, buoyancy)
    FOR EACH generator in body.force_generators:
      body.applied_force += evaluate_force_generator(generator, body, snapshot)

  -- 3b. Sync our bodies → Rapier rigid bodies
  FOR EACH body in snapshot.bodies:
    rapier_body = rapier_world.get_rigid_body(body.object_id)
    IF body.category == DYNAMIC:
      rapier_body.apply_force(body.applied_force)
      rapier_body.apply_torque(body.applied_torque)
    ELIF body.category == KINEMATIC:
      rapier_body.set_next_kinematic_position(
        body.position + body.scripted_velocity * dt
      )

  -- 3c. Step Rapier (this does integration + collision detect + resolve)
  rapier_world.step(dt)
  --   Rapier internally:
  --     1. Integrates velocities (semi-implicit Euler — matches ADR-003)
  --     2. Broad phase (sweep-and-prune)
  --     3. Narrow phase (GJK/EPA)
  --     4. Contact constraint solving (sequential impulses)
  --     5. Integrates positions

  -- 3d. Read back from Rapier → our bodies
  FOR EACH body in snapshot.bodies WHERE body.category == DYNAMIC:
    rapier_body = rapier_world.get_rigid_body(body.object_id)
    body.position = rapier_body.translation()
    body.orientation = rapier_body.rotation()
    body.velocity = rapier_body.linvel()
    body.angular_velocity = rapier_body.angvel()

  -- 3e. Velocity damping (post-Rapier, our tuning layer)
  FOR EACH body in snapshot.bodies WHERE body.category == DYNAMIC:
    body.velocity *= (1.0 - DAMPING_COEFFICIENT * dt)
    body.angular_velocity *= (1.0 - ANGULAR_DAMPING * dt)

  -- 3f. Clear forces
  FOR EACH body in snapshot.bodies WHERE body.category == DYNAMIC:
    body.applied_force = Vec3(0, 0, 0)
    body.applied_torque = Vec3(0, 0, 0)

  -- 3g. Collect collision events from Rapier
  FOR EACH contact_pair in rapier_world.contact_pairs():
    IF contact_pair.has_any_active_contact():
      emit COLLISION event with contact_point, normal, penetration

  RETURN snapshot (now post-physics)
```

### Rapier Configuration

```
RAPIER PIPELINE CONFIG:
  integration_parameters:
    dt:                    passed from run_tick
    min_ccd_dt:            0.001          -- CCD sub-step minimum (fast projectile safety)
    erp:                   0.8            -- error reduction parameter (contact stabilization)
    damping_ratio:         0.25           -- contact damping
    joint_erp:             1.0            -- joint constraint stiffness
    joint_damping_ratio:   1.0
    allowed_linear_error:  0.001          -- 1mm penetration tolerance
    max_penetration_correction: 0.2       -- max correction per step
    prediction_distance:   0.002          -- speculative contacts distance
    max_velocity_iterations: 4            -- velocity solver iterations
    max_velocity_friction_iterations: 8
    max_stabilization_iterations: 1
    interleave_restitution_and_friction: true
    num_solver_iterations: 4              -- position solver iterations
    num_additional_friction_iterations: 4
    num_internal_pgs_iterations: 1
```

### Body ↔ Rapier Mapping

```
BODY TO RAPIER MAPPING:

  body.category == DYNAMIC   → RigidBodyType::Dynamic
  body.category == STATIC    → RigidBodyType::Fixed
  body.category == KINEMATIC → RigidBodyType::KinematicPositionBased

  body.collision_shape == SPHERE     → ColliderBuilder::ball(radius)
  body.collision_shape == BOX        → ColliderBuilder::cuboid(hx, hy, hz)
  body.collision_shape == CONVEX_HULL → ColliderBuilder::convex_hull(&vertices)

  body.mass         → RigidBodyBuilder::additional_mass(mass)
  body.position     → RigidBodyBuilder::translation(x, y, z)
  body.orientation  → RigidBodyBuilder::rotation(quaternion)
  body.velocity     → body_handle.set_linvel(v)
  body.angular_velocity → body_handle.set_angvel(w)
```

---

## Stage 4: Game Rules

```
apply_game_rules(snapshot, dt) → (snapshot, simulation_events)

  events = []

  -- 4a. Entity AI evaluation (Phase 0: stub — NPCs stand still)
  -- Full implementation in Phase 1 with behavior trees
  FOR EACH entity in snapshot.entities WHERE entity.active_behavior != null:
    ai_action = evaluate_behavior_tree(entity, snapshot)
    IF ai_action != null:
      apply_ai_action(snapshot, entity, ai_action)
      events.push(AI_ACTION(entity.id, ai_action))

  -- 4b. State machine advancement
  FOR EACH entity in snapshot.entities WHERE entity.has_state_machine:
    IF entity.state_timer <= 0:
      next_state = entity.state_machine.transition(entity.current_state)
      IF next_state == END:
        events.push(STATE_EXHAUSTED(entity.id))
      ELSE:
        entity.current_state = next_state
        entity.state_timer = next_state.duration
    ELSE:
      entity.state_timer -= dt

  -- 4c. Trigger evaluation
  FOR EACH collision_event in this_tick_collisions:
    body_a = get_body(snapshot, collision_event.body_a_id)
    body_b = get_body(snapshot, collision_event.body_b_id)

    -- Check for on_collision triggers
    IF body_a.has_trigger(ON_COLLISION):
      fire_trigger(snapshot, body_a, ON_COLLISION, body_b)
    IF body_b.has_trigger(ON_COLLISION):
      fire_trigger(snapshot, body_b, ON_COLLISION, body_a)

  FOR EACH body in snapshot.bodies WHERE body.has_trigger(ON_REST):
    IF magnitude(body.velocity) < REST_VELOCITY_THRESHOLD
       AND body.rest_timer >= REST_DURATION_THRESHOLD:
      fire_trigger(snapshot, body, ON_REST)

  -- 4d. Domain boundary check
  FOR EACH body in snapshot.bodies WHERE body.category == DYNAMIC:
    IF NOT domain_contains(snapshot.domain_bounds, body.position):
      events.push(THRESHOLD_CROSSED(body.object_id))
      -- node-manager handles the actual handoff

  RETURN (snapshot, events)
```

---

## Stage 5: Diff

```
compute_diff(original_snapshot, final_snapshot) → tick_result

  state_changes = []

  -- Compare each body
  FOR EACH body in final_snapshot.bodies:
    original = find_body(original_snapshot, body.object_id)

    IF original == null:
      -- New body (created this tick)
      state_changes.push(state_change_event(CREATE, body))
    ELIF body != original:
      -- Changed body — emit what changed
      IF body.position != original.position OR body.orientation != original.orientation:
        state_changes.push(state_change_event(POSITION_UPDATE, body))
      IF body.properties != original.properties:
        state_changes.push(state_change_event(PROPERTY_CHANGE, body))
      IF body.state_enum != original.state_enum:
        state_changes.push(state_change_event(STATE_CHANGE, body))

  -- Check for removals
  FOR EACH body in original_snapshot.bodies:
    IF NOT exists_in(final_snapshot, body.object_id):
      state_changes.push(state_change_event(DESTROY, body))

  RETURN tick_result {
    next_tick_number: original_snapshot.tick_number + 1,
    state_changes:    state_changes,
    events:           all_simulation_events,
    rejected_requests: all_rejected_requests
  }
```

---

## Per-World Physics Configuration

Each world can override physics parameters. The simulation reads these from the snapshot, not from hardcoded constants.

```
WORLD_PHYSICS_CONFIG:
  gravity:              Vec3f32     -- default: (0, -9.8, 0)
  gravity_mode:         uint8       -- 0=directional, 1=spherical (toward world center),
                                    -- 2=zero, 3=zone-based
  gravity_center:       Vec3f64     -- used when gravity_mode == 1 (spherical)
  damping_coefficient:  float32     -- default: 0.01
  angular_damping:      float32     -- default: 0.05
  time_scale:           float32     -- default: 1.0 (time dilation per PRD §19)
  max_velocity:         float32     -- default: 300.0 m/s (speed cap prevents tunneling)
  enable_ccd:           bool        -- default: true (continuous collision for fast objects)
  gravity_zones:        list of gravity_zone  -- when gravity_mode == 3
```

```
GRAVITY_ZONE:
  bounds:     AABB64         -- axis-aligned bounding box
  gravity:    Vec3f32        -- local gravity override
  mode:       uint8          -- 0=directional, 1=spherical
  center:     Vec3f64        -- for spherical mode
  priority:   uint8          -- higher priority zones override lower ones
  blend_dist: float32        -- meters over which gravity transitions at zone boundary
```

### Spherical Gravity Calculation

```
compute_spherical_gravity(body, gravity_center, gravity_magnitude):
  direction = normalize(gravity_center - body.position)
  RETURN direction * gravity_magnitude * body.mass
```

---

## Constants

```
-- Physics (from simulation-contract + ADR-003)
TARGET_TICK_DURATION     = 0.02        -- 50 Hz (20ms per tick)
DEFAULT_GRAVITY          = (0, -9.8, 0)
DEFAULT_DAMPING          = 0.01
DEFAULT_ANGULAR_DAMPING  = 0.05
MAX_VELOCITY             = 300.0       -- m/s — prevents tunneling at 50Hz

-- Action limits
MAX_MOVE_FORCE           = 500.0       -- Newtons — clamp player input
PLAYER_MOVE_FORCE_MULTIPLIER = 50.0    -- translates normalized input direction to force
MAX_SPAWNS_PER_TICK      = 8           -- prevent spawn flooding
REST_VELOCITY_THRESHOLD  = 0.01        -- m/s — body considered "at rest"
REST_DURATION_THRESHOLD  = 1.0         -- seconds body must be below threshold

-- Rapier
RAPIER_SOLVER_ITERATIONS = 4
RAPIER_CCD_MIN_DT        = 0.001
RAPIER_ALLOWED_ERROR      = 0.001      -- 1mm penetration tolerance
```

---

## Determinism Contract

The simulation MUST be deterministic: identical inputs produce identical outputs. This is required for:

1. **Client-side prediction** — client runs the same simulation; if non-deterministic, reconciliation fails constantly
2. **Replay** — ticker log stores inputs, not full state; replaying inputs must reproduce the same state
3. **Testing** — physics tests must be repeatable

### What makes it deterministic

- Rapier is deterministic given the same rigid body set, same forces, same dt
- No floating-point non-determinism: operations are ordered identically every tick
- No randomness in the simulation pipeline (AI can use seeded RNG, seed stored in snapshot)
- Bodies are processed in a fixed order (sorted by object_id) at every stage
- HashMap iteration order is NOT used for any physics-critical path — use sorted vectors

### What could break determinism (guard against)

- Using `HashMap` iteration order for body processing (use `BTreeMap` or sorted `Vec`)
- Platform-dependent floating point (use `#[cfg(target_feature)]` to verify IEEE 754 compliance)
- Rapier version mismatch between server and client WASM build
- Unordered event processing (process collision events sorted by (body_a_id, body_b_id))

---

## Performance Budget

The node-manager allocates **17ms** of each 20ms tick to the simulation (3ms reserved for I/O, broadcasting, bookkeeping). Within those 17ms:

```
Stage 1 (Input validation):     < 0.5ms   -- O(N) where N = actions this tick
Stage 2 (Action application):   < 0.5ms   -- O(N) where N = validated actions
Stage 3 (Physics step):         < 14ms    -- O(M log M) where M = bodies in domain
  - Rapier step:                < 12ms    -- broad + narrow + solve
  - Body sync (to/from Rapier): < 2ms     -- linear scan
Stage 4 (Game rules):           < 1.5ms   -- O(E) where E = entities with AI
Stage 5 (Diff):                 < 0.5ms   -- O(M) linear scan
```

**Phase 0 capacity target**: 500 dynamic bodies + 50 connected players within 17ms tick budget on a single core.

---

## Phase 0 Scope

Phase 0 delivers the minimum simulation that proves the tick loop end-to-end: one player moves, physics applies, client sees the result.

**Included in Phase 0**:
- `run_tick` with all 5 stages
- Semi-implicit Euler integration via Rapier
- Collision detection and resolution via Rapier (sphere, box, convex hull)
- Player MOVE action → applied force → physics → position update
- Directional gravity (default Y-down)
- Velocity damping
- Domain boundary detection (THRESHOLD_CROSSED events)
- Determinism guarantee
- CREATE and DESTROY actions (basic spawn/remove)

**Deferred to Phase 1+**:
- Entity AI evaluation (stub: NPCs do nothing)
- State machine advancement (stub: no state machines)
- Trigger system (stub: triggers not fired)
- Spherical/zone-based gravity
- Force generators (attraction, orbit, repel, buoyancy)
- Custom physics profiles per object type
- Sub-stepping / CCD tuning
- Constraint/joint system

---

## Testing Requirements (before Phase 0 gate)

1. **Determinism test**: Run `run_tick` twice with identical inputs → outputs must be bitwise identical
2. **Gravity test**: Dynamic body at (0, 10, 0) with no floor → after 50 ticks (1 second), position.y ≈ 0.1 (free fall under 9.8 m/s²)
3. **Collision test**: Two spheres moving toward each other → they bounce, neither penetrates the other
4. **Player move test**: MOVE action with direction (1, 0, 0) → body gains positive x-velocity → position moves right
5. **Rejection test**: MOVE action targeting non-existent object → rejected_requests contains NOT_FOUND
6. **Boundary test**: Body moves past domain bounds → THRESHOLD_CROSSED event emitted
7. **Performance test**: 500 dynamic bodies, 10 player actions → `run_tick` completes in < 17ms
8. **Damping test**: Body with initial velocity and no forces → velocity decays toward zero over time
