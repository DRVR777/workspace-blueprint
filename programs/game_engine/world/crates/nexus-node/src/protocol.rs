//! Wire protocol — encode/decode binary messages.
//!
//! Header (16 bytes):
//!   [2] message_type   (u16 LE)
//!   [2] message_version (u16 LE)
//!   [4] sequence_number (u32 LE)
//!   [4] timestamp_ms    (u32 LE)
//!   [4] payload_length  (u32 LE)
//!
//! Message types (from shared/schemas/README.md):
//!   0x0001 ENTITY_POSITION_UPDATE (S→C)
//!   0x0004 TICK_SYNC (S→C)
//!   0x0005 PLAYER_JOINED (S→C)
//!   0x0006 PLAYER_LEFT (S→C)
//!   0x0100 HANDSHAKE (C→S)
//!   0x0101 HANDSHAKE_RESPONSE (S→C)
//!   0x0200 PLAYER_ACTION (C→S)

use nexus_core::types::PhysicsBody;

pub const HEADER_SIZE: usize = 16;

// Message type codes
pub const MSG_ENTITY_POSITION_UPDATE: u16 = 0x0001; // Legacy: kept for compat
pub const MSG_PHYSICS_DELTA: u16 = 0x0002;          // Newton-aware delta: only changed/non-inertial bodies
pub const MSG_FULL_SYNC: u16 = 0x0003;              // Full state: all dynamic bodies (periodic resync)
pub const MSG_TICK_SYNC: u16 = 0x0004;
pub const MSG_PLAYER_JOINED: u16 = 0x0005;
pub const MSG_PLAYER_LEFT: u16 = 0x0006;
pub const MSG_HANDSHAKE: u16 = 0x0100;
pub const MSG_HANDSHAKE_RESPONSE: u16 = 0x0101;
pub const MSG_PLAYER_ACTION: u16 = 0x0200;

// Sequence counter (global, atomic)
use std::sync::atomic::{AtomicU32, Ordering};
static SEQUENCE: AtomicU32 = AtomicU32::new(0);

fn next_sequence() -> u32 {
    SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// Encode a message header + payload into a binary frame.
fn encode_message(msg_type: u16, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());

    // Header
    buf.extend_from_slice(&msg_type.to_le_bytes());           // message_type
    buf.extend_from_slice(&1u16.to_le_bytes());               // message_version
    buf.extend_from_slice(&next_sequence().to_le_bytes());    // sequence_number
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32;
    buf.extend_from_slice(&ts.to_le_bytes());                 // timestamp_ms
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // payload_length

    // Payload
    buf.extend_from_slice(payload);
    buf
}

/// Decode a message header from raw bytes. Returns (type, payload_slice).
pub fn decode_header(data: &[u8]) -> Option<(u16, &[u8])> {
    if data.len() < HEADER_SIZE {
        return None;
    }

    let msg_type = u16::from_le_bytes([data[0], data[1]]);
    let payload_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;

    if data.len() < HEADER_SIZE + payload_len {
        return None;
    }

    Some((msg_type, &data[HEADER_SIZE..HEADER_SIZE + payload_len]))
}

// ============================================================================
// Encode outgoing messages
// ============================================================================

/// Encode HANDSHAKE_RESPONSE with the player's entity ID.
pub fn encode_handshake_response(entity_id: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&(entity_id as f64).to_le_bytes()); // f64 to match client
    encode_message(MSG_HANDSHAKE_RESPONSE, &payload)
}

/// Encode PLAYER_JOINED with entity ID.
pub fn encode_player_joined(entity_id: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(4);
    payload.extend_from_slice(&(entity_id as u32).to_le_bytes());
    encode_message(MSG_PLAYER_JOINED, &payload)
}

/// Encode PLAYER_LEFT with entity ID.
pub fn encode_player_left(entity_id: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(4);
    payload.extend_from_slice(&(entity_id as u32).to_le_bytes());
    encode_message(MSG_PLAYER_LEFT, &payload)
}

/// Encode TICK_SYNC with current tick number.
pub fn encode_tick_sync(tick: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&(tick as u32).to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes()); // padding
    encode_message(MSG_TICK_SYNC, &payload)
}

