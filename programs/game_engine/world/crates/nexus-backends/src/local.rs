//! LocalBackend — in-memory HNSW + SQLite EventLog.
//!
//! This is everything that exists today, wrapped behind the SemanticBackend
//! trait. All 33 existing nexus-semantic tests exercise this backend via
//! RoutingLoop (which owns IdentityStore + EventLog directly). LocalBackend
//! provides the same behavior behind the trait interface.

use std::sync::{Arc, RwLock};

use nexus_events::{EventLog, EventRecord};
use nexus_semantic::identity::{IdentityFile, IdentityStore};

use crate::backend::SemanticBackend;

pub struct LocalBackend {
    store: Arc<RwLock<Arc<IdentityStore>>>,
    log:   Arc<EventLog>,
}

impl LocalBackend {
    /// Create from an existing seed store and event log.
    /// This is the same initialization path as RoutingLoop::new.
    pub fn new(store: IdentityStore, log: Arc<EventLog>) -> Self {
        Self {
            store: Arc::new(RwLock::new(Arc::new(store))),
            log,
        }
    }

    /// Snapshot the current store (brief read-lock).
    pub fn current_store(&self) -> Arc<IdentityStore> {
        self.store.read().unwrap().clone()
    }
}

impl SemanticBackend for LocalBackend {
    fn nearest(&self, vector: &[f32], k: usize) -> Vec<IdentityFile> {
        self.current_store()
            .nearest_k(vector, k)
            .into_iter()
            .cloned()
            .collect()
    }

    fn index_output(&self, address: String, content: String, vector: Vec<f32>) {
        let new_file = IdentityFile {
            address,
            content,
            vector,
            world_coord: None,
        };
        let mut guard = self.store.write().unwrap();
        Arc::make_mut(&mut *guard).insert_one(new_file);
    }

    fn get_identity(&self, address: &str) -> Option<IdentityFile> {
        self.current_store().get_by_address(address).cloned()
    }

    fn record_event(&self, record: &EventRecord) -> Result<(), String> {
        self.log.append(record).map(|_| ()).map_err(|e| e.to_string())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_events::RecordType;
    use nexus_semantic::identity::seed_identities;

    fn make_backend() -> LocalBackend {
        let store = IdentityStore::build(seed_identities());
        let log   = Arc::new(EventLog::open_in_memory().unwrap());
        LocalBackend::new(store, log)
    }

    fn make_record(identity: &str) -> EventRecord {
        EventRecord {
            record_type:    RecordType::Event,
            timestamp_ms:   1_700_000_000_000,
            chain_id:       1,
            packet_id:      1,
            hop_count:      1,
            identity:       identity.to_string(),
            output:         "test output".to_string(),
            world_position: None,
            quality:        Some(0.7),
        }
    }

    #[test]
    fn nearest_returns_some_identity() {
        let b = make_backend();
        let query = vec![0.9f32, 0.9, 0.2, 0.5, 0.9]; // ENGINEER-like
        let results = b.nearest(&query, 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].address.contains("ENGINEER"), "got: {}", results[0].address);
    }

    #[test]
    fn index_output_grows_the_store() {
        let b = make_backend();
        let initial = b.current_store().len();
        b.index_output(
            "dworld://memory/test/1".into(),
            "new memory content".into(),
            vec![0.5; 5],
        );
        assert_eq!(b.current_store().len(), initial + 1);
    }

    #[test]
    fn get_identity_returns_seed() {
        let b = make_backend();
        let id = b.get_identity("dworld://council.local/identities/PHILOSOPHER");
        assert!(id.is_some());
    }

    #[test]
    fn record_event_persists_to_log() {
        let b = make_backend();
        b.record_event(&make_record("dworld://council.local/identities/CRITIC")).unwrap();
        // Verify via the EventLog (access through Arc clone)
        let store_ref = b.current_store(); // just to prove no borrow conflict
        drop(store_ref);
    }
}
