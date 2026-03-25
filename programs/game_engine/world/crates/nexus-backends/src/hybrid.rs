//! HybridBackend — fast routing + crash-safe persistence.
//!
//! The split:
//!
//!   QUERY PATH:   nearest / get_identity → LocalBackend (proven HNSW, warm)
//!   WRITE PATH:   index_output → both (keeps them in sync)
//!                 record_event → both (SQLite for hot queries, WITNESS_SEG for audit)
//!
//! On startup: HybridBackend::open(path, log) reads the .rvf file, rebuilds
//! the LocalBackend's IdentityStore from VEC_SEG, then takes over.
//! The .rvf file is the ground truth. The LocalBackend is the working index.
//!
//! If the LocalBackend diverges from the .rvf file (crash mid-write), the
//! .rvf file wins on restart: open() always rebuilds from the file.

use std::path::Path;
use std::sync::Arc;

use nexus_events::{EventLog, EventRecord};
use nexus_semantic::identity::{IdentityFile, IdentityStore};
use crate::backend::SemanticBackend;
use crate::local::LocalBackend;
use crate::rvf::RvfBackend;

pub struct HybridBackend {
    local: LocalBackend,
    rvf:   RvfBackend,
}

impl HybridBackend {
    /// Create a new hybrid backend.
    ///
    /// `path`   — path for the .rvf file (creates new).
    /// `seeds`  — initial identity files (written to both backends).
    /// `log`    — SQLite EventLog for the LocalBackend's hot-query path.
    pub fn create(
        path:  impl AsRef<Path>,
        seeds: Vec<IdentityFile>,
        log:   Arc<EventLog>,
    ) -> Result<Self, String> {
        let rvf   = RvfBackend::create_with_seeds(path, seeds.clone())?;
        let local = LocalBackend::new(IdentityStore::build(seeds), log);
        Ok(Self { local, rvf })
    }

    /// Open an existing .rvf file and rebuild the LocalBackend from it.
    ///
    /// This is the restart path. The .rvf file is the ground truth:
    /// everything in VEC_SEG is loaded into the LocalBackend's IdentityStore.
    pub fn open(path: impl AsRef<Path>, log: Arc<EventLog>) -> Result<Self, String> {
        let rvf = RvfBackend::open(path)?;

        // Reconstruct LocalBackend from RvfBackend's current store
        let files: Vec<IdentityFile> = rvf.iter_identities();
        let local = LocalBackend::new(IdentityStore::build(files), log);

        Ok(Self { local, rvf })
    }

    /// Create a COW branch at `dst_path`.
    /// The child's .rvf file is a copy of the parent. Both backends are independent.
    pub fn branch(
        &self,
        dst_path: impl AsRef<Path>,
        log:      Arc<EventLog>,
    ) -> Result<HybridBackend, String> {
        let child_rvf = self.rvf.branch(dst_path)?;
        let files: Vec<IdentityFile> = child_rvf.iter_identities();
        let child_local = LocalBackend::new(IdentityStore::build(files), log);
        Ok(HybridBackend { local: child_local, rvf: child_rvf })
    }
}

impl SemanticBackend for HybridBackend {
    fn nearest(&self, vector: &[f32], k: usize) -> Vec<IdentityFile> {
        // Query goes to LocalBackend — proven, warm HNSW.
        self.local.nearest(vector, k)
    }

    fn index_output(&self, address: String, content: String, vector: Vec<f32>) {
        // Write to both. RVF first (crash-safe persistence), then local (routing index).
        self.rvf.index_output(address.clone(), content.clone(), vector.clone());
        self.local.index_output(address, content, vector);
    }

    fn get_identity(&self, address: &str) -> Option<IdentityFile> {
        self.local.get_identity(address)
    }

    fn record_event(&self, record: &EventRecord) -> Result<(), String> {
        // Both logs get the event.
        self.rvf.record_event(record)?;
        self.local.record_event(record)
    }
}

// ─── Internal helper on RvfBackend ───────────────────────────────────────────
// We need to iterate all identity files from the RvfBackend to seed LocalBackend.
// Add this as an inherent method on RvfBackend (accessed here via pub(crate)).

