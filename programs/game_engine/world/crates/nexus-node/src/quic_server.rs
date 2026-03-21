//! QUIC transport server — native-client low-latency path (Quinn 0.11).
//!
//! Complements the WebSocket server (port 9001). Native clients connect here (port 9002).
//! Browser clients continue to use WebSocket; this is the upgrade path.
//!
//! ## Why QUIC over WebSocket for game servers
//!
//! WebSocket runs over TCP: a single dropped packet stalls ALL data until retransmitted
//! (head-of-line blocking). At 100Hz, a dropped physics delta waits for TCP retransmit
//! before the next tick can arrive. QUIC is UDP-based — dropped packets don't stall
//! subsequent ones, and UDP datagrams don't retransmit at all (best for physics: stale
//! frames are discarded anyway).
//!
//! ## Message routing (datagram vs stream)
//!
//! Inspects `msg_type` from the 16-byte wire header to choose transport:
//! - MSG_PHYSICS_DELTA (0x0002) → unreliable datagram (lost = fine, next tick replaces it)
//! - Everything else → reliable ordered unidirectional stream (HANDSHAKE, TICK_SYNC, etc.)
//!
//! ## TLS
//!
//! Self-signed certificate generated at startup via rcgen. For production, replace
//! with a proper cert from Let's Encrypt or similar.
//!
//! Spec: node-manager/MANIFEST.md "Transport layer"

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use quinn::{Connection, Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::sync::{RwLock, mpsc};

use nexus_core::types::{ChangeRequest, ChangeType};

use crate::{WorldState, QueuedAction};
use crate::clients::ClientManager;
use crate::protocol::{self, MSG_PHYSICS_DELTA};

/// Build a self-signed TLS certificate + Quinn ServerConfig.
///
/// Cert is valid for "localhost" and any connecting host that skips hostname verification.
/// In production, load a proper cert from disk instead.
pub fn make_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let certified = generate_simple_self_signed(vec!["localhost".to_string(), "nexus-node".to_string()])?;

    // rcgen 0.13: .der() returns a CertificateDer reference; clone to owned bytes
    let cert_der = certified.cert.der().to_vec();
    let key_der = certified.key_pair.serialize_der();

    let cert = CertificateDer::from(cert_der);
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));

    Ok(ServerConfig::with_single_cert(vec![cert], key)?)
}

