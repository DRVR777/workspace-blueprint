//! NEXUS Node — Layer 4+5
//!
//! Entry point of a world server process.
//! Runs the tick loop, accepts WebSocket connections, routes packets,
//! and broadcasts simulation results to connected clients.
//!
//! Spec: world/programs/node-manager/MANIFEST.md

mod tick_loop;
mod server;
mod quic_server;
mod protocol;
mod clients;

use nexus_core::math::{Vec3f64, Aabb64};
use nexus_core::config::WorldPhysicsConfig;
use nexus_core::types::{WorldStateSnapshot, PhysicsBody, ShapeParams, ChangeRequest};
use nexus_core::constants::{SECTOR_SIZE, QUIC_PORT};
use nexus_spatial::SpatialIndex;

use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use clients::ClientManager;

/// Shared world state accessible by tick loop and connection handlers.
pub struct WorldState {
    pub snapshot: WorldStateSnapshot,
    pub spatial_index: SpatialIndex,
    /// Next entity ID to assign (monotonically increasing).
    pub next_entity_id: u64,
}

impl WorldState {
    /// Spawn a new player entity at the spawn point. Returns the entity ID.
    pub fn spawn_player(&mut self) -> u64 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        // Capsule: half_height=0.5m + radius=0.3m → 1.6m tall humanoid.
        // Prevents avatar from spinning on contact; gives correct slope/step behavior.
        let body = PhysicsBody::new_dynamic(
            id,
            Vec3f64::new(0.0, 1.0, 5.0), // spawn position
            70.0, // ~human mass kg
            ShapeParams::Capsule { half_height: 0.5, radius: 0.3 },
        );

        self.snapshot.bodies.push(body);
        self.spatial_index.insert(id, Vec3f64::new(0.0, 1.0, 5.0), 0.8); // bounding_radius = 0.5+0.3

        tracing::info!("Spawned player entity {}", id);
        id
    }

    /// Remove a player entity.
    pub fn despawn_player(&mut self, entity_id: u64) {
        self.snapshot.bodies.retain(|b| b.object_id != entity_id);
        self.spatial_index.remove(entity_id);
        tracing::info!("Despawned player entity {}", entity_id);
    }
}

/// Action queued from a client connection to the tick loop.
pub struct QueuedAction {
    pub player_entity_id: u64,
    pub request: ChangeRequest,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9001);

    tracing::info!("NEXUS node starting on port {}", port);

    // Phase 0: hard-coded domain (1000³ units)
    let domain = Aabb64::new(Vec3f64::ZERO, Vec3f64::new(SECTOR_SIZE, SECTOR_SIZE, SECTOR_SIZE));

    // Add a ground plane (static body)
    let bodies = vec![PhysicsBody::new_static(
        0, // ground entity ID
        Vec3f64::new(0.0, -0.5, 0.0), // origin-centered ground — matches client Terrain at (0,0,0)
        ShapeParams::Box {
            half_extents: nexus_core::math::Vec3f32::new(500.0, 0.5, 500.0),
        },
    )];

    let spatial_index = SpatialIndex::new(domain);

    let world_state = Arc::new(RwLock::new(WorldState {
        snapshot: WorldStateSnapshot {
            tick_number: 0,
            timestamp_ms: 0,
            domain_id: 1,
            domain_bounds: domain,
            bodies,
            physics_config: WorldPhysicsConfig::default(),
        },
        spatial_index,
        next_entity_id: 100, // IDs 0-99 reserved for world objects
    }));

    // Action queue: clients push actions, tick loop drains them
    let (action_tx, action_rx) = mpsc::unbounded_channel::<QueuedAction>();

    // Client manager — holds per-client send channels; no global broadcast needed
    let client_manager = Arc::new(RwLock::new(ClientManager::new()));

    // Spawn tick loop and WebSocket server concurrently
    let tick_state = world_state.clone();
    let tick_clients = client_manager.clone();

    let server_state = world_state.clone();
    let server_clients = client_manager.clone();

    let quic_state = world_state.clone();
    let quic_action_tx = action_tx.clone();
    let quic_clients = client_manager.clone();

    tokio::select! {
        result = tick_loop::run(tick_state, action_rx, tick_clients) => {
            tracing::error!("Tick loop exited: {:?}", result);
        }
        result = server::run(port, server_state, action_tx, server_clients) => {
            tracing::error!("WebSocket server exited: {:?}", result);
        }
        result = quic_server::run(QUIC_PORT, quic_state, quic_action_tx, quic_clients) => {
            tracing::error!("QUIC server exited: {:?}", result);
        }
    }
}
