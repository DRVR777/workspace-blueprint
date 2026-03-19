# simulation — Build Contract (Phase 0)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: 5-stage pipeline, Rapier integration, physics config |
| `../../shared/contracts/simulation-contract.md` | `run_tick` signature, data shapes, invariants |
| `../../shared/contracts/world-state-contract.md` | object_record, entity_record, change_request shapes |
| `../../_planning/adr/ADR-003-physics-integrator.md` | Semi-implicit Euler — Rapier handles this internally |
| `../../_planning/adr/ADR-004-collision-detection.md` | Proven library behind contract — Rapier selected |
| `../../_planning/adr/ADR-015-technology-stack.md` | Rust + Rapier + bevy_ecs/hecs |
| `../spatial/MANIFEST.md` | Spatial index contract (query_radius, query_box used in Stage 4) |

Do NOT load: engine/ files, renderer, network schemas, client-side code.

---

## Process

1. **Set up Rust crate** with Rapier dependency:
   - `rapier3d` with `simd-stable` feature
   - Crate name: `nexus-simulation`
   - Expose `run_tick()` as the single public entry point

2. **Implement Stage 1 — Input Validation**:
   - Validate each `change_request` against the snapshot
   - Produce `validated_actions` and `rejected_requests`
   - All error codes match `change_error` in world-state-contract

3. **Implement Stage 2 — Action Application**:
   - MOVE → `applied_force` on body (NOT direct position set)
   - CREATE → new `RigidBody` + `Collider` in Rapier world
   - DESTROY → mark for removal after physics step
   - Emit `PLAYER_ACTION_RESULT` events

4. **Implement Stage 3 — Physics Step** (Rapier integration):
   - Sync our body state → Rapier rigid bodies
   - Apply world gravity + force generators to each dynamic body
   - Call `PhysicsPipeline::step()`
   - Read back positions/velocities from Rapier
   - Apply post-Rapier velocity damping
   - Collect collision events from Rapier's event queue
   - Clear accumulated forces

5. **Implement Stage 4 — Game Rules** (Phase 0 stubs):
   - Entity AI: stub (return no actions)
   - State machines: stub (no advancement)
   - Triggers: stub (not fired)
   - Domain boundary check: REAL — emit THRESHOLD_CROSSED if body exits domain AABB

6. **Implement Stage 5 — Diff**:
   - Compare final snapshot to original
   - Emit `state_change_event` for every position, property, or state change
   - Emit CREATE/DESTROY events for added/removed bodies
   - Package into `tick_result`

7. **Write tests** (all 8 from MANIFEST.md testing requirements):
   - Determinism, gravity, collision, player move, rejection, boundary, performance, damping

8. **Write `output/phase0-complete.md`**: tick performance at 500 bodies, determinism verified, all tests passing.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 4 | Physics step running: ball drops under gravity, two spheres collide | approve / revise Rapier config |
| Step 6 | Full `run_tick` returns correct `tick_result` for a test scenario | approve → write tests / revise |
| Step 7 | All 8 tests passing, performance within budget | approve → write output / fix failures |

---

## Audit

Before writing to `output/`:
- [ ] `run_tick` is deterministic: verified by bitwise comparison test
- [ ] `run_tick` produces no side effects (no I/O, no mutation outside return value)
- [ ] Bodies processed in sorted order (by object_id) at every stage
- [ ] No `HashMap` iteration in physics-critical paths — `BTreeMap` or sorted `Vec` only
- [ ] Rapier version pinned in `Cargo.toml` (no `*` or `^` — exact version)
- [ ] Gravity correctly applied per world config (not hardcoded)
- [ ] Player MOVE applies force, not direct position change
- [ ] THRESHOLD_CROSSED emitted when body exits domain AABB
- [ ] All 8 tests pass
- [ ] 500-body tick completes in < 17ms

---

## Outputs

| Output | Location |
|--------|----------|
| Simulation crate | `src/` (Rust) |
| Unit + integration tests | `src/tests/` or `tests/` |
| Phase 0 completion summary | `output/phase0-complete.md` |
