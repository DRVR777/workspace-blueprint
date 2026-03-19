//! NEXUS Node — Layer 4+5
//!
//! The entry point of a world server process.
//! Runs the tick loop, accepts WebSocket connections, routes packets,
//! and broadcasts simulation results to connected clients.
//!
//! Spec: world/programs/node-manager/MANIFEST.md

mod tick_loop;
mod server;

use nexus_core::math::{Vec3f64, Aabb64};
use nexus_core::config::WorldPhysicsConfig;
use nexus_core::types::WorldStateSnapshot;
use nexus_core::constants::SECTOR_SIZE;
use nexus_spatial::SpatialIndex;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared world state accessible by tick loop and connection handlers.
pub struct WorldState {
    pub snapshot: WorldStateSnapshot,
    pub spatial_index: SpatialIndex,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9001);

    tracing::info!("NEXUS node starting on port {}", port);

    // Phase 0: hard-coded domain
    let domain = Aabb64::new(Vec3f64::ZERO, Vec3f64::new(SECTOR_SIZE, SECTOR_SIZE, SECTOR_SIZE));
    let spatial_index = SpatialIndex::new(domain);

    let world_state = Arc::new(RwLock::new(WorldState {
        snapshot: WorldStateSnapshot {
            tick_number: 0,
            timestamp_ms: 0,
            domain_id: 1,
            domain_bounds: domain,
            bodies: Vec::new(),
            physics_config: WorldPhysicsConfig::default(),
        },
        spatial_index,
    }));

    // Spawn tick loop and WebSocket server concurrently
    let tick_state = world_state.clone();
    let server_state = world_state.clone();

    tokio::select! {
        result = tick_loop::run(tick_state) => {
            tracing::error!("Tick loop exited: {:?}", result);
        }
        result = server::run(port, server_state) => {
            tracing::error!("WebSocket server exited: {:?}", result);
        }
    }
}
