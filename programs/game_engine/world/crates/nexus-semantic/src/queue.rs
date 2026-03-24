//! Packet queue — async input channel for the routing loop.
//!
//! Thin wrapper around tokio::mpsc. The worker tasks pull packets from
//! the queue and call Router::route. No routing logic here.

use tokio::sync::mpsc;
use nexus_core::types::SemanticPacket;

/// The sender side of the packet queue.
pub type PacketSender = mpsc::UnboundedSender<SemanticPacket>;

/// The receiver side of the packet queue.
pub type PacketReceiver = mpsc::UnboundedReceiver<SemanticPacket>;

/// Create a new unbounded packet queue.
pub struct PacketQueue;

impl PacketQueue {
    pub fn channel() -> (PacketSender, PacketReceiver) {
        mpsc::unbounded_channel()
    }
}
