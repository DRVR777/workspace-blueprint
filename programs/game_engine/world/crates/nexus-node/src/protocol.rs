//! Wire protocol — encode/decode binary messages.
//!
//! Header (20 bytes, matches PacketHeader in nexus-core/src/types.rs):
//!   [0..2]   message_type   (u16 LE)
//!   [2..4]   message_version (u16 LE)
//!   [4..8]   sequence_number (u32 LE)
//!   [8..12]  timestamp_ms    (u32 LE)
//!   [12..16] payload_length  (u32 LE)
//!   [16..20] schema_id       (u32 LE) — identifies payload schema; 0 = untyped/legacy
//!
//! Message types (from shared/schemas/README.md):
//!   0x0001 ENTITY_POSITION_UPDATE (S→C)
//!   0x0004 TICK_SYNC (S→C)
//!   0x0005 PLAYER_JOINED (S→C)
//!   0x0006 PLAYER_LEFT (S→C)
//!   0x0100 HANDSHAKE (C→S)
//!   0x0101 HANDSHAKE_RESPONSE (S→C)
//!   0x0200 PLAYER_ACTION (C→S)

use nexus_core::types::{PhysicsBody, PacketHeader, SpatialManifest, AgentTask};
use nexus_schema::{schema_id, SchemaRegistry};

pub const HEADER_SIZE: usize = PacketHeader::SIZE; // 20

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
pub const MSG_ENTER: u16 = 0x0300;             // C→S: client requests world manifest
pub const MSG_SPATIAL_MANIFEST: u16 = 0x0301;  // S→C: world surface descriptor
pub const MSG_AGENT_TASK: u16 = 0x0400;        // C→S (agent): intent + selected action
pub const MSG_AGENT_BROADCAST: u16 = 0x0401;   // S→C (all): agent task broadcast to domain
// Re-export schema discovery message types so callers use protocol::MSG_SCHEMA_*
pub use nexus_schema::{MSG_SCHEMA_QUERY, MSG_SCHEMA_RESPONSE, MSG_SCHEMA_NOT_FOUND};

// Computed schema IDs — stable content-addressed hashes, not hardcoded constants.
// These are evaluated once and inlined by the compiler. Adding a new schema type
// requires only a new JSON file in world/schemas/ — no changes here.
#[allow(dead_code)] // used when physics delta encoding tags schema_id (next pass)
pub fn sid_physics_body()     -> u32 { schema_id("physics_body",     "1.0") }
pub fn sid_spatial_manifest() -> u32 { schema_id("spatial_manifest", "1.0") }
pub fn sid_agent_task()       -> u32 { schema_id("agent_task",       "1.0") }

// Sequence counter (global, atomic)
use std::sync::atomic::{AtomicU32, Ordering};
static SEQUENCE: AtomicU32 = AtomicU32::new(0);

fn next_sequence() -> u32 {
    SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// Encode a message header + payload, with an explicit schema_id.
fn encode_message_with_schema(msg_type: u16, payload: &[u8], schema_id: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());
    buf.extend_from_slice(&msg_type.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&next_sequence().to_le_bytes());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32;
    buf.extend_from_slice(&ts.to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&schema_id.to_le_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Encode a message header + payload into a binary frame.
fn encode_message(msg_type: u16, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());

    // Header (20 bytes — matches PacketHeader layout)
    buf.extend_from_slice(&msg_type.to_le_bytes());                // [0..2]  message_type
    buf.extend_from_slice(&1u16.to_le_bytes());                    // [2..4]  message_version
    buf.extend_from_slice(&next_sequence().to_le_bytes());         // [4..8]  sequence_number
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32;
    buf.extend_from_slice(&ts.to_le_bytes());                      // [8..12] timestamp_ms
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());  // [12..16] payload_length
    buf.extend_from_slice(&PacketHeader::SCHEMA_UNTYPED.to_le_bytes()); // [16..20] schema_id

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

/// Decode MSG_ENTER payload. Returns the world URI the client wants to enter.
/// Payload: [2] uri_len + [N] utf8 bytes. Empty URI means "default world".
pub fn decode_enter(payload: &[u8]) -> Option<String> {
    if payload.len() < 2 {
        return Some(String::new()); // empty = default world
    }
    let len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + len {
        return None;
    }
    String::from_utf8(payload[2..2 + len].to_vec()).ok()
}

// ============================================================================
// SpatialManifest encode/decode
// ============================================================================

fn push_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u16).to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn push_opt_str(buf: &mut Vec<u8>, s: &Option<String>) {
    match s {
        Some(v) => push_str(buf, v),
        None    => buf.extend_from_slice(&0u16.to_le_bytes()),
    }
}

/// Encode MSG_SPATIAL_MANIFEST with schema_id = SCHEMA_SPATIAL_MANIFEST.
///
/// Wire encoding:
///   [2+N] world_id (u16-len + UTF-8)
///   [2+N] geometry (u16-len + UTF-8; 0 = absent)
///   [1]   surface_count
///   for each surface: [2+N] name (u16-len + UTF-8)
///   [2+N] agent   (u16-len + UTF-8; 0 = absent)
///   [2+N] payment (u16-len + UTF-8; 0 = absent)
pub fn encode_spatial_manifest(m: &SpatialManifest) -> Vec<u8> {
    let mut payload = Vec::new();
    push_str(&mut payload, &m.world_id);
    push_opt_str(&mut payload, &m.geometry);
    payload.push(m.surface.len().min(255) as u8);
    for s in &m.surface {
        push_str(&mut payload, s);
    }
    push_opt_str(&mut payload, &m.agent);
    push_opt_str(&mut payload, &m.payment);

    encode_message_with_schema(MSG_SPATIAL_MANIFEST, &payload, sid_spatial_manifest())
}