/// Run the QUIC server.
///
/// Accepts native-client QUIC connections on `port`. Each connection gets the same
/// per-client channel as the WebSocket server — the tick loop is transport-agnostic.
pub async fn run(
    port: u16,
    state: Arc<RwLock<WorldState>>,
    action_tx: mpsc::UnboundedSender<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_config = make_server_config()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let endpoint = Endpoint::server(server_config, addr)?;

    tracing::info!("QUIC server listening on udp://{} (native clients)", addr);

    loop {
        match endpoint.accept().await {
            Some(incoming) => {
                let state = state.clone();
                let action_tx = action_tx.clone();
                let client_manager = client_manager.clone();

                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            tracing::info!("QUIC connection from {}", conn.remote_address());
                            if let Err(e) = handle_connection(conn, state, action_tx, client_manager).await {
                                tracing::warn!("QUIC connection error: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("QUIC incoming connection failed: {}", e);
                        }
                    }
                });
            }
            None => {
                tracing::info!("QUIC endpoint closed");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    conn: Connection,
    state: Arc<RwLock<WorldState>>,
    action_tx: mpsc::UnboundedSender<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let peer = conn.remote_address();

    // Per-client channel: tick loop pushes physics updates → this handler routes them
    // to datagram (PHYSICS_DELTA) or reliable stream (control messages).
    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // === HANDSHAKE via reliable bidirectional stream ===
    // Client opens a bidi stream, sends HANDSHAKE, we respond.
    let (mut send_stream, mut recv_stream) = conn.accept_bi().await?;

    // Read HANDSHAKE message
    let mut buf = vec![0u8; 512];
    let n = recv_stream.read(&mut buf).await?.unwrap_or(0);
    if n < protocol::HEADER_SIZE {
        return Err("QUIC: too short for HANDSHAKE".into());
    }

    let (msg_type, _payload) = protocol::decode_header(&buf[..n])
        .ok_or("QUIC: invalid HANDSHAKE header")?;

    if msg_type != protocol::MSG_HANDSHAKE {
        return Err(format!("QUIC: expected HANDSHAKE, got 0x{:04x}", msg_type).into());
    }

    // Spawn player entity
    let entity_id = {
        let mut world = state.write().await;
        world.spawn_player()
    };

    // Register in ClientManager (addr → per-client tx)
    {
        let mut cm = client_manager.write().await;
        cm.add(peer, entity_id, client_tx.clone());
    }

    // Send HANDSHAKE_RESPONSE via the bidi stream
    let response = protocol::encode_handshake_response(entity_id);
    send_stream.write_all(&response).await?;
    send_stream.finish()?;

    tracing::info!("QUIC {} HANDSHAKE accepted → entity {}", peer, entity_id);

    // Broadcast PLAYER_JOINED to all clients
    let joined_msg = protocol::encode_player_joined(entity_id);
    {
        let cm = client_manager.read().await;
        cm.send_to_all(joined_msg);
    }

    // Send initial TICK_SYNC + position update via datagram
    {
        let world = state.read().await;
        let tick_sync = protocol::encode_tick_sync(world.snapshot.tick_number);
        let _ = conn.send_datagram(Bytes::from(tick_sync));
        let positions = protocol::encode_position_updates(&world.snapshot.bodies);
        let _ = conn.send_datagram(Bytes::from(positions));
    }

    // === Main loop: receive client messages + forward tick-loop updates ===
    loop {
        tokio::select! {
            // Incoming datagram from client (PLAYER_ACTION)
            result = conn.read_datagram() => {
                match result {
                    Ok(data) => {
                        if let Some((msg_type, payload)) = protocol::decode_header(&data) {
                            if msg_type == protocol::MSG_PLAYER_ACTION {
                                if let Some((dx, dy, dz, seq, vehicle_mode, qx, qy, qz, qw, pos_x, pos_y, pos_z)) = protocol::decode_player_action(payload) {
                                    let mut action_payload = Vec::with_capacity(44);
                                    action_payload.extend_from_slice(&dx.to_le_bytes());
                                    action_payload.extend_from_slice(&dy.to_le_bytes());
                                    action_payload.extend_from_slice(&dz.to_le_bytes());
                                    action_payload.push(vehicle_mode);
                                    action_payload.extend_from_slice(&[0u8, 0u8, 0u8]);
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
                        }
                    }
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) |
                    Err(quinn::ConnectionError::ConnectionClosed { .. }) => {
                        tracing::info!("QUIC {} disconnected (entity {})", peer, entity_id);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("QUIC {} datagram error: {}", peer, e);
                        break;
                    }
                }
            }

            // Outbound: tick loop pushes filtered physics updates to client_rx
            data = client_rx.recv() => {
                match data {
                    Some(data) => {
                        if let Err(e) = send_to_client(&conn, data).await {
                            tracing::warn!("QUIC {} send error: {}", peer, e);
                            break;
                        }
                    }
                    None => break, // server shutting down
                }
            }
        }
    }

    // === Cleanup ===
    {
        let mut world = state.write().await;
        world.despawn_player(entity_id);
    }
    {
        let mut cm = client_manager.write().await;
        cm.remove(&peer);
    }
    let left_msg = protocol::encode_player_left(entity_id);
    {
        let cm = client_manager.read().await;
        cm.send_to_all(left_msg);
    }

    tracing::info!("QUIC {} cleaned up (entity {})", peer, entity_id);
    Ok(())
}

/// Route outbound data: physics deltas via unreliable datagram; everything else via
/// a new reliable unidirectional stream. The 2-byte msg_type in the wire header
/// determines the transport choice.
async fn send_to_client(
    conn: &Connection,
    data: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Inspect the message type from the 16-byte header
    let msg_type = if data.len() >= 2 {
        u16::from_le_bytes([data[0], data[1]])
    } else {
        0
    };

    if msg_type == MSG_PHYSICS_DELTA {
        // Unreliable datagram: fire-and-forget, zero retransmit cost.
        // Stale physics frames are superseded by the next tick anyway.
        conn.send_datagram(Bytes::from(data))?;
    } else {
        // Reliable ordered stream: FULL_SYNC, TICK_SYNC, PLAYER_JOINED/LEFT, HANDSHAKE_RESPONSE
        let mut stream = conn.open_uni().await?;
        stream.write_all(&data).await?;
        stream.finish()?;
    }
    Ok(())
}
