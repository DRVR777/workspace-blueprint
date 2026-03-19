//! Stage 1: Input Validation
//!
//! Validates each ChangeRequest against the current world snapshot.
//! Returns (validated_actions, rejected_requests).

use nexus_core::types::{
    WorldStateSnapshot, ChangeRequest, RejectedRequest, ChangeType, PhysicsBody,
};
use nexus_core::constants::{MAX_MOVE_FORCE, MAX_SPAWNS_PER_TICK};

/// Validated action — a ChangeRequest that passed all checks.
#[derive(Debug, Clone)]
pub struct ValidatedAction {
    pub request: ChangeRequest,
    pub clamped_direction: Option<nexus_core::math::Vec3f32>,
}

/// Validate all inputs against the snapshot. Returns (valid, rejected).
pub fn validate_inputs(
    snapshot: &WorldStateSnapshot,
    inputs: &[ChangeRequest],
) -> (Vec<ValidatedAction>, Vec<RejectedRequest>) {
    let mut validated = Vec::with_capacity(inputs.len());
    let mut rejected = Vec::new();
    let mut spawn_count = 0usize;

    for request in inputs {
        match validate_one(snapshot, request, &mut spawn_count) {
            Ok(action) => validated.push(action),
            Err(reason_code) => rejected.push(RejectedRequest {
                original_sequence_number: request.sequence_number,
                reason_code,
            }),
        }
    }

    (validated, rejected)
}

fn validate_one(
    snapshot: &WorldStateSnapshot,
    request: &ChangeRequest,
    spawn_count: &mut usize,
) -> Result<ValidatedAction, u8> {
    // CREATE doesn't need an existing object
    if request.change_type != ChangeType::Create {
        // Check object exists
        if !snapshot.bodies.iter().any(|b| b.object_id == request.object_id) {
            return Err(0x01); // NOT_FOUND
        }

        // Check domain ownership
        // (Phase 0: all objects are in our domain, skip this check)
    }

    match request.change_type {
        ChangeType::Move => {
            // Decode direction from payload (first 12 bytes = 3x f32)
            let direction = decode_vec3f32(&request.payload);
            let clamped = direction.clamped_magnitude(MAX_MOVE_FORCE);

            Ok(ValidatedAction {
                request: request.clone(),
                clamped_direction: Some(clamped),
            })
        }

        ChangeType::Create => {
            if *spawn_count >= MAX_SPAWNS_PER_TICK {
                return Err(0x04); // PHYSICS_VIOLATION (spawn limit)
            }
            *spawn_count += 1;
            Ok(ValidatedAction {
                request: request.clone(),
                clamped_direction: None,
            })
        }

        ChangeType::Destroy => {
            Ok(ValidatedAction {
                request: request.clone(),
                clamped_direction: None,
            })
        }

        ChangeType::Interact | ChangeType::PropertyChange => {
            Ok(ValidatedAction {
                request: request.clone(),
                clamped_direction: None,
            })
        }
    }
}

/// Decode a Vec3f32 from raw bytes (little-endian f32 x3).
fn decode_vec3f32(payload: &[u8]) -> nexus_core::math::Vec3f32 {
    if payload.len() < 12 {
        return nexus_core::math::Vec3f32::ZERO;
    }
    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let z = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    nexus_core::math::Vec3f32::new(x, y, z)
}
