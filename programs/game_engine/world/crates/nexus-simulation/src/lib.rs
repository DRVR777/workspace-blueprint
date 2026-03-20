//! NEXUS Simulation — Layer 3
//!
//! The pure-function simulation core. Takes a world snapshot + inputs,
//! advances physics by one tick via Rapier, resolves collisions, processes
//! player actions, runs game rules, and returns a TickResult.
//!
//! No side effects. No I/O. Deterministic: same inputs → same outputs.
//!
//! Spec: world/programs/simulation/MANIFEST.md
//! Contract: shared/contracts/simulation-contract.md

mod validate;
mod actions;
mod physics;
mod rules;
mod diff;

use nexus_core::types::{WorldStateSnapshot, ChangeRequest, TickResult, PhysicsBody};

/// Pure-function version: takes immutable snapshot, returns diffs.
/// Use for replay, testing, and determinism verification.
pub fn run_tick(
    snapshot: &WorldStateSnapshot,
    inputs: &[ChangeRequest],
    dt: f32,
) -> TickResult {
    let mut bodies = snapshot.bodies.clone();
    run_tick_internal(&snapshot, &mut bodies, inputs, dt)
}

/// In-place version: mutates snapshot.bodies directly and returns the result.
/// Use in the tick loop for performance (avoids clone + re-apply).
pub fn run_tick_mut(
    snapshot: &mut WorldStateSnapshot,
    inputs: &[ChangeRequest],
    dt: f32,
) -> TickResult {
    // Save original bodies for diff
    let original = snapshot.bodies.clone();

    // Run the pipeline on the actual bodies
    let (validated, rejected) = validate::validate_inputs_from_bodies(&snapshot.bodies, &snapshot.domain_bounds, snapshot.domain_id, inputs);
    let action_events = actions::apply_actions(&mut snapshot.bodies, &validated);
    let collision_events = physics::physics_step(&mut snapshot.bodies, &snapshot.physics_config, dt);
    let rule_events = rules::apply_game_rules(&snapshot.bodies, &snapshot.domain_bounds, &collision_events, dt);
    let state_changes = diff::compute_diff(&original, &snapshot.bodies, snapshot.timestamp_ms);

    let mut events = Vec::new();
    events.extend(action_events);
    events.extend(collision_events);
    events.extend(rule_events);

    TickResult {
        next_tick_number: snapshot.tick_number + 1,
        state_changes,
        events,
        rejected_requests: rejected,
    }
}

/// Internal pipeline — operates on a mutable body vec.
fn run_tick_internal(
    snapshot: &WorldStateSnapshot,
    bodies: &mut Vec<PhysicsBody>,
    inputs: &[ChangeRequest],
    dt: f32,
) -> TickResult {
    let original_bodies = bodies.clone();

    // Stage 1: Validate inputs
    let (validated, rejected) = validate::validate_inputs(snapshot, inputs);

    // Stage 2: Apply validated actions to bodies
    let action_events = actions::apply_actions(bodies, &validated);

    // Stage 3: Physics step via Rapier
    let collision_events = physics::physics_step(bodies, &snapshot.physics_config, dt);

    // Stage 4: Game rules (Phase 0: boundary check only)
    let rule_events = rules::apply_game_rules(bodies, &snapshot.domain_bounds, &collision_events, dt);

    // Stage 5: Diff
    let state_changes = diff::compute_diff(&original_bodies, bodies, snapshot.timestamp_ms);

    let mut events = Vec::new();
    events.extend(action_events);
    events.extend(collision_events);
    events.extend(rule_events);

    TickResult {
        next_tick_number: snapshot.tick_number + 1,
        state_changes,
        events,
        rejected_requests: rejected,
    }
}
