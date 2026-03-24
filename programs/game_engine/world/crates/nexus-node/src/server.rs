//! WebSocket server — accepts connections, handles HANDSHAKE, routes PLAYER_ACTION.
//!
//! Spec: node-manager/MANIFEST.md
//!
//! Connection lifecycle:
//!   1. Client connects via WebSocket
//!   2. Client sends HANDSHAKE
//!   3. Server spawns player entity in world state
//!   4. Server sends HANDSHAKE_RESPONSE with entity ID
//!   5. Server broadcasts PLAYER_JOINED to all other clients via ClientManager
//!   6. Client sends PLAYER_ACTION messages (movement input)
//!   7. Server queues actions for tick loop
//!   8. On disconnect: despawn entity, broadcast PLAYER_LEFT
//!
//! Each client gets a dedicated mpsc channel (client_tx stored in ClientManager,
//! client_rx used in this handler). The tick loop sends filtered physics updates
//! per-client via those channels. Global events (PLAYER_JOINED/LEFT, TICK_SYNC)
//! go to all clients via ClientManager::send_to_all.
//!
//! DEBUG LOGGING CHECKPOINTS:
//!   [L1] TCP_ACCEPT   — Connection reached server socket
//!   [L2] WS_UPGRADE   — WebSocket handshake completed
//!   [L3] PLAYER_JOIN  — Entity spawned, registered
//!   [L4] BROADCAST     — State sent to clients

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use std::sync::atomic::{AtomicU64, Ordering};

use nexus_core::types::{ChangeRequest, ChangeType, SpatialManifest};
use nexus_schema::SchemaRegistry;

use crate::{WorldState, QueuedAction};
use crate::clients::ClientManager;
use crate::protocol;

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn log_checkpoint(conn_id: u64, layer: &str, peer: &SocketAddr, msg: &str) {
    eprintln!("[{:>13}] [{}] conn={:03} {} {}", 
        timestamp_ms(), layer, conn_id, peer, msg);
}

