//! Client manager — tracks connected clients and their per-client send channels.

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;

/// Information about a connected client.
#[derive(Debug)]
pub struct ClientInfo {
    pub entity_id: u64,
    /// Send end of the per-client channel. Tick loop pushes filtered physics updates here.
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
}

/// Manages the mapping between client connections and game entities.
pub struct ClientManager {
    /// addr → client info
    clients: HashMap<SocketAddr, ClientInfo>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Register a new client with their assigned entity ID and send channel.
    pub fn add(&mut self, addr: SocketAddr, entity_id: u64, tx: mpsc::UnboundedSender<Vec<u8>>) {
        self.clients.insert(addr, ClientInfo { entity_id, tx });
    }

    /// Remove a client, returns their entity ID if they existed.
    pub fn remove(&mut self, addr: &SocketAddr) -> Option<u64> {
        self.clients.remove(addr).map(|c| c.entity_id)
    }

    /// Get entity ID for a client address.
    #[allow(dead_code)] // Phase 1: used for targeted server→client messaging
    pub fn get_entity_id(&self, addr: &SocketAddr) -> Option<u64> {
        self.clients.get(addr).map(|c| c.entity_id)
    }

    /// Number of connected clients.
    pub fn count(&self) -> usize {
        self.clients.len()
    }

    /// All connected client addresses (for broadcasting).
    #[allow(dead_code)] // Phase 1: used for admin/debug tooling
    pub fn all_addrs(&self) -> Vec<SocketAddr> {
        self.clients.keys().cloned().collect()
    }

    /// Send data to every connected client.
    pub fn send_to_all(&self, data: Vec<u8>) {
        for client in self.clients.values() {
            let _ = client.tx.send(data.clone());
        }
    }

    /// Iterate over all connected clients (for per-client interest management).
    pub fn iter_clients(&self) -> impl Iterator<Item = &ClientInfo> {
        self.clients.values()
    }
}
