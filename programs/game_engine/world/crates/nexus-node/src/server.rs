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

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};

use nexus_core::types::{ChangeRequest, ChangeType};

use crate::{WorldState, QueuedAction};
use crate::clients::ClientManager;
use crate::protocol;

/// Run the WebSocket server.
pub async fn run(
    port: u16,
    state: Arc<RwLock<WorldState>>,
    action_tx: mpsc::UnboundedSender<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on ws://{}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        tracing::info!("New TCP connection from {}", peer_addr);

        let state = state.clone();
        let action_tx = action_tx.clone();
        let client_manager = client_manager.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                stream, peer_addr, state, action_tx, client_manager,
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    tracing::info!("{} WebSocket upgraded", peer_addr);

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
                        {
                            let mut cm = client_manager.write().await;
                            cm.add(peer_addr, entity_id, client_tx.clone());
                        }

                        // Send HANDSHAKE_RESPONSE
                        let response = protocol::encode_handshake_response(entity_id);
                        ws_write.send(Message::Binary(response)).await?;
                        tracing::info!("{} HANDSHAKE accepted → entity {}", peer_addr, entity_id);

                        // Broadcast PLAYER_JOINED to all clients (including the new one)
                        let joined_msg = protocol::encode_player_joined(entity_id);
                        {
                            let cm = client_manager.read().await;
                            cm.send_to_all(joined_msg);
                        }

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
    {
        let mut cm = client_manager.write().await;
        cm.remove(&peer_addr);
    }

    let left_msg = protocol::encode_player_left(entity_id);
    {
        let cm = client_manager.read().await;
        cm.send_to_all(left_msg);
    }

    tracing::info!("{} cleaned up (entity {})", peer_addr, entity_id);
    Ok(())
}
