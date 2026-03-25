//! Binary frame codec for .rvf files.
//!
//! Every frame on disk:
//!   [1]  frame_type
//!   [4]  payload_len (LE u32)
//!   [N]  payload
//!   [4]  crc32 (CRC32 of type ++ payload_len_bytes ++ payload)
//!
//! A partial frame at EOF (bad CRC or missing bytes) is silently dropped —
//! this is the crash-safety guarantee: a frame is either fully written or ignored.

use crc32fast::Hasher as Crc32Hasher;
use sha2::{Digest, Sha256};

// ─── Frame type codes ─────────────────────────────────────────────────────────

pub const FRAME_MANIFEST: u8 = 1;
pub const FRAME_VEC:      u8 = 2;
pub const FRAME_WITNESS:  u8 = 3;

// ─── VecEntry ─────────────────────────────────────────────────────────────────

/// One identity file serialized for storage in a VEC_ENTRY frame.
/// Contains everything needed to reconstruct an IdentityFile on startup
/// (world_coord is excluded — layout re-runs on load).
#[derive(Debug, Clone)]
pub struct VecEntry {
    pub address: String,
    pub content: String,
    pub vector:  Vec<f32>,
}

impl VecEntry {
    /// Encode to the VEC_ENTRY payload wire format.
    ///
    /// Layout:
    ///   [4]  addr_len
    ///   [4]  content_len
    ///   [4]  dims
    ///   [addr_len] address (UTF-8)
    ///   [content_len] content (UTF-8)
    ///   [dims * 4] vector (f32 LE each)
    pub fn encode(&self) -> Vec<u8> {
        let ab = self.address.as_bytes();
        let cb = self.content.as_bytes();
        let mut buf = Vec::with_capacity(12 + ab.len() + cb.len() + self.vector.len() * 4);
        buf.extend_from_slice(&(ab.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(cb.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(self.vector.len() as u32).to_le_bytes());
        buf.extend_from_slice(ab);
        buf.extend_from_slice(cb);
        for &f in &self.vector {
            buf.extend_from_slice(&f.to_le_bytes());
        }
        buf
    }

    /// Decode from a VEC_ENTRY payload.
    pub fn decode(payload: &[u8]) -> Result<Self, FrameError> {
        if payload.len() < 12 {
            return Err(FrameError::TooShort);
        }
        let addr_len    = u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize;
        let content_len = u32::from_le_bytes(payload[4..8].try_into().unwrap()) as usize;
        let dims        = u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;

        let needed = 12 + addr_len + content_len + dims * 4;
        if payload.len() < needed {
            return Err(FrameError::TooShort);
        }

        let mut off = 12;
        let address = std::str::from_utf8(&payload[off..off + addr_len])
            .map_err(|_| FrameError::InvalidUtf8)?
            .to_string();
        off += addr_len;
        let content = std::str::from_utf8(&payload[off..off + content_len])
            .map_err(|_| FrameError::InvalidUtf8)?
            .to_string();
        off += content_len;

        let mut vector = Vec::with_capacity(dims);
        for i in 0..dims {
            let s = off + i * 4;
            vector.push(f32::from_le_bytes(payload[s..s + 4].try_into().unwrap()));
        }

        Ok(VecEntry { address, content, vector })
    }
}

// ─── WitnessEntry ─────────────────────────────────────────────────────────────

/// One event record serialized for the tamper-evident WITNESS_ENTRY chain.
///
/// Layout of the WITNESS_ENTRY payload:
///   [32]  prev_hash    — SHA-256 of previous entry's body; zeros for first
///   [8]   timestamp_ms — u64 LE
///   [8]   chain_id     — u64 LE
///   [8]   packet_id    — u64 LE
///   [4]   hop_count    — u32 LE
///   [4]   record_type  — u32 LE (0=Event, 1=Signal)
///   [4]   identity_len — u32 LE
///   [4]   output_len   — u32 LE
///   [4]   quality_raw  — u32 LE (0xFFFFFFFF = None; else f32 bits)
///   [4]   world_x_raw  — u32 LE (0xFFFFFFFF = absent)
///   [4]   world_y_raw  — u32 LE
///   [4]   world_z_raw  — u32 LE
///   [identity_len] identity — UTF-8
///   [output_len]   output   — UTF-8
///   [32]  entry_hash   — SHA-256(everything above, not including entry_hash)
#[derive(Debug, Clone)]
pub struct WitnessEntry {
    pub prev_hash:     [u8; 32],
    pub timestamp_ms:  u64,
    pub chain_id:      u64,
    pub packet_id:     u64,
    pub hop_count:     u32,
    pub record_type:   u32,
    pub quality:       Option<f32>,
    pub world_position: Option<[f32; 3]>,
    pub identity:      String,
    pub output:        String,
    /// SHA-256 of the payload minus the last 32 bytes.
    /// Computed on encode; verified on decode.
    pub entry_hash:    [u8; 32],
}

const ABSENT: u32 = 0xFFFF_FFFF;
const FIXED_HEADER: usize = 32 + 8 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4; // = 96

impl WitnessEntry {
    pub fn encode(&self) -> Vec<u8> {
        let ib = self.identity.as_bytes();
        let ob = self.output.as_bytes();

        let mut body = Vec::with_capacity(FIXED_HEADER + ib.len() + ob.len());
        body.extend_from_slice(&self.prev_hash);
        body.extend_from_slice(&self.timestamp_ms.to_le_bytes());
        body.extend_from_slice(&self.chain_id.to_le_bytes());
        body.extend_from_slice(&self.packet_id.to_le_bytes());
        body.extend_from_slice(&self.hop_count.to_le_bytes());
        body.extend_from_slice(&self.record_type.to_le_bytes());
        body.extend_from_slice(&(ib.len() as u32).to_le_bytes());
        body.extend_from_slice(&(ob.len() as u32).to_le_bytes());
        body.extend_from_slice(&match self.quality {
            Some(q) => q.to_bits().to_le_bytes(),
            None    => ABSENT.to_le_bytes(),
        });
        match self.world_position {
            Some(p) => {
                body.extend_from_slice(&p[0].to_bits().to_le_bytes());
                body.extend_from_slice(&p[1].to_bits().to_le_bytes());
                body.extend_from_slice(&p[2].to_bits().to_le_bytes());
            }
            None => {
                body.extend_from_slice(&ABSENT.to_le_bytes());
                body.extend_from_slice(&ABSENT.to_le_bytes());
                body.extend_from_slice(&ABSENT.to_le_bytes());
            }
        }
        body.extend_from_slice(ib);
        body.extend_from_slice(ob);

        let entry_hash: [u8; 32] = Sha256::digest(&body).into();
        body.extend_from_slice(&entry_hash);
        body
    }

    pub fn decode(payload: &[u8]) -> Result<Self, FrameError> {
        if payload.len() < FIXED_HEADER + 32 {
            return Err(FrameError::TooShort);
        }

        let mut off = 0;
        let mut take4 = |o: &mut usize| -> [u8; 4] {
            let b: [u8; 4] = payload[*o..*o + 4].try_into().unwrap();
            *o += 4;
            b
        };
        let mut take8 = |o: &mut usize| -> [u8; 8] {
            let b: [u8; 8] = payload[*o..*o + 8].try_into().unwrap();
            *o += 8;
            b
        };
        let mut take32 = |o: &mut usize| -> [u8; 32] {
            let b: [u8; 32] = payload[*o..*o + 32].try_into().unwrap();
            *o += 32;
            b
        };

        let prev_hash    = take32(&mut off);
        let timestamp_ms = u64::from_le_bytes(take8(&mut off));
        let chain_id     = u64::from_le_bytes(take8(&mut off));
        let packet_id    = u64::from_le_bytes(take8(&mut off));
        let hop_count    = u32::from_le_bytes(take4(&mut off));
        let record_type  = u32::from_le_bytes(take4(&mut off));
        let identity_len = u32::from_le_bytes(take4(&mut off)) as usize;
        let output_len   = u32::from_le_bytes(take4(&mut off)) as usize;
        let quality_raw  = u32::from_le_bytes(take4(&mut off));
        let wx_raw       = u32::from_le_bytes(take4(&mut off));
        let wy_raw       = u32::from_le_bytes(take4(&mut off));
        let wz_raw       = u32::from_le_bytes(take4(&mut off));

        let needed = off + identity_len + output_len + 32;
        if payload.len() < needed {
            return Err(FrameError::TooShort);
        }

        let identity = std::str::from_utf8(&payload[off..off + identity_len])
            .map_err(|_| FrameError::InvalidUtf8)?.to_string();
        off += identity_len;
        let output = std::str::from_utf8(&payload[off..off + output_len])
            .map_err(|_| FrameError::InvalidUtf8)?.to_string();
        off += output_len;

        let entry_hash: [u8; 32] = payload[off..off + 32].try_into().unwrap();

        // Verify hash chain integrity.
        let computed: [u8; 32] = Sha256::digest(&payload[..off]).into();
        if computed != entry_hash {
            return Err(FrameError::HashMismatch);
        }

        let quality = if quality_raw == ABSENT {
            None
        } else {
            Some(f32::from_bits(quality_raw))
        };
        let world_position = if wx_raw == ABSENT {
            None
        } else {
            Some([
                f32::from_bits(wx_raw),
                f32::from_bits(wy_raw),
                f32::from_bits(wz_raw),
            ])
        };

        Ok(WitnessEntry {
            prev_hash,
            timestamp_ms,
            chain_id,
            packet_id,
            hop_count,
            record_type,
            quality,
            world_position,
            identity,
            output,
            entry_hash,
        })
    }

    /// Compute the entry_hash for this entry (used when building the next entry's prev_hash).
    pub fn compute_hash(payload_without_hash: &[u8]) -> [u8; 32] {
        Sha256::digest(payload_without_hash).into()
    }
}

// ─── Frame I/O ────────────────────────────────────────────────────────────────

/// Encode one frame to bytes (type + len + payload + crc32).
pub fn encode_frame(frame_type: u8, payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(1 + 4 + payload.len() + 4);
    frame.push(frame_type);
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(payload);

    let mut h = Crc32Hasher::new();
    h.update(&[frame_type]);
    h.update(&len.to_le_bytes());
    h.update(payload);
    frame.extend_from_slice(&h.finalize().to_le_bytes());
    frame
}

/// Read one frame from `data` at byte offset `pos`.
/// Returns `(frame_type, payload, new_pos)` or `None` if at EOF or CRC error.
pub fn decode_frame(data: &[u8], pos: usize) -> Option<(u8, Vec<u8>, usize)> {
    if pos + 5 > data.len() {
        return None; // not enough for type + len
    }
    let frame_type = data[pos];
    let payload_len = u32::from_le_bytes(data[pos + 1..pos + 5].try_into().ok()?) as usize;
    let end = pos + 5 + payload_len;
    if end + 4 > data.len() {
        return None; // payload + crc not fully written (crash-truncated)
    }
    let payload = data[pos + 5..end].to_vec();
    let stored_crc = u32::from_le_bytes(data[end..end + 4].try_into().ok()?);

    let mut h = Crc32Hasher::new();
    h.update(&[frame_type]);
    h.update(&(payload_len as u32).to_le_bytes());
    h.update(&payload);
    if h.finalize() != stored_crc {
        return None; // corrupted frame, stop scanning
    }

    Some((frame_type, payload, end + 4))
}

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("payload too short")]
    TooShort,
    #[error("invalid UTF-8 in frame")]
    InvalidUtf8,
    #[error("hash mismatch in witness entry")]
    HashMismatch,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_entry_round_trips() {
        let entry = VecEntry {
            address: "dworld://council.local/identities/PHILOSOPHER".into(),
            content: "You are PHILOSOPHER.".into(),
            vector:  vec![0.2, 0.1, 0.8, 0.9, 0.7],
        };
        let payload = entry.encode();
        let decoded = VecEntry::decode(&payload).unwrap();
        assert_eq!(decoded.address, entry.address);
        assert_eq!(decoded.content, entry.content);
        assert_eq!(decoded.vector.len(), entry.vector.len());
        for (a, b) in decoded.vector.iter().zip(entry.vector.iter()) {
            assert!((a - b).abs() < 1e-6, "vector mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn frame_crc_round_trips() {
        let payload = b"hello world";
        let frame = encode_frame(FRAME_VEC, payload);
        let (ft, p, end) = decode_frame(&frame, 0).unwrap();
        assert_eq!(ft, FRAME_VEC);
        assert_eq!(p, payload);
        assert_eq!(end, frame.len());
    }

    #[test]
    fn corrupt_frame_returns_none() {
        let mut frame = encode_frame(FRAME_VEC, b"test payload");
        let last = frame.len() - 1;
        frame[last] ^= 0xFF; // flip bits in CRC
        assert!(decode_frame(&frame, 0).is_none());
    }

    #[test]
    fn witness_entry_round_trips() {
        let body_payload = {
            let e = WitnessEntry {
                prev_hash:      [0u8; 32],
                timestamp_ms:   1_700_000_000_000,
                chain_id:       42,
                packet_id:      7,
                hop_count:      2,
                record_type:    0,
                quality:        Some(0.85),
                world_position: Some([1.0, 2.0, 3.0]),
                identity:       "dworld://council.local/identities/ENGINEER".into(),
                output:         "Build the thing.".into(),
                entry_hash:     [0u8; 32], // overwritten by encode
            };
            e.encode()
        };
        let decoded = WitnessEntry::decode(&body_payload).unwrap();
        assert_eq!(decoded.chain_id, 42);
        assert_eq!(decoded.packet_id, 7);
        assert!((decoded.quality.unwrap() - 0.85).abs() < 1e-5);
        let p = decoded.world_position.unwrap();
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert_eq!(decoded.identity, "dworld://council.local/identities/ENGINEER");
        assert_eq!(decoded.output, "Build the thing.");
    }

    #[test]
    fn witness_entry_none_fields() {
        let payload = WitnessEntry {
            prev_hash:      [0u8; 32],
            timestamp_ms:   0,
            chain_id:       1,
            packet_id:      1,
            hop_count:      0,
            record_type:    1,
            quality:        None,
            world_position: None,
            identity:       "addr".into(),
            output:         "msg".into(),
            entry_hash:     [0u8; 32],
        }.encode();
        let decoded = WitnessEntry::decode(&payload).unwrap();
        assert!(decoded.quality.is_none());
        assert!(decoded.world_position.is_none());
    }
}
