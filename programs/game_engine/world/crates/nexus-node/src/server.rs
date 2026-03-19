//! WebSocket server — accepts client connections and routes packets.
//!
//! Spec: node-manager/MANIFEST.md "Client connection handling"

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

use crate::WorldState;

/// Run the WebSocket server on the given port.
pub async fn run(
    port: u16,
    state: Arc<RwLock<WorldState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on ws://{}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        tracing::info!("New connection from {}", peer_addr);

        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, state).await {
                tracing::error!("Connection {} error: {}", peer_addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    state: Arc<RwLock<WorldState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    tracing::info!("WebSocket upgraded for {}", peer_addr);

    // TODO Phase 0:
    // 1. Read HANDSHAKE message from client
    // 2. Validate auth token via session contract
    // 3. Send HANDSHAKE_RESPONSE (ACCEPTED)
    // 4. Send STATE_SNAPSHOT with current world state
    // 5. Broadcast PLAYER_JOINED to other clients
    // 6. Loop: read PLAYER_ACTION messages, enqueue in action queue

    // For now: echo messages back (proves connectivity)
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_binary() || msg.is_text() {
                    let data = msg.into_data();
                    tracing::debug!("Received {} bytes from {}", data.len(), peer_addr);

                    // Echo back for Phase 0 testing
                    write.send(tokio_tungstenite::tungstenite::Message::Binary(data.into()))
                        .await?;
                } else if msg.is_close() {
                    tracing::info!("Client {} disconnected", peer_addr);
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Read error from {}: {}", peer_addr, e);
                break;
            }
        }
    }

    // TODO: broadcast PLAYER_LEFT to other clients
    tracing::info!("Connection closed for {}", peer_addr);
    Ok(())
}
