//! RvfBackend — a world is a single .rvf file.
//!
//! On open: reads all VEC_ENTRY frames → rebuilds IdentityStore in memory.
//! On index_output: inserts into the in-memory store AND appends a VEC_ENTRY frame.
//! On record_event: appends a WITNESS_ENTRY frame (hash-chained, tamper-evident).
//! On nearest/get_identity: queries the in-memory store (same HNSW as LocalBackend).
//!
//! The .rvf file IS the world's ground truth.
//! The in-memory IdentityStore is the routing index — always reconstructible from the file.
//! Same relationship as a database file and its buffer pool.

use std::path::Path;
use std::sync::{Arc, RwLock};

use nexus_events::{EventRecord, RecordType};
use nexus_semantic::identity::{IdentityFile, IdentityStore};
use rvf_runtime::{RvfFile, VecEntry};

use crate::backend::SemanticBackend;

pub struct RvfBackend {
    pub(crate) file:  RvfFile,
    store: Arc<RwLock<Arc<IdentityStore>>>,
}

impl RvfBackend {
    /// Create a new .rvf file at `path`.
    /// Starts with an empty identity store; no seed identities.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = RvfFile::create(path, r#"{"version":1}"#)
            .map_err(|e| e.to_string())?;
        let store = IdentityStore::build(vec![]);
        Ok(Self {
            file,
            store: Arc::new(RwLock::new(Arc::new(store))),
        })
    }

    /// Create a new .rvf file at `path` seeded with the given identity files.
    pub fn create_with_seeds(
        path: impl AsRef<Path>,
        seeds: Vec<IdentityFile>,
    ) -> Result<Self, String> {
        let path = path.as_ref();
        let file = RvfFile::create(path, r#"{"version":1}"#)
            .map_err(|e| e.to_string())?;

        // Write seeds as VEC_ENTRY frames
        for f in &seeds {
            file.append_vec(&VecEntry {
                address: f.address.clone(),
                content: f.content.clone(),
                vector:  f.vector.clone(),
            }).map_err(|e| e.to_string())?;
        }

        let store = IdentityStore::build(seeds);
        Ok(Self {
            file,
            store: Arc::new(RwLock::new(Arc::new(store))),
        })
    }

    /// Open an existing .rvf file and reconstruct the IdentityStore from VEC_ENTRY frames.
    ///
    /// This is the restart path: write 10 outputs, drop, reopen — all 10 come back.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = RvfFile::open(path).map_err(|e| e.to_string())?;

        // Rebuild IdentityStore from all VEC_ENTRY frames
        let entries = file.iter_vec_entries();
        let identity_files: Vec<IdentityFile> = entries.into_iter().map(|e| IdentityFile {
            address:    e.address,
            content:    e.content,
            vector:     e.vector,
            world_coord: None, // layout re-runs after load if needed
        }).collect();

        let store = IdentityStore::build(identity_files);
        Ok(Self {
            file,
            store: Arc::new(RwLock::new(Arc::new(store))),
        })
    }

    /// Create a COW child branch at `dst_path`.
    ///
    /// The child starts with all parent frames + gets its own write handle.
    /// Writes to child are independent of the parent after branching.
    pub fn branch(&self, dst_path: impl AsRef<Path>) -> Result<RvfBackend, String> {
        let child_file = self.file.branch(dst_path).map_err(|e| e.to_string())?;
        let store = self.store.read().unwrap().as_ref().clone();
        Ok(RvfBackend {
            file:  child_file,
            store: Arc::new(RwLock::new(Arc::new(store))),
        })
    }

    fn current_store(&self) -> Arc<IdentityStore> {
        self.store.read().unwrap().clone()
    }
}

impl SemanticBackend for RvfBackend {
    fn nearest(&self, vector: &[f32], k: usize) -> Vec<IdentityFile> {
        self.current_store()
            .nearest_k(vector, k)
            .into_iter()
            .cloned()
            .collect()
    }

