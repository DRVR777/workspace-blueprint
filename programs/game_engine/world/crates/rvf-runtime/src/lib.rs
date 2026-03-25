//! RVF — Reality Volume File.
//!
//! A world in a file. Crash-safe, append-only, tamper-evident.
//!
//! # File layout
//!
//! ```text
//! [48 bytes] Header
//!   magic:       [u8; 8]  = b"NEXUS.RV"
//!   version:     u32 LE   = 1
//!   flags:       u32 LE   (FLAG_IS_BRANCH = 0x01)
//!   parent_hash: [u8; 32] (SHA-256 of parent file at branch point; zeros if root)
//!
//! [N typed frames]
//!   frame_type:  u8       (FRAME_MANIFEST=1, FRAME_VEC=2, FRAME_WITNESS=3)
//!   payload_len: u32 LE
//!   payload:     N bytes
//!   crc32:       u32 LE   (CRC32 of frame_type ++ payload_len_bytes ++ payload)
//! ```
//!
//! # Segment semantics
//!
//! - **MANIFEST** frames — world descriptor JSON, written once at creation.
//! - **VEC_ENTRY** frames — one identity file per frame: address + content + vector.
//!   Reading the file from start → reconstructs the full identity store.
//! - **WITNESS_ENTRY** frames — one event record per frame with SHA-256 hash chain.
//!   The chain makes the history tamper-evident: altering any entry invalidates
//!   every subsequent entry_hash.
//!
//! # COW branching
//!
//! `RvfFile::branch(src, dst)` copies `src` to `dst`, sets FLAG_IS_BRANCH and
//! writes the parent_hash into the new file's header. The child can be written
//! independently. On open, scanning starts from byte 48 and reads all frames —
//! parent frames and child-appended frames are indistinguishable (same format).

pub mod frame;
pub mod rvf_file;

pub use frame::{VecEntry, WitnessEntry};
pub use rvf_file::{RvfFile, RvfError};
