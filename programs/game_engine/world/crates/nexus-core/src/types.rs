//! Entity, body, and world-state types.
//!
//! These map directly to the data shapes in simulation-contract.md
//! and world-state-contract.md. When the spec says "physics_body",
//! this is the Rust struct.

use crate::math::{Vec3f32, Vec3f64, Quat32, Aabb64};

// =============================================================================
// Packet Header — universal wire header for all NEXUS messages
// =============================================================================

/// The 20-byte header prepended to every message on the wire.
///
/// Layout (little-endian):
///   [0..2]  msg_type    — identifies the message schema
///   [2..4]  version     — codec version
///   [4..8]  sequence    — monotonically increasing counter
///   [8..12] timestamp   — Unix ms (lower 32 bits)
///   [12..16] payload_len — byte length of payload that follows
///   [16..20] schema_id  — identifies the payload schema for self-describing decode
///
/// `schema_id` is the field that makes this protocol self-describing.
/// A receiver that does not recognise `msg_type` can still route the packet
/// to the correct decoder by `schema_id`, and a receiver that knows neither
/// can safely skip `payload_len` bytes and move to the next frame.
/// This is what makes AGENT_TASK, SPATIAL_MANIFEST, KNOWLEDGE_QUERY, and
/// any future packet type expressible without modifying the physics layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub msg_type:    u16,
    pub version:     u16,
    pub sequence:    u32,
    pub timestamp:   u32,
    pub payload_len: u32,
    pub schema_id:   u32,
}

// =============================================================================
// SpatialManifest — world surface descriptor
// =============================================================================

/// Describes what is reachable and actionable at a spatial address.
///
/// Sent by the server in response to an ENTER request (MSG_ENTER = 0x0300).
/// Carried with schema_id = SCHEMA_SPATIAL_MANIFEST (0x00000002).
///
/// This is the HTTP/0.9 response for space: "you entered dworld://X, here is what it is."
/// Every field after `world_id` is optional — a minimal world advertises only its address.
///
/// Wire encoding (all strings are u16-length-prefixed UTF-8, 0 = absent):
///   [2+N] world_id    — dworld:// URI of this world
///   [2+N] geometry    — URL of 3D asset (IPFS hash, CDN URL) or empty
///   [1]   surface_count — number of named actions available
///   for each surface: [2+N] action name
///   [2+N] agent       — HTTPS endpoint of the agent that governs this world
///   [2+N] payment     — payment address (Solana pubkey, Ethereum address, etc.)
///   [2+N] semantic_identity — dworld:// address of the identity file governing this world
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialManifest {
    /// The canonical address of this world.
    pub world_id:  URI,
    /// URL or IPFS hash of the world's primary 3D geometry asset.
    pub geometry:  Option<String>,
    /// Named actions available to any Body that enters this world.
    /// Examples: "browse", "build", "talk", "trade", "vote"
    pub surface:   Vec<String>,
    /// HTTPS endpoint of the AI agent governing this world, if any.
    pub agent:     Option<String>,
    /// Payment address for access or actions that have a cost, if any.
    pub payment:   Option<String>,
    /// dworld:// address of the identity file that governs this world's semantic behavior.
    ///
    /// This is the link between the physics layer and the semantic layer.
    /// When a player enters this world, the server routes a SemanticPacket
    /// to this identity. The packet's path through the identity field leaves
    /// a physical trail via world_position on each hop.
    ///
    /// None = world has no semantic identity (pure physics, no routing).
    pub semantic_identity: Option<URI>,
}

impl SpatialManifest {
    /// Construct the manifest for the default physics world.
    /// This is what the server sends when a client enters with no specific address.
    pub fn default_world() -> Self {
        Self {
            world_id: "dworld://nexus.local/".to_string(),
            geometry: None,
            surface: vec![
                "move".to_string(),
                "build".to_string(),
                "talk".to_string(),
            ],
            agent: None,
            payment: None,
            semantic_identity: None,
        }
    }
}

