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
use nexus_schema::SchemaRegistry;
use nexus_semantic::{IdentityStore, HttpState, http_router};
use nexus_semantic::identity::seed_identities;
use nexus_semantic::llm::LocalEmbedClient;
use nexus_semantic::worker::RoutingLoop;
use nexus_events::EventLog;

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::{RwLock, mpsc};
use axum;

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

    let semantic_port: u16 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9002);

    tracing::info!("NEXUS node starting on port {} (semantic HTTP: {})", port, semantic_port);

    // Load schema registry from world/schemas/ (env override: NEXUS_SCHEMAS_DIR)
    let schemas_dir = std::env::var("NEXUS_SCHEMAS_DIR")
        .unwrap_or_else(|_| "schemas".to_string());
    let schema_registry = Arc::new(
        SchemaRegistry::load_from_dir(std::path::Path::new(&schemas_dir))
    );
    tracing::info!("Schema registry: {} schemas loaded from {}", schema_registry.schema_count(), schemas_dir);

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
    let server_schemas = schema_registry.clone();

    let quic_state = world_state.clone();
    let quic_action_tx = action_tx.clone();
    let quic_clients = client_manager.clone();

    // ── Semantic network bootstrap ────────────────────────────────────────────
    // Seed the identity store with architecture documents, then start the
    // RoutingLoop.  The HTTP server makes the field callable from outside.
    let event_log = Arc::new(
        EventLog::open("nexus-events.db").unwrap_or_else(|_| {
            tracing::warn!("Could not open nexus-events.db; using in-memory log");
            EventLog::open_in_memory().unwrap()
        })
    );

    let llm: Arc<dyn nexus_semantic::LlmClient> = {
        match LocalEmbedClient::new() {
            Ok(c) => {
                tracing::info!("Semantic: local embedding model loaded");
                Arc::new(c)
            }
            Err(e) => {
                tracing::warn!("Semantic: local embed failed ({e}), using mock LLM");
                Arc::new(nexus_semantic::MockLlmClient::new())
            }
        }
    };

    let seed_store = IdentityStore::build(seed_identities());
    let (routing_loop, rx) = RoutingLoop::new(seed_store, llm, Arc::clone(&event_log));
    let _routing_handle = Arc::clone(&routing_loop).spawn(rx);
    tracing::info!("Semantic routing loop started");

    let http_state = Arc::new(HttpState {
        routing_loop,
        event_log,
        next_chain_id: Arc::new(AtomicU64::new(1)),
    });
    let semantic_router = http_router(http_state);

    tokio::select! {
        result = tick_loop::run(tick_state, action_rx, tick_clients) => {
            tracing::error!("Tick loop exited: {:?}", result);
        }
        result = server::run(port, server_state, action_tx, server_clients, server_schemas) => {
            tracing::error!("WebSocket server exited: {:?}", result);
        }
        result = quic_server::run(QUIC_PORT, quic_state, quic_action_tx, quic_clients) => {
            tracing::error!("QUIC server exited: {:?}", result);
        }
        result = async {
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], semantic_port));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            tracing::info!("Semantic HTTP listening on :{semantic_port}");
            axum::serve(listener, semantic_router).await
        } => {
            tracing::error!("Semantic HTTP server exited: {:?}", result);
        }
    }
}
