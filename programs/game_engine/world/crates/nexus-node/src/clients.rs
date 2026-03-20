//! Client manager — tracks connected clients and their entity IDs.

use std::collections::HashMap;
use std::net::SocketAddr;

/// Information about a connected client.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub entity_id: u64,
    pub addr: SocketAddr,
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

    /// Register a new client with their assigned entity ID.
    pub fn add(&mut self, addr: SocketAddr, entity_id: u64) {
        self.clients.insert(addr, ClientInfo { entity_id, addr });
    }

    /// Remove a client, returns their entity ID if they existed.
    pub fn remove(&mut self, addr: &SocketAddr) -> Option<u64> {
        self.clients.remove(addr).map(|c| c.entity_id)
    }

    /// Get entity ID for a client address.
    pub fn get_entity_id(&self, addr: &SocketAddr) -> Option<u64> {
        self.clients.get(addr).map(|c| c.entity_id)
    }

    /// Number of connected clients.
    pub fn count(&self) -> usize {
        self.clients.len()
    }

    /// All connected client addresses (for broadcasting).
    pub fn all_addrs(&self) -> Vec<SocketAddr> {
        self.clients.keys().cloned().collect()
    }
}
