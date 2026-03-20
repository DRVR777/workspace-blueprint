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
pub const MSG_ENTITY_POSITION_UPDATE: u16 = 0x0001;
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

/// Encode ENTITY_POSITION_UPDATE for all dynamic bodies.
/// Each entity: 24 bytes (id:u32, x:f32, y:f32, z:f32, yaw:f32, flags:u32)
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

// ============================================================================
// Decode incoming messages
// ============================================================================

/// Decode PLAYER_ACTION payload → (direction_x, direction_y, direction_z)
pub fn decode_player_action(payload: &[u8]) -> Option<(f32, f32, f32)> {
    if payload.len() < 12 {
        return None;
    }
    let x = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let y = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let z = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    Some((x, y, z))
}