/// Run the WebSocket server.
pub async fn run(
    port: u16,
    state: Arc<RwLock<WorldState>>,
    action_tx: mpsc::UnboundedSender<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
    schema_registry: Arc<SchemaRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on ws://{}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        
        // [L1] TCP_ACCEPT — Connection reached server socket
        let conn_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        log_checkpoint(conn_id, "TCP_ACCEPT", &peer_addr, "SYN received, accepting");
        tracing::info!("[L1] TCP_ACCEPT conn={:03} from {}", conn_id, peer_addr);

        let state = state.clone();
        let action_tx = action_tx.clone();
        let client_manager = client_manager.clone();

        let schema_registry = schema_registry.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                stream, peer_addr, state, action_tx, client_manager, schema_registry, conn_id,
            ).await {
                tracing::error!("Connection {} error: {}", peer_addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    state: Arc<RwLock<WorldState>>,
    action_tx: mpsc::UnboundedSender<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
    schema_registry: Arc<SchemaRegistry>,
    conn_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // [L2] WS_UPGRADE — WebSocket handshake completed
    log_checkpoint(conn_id, "WS_UPGRADE", &peer_addr, "HTTP→WS upgrade complete");
    tracing::info!("[L2] WS_UPGRADE conn={:03} {} WebSocket established", conn_id, peer_addr);

    // Per-client channel: tick loop pushes filtered physics updates via client_tx,
    // this handler forwards them to the WebSocket via client_rx.
    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // === Wait for HANDSHAKE ===
    let entity_id = loop {
        match ws_read.next().await {
            Some(Ok(msg)) if msg.is_binary() => {
                let data = msg.into_data();
                if let Some((msg_type, _payload)) = protocol::decode_header(&data) {
                    if msg_type == protocol::MSG_HANDSHAKE {
                        // Spawn player entity
                        let entity_id = {
                            let mut world = state.write().await;
                            world.spawn_player()
                        };

                        // Register client — stores client_tx so tick loop can send to it
                        let client_count = {
                            let mut cm = client_manager.write().await;
                            cm.add(peer_addr, entity_id, client_tx.clone());
                            cm.count()
                        };

                        // Send HANDSHAKE_RESPONSE
                        let response = protocol::encode_handshake_response(entity_id);
                        ws_write.send(Message::Binary(response)).await?;
                        tracing::info!("{} HANDSHAKE accepted → entity {}", peer_addr, entity_id);

                        // [L3] PLAYER_JOIN — Entity spawned and registered
                        log_checkpoint(conn_id, "PLAYER_JOIN", &peer_addr, 
                            &format!("entity={} total_clients={}", entity_id, client_count));
                        tracing::info!("[L3] PLAYER_JOIN conn={:03} {} entity={} clients={}", 
                            conn_id, peer_addr, entity_id, client_count);

                        // Broadcast PLAYER_JOINED to all clients (including the new one)
                        let joined_msg = protocol::encode_player_joined(entity_id);
                        let broadcast_count = {
                            let cm = client_manager.read().await;
                            cm.send_to_all(joined_msg.clone());
                            cm.count()
                        };
                        
                        // [L4] BROADCAST — State sent to all clients
                        log_checkpoint(conn_id, "BROADCAST", &peer_addr, 
                            &format!("PLAYER_JOINED sent to {} clients", broadcast_count));
                        tracing::info!("[L4] BROADCAST conn={:03} {} PLAYER_JOINED→{} clients", 
                            conn_id, peer_addr, broadcast_count);

                        // Send initial TICK_SYNC
                        let tick = {
                            let world = state.read().await;
                            world.snapshot.tick_number
                        };
                        let sync_msg = protocol::encode_tick_sync(tick);
                        ws_write.send(Message::Binary(sync_msg)).await?;

                        // Send initial position update (all current entities)
                        let positions = {
                            let world = state.read().await;
                            protocol::encode_position_updates(&world.snapshot.bodies)
                        };
                        ws_write.send(Message::Binary(positions)).await?;

                        break entity_id;
                    }
                }
            }
            Some(Ok(msg)) if msg.is_close() => {
                tracing::info!("{} disconnected before handshake", peer_addr);
                return Ok(());
            }
            Some(Err(e)) => {
                tracing::warn!("{} error before handshake: {}", peer_addr, e);
                return Ok(());
            }
            None => {
                tracing::info!("{} connection closed before handshake", peer_addr);
                return Ok(());
            }
            _ => continue, // skip non-binary messages
        }
    };

    // === Main loop: read client actions + forward per-client physics updates ===
    loop {
        tokio::select! {
            // Client sends a message
            msg = ws_read.next() => {
                match msg {
                    Some(Ok(msg)) if msg.is_binary() => {
                        let data = msg.into_data();
                        match protocol::decode_header(&data) {
                            Some((protocol::MSG_PLAYER_ACTION, payload)) => {
                                if let Some((dx, dy, dz, seq, vehicle_mode, qx, qy, qz, qw, pos_x, pos_y, pos_z)) = protocol::decode_player_action(payload) {
                                    // Pack ChangeRequest payload: direction + vehicle state + position
                                    // Layout: [dx][dy][dz][vehicle_mode][pad:3][qx][qy][qz][qw][pos_x][pos_y][pos_z]
                                    let mut action_payload = Vec::with_capacity(44);
                                    action_payload.extend_from_slice(&dx.to_le_bytes());
                                    action_payload.extend_from_slice(&dy.to_le_bytes());
                                    action_payload.extend_from_slice(&dz.to_le_bytes());
                                    action_payload.push(vehicle_mode);
                                    action_payload.extend_from_slice(&[0u8, 0u8, 0u8]); // padding
                                    action_payload.extend_from_slice(&qx.to_le_bytes());
                                    action_payload.extend_from_slice(&qy.to_le_bytes());
                                    action_payload.extend_from_slice(&qz.to_le_bytes());
                                    action_payload.extend_from_slice(&qw.to_le_bytes());
                                    action_payload.extend_from_slice(&pos_x.to_le_bytes());
                                    action_payload.extend_from_slice(&pos_y.to_le_bytes());
                                    action_payload.extend_from_slice(&pos_z.to_le_bytes());

                                    let request = ChangeRequest {
                                        source: entity_id,
                                        change_type: ChangeType::Move,
                                        object_id: entity_id,
                                        sequence_number: seq,
                                        requires_ack: false,
                                        payload: action_payload,
                                    };

                                    let _ = action_tx.send(QueuedAction {
                                        player_entity_id: entity_id,
                                        request,
                                    });
                                }
                            }
                            Some((protocol::MSG_SCHEMA_QUERY, payload)) => {
                                // Client received an unknown schema_id and is asking what it means.
                                // Respond with the JSON descriptor from the registry.
                                // This is the entire schema discovery protocol — one request, one response.
                                if let Some(queried_id) = protocol::decode_schema_query(payload) {
                                    let response = protocol::encode_schema_response(queried_id, &schema_registry);
                                    tracing::debug!("{} SCHEMA_QUERY 0x{:08x}", peer_addr, queried_id);
                                    ws_write.send(Message::Binary(response)).await?;
                                }
                            }
                            Some((protocol::MSG_AGENT_TASK, payload)) => {
                                // An AI agent has sent a task. Decode it and broadcast
                                // to all clients as MSG_AGENT_BROADCAST.
                                // Physics loop is not involved — this bypasses it entirely.
                                if let Some(task) = protocol::decode_agent_task(payload) {
                                    tracing::info!(
                                        "{} AGENT_TASK {} intent={:?} action={:?}",
                                        peer_addr, task.task_id, task.intent, task.action
                                    );
                                    let broadcast = protocol::encode_agent_broadcast(&task);
                                    let cm = client_manager.read().await;
                                    cm.send_to_all(broadcast);
                                }
                            }
                            Some((protocol::MSG_ENTER, payload)) => {
                                // Client is requesting the spatial manifest for a world.
                                // Decode the URI they want; default world if empty or unrecognised.
                                let world_id = protocol::decode_enter(payload)
                                    .unwrap_or_default();
                                let manifest = if world_id.is_empty() || world_id == "dworld://nexus.local/" {
                                    SpatialManifest::default_world()
                                } else {
                                    // Unknown address — return default world manifest.
                                    // Future: look up world registry.
                                    SpatialManifest::default_world()
                                };
                                let response = protocol::encode_spatial_manifest(&manifest);
                                tracing::info!("{} ENTER {:?} → manifest ({}B)", peer_addr, world_id, response.len());
                                ws_write.send(Message::Binary(response)).await?;
                            }
                            _ => {
                                // Unknown or unparseable message — ignore
                            }
                        }
                    }
                    Some(Ok(msg)) if msg.is_close() => {
                        tracing::info!("{} disconnected (entity {})", peer_addr, entity_id);
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!("{} read error: {}", peer_addr, e);
                        break;
                    }
                    None => {
                        tracing::info!("{} connection closed (entity {})", peer_addr, entity_id);
                        break;
                    }
                    _ => continue,
                }
            }

            // Tick loop sends filtered physics updates for this client
            data = client_rx.recv() => {
                match data {
                    Some(data) => {
                        if let Err(e) = ws_write.send(Message::Binary(data)).await {
                            tracing::warn!("{} write error: {}", peer_addr, e);
                            break;
                        }
                    }
                    None => break, // channel closed (server shutting down)
                }
            }
        }
    }

    // === Cleanup: despawn entity, broadcast PLAYER_LEFT ===
    {
        let mut world = state.write().await;
        world.despawn_player(entity_id);
    }
    let remaining_clients = {
        let mut cm = client_manager.write().await;
        cm.remove(&peer_addr);
        cm.count()
    };

    let left_msg = protocol::encode_player_left(entity_id);
    {
        let cm = client_manager.read().await;
        cm.send_to_all(left_msg);
    }

    // [DISCONNECT] Player left
    log_checkpoint(conn_id, "PLAYER_LEFT", &peer_addr, 
        &format!("entity={} remaining_clients={}", entity_id, remaining_clients));
    tracing::info!("[DISCONNECT] conn={:03} {} entity={} left, {} clients remain", 
        conn_id, peer_addr, entity_id, remaining_clients);
    Ok(())
}