/// Encode ENTITY_POSITION_UPDATE for all dynamic bodies (legacy format, v1).
/// Each entity: 24 bytes (id:u32, x:f32, y:f32, z:f32, yaw:f32, flags:u32)
/// Kept for backward compatibility. New code should use encode_physics_delta / encode_full_sync.
pub fn encode_position_updates(bodies: &[PhysicsBody]) -> Vec<u8> {
    let dynamic_bodies: Vec<&PhysicsBody> = bodies.iter()
        .filter(|b| b.is_dynamic())
        .collect();

    let entity_size = 24usize;
    let mut payload = Vec::with_capacity(dynamic_bodies.len() * entity_size);

    for body in &dynamic_bodies {
        payload.extend_from_slice(&(body.object_id as u32).to_le_bytes());
        payload.extend_from_slice(&(body.position.x as f32).to_le_bytes());
        payload.extend_from_slice(&(body.position.y as f32).to_le_bytes());
        payload.extend_from_slice(&(body.position.z as f32).to_le_bytes());
        // Compute yaw from quaternion (standard quaternion → euler Y extraction)
        let siny_cosp = 2.0 * (body.orientation.w * body.orientation.y
            + body.orientation.x * body.orientation.z);
        let cosy_cosp = 1.0 - 2.0 * (body.orientation.y * body.orientation.y
            + body.orientation.z * body.orientation.z);
        let yaw = siny_cosp.atan2(cosy_cosp);
        payload.extend_from_slice(&yaw.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes()); // flags (reserved)
    }

    encode_message(MSG_ENTITY_POSITION_UPDATE, &payload)
}