// =============================================================================
// AgentTask — intent packet from an AI agent to the world
// =============================================================================

/// A task emitted by an AI agent after reading a SpatialManifest.
///
/// Sent by the agent client via MSG_AGENT_TASK (0x0400), schema_id = SCHEMA_AGENT_TASK.
/// Broadcast by the server to all clients in the domain.
/// Physics loop has no visibility into this type.
///
/// Wire encoding:
///   [8] task_id       (u64 LE)
///   [8] origin_id     (u64 LE) — entity ID of the agent; 0 = anonymous
///   [2+N] intent      (u16-len + UTF-8) — natural language statement of intent
///   [2+N] action      (u16-len + UTF-8) — one action from the surface vocabulary
///   [1]   context_count
///   for each: [8] object_id (u64 LE)
///   [4] deadline_ms   (u32 LE, 0 = no deadline)
#[derive(Debug, Clone, PartialEq)]
pub struct AgentTask {
    /// Monotonically increasing task ID (set by the agent).
    pub task_id:    u64,
    /// Entity ID of the agent body that issued this task. 0 = anonymous.
    pub origin_id:  ObjectId,
    /// Natural language statement of what the agent intends to do.
    pub intent:     String,
    /// The specific surface action selected from the SpatialManifest.
    pub action:     String,
    /// Object IDs the agent is acting on or observing (empty = world-scope).
    pub context:    Vec<ObjectId>,
    /// Deadline in ms from now. None = best effort, no expiry.
    pub deadline_ms: Option<u32>,
}

impl PacketHeader {
    pub const SIZE: usize = 20;

    /// Bootstrap schema IDs — hardcoded forever. Everything else lives in
    /// `world/schemas/*.json` and is discovered via nexus-schema's registry.
    ///
    /// See `nexus_schema::schema_id(name, version)` to compute any other ID.
    pub const SCHEMA_UNTYPED:  u32 = 0; // legacy / unknown — decode by msg_type only
    pub const SCHEMA_REGISTRY: u32 = 1; // schema discovery — MSG_SCHEMA_QUERY/RESPONSE
}

/// A Universal Resource Identifier — addressing scheme for worlds, agents, and assets.
/// Format: `dworld://<host>/<path>` for spatial locations,
///          `https://` for external resources, `ipfs://` for content-addressed assets.
pub type URI = String;

/// Unique identifier for every object/entity in the world. Never reused.
pub type ObjectId = u64;
pub type EntityId = u64;
pub type DomainId = u64;
pub type PlayerId = u64;

/// Timestamp in milliseconds since Unix epoch.
pub type TimestampMs = u64;

// =============================================================================
// Physics Body — from simulation-contract.md
// =============================================================================

/// Body category determines how the physics engine treats this body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BodyCategory {
    /// Affected by forces, collisions, gravity.
    Dynamic = 0,
    /// Never moves. Infinite mass. Terrain, walls, floors.
    Static = 1,
    /// Moves on a scripted path, not affected by forces. NPCs, elevators.
    Kinematic = 2,
}

/// Network motion state — Newton's 1st Law applied to bandwidth.
///
/// Computed each tick by the physics system after the Rapier step.
/// Determines whether the client needs a position update (ΣF≠0)
/// or can predict locally (ΣF≈0 → velocity is constant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MotionState {
    /// ΣF ≈ 0: velocity is constant plus predictable damping.
    /// Client predicts: pos += vel * dt. No server update needed.
    Inertial = 0,
    /// ΣF ≠ 0: player or agent applied force this tick.
    /// Client must receive new state.
    Accelerating = 1,
    /// Collision impulse applied — velocity changed discontinuously.
    /// Client must snap to server state immediately.
    Collision = 2,
}

