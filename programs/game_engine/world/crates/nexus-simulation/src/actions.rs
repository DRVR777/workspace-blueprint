//! Stage 2: Action Application
//!
//! Applies validated player actions to the body list.
//! MOVE → applies force (NOT direct position change).
//! CREATE → spawns new body.
//! DESTROY → marks for removal.

use nexus_core::math::{Vec3f32, Quat32};
use nexus_core::types::{
    PhysicsBody, SimulationEvent, SimulationEventType, ChangeType, ShapeParams,
};
use nexus_core::constants::{PLAYER_MOVE_SPEED, PLAYER_JUMP_SPEED};

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
                    if let Some(body) = bodies.iter_mut().find(|b| b.object_id == action.request.object_id) {
                        if body.is_dynamic() {
                            let p = &action.request.payload;

                            // Extended payload (bytes 12+): vehicle_mode + quaternion + position.
                            // Layout: [1] vehicle_mode, [3] padding, [4] qx..qw, [4] px..pz
                            if p.len() >= 32 {
                                body.vehicle_mode = p[12];
                                let qx = f32::from_le_bytes([p[16], p[17], p[18], p[19]]);
                                let qy = f32::from_le_bytes([p[20], p[21], p[22], p[23]]);
                                let qz = f32::from_le_bytes([p[24], p[25], p[26], p[27]]);
                                let qw = f32::from_le_bytes([p[28], p[29], p[30], p[31]]);
                                body.orientation = Quat32::new(qx, qy, qz, qw);
                            } else {
                                body.vehicle_mode = 0;
                            }

                            if body.vehicle_mode != 0 {
                                // Vehicle mode: client is authoritative about position.
                                // Teleport body to pilot's local position each tick so observers
                                // track the actual flight path, not server physics integration.
                                if p.len() >= 44 {
                                    let px = f32::from_le_bytes([p[32], p[33], p[34], p[35]]);
                                    let py = f32::from_le_bytes([p[36], p[37], p[38], p[39]]);
                                    let pz = f32::from_le_bytes([p[40], p[41], p[42], p[43]]);
                                    body.position = nexus_core::math::Vec3f64::new(
                                        px as f64, py as f64, pz as f64,
                                    );
                                }
                                // Set velocity from nose direction so observers can extrapolate
                                body.velocity = Vec3f32::new(
                                    direction.x * PLAYER_MOVE_SPEED,
                                    direction.y * PLAYER_MOVE_SPEED,
                                    direction.z * PLAYER_MOVE_SPEED,
                                );
                            } else {
                                // Ground movement: standard velocity-based control
                                let new_vx = direction.x * PLAYER_MOVE_SPEED;
                                let new_vz = direction.z * PLAYER_MOVE_SPEED;
                                let new_vy = if direction.y > 0.0 && body.velocity.y.abs() < 0.5 {
                                    PLAYER_JUMP_SPEED
                                } else {
                                    body.velocity.y
                                };
                                body.velocity = Vec3f32::new(new_vx, new_vy, new_vz);
                            }
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