/// Encode PHYSICS_DELTA (0x0002) — Newton-aware delta update.
///
/// Payload layout:
///   [4] ack_seq (u32 LE) — last input sequence number processed for this client.
///         Client uses this to prune its input buffer (discard inputs with seq ≤ ack_seq).
///         Use 0 if the server hasn't processed any input from this client yet.
///   [N × 26] entity records:
///     [4] entity_id (u32)
///     [1] motion_state (u8: 0=inertial, 1=accelerating, 2=collision)
///     [1] vehicle_mode (u8: 0=on foot, 1=plane/fly)
///     [2] x (i16, 1 unit = 1/32 m, range ±1024 m)
///     [2] y (i16)
///     [2] z (i16)
///     [2] vx (i16, 1 unit = 1/32 m/s, range ±1024 m/s)
///     [2] vy (i16)
///     [2] vz (i16)
///     [2] oqx (i16, value × 32767 → -1..1)
///     [2] oqy (i16)
///     [2] oqz (i16)
///     [2] oqw (i16)
///
/// Clients use ack_seq for server reconciliation, velocity+motion_state for prediction.
/// vehicle_mode and orientation quaternion drive client-side avatar/vehicle mesh selection.
pub fn encode_physics_delta(bodies_to_send: &[&PhysicsBody], ack_seq: u32) -> Vec<u8> {
    const ENTITY_SIZE: usize = 26;
    // 4-byte ack_seq header + per-entity data
    let mut payload = Vec::with_capacity(4 + bodies_to_send.len() * ENTITY_SIZE);
    payload.extend_from_slice(&ack_seq.to_le_bytes());

    for body in bodies_to_send {
        // Quantize position: 1/32 m precision, ±1024 m range
        let qx = ((body.position.x as f32) * 32.0).clamp(-32768.0, 32767.0) as i16;
        let qy = ((body.position.y as f32) * 32.0).clamp(-32768.0, 32767.0) as i16;
        let qz = ((body.position.z as f32) * 32.0).clamp(-32768.0, 32767.0) as i16;

        // Quantize velocity: 1/32 m/s precision, ±1024 m/s range
        let qvx = (body.velocity.x * 32.0).clamp(-32768.0, 32767.0) as i16;
        let qvy = (body.velocity.y * 32.0).clamp(-32768.0, 32767.0) as i16;
        let qvz = (body.velocity.z * 32.0).clamp(-32768.0, 32767.0) as i16;

        // Quantize orientation quaternion: i16 scaled by 32767
        let oqx = (body.orientation.x * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let oqy = (body.orientation.y * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let oqz = (body.orientation.z * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let oqw = (body.orientation.w * 32767.0).clamp(-32768.0, 32767.0) as i16;

        payload.extend_from_slice(&(body.object_id as u32).to_le_bytes());
        payload.push(body.motion_state as u8);
        payload.push(body.vehicle_mode);
        payload.extend_from_slice(&qx.to_le_bytes());
        payload.extend_from_slice(&qy.to_le_bytes());
        payload.extend_from_slice(&qz.to_le_bytes());
        payload.extend_from_slice(&qvx.to_le_bytes());
        payload.extend_from_slice(&qvy.to_le_bytes());
        payload.extend_from_slice(&qvz.to_le_bytes());
        payload.extend_from_slice(&oqx.to_le_bytes());
        payload.extend_from_slice(&oqy.to_le_bytes());
        payload.extend_from_slice(&oqz.to_le_bytes());
        payload.extend_from_slice(&oqw.to_le_bytes());
    }

    encode_message(MSG_PHYSICS_DELTA, &payload)
}

/// Encode FULL_SYNC (0x0003) — full state for a set of bodies (caller filters by interest).
/// Same payload format as PHYSICS_DELTA (ack_seq header + N×26 entity records).
/// Sent periodically (every PREDICTION_HORIZON_TICKS) to correct client prediction drift.
pub fn encode_full_sync(bodies: &[&PhysicsBody], ack_seq: u32) -> Vec<u8> {
    // Reuse the delta encoder — same wire format, different message type header
    let mut full_sync = encode_physics_delta(bodies, ack_seq);
    let type_bytes = MSG_FULL_SYNC.to_le_bytes();
    full_sync[0] = type_bytes[0];
    full_sync[1] = type_bytes[1];
    full_sync
}

// ============================================================================
// Decode incoming messages
// ============================================================================

/// Decode PLAYER_ACTION payload.
///
/// Returns: (dx, dy, dz, seq, vehicle_mode, qx, qy, qz, qw, pos_x, pos_y, pos_z)
///
/// Base wire format (16 bytes):
///   [4] dx, [4] dy, [4] dz, [4] seq
///
/// Extended vehicle format (48 bytes):
///   [4] dx, [4] dy, [4] dz, [4] seq
///   [1] vehicle_mode, [3] padding
///   [4] qx, [4] qy, [4] qz, [4] qw
///   [4] pos_x, [4] pos_y, [4] pos_z   ← client-authoritative position for vehicles
///
/// pos_x/y/z allow the server to directly track the client's plane position
/// instead of integrating velocity. Defaults to (0,0,0) when not present.
pub fn decode_player_action(payload: &[u8]) -> Option<(f32, f32, f32, u32, u8, f32, f32, f32, f32, f32, f32, f32)> {
    if payload.len() < 12 {
        return None;
    }
    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let z = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    let seq = if payload.len() >= 16 {
        u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]])
    } else {
        0
    };
    let (vehicle_mode, qx, qy, qz, qw) = if payload.len() >= 36 {
        let vm = payload[16];
        let qx = f32::from_le_bytes([payload[20], payload[21], payload[22], payload[23]]);
        let qy = f32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]);
        let qz = f32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]);
        let qw = f32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]);
        (vm, qx, qy, qz, qw)
    } else {
        (0u8, 0.0f32, 0.0f32, 0.0f32, 1.0f32)
    };
    let (pos_x, pos_y, pos_z) = if payload.len() >= 48 {
        let px = f32::from_le_bytes([payload[36], payload[37], payload[38], payload[39]]);
        let py = f32::from_le_bytes([payload[40], payload[41], payload[42], payload[43]]);
        let pz = f32::from_le_bytes([payload[44], payload[45], payload[46], payload[47]]);
        (px, py, pz)
    } else {
        (0.0f32, 0.0f32, 0.0f32)
    };
    Some((x, y, z, seq, vehicle_mode, qx, qy, qz, qw, pos_x, pos_y, pos_z))
}