/// Collision shape type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CollisionShape {
    Sphere = 0,
    Box = 1,
    ConvexHull = 2,
    /// Y-axis aligned capsule. Used for humanoid characters.
    Capsule = 3,
}

/// Shape parameters. Size depends on shape type.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeParams {
    Sphere { radius: f32 },
    Box { half_extents: Vec3f32 },
    ConvexHull { vertices: Vec<Vec3f32> },
    /// Y-axis aligned capsule: cylinder of `2*half_height` capped by hemispheres of `radius`.
    /// Total height = 2 * (half_height + radius).
    /// Use for humanoid avatars: half_height=0.5, radius=0.3 → 1.6m tall.
    Capsule { half_height: f32, radius: f32 },
}

/// A physics body in the simulation. Maps to PHYSICS_BODY in simulation-contract.md.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBody {
    pub object_id: ObjectId,
    pub category: BodyCategory,

    // Present for all categories
    pub position: Vec3f64,
    pub orientation: Quat32,
    pub shape: ShapeParams,

    // Dynamic only
    pub velocity: Vec3f32,
    pub angular_velocity: Vec3f32,
    pub mass: f32,
    pub moment_of_inertia: f32,
    pub applied_force: Vec3f32,
    pub applied_torque: Vec3f32,

    // Kinematic only
    pub scripted_velocity: Vec3f32,

    /// Motion state computed by the physics system each tick.
    /// Drives client-side prediction: Inertial bodies skip server updates.
    pub motion_state: MotionState,

    /// Vehicle the entity is currently piloting.
    /// 0 = on foot, 1 = plane/fly mode.
    /// Set from client PLAYER_ACTION payload; broadcast to all nearby clients.
    pub vehicle_mode: u8,
}

impl PhysicsBody {
    /// Create a new dynamic body at a position with a collision shape.
    pub fn new_dynamic(id: ObjectId, position: Vec3f64, mass: f32, shape: ShapeParams) -> Self {
        Self {
            object_id: id,
            category: BodyCategory::Dynamic,
            position,
            orientation: Quat32::IDENTITY,
            shape,
            velocity: Vec3f32::ZERO,
            angular_velocity: Vec3f32::ZERO,
            mass,
            moment_of_inertia: mass, // simplified: use mass as MoI
            applied_force: Vec3f32::ZERO,
            applied_torque: Vec3f32::ZERO,
            scripted_velocity: Vec3f32::ZERO,
            motion_state: MotionState::Inertial,
            vehicle_mode: 0,
        }
    }

    /// Create a new static body (terrain, walls).
    pub fn new_static(id: ObjectId, position: Vec3f64, shape: ShapeParams) -> Self {
        Self {
            object_id: id,
            category: BodyCategory::Static,
            position,
            orientation: Quat32::IDENTITY,
            shape,
            velocity: Vec3f32::ZERO,
            angular_velocity: Vec3f32::ZERO,
            mass: 0.0,
            moment_of_inertia: 0.0,
            applied_force: Vec3f32::ZERO,
            applied_torque: Vec3f32::ZERO,
            scripted_velocity: Vec3f32::ZERO,
            motion_state: MotionState::Inertial,
            vehicle_mode: 0,
        }
    }

