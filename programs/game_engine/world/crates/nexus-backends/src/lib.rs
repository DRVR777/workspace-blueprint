//! Three backends, one trait.
//!
//! `SemanticBackend` defines the four operations the routing loop
//! needs from storage. Three implementations prove different layers:
//!
//! | Backend      | Proves                                              |
//! |--------------|-----------------------------------------------------|
//! | LocalBackend | Routing logic is correct (existing 33 tests)        |
//! | RvfBackend   | A world can be a single file                        |
//! | HybridBackend| They compose: fast routing + crash-safe persistence |
//!
//! The routing loop sits above this layer. It holds a `Box<dyn SemanticBackend>`
//! (or `Arc<dyn SemanticBackend>`) and calls the four methods. Swapping
//! backends is a constructor change — nothing else moves.

pub mod backend;
pub mod local;
pub mod rvf;
pub mod hybrid;

pub use backend::SemanticBackend;
pub use local::LocalBackend;
pub use rvf::RvfBackend;
pub use hybrid::HybridBackend;
