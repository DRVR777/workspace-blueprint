# local-simulation — Build Contract (Phase 0)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: prediction algorithm, reconciliation thresholds, player self-correction |
| `../../shared/contracts/world-state-contract.md` | entity_record shape (position, velocity, prediction_age fields) |
| `../../shared/schemas/entity_position_update.fbs` | authoritative server update shape |

Do NOT load: world/ files, server simulation-contract, renderer, asset schemas.

---

## Process

1. Add `prediction_age: float32` and `reconcile_target: Vec3f32` and `reconcile_timer: float32` fields to the client's entity record (these are client-only — they do not exist in the server's world-state-contract).

2. Implement `predict_frame(dt: float32)` (MANIFEST.md §"EACH FRAME"):
   - Step 1: Apply pending player input immediately to `local_player.position` with terrain floor clamp
   - Step 2: For each entity in `nearby_entities`: advance position by `velocity * dt`, clamp to terrain height, increment `prediction_age`. Skip entities with `prediction_age > 500ms`.

3. Implement `apply_server_update(updates: EntityPositionUpdate)` (MANIFEST.md §"WHEN server update arrives"):
   - For each entity_update U: compute `discrepancy = distance(U.position, E.position)`
   - Apply the correct case: snap (< 0.1), lerp slow (< 2.0), lerp fast (< 10.0), teleport (≥ 10.0)
   - Reset `prediction_age = 0` and update `velocity`

4. Implement `reconcile_frame(dt: float32)`: advance all active `reconcile_timer` entities toward their `reconcile_target` using the lerp defined in MANIFEST.md.

5. Implement `apply_player_correction(server_position: Vec3f64)` (MANIFEST.md §"Player self-correction"):
   - If discrepancy < 0.5: soft lerp with factor 0.3
   - If discrepancy ≥ 0.5: instant snap + trigger visual correction effect (stub in Phase 0: log the snap)

6. Write unit tests:
   - Verify an entity with velocity (1, 0, 0) at position (0, 0, 0) predicts to (1, 0, 0) after 1 second
   - Verify a server update with small discrepancy (0.05) applies a direct snap
   - Verify a server update with medium discrepancy (1.0) starts a lerp and completes within RECONCILE_TIME
   - Verify a server update with large discrepancy (15.0) applies an instant teleport
   - Verify entities stop being predicted after `prediction_age > 500ms`

7. Write `output/phase0-complete.md`: prediction accuracy (average discrepancy at reconciliation), test results, observed visual smoothness notes.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 3 | Prediction + reconciliation working — unit test results for all 4 discrepancy cases | approve / adjust thresholds |
| Step 6 | Full test suite results | approve → write output / fix failures |

---

## Audit

Before writing to `output/`:
- [ ] `prediction_age` field correctly stops prediction at 500ms (not 501ms, not 499ms — exact)
- [ ] All 4 reconciliation cases (snap / lerp-slow / lerp-fast / teleport) implemented with thresholds from MANIFEST.md
- [ ] Player self-correction does NOT lerp teleports (instant snap for discrepancy ≥ 0.5)
- [ ] `predict_frame` does NOT modify `entity.velocity` — prediction uses velocity read-only
- [ ] `apply_server_update` sets `prediction_age = 0` — never leaves prediction_age stale after authoritative update
- [ ] All 5 unit tests pass

---

## Outputs

| Output | Location |
|--------|----------|
| Client prediction implementation | `src/` |
| Unit tests | `src/tests/` |
| Phase 0 completion summary | `output/phase0-complete.md` |