#[allow(dead_code)] // used by agent runtime (Build 2) and tests
fn read_str<'a>(data: &'a [u8], pos: &mut usize) -> Option<String> {
    if *pos + 2 > data.len() { return None; }
    let len = u16::from_le_bytes([data[*pos], data[*pos + 1]]) as usize;
    *pos += 2;
    if *pos + len > data.len() { return None; }
    let s = String::from_utf8(data[*pos..*pos + len].to_vec()).ok()?;
    *pos += len;
    Some(s)
}

#[allow(dead_code)]
fn read_opt_str(data: &[u8], pos: &mut usize) -> Option<Option<String>> {
    let s = read_str(data, pos)?;
    Some(if s.is_empty() { None } else { Some(s) })
}

// ============================================================================
// AgentTask encode/decode
// ============================================================================

/// Encode MSG_AGENT_BROADCAST — the server's outbound broadcast of an agent task.
/// Uses schema_id = SCHEMA_AGENT_TASK so any receiver can identify it without knowing msg_type.
pub fn encode_agent_broadcast(task: &AgentTask) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&task.task_id.to_le_bytes());
    payload.extend_from_slice(&task.origin_id.to_le_bytes());
    push_str(&mut payload, &task.intent);
    push_str(&mut payload, &task.action);
    payload.push(task.context.len().min(255) as u8);
    for &id in &task.context {
        payload.extend_from_slice(&id.to_le_bytes());
    }
    payload.extend_from_slice(&task.deadline_ms.unwrap_or(0).to_le_bytes());
    encode_message_with_schema(MSG_AGENT_BROADCAST, &payload, sid_agent_task())
}

/// Decode an incoming MSG_AGENT_TASK payload sent by an agent client.
/// Returns None if the payload is malformed.
pub fn decode_agent_task(payload: &[u8]) -> Option<AgentTask> {
    if payload.len() < 16 { return None; }
    let task_id   = u64::from_le_bytes(payload[0..8].try_into().ok()?);
    let origin_id = u64::from_le_bytes(payload[8..16].try_into().ok()?);
    let mut pos = 16;
    let intent = read_str(payload, &mut pos)?;
    let action = read_str(payload, &mut pos)?;
    if pos >= payload.len() { return None; }
    let ctx_count = payload[pos] as usize;
    pos += 1;
    let mut context = Vec::with_capacity(ctx_count);
    for _ in 0..ctx_count {
        if pos + 8 > payload.len() { return None; }
        let id = u64::from_le_bytes(payload[pos..pos + 8].try_into().ok()?);
        context.push(id);
        pos += 8;
    }
    let deadline_ms = if pos + 4 <= payload.len() {
        let ms = u32::from_le_bytes(payload[pos..pos + 4].try_into().ok()?);
        if ms == 0 { None } else { Some(ms) }
    } else {
        None
    };
    Some(AgentTask { task_id, origin_id, intent, action, context, deadline_ms })
}

// ============================================================================
// Schema discovery — MSG_SCHEMA_QUERY / MSG_SCHEMA_RESPONSE
// ============================================================================

/// Decode a MSG_SCHEMA_QUERY payload. Returns the queried schema_id.
/// Payload: [4] schema_id (u32 LE).
pub fn decode_schema_query(payload: &[u8]) -> Option<u32> {
    if payload.len() < 4 { return None; }
    Some(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]))
}

/// Encode MSG_SCHEMA_RESPONSE: [4] schema_id + [N] JSON descriptor bytes.
pub fn encode_schema_response(queried_id: u32, registry: &SchemaRegistry) -> Vec<u8> {
    match registry.to_json_bytes(queried_id) {
        Some(json) => {
            let mut payload = Vec::with_capacity(4 + json.len());
            payload.extend_from_slice(&queried_id.to_le_bytes());
            payload.extend_from_slice(&json);
            encode_message_with_schema(MSG_SCHEMA_RESPONSE, &payload, PacketHeader::SCHEMA_REGISTRY)
        }
        None => {
            // Schema not in registry — send NOT_FOUND with just the queried ID
            let payload = queried_id.to_le_bytes().to_vec();
            encode_message_with_schema(MSG_SCHEMA_NOT_FOUND, &payload, PacketHeader::SCHEMA_REGISTRY)
        }
    }
}

/// Decode a SpatialManifest from a MSG_SPATIAL_MANIFEST payload.
#[allow(dead_code)]
pub fn decode_spatial_manifest(payload: &[u8]) -> Option<SpatialManifest> {
    let mut pos = 0;
    let world_id = read_str(payload, &mut pos)?;
    let geometry = read_opt_str(payload, &mut pos)?;
    if pos >= payload.len() { return None; }
    let surface_count = payload[pos] as usize;
    pos += 1;
    let mut surface = Vec::with_capacity(surface_count);
    for _ in 0..surface_count {
        surface.push(read_str(payload, &mut pos)?);
    }
    let agent   = read_opt_str(payload, &mut pos)?;
    let payment = read_opt_str(payload, &mut pos)?;
    Some(SpatialManifest { world_id, geometry, surface, agent, payment, semantic_identity: None })
}
