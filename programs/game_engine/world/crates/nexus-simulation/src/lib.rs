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

/// The single public entry point of the simulation layer.
///
/// Called once per tick by the node-manager.
/// Returns everything that changed — the node-manager writes the results.
///
/// 5-stage pipeline:
///   1. Input Validation   → filter bad requests
///   2. Action Application → apply forces, spawn/destroy
///   3. Physics Step       → Rapier integration + collision
///   4. Game Rules         → AI, triggers, boundary check (stub Phase 0)
///   5. Diff               → compare before/after, emit state changes
pub fn run_tick(
    snapshot: &WorldStateSnapshot,
    inputs: &[ChangeRequest],
    dt: f32,
) -> TickResult {
    // Clone bodies — simulation mutates this working copy, original is for diff
    let mut bodies = snapshot.bodies.clone();

    // Stage 1: Validate inputs
    let (validated, rejected) = validate::validate_inputs(snapshot, inputs);

    // Stage 2: Apply validated actions to bodies
    let action_events = actions::apply_actions(&mut bodies, &validated);

    // Stage 3: Physics step via Rapier
    let collision_events = physics::physics_step(
        &mut bodies,
        &snapshot.physics_config,
        dt,
    );

    // Stage 4: Game rules (Phase 0: boundary check only, AI/triggers stubbed)
    let rule_events = rules::apply_game_rules(
        &mut bodies,
        &snapshot.domain_bounds,
        &collision_events,
        dt,
    );

    // Stage 5: Diff — compare original snapshot.bodies to final bodies
    let state_changes = diff::compute_diff(
        &snapshot.bodies,
        &bodies,
        snapshot.timestamp_ms,
    );

    // Combine all events
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
