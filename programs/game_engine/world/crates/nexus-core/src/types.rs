//! Entity, body, and world-state types.
//!
//! These map directly to the data shapes in simulation-contract.md
//! and world-state-contract.md. When the spec says "physics_body",
//! this is the Rust struct.

use crate::math::{Vec3f32, Vec3f64, Quat32, Aabb64};

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
