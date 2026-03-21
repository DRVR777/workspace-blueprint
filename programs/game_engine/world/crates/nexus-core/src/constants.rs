//! Engine-wide constants.
//!
//! Every magic number in the engine lives here. No magic numbers in other modules.
//! Sources: simulation-contract.md, ADR-001, ADR-003, simulation MANIFEST.md.

// =============================================================================
// Tick timing
// =============================================================================

/// Target tick duration in seconds (100 Hz).
pub const TARGET_TICK_DURATION: f32 = 0.01;

/// Maximum allowed dt — caps simulation step if a tick takes too long.
pub const MAX_TICK_DT: f32 = 0.05;

/// Target ticks per second.
pub const TARGET_TICK_RATE: u32 = 100;

// =============================================================================
// Physics defaults (ADR-003)
// =============================================================================

/// Default linear velocity damping per second.
pub const DEFAULT_DAMPING: f32 = 0.01;

/// Default angular velocity damping per second.
pub const DEFAULT_ANGULAR_DAMPING: f32 = 0.05;

/// Maximum velocity magnitude (prevents tunneling at 50Hz).
pub const MAX_VELOCITY: f32 = 300.0;

// =============================================================================
// Spatial (ADR-001, spatial MANIFEST.md)
// =============================================================================

/// Sector size in world units.
pub const SECTOR_SIZE: f64 = 1000.0;

/// Maximum objects per octree leaf before subdivision.
pub const MAX_OBJECTS_PER_LEAF: usize = 32;

/// Minimum objects per leaf — below this AND parent total < MERGE_THRESHOLD, merge.
pub const MIN_OBJECTS_PER_LEAF: usize = 8;

/// Parent merges children if total objects < this.
pub const MERGE_THRESHOLD: usize = 24;

/// Maximum octree depth (safeguard against degenerate cases).
pub const MAX_TREE_DEPTH: usize = 16;

// =============================================================================
// Action limits (simulation MANIFEST.md)
// =============================================================================

/// Maximum force magnitude from a player MOVE action (Newtons).
pub const MAX_MOVE_FORCE: f32 = 500.0;

/// Target walk speed for player movement (m/s).
/// Direct velocity set — no force accumulation. Matches client SPEED constant.
pub const PLAYER_MOVE_SPEED: f32 = 5.0;

/// Upward velocity applied on jump (m/s). Matches client JUMP_VELOCITY constant.
pub const PLAYER_JUMP_SPEED: f32 = 6.0;

/// Translates normalized input direction to force (legacy — kept for non-player dynamic bodies).
pub const PLAYER_MOVE_FORCE_MULTIPLIER: f32 = 50.0;

/// Maximum object spawns per tick (prevent spawn flooding).
pub const MAX_SPAWNS_PER_TICK: usize = 8;

/// Velocity below which a body is considered "at rest" (m/s).
pub const REST_VELOCITY_THRESHOLD: f32 = 0.05;

/// Squared velocity below which a body is tagged Inertial (m²/s²).
/// With direct-velocity movement, motion_state is based on speed not applied_force.
/// Bodies below this threshold are at rest → client needs no delta updates.
/// Bodies above it are moving → tagged Accelerating → delta sent every tick.
/// 0.05 m/s threshold: REST_VELOCITY_THRESHOLD² = 0.0025
pub const REST_VELOCITY_SQ: f32 = REST_VELOCITY_THRESHOLD * REST_VELOCITY_THRESHOLD;

/// Squared applied-force magnitude below which a body is tagged Inertial (N²).
/// Legacy — kept for non-player dynamic bodies that still use force accumulation.
pub const FORCE_THRESHOLD_SQ: f32 = 0.1;

/// Ticks between mandatory full-state resyncs for continuously inertial bodies.
/// At 100 Hz: 150 ticks = 1.5 seconds. Corrects accumulated client prediction drift.
/// Nemotron review: 3s was too long for continuously moving players.
pub const PREDICTION_HORIZON_TICKS: u64 = 150;

/// Duration body must be below threshold to trigger ON_REST (seconds).
pub const REST_DURATION_THRESHOLD: f32 = 1.0;

// =============================================================================
// Networking
// =============================================================================

/// Default server-side visibility radius (units).
pub const DEFAULT_VISIBILITY_RADIUS: f64 = 500.0;

/// WebSocket server port (browser clients, legacy).
pub const WEBSOCKET_PORT: u16 = 9001;

/// QUIC/Quinn server port (native clients — UDP, lower latency).
/// Browser clients use WebSocket; native clients use QUIC datagrams for physics.
pub const QUIC_PORT: u16 = 9002;

/// Base visibility radius for dynamic computation.
pub const BASE_VISIBILITY_RADIUS: f64 = 500.0;

/// Minimum visibility radius even under extreme load (50% of base).
pub const MIN_VISIBILITY_RADIUS: f64 = 250.0;

// =============================================================================
// Performance budgets (ms)
// =============================================================================

/// Total tick budget in milliseconds.
pub const TICK_BUDGET_MS: f32 = 10.0;

/// Simulation budget within the tick (node-manager reserves 2ms for I/O).
pub const SIMULATION_BUDGET_MS: f32 = 8.0;

/// Tick duration above which load warnings begin.
pub const HIGH_LOAD_THRESHOLD_MS: f32 = 9.0;

/// Number of consecutive overloaded ticks before requesting a domain split.
pub const LOAD_GRACE_TICKS: u32 = 100;
