//! Stage 4: Game Rules
//!
//! Phase 0: Only domain boundary detection (THRESHOLD_CROSSED).
//! Entity AI, state machines, and triggers are stubbed.

use nexus_core::math::Aabb64;
use nexus_core::types::{PhysicsBody, BodyCategory, SimulationEvent, SimulationEventType};

/// Apply game rules to the post-physics snapshot.
/// Phase 0: checks if dynamic bodies have exited the domain bounds.
pub fn apply_game_rules(
    bodies: &[PhysicsBody],
    domain_bounds: &Aabb64,
    _collision_events: &[SimulationEvent],
    _dt: f32,
) -> Vec<SimulationEvent> {
    let mut events = Vec::new();

    // 4d. Domain boundary check — emit THRESHOLD_CROSSED if body exits domain
    for body in bodies {
        if body.category != BodyCategory::Dynamic {
            continue;
        }

        if !domain_bounds.contains_point(body.position) {
            events.push(SimulationEvent {
                event_type: SimulationEventType::ThresholdCrossed,
                object_id: body.object_id,
                other_id: 0,
                payload: Vec::new(),
            });
        }
    }

    // 4a. Entity AI evaluation — Phase 0 stub: no AI
    // 4b. State machine advancement — Phase 0 stub: no state machines
    // 4c. Trigger evaluation — Phase 0 stub: no triggers

    events
}
