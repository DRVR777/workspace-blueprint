//! Stage 2: Action Application
//!
//! Applies validated player actions to the body list.
//! MOVE → applies force (NOT direct position change).
//! CREATE → spawns new body.
//! DESTROY → marks for removal.

use nexus_core::math::Vec3f32;
use nexus_core::types::{
    PhysicsBody, SimulationEvent, SimulationEventType, ChangeType, ShapeParams,
};
use nexus_core::constants::PLAYER_MOVE_FORCE_MULTIPLIER;

use crate::validate::ValidatedAction;

/// Apply all validated actions to the body list. Returns action result events.
pub fn apply_actions(
    bodies: &mut Vec<PhysicsBody>,
    actions: &[ValidatedAction],
) -> Vec<SimulationEvent> {
    let mut events = Vec::new();
    let mut to_destroy: Vec<u64> = Vec::new();

    for action in actions {
        match action.request.change_type {
            ChangeType::Move => {
                if let Some(direction) = action.clamped_direction {
                    // Find the body and apply force
                    if let Some(body) = bodies.iter_mut().find(|b| b.object_id == action.request.object_id) {
                        if body.is_dynamic() {
                            body.applied_force += direction * PLAYER_MOVE_FORCE_MULTIPLIER;
                        }
                    }
                }

                events.push(SimulationEvent {
                    event_type: SimulationEventType::PlayerActionResult,
                    object_id: action.request.object_id,
                    other_id: 0,
                    payload: action.request.sequence_number.to_le_bytes().to_vec(),
                });
            }

            ChangeType::Create => {
                // Phase 0: create a simple dynamic sphere at the position encoded in payload
                let pos = decode_position(&action.request.payload);
                let new_body = PhysicsBody::new_dynamic(
                    action.request.object_id,
                    pos,
                    1.0, // default mass
                    ShapeParams::Sphere { radius: 0.5 },
                );
                bodies.push(new_body);

                events.push(SimulationEvent {
                    event_type: SimulationEventType::PlayerActionResult,
                    object_id: action.request.object_id,
                    other_id: 0,
                    payload: action.request.sequence_number.to_le_bytes().to_vec(),
                });
            }

            ChangeType::Destroy => {
                to_destroy.push(action.request.object_id);

                events.push(SimulationEvent {
                    event_type: SimulationEventType::PlayerActionResult,
                    object_id: action.request.object_id,
                    other_id: 0,
                    payload: action.request.sequence_number.to_le_bytes().to_vec(),
                });
            }

            ChangeType::Interact | ChangeType::PropertyChange => {
                // Phase 0: no-op for these types
                events.push(SimulationEvent {
                    event_type: SimulationEventType::PlayerActionResult,
                    object_id: action.request.object_id,
                    other_id: 0,
                    payload: action.request.sequence_number.to_le_bytes().to_vec(),
                });
            }
        }
    }

    // Remove destroyed bodies
    bodies.retain(|b| !to_destroy.contains(&b.object_id));

    events
}

fn decode_position(payload: &[u8]) -> nexus_core::math::Vec3f64 {
    if payload.len() < 24 {
        return nexus_core::math::Vec3f64::ZERO;
    }
    let x = f64::from_le_bytes(payload[0..8].try_into().unwrap());
    let y = f64::from_le_bytes(payload[8..16].try_into().unwrap());
    let z = f64::from_le_bytes(payload[16..24].try_into().unwrap());
    nexus_core::math::Vec3f64::new(x, y, z)
}