    /// Create a kinematic body (NPC, elevator).
    pub fn new_kinematic(id: ObjectId, position: Vec3f64, shape: ShapeParams) -> Self {
        Self {
            object_id: id,
            category: BodyCategory::Kinematic,
            position,
            orientation: Quat32::IDENTITY,
            shape,
            velocity: Vec3f32::ZERO,
            angular_velocity: Vec3f32::ZERO,
            mass: 0.0,
            moment_of_inertia: 0.0,
            applied_force: Vec3f32::ZERO,
            applied_torque: Vec3f32::ZERO,
            scripted_velocity: Vec3f32::ZERO,
            motion_state: MotionState::Inertial,
            vehicle_mode: 0,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        self.category == BodyCategory::Dynamic
    }

    /// Bounding radius for broad-phase collision and visibility culling.
    pub fn bounding_radius(&self) -> f32 {
        match &self.shape {
            ShapeParams::Sphere { radius } => *radius,
            ShapeParams::Box { half_extents } => half_extents.magnitude(),
            ShapeParams::ConvexHull { vertices } => {
                vertices.iter()
                    .map(|v| v.magnitude())
                    .fold(0.0f32, f32::max)
            }
            ShapeParams::Capsule { half_height, radius } => half_height + radius,
        }
    }
}

// =============================================================================
// Change Request — from world-state-contract.md
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChangeType {
    Move = 0,
    PropertyChange = 1,
    Create = 2,
    Destroy = 3,
    Interact = 4,
}

#[derive(Debug, Clone)]
pub struct ChangeRequest {
    pub source: PlayerId,
    pub change_type: ChangeType,
    pub object_id: ObjectId,
    pub sequence_number: u32,
    pub requires_ack: bool,
    pub payload: Vec<u8>,
}

// =============================================================================
// Tick Result — from simulation-contract.md
// =============================================================================

#[derive(Debug, Clone)]
pub struct TickResult {
    pub next_tick_number: u64,
    pub state_changes: Vec<StateChangeEvent>,
    pub events: Vec<SimulationEvent>,
    pub rejected_requests: Vec<RejectedRequest>,
}

#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub sequence: u64,
    pub timestamp_ms: TimestampMs,
    pub object_id: ObjectId,
    pub change_type: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SimulationEventType {
    Collision = 0x0001,
    ThresholdCrossed = 0x0002,
    StateExhausted = 0x0003,
    AiAction = 0x0004,
    PlayerActionResult = 0x0005,
}

#[derive(Debug, Clone)]
pub struct SimulationEvent {
    pub event_type: SimulationEventType,
    pub object_id: ObjectId,
    pub other_id: ObjectId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RejectedRequest {
    pub original_sequence_number: u32,
    pub reason_code: u8,
}

// =============================================================================
// World State Snapshot — from simulation-contract.md
// =============================================================================

#[derive(Debug, Clone)]
pub struct WorldStateSnapshot {
    pub tick_number: u64,
    pub timestamp_ms: TimestampMs,
    pub domain_id: DomainId,
    pub domain_bounds: Aabb64,
    pub bodies: Vec<PhysicsBody>,
    pub physics_config: WorldPhysicsConfig,
}

// Re-export WorldPhysicsConfig from config module
pub use crate::config::WorldPhysicsConfig;

// =============================================================================
// SemanticPacket — the atom of the semantic routing layer
//
// A packet is the unit of work that moves through the identity-file network.
// It carries typed data (one of 5 kinds) and a metadata chain that grows
// with every hop. The chain is the packet's working memory — the accumulated
// record of every identity file that shaped it. No agent stores state.
// The packet carries everything forward.
//
// Five data types cover everything that can move through the network:
//
//   Text     — knowledge, natural language, questions, answers
//   Program  — executable code, config, schema definitions
//   Spatial  — geometry, position, SpatialManifest payloads
//   Signal   — events, measurements, quality scores, ticks
//   Identity — identity file content, agent profiles
//
// These are not arbitrary. They map to the five things that can exist
// as a node in the semantic field: a thought, a program, a place,
// an event, and a perspective. Every other data shape is a combination
// of these five.
// =============================================================================

/// The five kinds of payload a SemanticPacket can carry.
///
/// Typed so that identity files can declare which types they accept
/// in their surface field — a Text-only identity will not activate
/// on a Program packet. Type mismatch is total internal reflection.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketData {
    /// Natural language content: questions, answers, summaries, knowledge.
    Text(String),

