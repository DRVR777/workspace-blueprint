//! nexus-semantic — semantic routing layer
//!
//! This crate is bone 1b: the routing loop that `SemanticPacket` runs through.
//!
//! Architecture:
//!
//!   IdentityStore  — holds identity files + HNSW index over their vectors
//!   LlmClient      — trait: embed text, call LLM with identity+packet context
//!   Router         — the loop: nearest identity → one call → push_hop → requeue/terminate
//!   PacketQueue    — async mpsc wrapper; the input side of the routing loop
//!
//! The routing loop (Router::route) in five steps:
//!
//!   1. Embed the packet's current data text
//!   2. Query IdentityStore for nearest identity file (HNSW, k=1)
//!   3. One LLM call: identity content + packet chain → output string
//!   4. push_hop() on the packet: records identity, vector, timestamp
//!   5. Return RouteResult::Continue(new_packet) or RouteResult::Terminal(output)
//!
//! The caller owns the queue and the loop. Router::route is one step.
//! Threading: each worker task holds an Arc<Router> and pulls from the queue.

pub mod identity;
pub mod llm;
pub mod router;
pub mod queue;
pub mod worker;
pub mod http;
pub mod layout;

pub use identity::{IdentityFile, IdentityStore};
pub use llm::{LlmClient, MockLlmClient};
pub use router::{Router, RouteResult};
pub use queue::PacketQueue;
pub use worker::{RoutingLoop, extract_continuation};
pub use http::{HttpState, router as http_router};
pub use layout::{apply_layout, force_directed_layout};
