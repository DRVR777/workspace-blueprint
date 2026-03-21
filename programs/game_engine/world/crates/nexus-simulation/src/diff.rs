//! Stage 5: Diff
//!
//! Compares the original snapshot bodies to the post-simulation bodies.
//! Emits StateChangeEvents for everything that changed.

use nexus_core::types::{PhysicsBody, StateChangeEvent, TimestampMs};

/// Object change types (matches ObjectChangeType in object_state_change.fbs).
const CHANGE_POSITION_UPDATE: u8 = 0;
const CHANGE_CREATE: u8 = 1;
const CHANGE_DESTROY: u8 = 2;
#[allow(dead_code)] // Phase 1: used when object properties change (health, state, etc.)
const CHANGE_PROPERTY: u8 = 3;

/// Compare original bodies to final bodies. Emit state changes for differences.
pub fn compute_diff(
    original: &[PhysicsBody],
    final_bodies: &[PhysicsBody],
    timestamp_ms: TimestampMs,
) -> Vec<StateChangeEvent> {
    let mut changes = Vec::new();
    let mut seq: u64 = 0;

    // Check for changed and new bodies
    for body in final_bodies {
        if let Some(orig) = original.iter().find(|b| b.object_id == body.object_id) {
            // Existing body — check if position/orientation changed
            if position_changed(orig, body) {
                seq += 1;
                changes.push(StateChangeEvent {
                    sequence: seq,
                    timestamp_ms,
                    object_id: body.object_id,
                    change_type: CHANGE_POSITION_UPDATE,
                    payload: encode_position_update(body),
                });
            }
        } else {
            // New body (created this tick)
            seq += 1;
            changes.push(StateChangeEvent {
                sequence: seq,
                timestamp_ms,
                object_id: body.object_id,
                change_type: CHANGE_CREATE,
                payload: encode_position_update(body),
            });
        }
    }

    // Check for destroyed bodies
    for orig in original {
        if !final_bodies.iter().any(|b| b.object_id == orig.object_id) {
            seq += 1;
            changes.push(StateChangeEvent {
                sequence: seq,
                timestamp_ms,
                object_id: orig.object_id,
                change_type: CHANGE_DESTROY,
                payload: Vec::new(),
            });
        }
    }

    changes
}

/// Check if a body's position or orientation changed meaningfully.
fn position_changed(a: &PhysicsBody, b: &PhysicsBody) -> bool {
    const POS_EPSILON: f64 = 0.0001;
    const VEL_EPSILON: f32 = 0.0001;

    let dp = a.position.distance_to(b.position);
    if dp > POS_EPSILON {
        return true;
    }

    let dv = (a.velocity - b.velocity).magnitude();
    if dv > VEL_EPSILON {
        return true;
    }

    false
}

/// Encode position + velocity as bytes for the state change payload.
fn encode_position_update(body: &PhysicsBody) -> Vec<u8> {
    let mut payload = Vec::with_capacity(36);
    // Position (3x f64 = 24 bytes)
    payload.extend_from_slice(&(body.position.x as f32).to_le_bytes());
    payload.extend_from_slice(&(body.position.y as f32).to_le_bytes());
    payload.extend_from_slice(&(body.position.z as f32).to_le_bytes());
    // Velocity (3x f32 = 12 bytes)
    payload.extend_from_slice(&body.velocity.x.to_le_bytes());
    payload.extend_from_slice(&body.velocity.y.to_le_bytes());
    payload.extend_from_slice(&body.velocity.z.to_le_bytes());
    payload
}
