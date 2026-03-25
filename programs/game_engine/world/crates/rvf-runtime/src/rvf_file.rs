//! RvfFile — the file handle for a .rvf world file.
//!
//! Thread-safe via `Mutex<Vec<u8>>` (in-memory backing for tests)
//! or `Mutex<File>` (on-disk). For simplicity we use a single backing
//! model: the file is read fully into memory on open, and appended
//! via the OS file handle on write.  This is correct for world files
//! that fit in RAM (typical world < 100 MB).
//!
//! Write path: append frame bytes to file, fsync.
//! Read path: re-read from stored bytes snapshot (no seek required).

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use crate::frame::{
    decode_frame, encode_frame, FrameError,
    VecEntry, WitnessEntry,
    FRAME_MANIFEST, FRAME_VEC, FRAME_WITNESS,
};

// ─── File header constants ────────────────────────────────────────────────────

const MAGIC:        &[u8; 8] = b"NEXUS.RV";
const VERSION:      u32      = 1;
const FLAG_BRANCH:  u32      = 0x01;
const HEADER_LEN:   usize    = 48; // magic(8) + version(4) + flags(4) + parent_hash(32)

// ─── RvfFile ──────────────────────────────────────────────────────────────────

pub struct RvfFile {
    path:   PathBuf,
    inner:  Mutex<RvfInner>,
}

struct RvfInner {
    /// All bytes from the file including header, loaded at open.
    /// New frames are appended here AND to the on-disk file.
    data: Vec<u8>,
    /// OS file handle for appending.
    file: File,
    /// SHA-256 of the last WITNESS_ENTRY's payload body (for hash chain).
    last_witness_hash: [u8; 32],
}

impl RvfFile {
    /// Create a new .rvf file at `path` with the given manifest JSON.
    /// Overwrites if exists.
    pub fn create(path: impl AsRef<Path>, manifest_json: &str) -> Result<Self, RvfError> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .create(true).write(true).truncate(true)
            .open(&path)?;

        let mut data = Vec::with_capacity(HEADER_LEN + 256);

        // Header
        data.extend_from_slice(MAGIC);
        data.extend_from_slice(&VERSION.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes()); // flags
        data.extend_from_slice(&[0u8; 32]);           // parent_hash (zeros = root)

        // Manifest frame
        let mf = encode_frame(FRAME_MANIFEST, manifest_json.as_bytes());
        data.extend_from_slice(&mf);

        file.write_all(&data)?;
        file.flush()?;