    fn index_output(&self, address: String, content: String, vector: Vec<f32>) {
        // Persist first (crash-safe: if we die here the memory write didn't happen
        // but the frame is either fully written or CRC-failed, never half-applied)
        let _ = self.file.append_vec(&VecEntry {
            address: address.clone(),
            content: content.clone(),
            vector:  vector.clone(),
        });

        // Then update in-memory store
        let new_file = IdentityFile { address, content, vector, world_coord: None };
        let mut guard = self.store.write().unwrap();
        Arc::make_mut(&mut *guard).insert_one(new_file);
    }

    fn get_identity(&self, address: &str) -> Option<IdentityFile> {
        self.current_store().get_by_address(address).cloned()
    }

    fn record_event(&self, record: &EventRecord) -> Result<(), String> {
        let ts = record.timestamp_ms;
        let record_type = match record.record_type {
            RecordType::Event  => 0,
            RecordType::Signal => 1,
        };
        self.file.append_witness(
            ts,
            record.chain_id,
            record.packet_id,
            record.hop_count,
            record_type,
            record.quality,
            record.world_position,
            &record.identity,
            &record.output,
        ).map(|_| ()).map_err(|e| e.to_string())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_events::RecordType;
    use nexus_semantic::identity::seed_identities;

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(name)
    }

    fn make_record() -> EventRecord {
        EventRecord {
            record_type:    RecordType::Event,
            timestamp_ms:   1_700_000_000_000,
            chain_id:       1,
            packet_id:      1,
            hop_count:      1,
            identity:       "dworld://council.local/identities/ENGINEER".into(),
            output:         "Build the thing.".into(),
            world_position: None,
            quality:        Some(0.9),
        }
    }

    #[test]
    fn create_and_nearest_seed() {
        let p = tmp("rvf_backend_seeds.rvf");
        let b = RvfBackend::create_with_seeds(&p, seed_identities()).unwrap();
        let q = vec![0.9f32, 0.9, 0.2, 0.5, 0.9]; // ENGINEER
        let hits = b.nearest(&q, 1);
        assert!(hits[0].address.contains("ENGINEER"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn index_output_persists_and_queryable() {
        let p = tmp("rvf_backend_index.rvf");
        let b = RvfBackend::create(&p).unwrap();
        b.index_output(
            "dworld://memory/1".into(),
            "memory content".into(),
            vec![0.1, 0.2, 0.3, 0.4, 0.5],
        );
        assert_eq!(b.current_store().len(), 1);

        // Verify it's on disk too
        let entries = b.file.iter_vec_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].address, "dworld://memory/1");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn open_restores_indexed_outputs() {
        let p = tmp("rvf_backend_restore.rvf");
        {
            let b = RvfBackend::create(&p).unwrap();
            for i in 0..5u32 {
                b.index_output(
                    format!("dworld://memory/{i}"),
                    format!("content {i}"),
                    vec![i as f32 * 0.1, 0.5, 0.5, 0.5, 0.5],
                );
            }
        } // drop

        let b2 = RvfBackend::open(&p).unwrap();
        assert_eq!(b2.current_store().len(), 5);
        assert!(b2.get_identity("dworld://memory/3").is_some());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn record_event_writes_witness_frame() {
        let p = tmp("rvf_backend_witness.rvf");
        let b = RvfBackend::create(&p).unwrap();
        b.record_event(&make_record()).unwrap();

        let entries = b.file.iter_witness_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].chain_id, 1);
        assert_eq!(entries[0].output, "Build the thing.");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn branch_is_independent() {
        let parent_p = tmp("rvf_branch_parent.rvf");
        let child_p  = tmp("rvf_branch_child.rvf");

        let parent = RvfBackend::create(&parent_p).unwrap();
        parent.index_output("dworld://p/1".into(), "parent".into(), vec![1.0, 0.0, 0.0, 0.0, 0.0]);

        let child = parent.branch(&child_p).unwrap();
        child.index_output("dworld://c/1".into(), "child".into(), vec![0.0, 1.0, 0.0, 0.0, 0.0]);

        assert_eq!(parent.current_store().len(), 1); // parent unchanged
        assert_eq!(child.current_store().len(), 2);  // child has parent's + its own

        let _ = std::fs::remove_file(&parent_p);
        let _ = std::fs::remove_file(&child_p);
    }
}