    /// Executable or interpretable code, config files, schema definitions,
    /// build instructions. The content is meant to be run, not just read.
    Program {
        /// Language or runtime identifier: "rust", "python", "json", "shell", etc.
        lang: String,
        /// Source code or serialized program content.
        source: String,
        /// Optional entry point or invocation hint.
        entrypoint: Option<String>,
    },

    /// 3D spatial data: geometry descriptors, SpatialManifest payloads,
    /// positions, transforms, bounding volumes.
    Spatial {
        /// dworld:// URI this geometry is associated with.
        address: URI,
        /// Serialized spatial payload (FlatBuffers bytes or JSON).
        payload: Vec<u8>,
    },

    /// Events, measurements, quality scores, tick records.
    /// The immutable signal layer — what happened, when, and how well.
    Signal {
        /// Event type identifier (e.g. "quality_score", "tick", "activation").
        kind: String,
        /// Numeric measurement attached to this signal. None = boolean event.
        value: Option<f64>,
        /// Raw payload for structured signal data.
        payload: Vec<u8>,
    },

    /// Identity file content — the lens itself traveling as data.
    /// Used when identity files are created, updated, or transmitted
    /// as keys between worlds.
    Identity {
        /// dworld:// address of this identity file.
        address: URI,
        /// Markdown content of the identity file.
        content: String,
        /// Embedding vector. None = not yet indexed.
        /// Length depends on the embedding model (384 for AllMiniLML6V2).
        vector: Option<Vec<f32>>,
    },
}

/// One entry in the packet's provenance chain.
///
/// Written by the routing layer after each activation. The packet
/// carries its own history. No external memory needed.
#[derive(Debug, Clone, PartialEq)]
pub struct HopRecord {
    /// Monotonically increasing hop number within this chain (0-indexed).
    pub hop:          u32,
    /// Timestamp of this activation.
    pub timestamp_ms: TimestampMs,
    /// dworld:// address of the identity file that processed this hop.
    pub identity:     URI,
    /// Embedding vector of the identity file at activation time.
    /// Used for drift detection and quality attribution.
    /// Length depends on the embedding model (384 for AllMiniLML6V2).
    pub vector:       Option<Vec<f32>>,
    /// 3D world coordinate of the identity file at activation time.
    /// Set from IdentityFile::world_coord at the moment of routing.
    /// None = identity has no physical position (pre-layout).
    /// Together, the hop records form the packet's physical trail through the world.
    pub world_coord:  Option<[f32; 3]>,
    /// Quality score assigned to the output of this hop. None = not yet scored.
    pub quality:      Option<f32>,
}

/// The semantic packet — atom of the identity-file routing network.
///
/// Enters the field with `data` and zero hops. Each time it activates
/// an identity file, a `HopRecord` is appended to `meta`. The chain
/// grows until the packet reaches a terminal condition (depth exceeded,
/// output classified as final, or quality threshold met).
///
/// The packet is the conversation. Identity files are lenses it passes through.
/// The hop chain is its working memory. The world_position is its physical trail.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticPacket {
    /// Unique identifier for this packet. Never reused.
    pub id:             u64,
    /// Chain ID — groups all packets that belong to one logical request.
    /// Set by the originating caller. All hops in one reasoning chain share this.
    pub chain_id:       u64,
    /// The typed payload.
    pub data:           PacketData,
    /// Growing provenance chain. Empty on creation. One record per hop.
    pub meta:           Vec<HopRecord>,
    /// Maximum number of hops before this packet is forced to terminal.
    /// Prevents infinite loops. Default: 16.
    pub depth_limit:    u32,
    /// Origin address — who sent this packet into the field.
    pub origin:         URI,
    /// Terminal flag. Set by the routing layer when output is final.
    pub terminal:       bool,
    /// Current 3D position in the physical world.
    /// Updated in push_hop to the identity file's world_coord.
    /// None until the packet hops to an identity that has a world_coord.
    /// The full trail is in meta[*].world_coord — this field is the most recent.
    pub world_position: Option<[f32; 3]>,
}

