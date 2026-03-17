---
name: local-simulation
parent: engine
type: sub-program
status: specced
phase: 0
layer: 3 (engine primitive)
---

# local-simulation — Client-Side Prediction and Reconciliation

**What this sub-program is**: The client's prediction of how the world is moving, running between server updates. Because the server only sends 50 updates per second but the client renders 60+ frames per second, the client must extrapolate entity positions for the frames between updates. This sub-program does that extrapolation and corrects it when the server's authoritative update arrives.

**What it owns**:
- Velocity integration for all nearby entities (predict position from last known velocity)
- Simple terrain collision for predicted entities (stop at ground level)
- Server reconciliation: when authoritative update arrives, smoothly correct prediction error
- Player input application: apply the player's own movement locally before server confirms it (optimistic local response)

**What it does NOT own**:
- Authoritative physics (that is world/simulation/ on the server)
- The rendering of entities (that is engine/renderer/)
- Receiving network messages (that is the network layer above)
- The client world state object (it modifies the state, but does not own it)

**Prediction algorithm**:

```
EACH FRAME (called before rendering):

  dt = time since last frame (capped at MAX_PREDICTION_DT = 100ms)

  [Step 1] Apply player input immediately (optimistic):
    IF there is pending player movement input:
      local_player.position += input_direction * PLAYER_SPEED * dt
      local_player.position.y = max(local_player.position.y, terrain_height(local_player.position.xz))
      -- Don't send the input yet — it is batched and sent to server at tick rate

  [Step 2] Predict all other entities:
    FOR EACH entity E in nearby_entities:
      IF E.prediction_age > MAX_PREDICTION_AGE (500ms):
        -- Stop predicting — entity is too stale, wait for server update
        CONTINUE
      E.position += E.velocity * dt
      E.position.y = max(E.position.y, terrain_height(E.position.xz))
      E.prediction_age += dt

WHEN server update arrives (async, from network thread):
  FOR EACH entity_update U in server_update:
    E = nearby_entities[U.entity_id]
    IF E not found:
      ADD E to nearby_entities with U's state, prediction_age = 0
      CONTINUE

    discrepancy = distance(U.position, E.position)

    IF discrepancy < SNAP_SMALL (0.1 units):
      -- Imperceptible difference — apply directly
      E.position = U.position
    ELSE IF discrepancy < SNAP_MEDIUM (2.0 units):
      -- Small visible difference — smooth lerp over RECONCILE_TIME (100ms)
      E.reconcile_target = U.position
      E.reconcile_timer = RECONCILE_TIME
    ELSE IF discrepancy < SNAP_LARGE (10.0 units):
      -- Large difference (lag spike or fast movement) — fast lerp
      E.reconcile_target = U.position
      E.reconcile_timer = RECONCILE_TIME * 0.25
    ELSE:
      -- Teleport-scale difference — instant snap (never lerp a teleport)
      E.position = U.position

    E.velocity = U.velocity
    E.prediction_age = 0

  -- Apply ongoing reconciliation lerps
  FOR EACH entity E with active reconcile_timer:
    t = 1.0 - (E.reconcile_timer / RECONCILE_TIME_INITIAL)
    E.position = LERP(E.position, E.reconcile_target, LERP_SPEED * dt)
    E.reconcile_timer -= dt
    IF E.reconcile_timer <= 0:
      E.position = E.reconcile_target
```

**Player self-correction** (the hardest case):
The player's own movement is applied locally immediately (optimistic). When the server confirms a different position (e.g., the player was blocked by an object they didn't know about), the correction must be applied smoothly:

```
WHEN server confirms player position P_server but local prediction is P_local:
  IF distance(P_server, P_local) < PLAYER_CORRECTION_THRESHOLD (0.5 units):
    local_player.position = LERP(local_player.position, P_server, 0.3)
  ELSE:
    -- Significant correction needed (teleport, respawn, major lag)
    local_player.position = P_server
    play_correction_visual_effect()
```

**Phase 0 scope**: Full prediction for player and other visible players. Simple terrain collision only (no object collision in prediction — objects are static in Phase 0). Full reconciliation.