impl RvfBackend {
    pub(crate) fn iter_identities(&self) -> Vec<IdentityFile> {
        self.file.iter_vec_entries().into_iter().map(|e| IdentityFile {
            address:    e.address,
            content:    e.content,
            vector:     e.vector,
            world_coord: None,
        }).collect()
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

    fn make_event(id: u64) -> EventRecord {
        EventRecord {
            record_type:    RecordType::Event,
            timestamp_ms:   1_700_000_000_000 + id * 1000,
            chain_id:       id,
            packet_id:      id,
            hop_count:      1,
            identity:       "dworld://council.local/identities/ENGINEER".into(),
            output:         format!("output {id}"),
            world_position: None,
            quality:        Some(0.8),
        }
    }

    #[test]
    fn hybrid_nearest_goes_to_local() {
        let p   = tmp("hybrid_nearest.rvf");
        let log = Arc::new(EventLog::open_in_memory().unwrap());
        let b   = HybridBackend::create(&p, seed_identities(), log).unwrap();

        let q = vec![0.9f32, 0.9, 0.2, 0.5, 0.9]; // ENGINEER
        let hits = b.nearest(&q, 1);
        assert!(hits[0].address.contains("ENGINEER"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn hybrid_index_output_syncs_both() {
        let p   = tmp("hybrid_index.rvf");
        let log = Arc::new(EventLog::open_in_memory().unwrap());
        let b   = HybridBackend::create(&p, vec![], log).unwrap();

        b.index_output("dworld://m/1".into(), "c".into(), vec![0.5; 5]);

        // Local has it
        assert!(b.local.get_identity("dworld://m/1").is_some());
        // RVF has it
        assert!(b.rvf.get_identity("dworld://m/1").is_some());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn hybrid_record_event_goes_to_both() {
        let p   = tmp("hybrid_events.rvf");
        let log = Arc::new(EventLog::open_in_memory().unwrap());
        let b   = HybridBackend::create(&p, vec![], Arc::clone(&log)).unwrap();

        b.record_event(&make_event(1)).unwrap();

        // Witness in RVF
        let witnesses = b.rvf.file.iter_witness_entries();
        assert_eq!(witnesses.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    /// THE KEY TEST: write 10 outputs, drop the backend, reopen from .rvf,
    /// verify all 10 are retrievable by nearest-neighbor query.
    ///
    /// If this passes: the world is a file. The routing is in memory.
    /// They're the same thing at different resolutions.
    #[test]
    fn hybrid_survives_restart() {
        let p   = tmp("hybrid_restart.rvf");
        let vectors: Vec<Vec<f32>> = (0..10u32).map(|i| {
            let mut v = vec![0.0f32; 5];
            v[i as usize % 5] = 1.0;
            v[i as usize / 5] = 0.5;
            v
        }).collect();
        let addresses: Vec<String> = (0..10u32)
            .map(|i| format!("dworld://memory/item-{i}"))
            .collect();

        // ── Phase 1: write 10 outputs ──────────────────────────────────────
        {
            let log = Arc::new(EventLog::open_in_memory().unwrap());
            let b   = HybridBackend::create(&p, vec![], log).unwrap();
            for (addr, vec) in addresses.iter().zip(vectors.iter()) {
                b.index_output(addr.clone(), format!("content for {addr}"), vec.clone());
            }
            // b drops here, file is flushed
        }

        // ── Phase 2: reopen and verify all 10 are retrievable ─────────────
        let log2 = Arc::new(EventLog::open_in_memory().unwrap());
        let b2 = HybridBackend::open(&p, log2).unwrap();

        assert_eq!(
            b2.local.current_store().len(), 10,
            "store should have all 10 entries after restart"
        );

        for (addr, vec) in addresses.iter().zip(vectors.iter()) {
            let hits = b2.nearest(vec, 1);
            assert!(
                !hits.is_empty(),
                "no result for vector corresponding to {addr}"
            );
            assert_eq!(
                hits[0].address, *addr,
                "wrong nearest neighbor after restart: expected {addr}, got {}",
                hits[0].address
            );
        }

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn hybrid_branch_is_independent() {
        let parent_p = tmp("hybrid_branch_parent.rvf");
        let child_p  = tmp("hybrid_branch_child.rvf");

        let log_p = Arc::new(EventLog::open_in_memory().unwrap());
        let parent = HybridBackend::create(&parent_p, vec![], log_p).unwrap();
        parent.index_output("dworld://p/1".into(), "parent".into(), vec![1.0, 0.0, 0.0, 0.0, 0.0]);

        let log_c = Arc::new(EventLog::open_in_memory().unwrap());
        let child = parent.branch(&child_p, log_c).unwrap();
        child.index_output("dworld://c/1".into(), "child".into(), vec![0.0, 1.0, 0.0, 0.0, 0.0]);

        // Parent unchanged
        assert_eq!(parent.nearest(&[1.0, 0.0, 0.0, 0.0, 0.0], 5).len(), 1);
        // Child has both
        assert_eq!(child.nearest(&[1.0, 0.0, 0.0, 0.0, 0.0], 5).len(), 2);

        let _ = std::fs::remove_file(&parent_p);
        let _ = std::fs::remove_file(&child_p);
    }
}