impl SemanticPacket {
    /// Create a new packet with no hop history.
    pub fn new(id: u64, chain_id: u64, data: PacketData, origin: URI) -> Self {
        Self {
            id,
            chain_id,
            data,
            meta: Vec::new(),
            depth_limit: 16,
            origin,
            terminal: false,
            world_position: None,
        }
    }

    /// Record a hop. Called by the routing layer after each identity activation.
    ///
    /// `world_coord` is the 3D position of the identity file in the physical world.
    /// When Some, it updates `self.world_position` and is stored in the HopRecord.
    /// The sequence of HopRecord::world_coord values is the packet's physical trail.
    ///
    /// Returns the new hop number.
    pub fn push_hop(
        &mut self,
        timestamp_ms: TimestampMs,
        identity: URI,
        vector: Option<Vec<f32>>,
        world_coord: Option<[f32; 3]>,
    ) -> u32 {
        if let Some(coord) = world_coord {
            self.world_position = Some(coord);
        }
        let hop = self.meta.len() as u32;
        self.meta.push(HopRecord {
            hop,
            timestamp_ms,
            identity,
            vector,
            world_coord,
            quality: None,
        });
        hop
    }

    /// Set the quality score on the most recent hop.
    /// Called by the quality-score system after evaluating the hop's output.
    pub fn score_last_hop(&mut self, quality: f32) {
        if let Some(record) = self.meta.last_mut() {
            record.quality = Some(quality);
        }
    }

    /// True if this packet has exceeded its depth limit.
    pub fn depth_exceeded(&self) -> bool {
        self.meta.len() as u32 >= self.depth_limit
    }

    /// The identity file that last processed this packet, if any.
    pub fn last_identity(&self) -> Option<&str> {
        self.meta.last().map(|r| r.identity.as_str())
    }

    /// Mean quality score across all scored hops. None if no hops are scored.
    pub fn mean_quality(&self) -> Option<f32> {
        let scored: Vec<f32> = self.meta.iter()
            .filter_map(|r| r.quality)
            .collect();
        if scored.is_empty() {
            None
        } else {
            Some(scored.iter().sum::<f32>() / scored.len() as f32)
        }
    }
}

