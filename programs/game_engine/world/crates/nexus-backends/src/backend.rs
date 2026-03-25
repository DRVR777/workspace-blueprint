//! SemanticBackend trait — the four operations the routing loop needs.
//!
//! All methods take `&self` (interior mutability in implementations).
//! All methods are synchronous — async wrappers live in the routing loop
//! layer above (e.g., spawn_blocking for I/O-heavy backends).

use nexus_events::EventRecord;
use nexus_semantic::identity::IdentityFile;

/// The four operations every semantic backend must support.
///
/// Implementors:
///   `LocalBackend`  — in-memory HNSW + SQLite
///   `RvfBackend`    — in-memory HNSW rebuilt from .rvf VEC_SEG on open
///   `HybridBackend` — queries via Local, writes to both Local + Rvf
pub trait SemanticBackend: Send + Sync {
    /// Return up to `k` identity files nearest to `vector`, closest first.
    fn nearest(&self, vector: &[f32], k: usize) -> Vec<IdentityFile>;

    /// Embed and index a new output as a memory identity file.
    ///
    /// For `LocalBackend`: inserts into the in-memory HNSW.
    /// For `RvfBackend`:   inserts into HNSW + appends a VEC_ENTRY frame.
    /// For `HybridBackend`: both of the above.
    fn index_output(&self, address: String, content: String, vector: Vec<f32>);

    /// Look up an identity file by its dworld:// address.
    fn get_identity(&self, address: &str) -> Option<IdentityFile>;

    /// Record a terminal event (or signal) in the backend's event log.
    ///
    /// For `LocalBackend`: appends to SQLite.
    /// For `RvfBackend`:   appends a WITNESS_ENTRY frame.
    /// For `HybridBackend`: both.
    fn record_event(&self, record: &EventRecord) -> Result<(), String>;
}