        Ok(Self {
            path,
            inner: Mutex::new(RvfInner {
                data,
                file,
                last_witness_hash: [0u8; 32],
            }),
        })
    }

    /// Open an existing .rvf file at `path`.
    /// Validates the magic bytes and loads all frames into memory.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RvfError> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true).write(true).append(true)
            .open(&path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < HEADER_LEN {
            return Err(RvfError::InvalidHeader("file too short".into()));
        }
        if &data[0..8] != MAGIC {
            return Err(RvfError::InvalidHeader("bad magic".into()));
        }

        // Scan WITNESS frames to find the last hash
        let mut last_witness_hash = [0u8; 32];
        let mut pos = HEADER_LEN;
        while let Some((ftype, payload, next)) = decode_frame(&data, pos) {
            if ftype == FRAME_WITNESS {
                if let Ok(e) = WitnessEntry::decode(&payload) {
                    last_witness_hash = e.entry_hash;
                }
            }
            pos = next;
        }

        Ok(Self {
            path,
            inner: Mutex::new(RvfInner { data, file, last_witness_hash }),
        })
    }

    /// Create a COW branch of this file at `dst_path`.
    ///
    /// The child starts as a byte-for-byte copy of the parent.
    /// The child's header is updated: FLAG_BRANCH is set and parent_hash
    /// is set to the SHA-256 of the parent's full data at this moment.
    /// Future writes to either file are independent.
    pub fn branch(&self, dst_path: impl AsRef<Path>) -> Result<RvfFile, RvfError> {
        let dst = dst_path.as_ref().to_path_buf();
        let inner = self.inner.lock().unwrap();

        // SHA-256 of parent data = the parent hash recorded in child
        let parent_hash: [u8; 32] = Sha256::digest(&inner.data).into();

        // Write child file = copy of parent data
        std::fs::copy(&self.path, &dst)?;

        // Patch the child header: set FLAG_BRANCH at bytes [12..16],
        // write parent_hash at bytes [16..48]
        let mut child_data = inner.data.clone();
        let flags: u32 = u32::from_le_bytes(child_data[12..16].try_into().unwrap()) | FLAG_BRANCH;
        child_data[12..16].copy_from_slice(&flags.to_le_bytes());
        child_data[16..48].copy_from_slice(&parent_hash);

        // Overwrite the file with patched header
        let mut child_file = OpenOptions::new().write(true).truncate(true).open(&dst)?;
        child_file.write_all(&child_data)?;
        child_file.flush()?;
        drop(child_file);

        // Open child for append
        RvfFile::open(dst)
    }

    // ── Append operations ─────────────────────────────────────────────────────

    /// Append a VEC_ENTRY frame. Crash-safe: partial writes are ignored on reload.
    pub fn append_vec(&self, entry: &VecEntry) -> Result<(), RvfError> {
        let payload = entry.encode();
        let frame = encode_frame(FRAME_VEC, &payload);
        self.append_frame(&frame)
    }

    /// Append a WITNESS_ENTRY frame. Automatically chains from the previous entry.
    ///
    /// Returns the entry_hash of the appended entry (use as prev_hash for next).
    pub fn append_witness(
        &self,
        timestamp_ms:   u64,
        chain_id:       u64,
        packet_id:      u64,
        hop_count:      u32,
        record_type:    u32,
        quality:        Option<f32>,
        world_position: Option<[f32; 3]>,
        identity:       &str,
        output:         &str,
    ) -> Result<[u8; 32], RvfError> {
        let mut inner = self.inner.lock().unwrap();

        let entry = WitnessEntry {
            prev_hash:      inner.last_witness_hash,
            timestamp_ms,
            chain_id,
            packet_id,
            hop_count,
            record_type,
            quality,
            world_position,
            identity: identity.to_string(),
            output:   output.to_string(),
            entry_hash: [0u8; 32], // computed by encode
        };

        let payload = entry.encode();
        // entry_hash is the last 32 bytes of the payload
        let entry_hash: [u8; 32] = payload[payload.len() - 32..].try_into().unwrap();

        let frame = encode_frame(FRAME_WITNESS, &payload);
        inner.data.extend_from_slice(&frame);
        inner.file.write_all(&frame)?;
        inner.file.flush()?;
        inner.last_witness_hash = entry_hash;

        Ok(entry_hash)
    }

    // ── Read operations ───────────────────────────────────────────────────────

    /// Iterate all VEC_ENTRY frames in file order.
    pub fn iter_vec_entries(&self) -> Vec<VecEntry> {
        let inner = self.inner.lock().unwrap();
        let mut result = Vec::new();
        let mut pos = HEADER_LEN;
        while let Some((ftype, payload, next)) = decode_frame(&inner.data, pos) {
            if ftype == FRAME_VEC {
                if let Ok(e) = VecEntry::decode(&payload) {
                    result.push(e);
                }
            }
            pos = next;
        }
        result
    }

    /// Iterate all WITNESS_ENTRY frames in file order.
    pub fn iter_witness_entries(&self) -> Vec<WitnessEntry> {
        let inner = self.inner.lock().unwrap();
        let mut result = Vec::new();
        let mut pos = HEADER_LEN;
        while let Some((ftype, payload, next)) = decode_frame(&inner.data, pos) {
            if ftype == FRAME_WITNESS {
                if let Ok(e) = WitnessEntry::decode(&payload) {
                    result.push(e);
                }
            }
            pos = next;
        }
        result
    }

    /// Read the manifest JSON (first MANIFEST frame in the file).
    pub fn manifest_json(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        let mut pos = HEADER_LEN;
        while let Some((ftype, payload, next)) = decode_frame(&inner.data, pos) {
            if ftype == FRAME_MANIFEST {
                return String::from_utf8(payload).ok();
            }
            pos = next;
        }
        None
    }

    /// Return the path this file is stored at.
    pub fn path(&self) -> &Path { &self.path }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn append_frame(&self, frame: &[u8]) -> Result<(), RvfError> {
        let mut inner = self.inner.lock().unwrap();
        inner.data.extend_from_slice(frame);
        inner.file.write_all(frame)?;
        inner.file.flush()?;
        Ok(())
    }
}

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum RvfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error("frame decode error: {0}")]
    Frame(#[from] FrameError),
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn create_open_and_read_manifest() {
        let p = tmp_path("test_rvf_manifest.rvf");
        let manifest = r#"{"world_id":"test","name":"World-1"}"#;
        RvfFile::create(&p, manifest).unwrap();

        let f = RvfFile::open(&p).unwrap();
        assert_eq!(f.manifest_json().as_deref(), Some(manifest));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn vec_entries_survive_reopen() {
        let p = tmp_path("test_rvf_vec.rvf");
        {
            let f = RvfFile::create(&p, "{}").unwrap();
            for i in 0..5u32 {
                f.append_vec(&VecEntry {
                    address: format!("dworld://test/{i}"),
                    content: format!("content {i}"),
                    vector:  vec![i as f32 * 0.1, 0.5, 0.9],
                }).unwrap();
            }
        } // drop + flush

        let f = RvfFile::open(&p).unwrap();
        let entries = f.iter_vec_entries();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].address, "dworld://test/0");
        assert_eq!(entries[4].address, "dworld://test/4");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn witness_hash_chain_is_linked() {
        let p = tmp_path("test_rvf_witness.rvf");
        let f = RvfFile::create(&p, "{}").unwrap();

        let h1 = f.append_witness(1000, 1, 1, 0, 0, None, None, "id1", "out1").unwrap();
        let h2 = f.append_witness(2000, 2, 2, 1, 0, Some(0.9), None, "id2", "out2").unwrap();

        let entries = f.iter_witness_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].prev_hash, [0u8; 32]); // first entry: zeros
        assert_eq!(entries[0].entry_hash, h1);
        assert_eq!(entries[1].prev_hash, h1);         // second chains from first
        assert_eq!(entries[1].entry_hash, h2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn branch_creates_independent_child() {
        let parent_p = tmp_path("test_rvf_parent.rvf");
        let child_p  = tmp_path("test_rvf_child.rvf");

        let parent = RvfFile::create(&parent_p, r#"{"world":"parent"}"#).unwrap();
        parent.append_vec(&VecEntry {
            address: "dworld://parent/id1".into(),
            content: "parent content".into(),
            vector:  vec![1.0, 0.0],
        }).unwrap();

        let child = parent.branch(&child_p).unwrap();

        // Parent entries visible in child
        let child_entries = child.iter_vec_entries();
        assert_eq!(child_entries.len(), 1);

        // Child gets its own entry
        child.append_vec(&VecEntry {
            address: "dworld://child/id1".into(),
            content: "child content".into(),
            vector:  vec![0.0, 1.0],
        }).unwrap();

        // Parent unchanged
        assert_eq!(parent.iter_vec_entries().len(), 1);
        // Child has both
        assert_eq!(child.iter_vec_entries().len(), 2);

        let _ = std::fs::remove_file(&parent_p);
        let _ = std::fs::remove_file(&child_p);
    }
}