// =============================================================================
// Collision Data — from simulation-contract.md
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub struct CollisionData {
    pub contact_point: Vec3f32,
    pub contact_normal: Vec3f32,
    pub penetration_depth: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionPair {
    pub body_a_id: ObjectId,
    pub body_b_id: ObjectId,
    pub data: CollisionData,
}

#[cfg(test)]
mod semantic_packet_tests {
    use super::*;

    fn ts() -> TimestampMs { 1_000_000 }

    #[test]
    fn new_packet_has_no_hops() {
        let p = SemanticPacket::new(1, 1, PacketData::Text("hello".into()), "dworld://test/".into());
        assert!(p.meta.is_empty());
        assert!(!p.terminal);
        assert!(!p.depth_exceeded());
        assert_eq!(p.last_identity(), None);
        assert_eq!(p.mean_quality(), None);
        assert_eq!(p.world_position, None);
    }

    #[test]
    fn world_position_tracks_hop_trail() {
        let mut p = SemanticPacket::new(8, 8, PacketData::Text("q".into()), "dworld://x/".into());
        // First hop: identity has no world coord
        p.push_hop(ts(), "dworld://a/".into(), None, None);
        assert_eq!(p.world_position, None);
        assert_eq!(p.meta[0].world_coord, None);
        // Second hop: identity has a world coord
        p.push_hop(ts() + 1, "dworld://b/".into(), None, Some([10.0, 20.0, 30.0]));
        assert_eq!(p.world_position, Some([10.0, 20.0, 30.0]));
        assert_eq!(p.meta[1].world_coord, Some([10.0, 20.0, 30.0]));
        // Third hop: identity has a different world coord — packet moves
        p.push_hop(ts() + 2, "dworld://c/".into(), None, Some([40.0, 50.0, 60.0]));
        assert_eq!(p.world_position, Some([40.0, 50.0, 60.0]));
        // Full trail preserved in hop records
        assert_eq!(p.meta[0].world_coord, None);
        assert_eq!(p.meta[1].world_coord, Some([10.0, 20.0, 30.0]));
        assert_eq!(p.meta[2].world_coord, Some([40.0, 50.0, 60.0]));
    }

    #[test]
    fn hop_chain_grows_with_each_activation() {
        let mut p = SemanticPacket::new(2, 2, PacketData::Text("q".into()), "dworld://origin/".into());
        let h0 = p.push_hop(ts(), "dworld://identity/A".into(), None, None);
        let h1 = p.push_hop(ts() + 1, "dworld://identity/B".into(), Some(vec![0.8f32, 0.3, 0.5, 0.9, 0.7]), Some([1.0, 2.0, 3.0]));
        assert_eq!(h0, 0);
        assert_eq!(h1, 1);
        assert_eq!(p.meta.len(), 2);
        assert_eq!(p.last_identity(), Some("dworld://identity/B"));
    }

    #[test]
    fn quality_scoring_on_last_hop() {
        let mut p = SemanticPacket::new(3, 3, PacketData::Text("q".into()), "dworld://x/".into());
        p.push_hop(ts(), "dworld://identity/A".into(), None, None);
        p.push_hop(ts() + 1, "dworld://identity/B".into(), None, None);
        p.score_last_hop(0.9);
        assert_eq!(p.meta[0].quality, None);
        assert_eq!(p.meta[1].quality, Some(0.9));
    }

    #[test]
    fn mean_quality_ignores_unscored_hops() {
        let mut p = SemanticPacket::new(4, 4, PacketData::Text("q".into()), "dworld://x/".into());
        p.push_hop(ts(), "dworld://a/".into(), None, None);
        p.push_hop(ts(), "dworld://b/".into(), None, None);
        p.push_hop(ts(), "dworld://c/".into(), None, None);
        // Score only first and last
        p.meta[0].quality = Some(0.8);
        p.meta[2].quality = Some(0.4);
        let mean = p.mean_quality().unwrap();
        assert!((mean - 0.6).abs() < 1e-5);
    }

    #[test]
    fn depth_exceeded_when_hops_reach_limit() {
        let mut p = SemanticPacket::new(5, 5, PacketData::Text("q".into()), "dworld://x/".into());
        p.depth_limit = 3;
        for i in 0..3 {
            assert!(!p.depth_exceeded());
            p.push_hop(ts() + i as u64, format!("dworld://{i}/"), None, None);
        }
        assert!(p.depth_exceeded());
    }

    #[test]
    fn program_packet_carries_lang_and_source() {
        let p = SemanticPacket::new(6, 6,
            PacketData::Program {
                lang: "rust".into(),
                source: "fn main() {}".into(),
                entrypoint: Some("main".into()),
            },
            "dworld://compiler/".into(),
        );
        match &p.data {
            PacketData::Program { lang, source, entrypoint } => {
                assert_eq!(lang, "rust");
                assert_eq!(source, "fn main() {}");
                assert_eq!(entrypoint.as_deref(), Some("main"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn identity_packet_carries_vector() {
        let vec = vec![0.9f32, 0.1, 0.5, 0.8, 0.3];
        let p = SemanticPacket::new(7, 7,
            PacketData::Identity {
                address: "dworld://council/ORACLE".into(),
                content: "# ORACLE\nYou are the oracle.".into(),
                vector: Some(vec.clone()),
            },
            "dworld://make_agent/".into(),
        );
        match &p.data {
            PacketData::Identity { vector, .. } => {
                assert_eq!(*vector, Some(vec));
            }
            _ => panic!("wrong variant"),
        }
    }
}
