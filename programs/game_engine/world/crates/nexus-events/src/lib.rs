//! nexus-events — append-only event log for the NEXUS semantic network.
//!
//! Bone 2: every terminal packet writes one record here.
//!
//! Two record types:
//!   EVENT  (0) — a terminal SemanticPacket: routing completed, output produced
//!   SIGNAL (1) — a quality score update, drift alert, or reformation trigger
//!
//! One SQLite table. Two indices (chain_id, timestamp_ms).
//! Append-only. Quality scores written back after evaluation.
//!
//! Thread safety: Mutex<Connection>. Writes are synchronous and blocking.
//! In the worker loop (bone 3a), wrap appends in tokio::task::spawn_blocking.
//!
//! Architecture note: this is the network's memory *across* packets.
//! The in-packet hop chain is working memory (forgotten when the packet dies).
//! The event log is long-term memory (the permanent record of what happened).

use rusqlite::{params, Connection};
use std::sync::Mutex;
use nexus_core::types::SemanticPacket;

// ─── Record types ─────────────────────────────────────────────────────────────

/// The two record types stored in the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    /// A terminal routing result: SemanticPacket reached a terminal condition.
    /// Written by the worker loop when RouteResult::Terminal fires.
    Event = 0,
    /// A quality score update, drift signal, or reformation trigger.
    /// Written by the quality scorer (bone 4) after evaluating an Event record.
    Signal = 1,
}

impl RecordType {
    fn from_i64(v: i64) -> Self {
        match v {
            0 => Self::Event,
            _ => Self::Signal,
        }
    }
}

/// One record in the event log.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    /// Event (routing terminal) or Signal (quality score / drift alert).
    pub record_type: RecordType,
    /// Unix timestamp in milliseconds when this record was written.
    pub timestamp_ms: u64,
    /// Chain ID — groups all packets in one logical request.
    pub chain_id: u64,
    /// Packet ID — unique per packet.
    pub packet_id: u64,
    /// Number of hops the packet took before reaching terminal.
    pub hop_count: u32,
    /// dworld:// address of the last identity file that processed this packet.
    pub identity: String,
    /// Terminal output text produced by the last hop.
    pub output: String,
    /// 3D world position when the packet reached terminal.
    /// None if no world_coord has been assigned by the layout algorithm yet.
    pub world_position: Option<[f32; 3]>,
    /// Mean quality score across all scored hops. None until the quality scorer runs.
    pub quality: Option<f32>,
}

impl EventRecord {
    /// Construct an EVENT record from a terminal SemanticPacket.
    ///
    /// Called by the worker loop after Router::route returns Terminal.
    /// `output` is the string from RouteResult::Terminal { output, .. }.
    /// `timestamp_ms` is the time the Terminal was recorded (not when routing started).
    pub fn from_terminal(packet: &SemanticPacket, output: &str, timestamp_ms: u64) -> Self {
        Self {
            record_type: RecordType::Event,
            timestamp_ms,
            chain_id: packet.chain_id,
            packet_id: packet.id,
            hop_count: packet.meta.len() as u32,
            identity: packet
                .last_identity()
                .unwrap_or("unknown")
                .to_string(),
            output: output.to_string(),
            world_position: packet.world_position,
            quality: packet.mean_quality(),
        }
    }
}

// ─── EventLog ─────────────────────────────────────────────────────────────────

/// The event log — append-only record of every terminal packet.
pub struct EventLog {
    conn: Mutex<Connection>,
}

impl EventLog {
    /// Open or create a persistent event log at `path`.
    /// Creates the schema on first open.
    pub fn open(path: &str) -> Result<Self, EventLogError> {
        let conn = Connection::open(path).map_err(EventLogError::Sqlite)?;
        let log = Self { conn: Mutex::new(conn) };
        log.init_schema()?;
        Ok(log)
    }

    /// Create an in-memory event log. Used in tests and dry-run mode.
    pub fn open_in_memory() -> Result<Self, EventLogError> {
        let conn = Connection::open_in_memory().map_err(EventLogError::Sqlite)?;
        let log = Self { conn: Mutex::new(conn) };
        log.init_schema()?;
        Ok(log)
    }

    fn init_schema(&self) -> Result<(), EventLogError> {
        self.conn.lock().unwrap().execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id           INTEGER PRIMARY KEY,
                record_type  INTEGER NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                chain_id     INTEGER NOT NULL,
                packet_id    INTEGER NOT NULL,
                hop_count    INTEGER NOT NULL,
                identity     TEXT    NOT NULL,
                output       TEXT    NOT NULL,
                world_x      REAL,
                world_y      REAL,
                world_z      REAL,
                quality      REAL
            );
            CREATE INDEX IF NOT EXISTS events_chain
                ON events(chain_id);
            CREATE INDEX IF NOT EXISTS events_timestamp
                ON events(timestamp_ms);",
        )
        .map_err(EventLogError::Sqlite)
    }

    /// Append one record. Returns the SQLite row ID assigned.
    pub fn append(&self, record: &EventRecord) -> Result<i64, EventLogError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events
             (record_type, timestamp_ms, chain_id, packet_id, hop_count,
              identity, output, world_x, world_y, world_z, quality)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                record.record_type as i64,
                record.timestamp_ms as i64,
                record.chain_id as i64,
                record.packet_id as i64,
                record.hop_count as i64,
                record.identity,
                record.output,
                record.world_position.map(|p| p[0] as f64),
                record.world_position.map(|p| p[1] as f64),
                record.world_position.map(|p| p[2] as f64),
                record.quality.map(|q| q as f64),
            ],
        )
        .map_err(EventLogError::Sqlite)?;
        Ok(conn.last_insert_rowid())
    }

    /// Write a quality score back to an existing record.
    ///
    /// Called by the quality scorer (bone 4) after evaluating a terminal output.
    /// This is the only mutation allowed on the log — everything else is append-only.
    pub fn score(
        &self,
        chain_id: u64,
        packet_id: u64,
        quality: f32,
    ) -> Result<(), EventLogError> {
        self.conn.lock().unwrap().execute(
            "UPDATE events SET quality = ?1
             WHERE chain_id = ?2 AND packet_id = ?3 AND record_type = 0",
            params![quality as f64, chain_id as i64, packet_id as i64],
        )
        .map_err(EventLogError::Sqlite)?;
        Ok(())
    }

    /// All records for a chain, ordered by timestamp ascending.
    pub fn chain(&self, chain_id: u64) -> Result<Vec<EventRecord>, EventLogError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT record_type, timestamp_ms, chain_id, packet_id, hop_count,
                        identity, output, world_x, world_y, world_z, quality
                 FROM events WHERE chain_id = ?1 ORDER BY timestamp_ms",
            )
            .map_err(EventLogError::Sqlite)?;
        collect_records(stmt.query_map(params![chain_id as i64], row_to_record))
    }

    /// The most recent `limit` records across all chains, newest first.
    pub fn recent(&self, limit: usize) -> Result<Vec<EventRecord>, EventLogError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT record_type, timestamp_ms, chain_id, packet_id, hop_count,
                        identity, output, world_x, world_y, world_z, quality
                 FROM events ORDER BY timestamp_ms DESC LIMIT ?1",
            )
            .map_err(EventLogError::Sqlite)?;
        collect_records(stmt.query_map(params![limit as i64], row_to_record))
    }

    /// Total number of records in the log.
    pub fn len(&self) -> Result<usize, EventLogError> {
        let count: i64 = self
            .conn
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .map_err(EventLogError::Sqlite)?;
        Ok(count as usize)
    }

    pub fn is_empty(&self) -> Result<bool, EventLogError> {
        Ok(self.len()? == 0)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<EventRecord> {
    let wx: Option<f64> = row.get(7)?;
    let wy: Option<f64> = row.get(8)?;
    let wz: Option<f64> = row.get(9)?;
    let world_position = match (wx, wy, wz) {
        (Some(x), Some(y), Some(z)) => Some([x as f32, y as f32, z as f32]),
        _ => None,
    };
    Ok(EventRecord {
        record_type:  RecordType::from_i64(row.get::<_, i64>(0)?),
        timestamp_ms: row.get::<_, i64>(1)? as u64,
        chain_id:     row.get::<_, i64>(2)? as u64,
        packet_id:    row.get::<_, i64>(3)? as u64,
        hop_count:    row.get::<_, i64>(4)? as u32,
        identity:     row.get(5)?,
        output:       row.get(6)?,
        world_position,
        quality:      row.get::<_, Option<f64>>(10)?.map(|q| q as f32),
    })
}

fn collect_records(
    mapped: rusqlite::Result<rusqlite::MappedRows<impl FnMut(&rusqlite::Row) -> rusqlite::Result<EventRecord>>>,
) -> Result<Vec<EventRecord>, EventLogError> {
    mapped
        .map_err(EventLogError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(EventLogError::Sqlite)
}

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum EventLogError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::types::{PacketData, SemanticPacket};

    fn ts() -> u64 { 1_700_000_000_000 }

    fn make_event(chain_id: u64, packet_id: u64) -> EventRecord {
        EventRecord {
            record_type:  RecordType::Event,
            timestamp_ms: ts(),
            chain_id,
            packet_id,
            hop_count:    3,
            identity:     "dworld://council.local/identities/PHILOSOPHER".into(),
            output:       "The structure beneath the question is recursion.".into(),
            world_position: None,
            quality:      None,
        }
    }

    #[test]
    fn append_and_retrieve_by_chain() {
        let log = EventLog::open_in_memory().unwrap();
        assert!(log.is_empty().unwrap());

        let row_id = log.append(&make_event(42, 1)).unwrap();
        assert_eq!(row_id, 1);
        assert_eq!(log.len().unwrap(), 1);

        let chain = log.chain(42).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].chain_id, 42);
        assert_eq!(chain[0].packet_id, 1);
        assert_eq!(chain[0].hop_count, 3);
        assert_eq!(chain[0].output, "The structure beneath the question is recursion.");
        assert_eq!(chain[0].quality, None);
        assert_eq!(chain[0].record_type, RecordType::Event);
    }

    #[test]
    fn score_writes_quality_back() {
        let log = EventLog::open_in_memory().unwrap();
        log.append(&make_event(1, 1)).unwrap();
        assert_eq!(log.chain(1).unwrap()[0].quality, None);

        log.score(1, 1, 0.85).unwrap();

        let quality = log.chain(1).unwrap()[0].quality.unwrap();
        assert!((quality - 0.85).abs() < 1e-5, "quality={quality}");
    }

    #[test]
    fn recent_returns_newest_first() {
        let log = EventLog::open_in_memory().unwrap();
        for i in 0..5u64 {
            let mut r = make_event(i, i);
            r.timestamp_ms = ts() + i * 1_000;
            log.append(&r).unwrap();
        }

        let recent = log.recent(3).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].chain_id, 4); // newest first
        assert_eq!(recent[1].chain_id, 3);
        assert_eq!(recent[2].chain_id, 2);
    }

    #[test]
    fn world_position_round_trips() {
        let log = EventLog::open_in_memory().unwrap();
        let mut r = make_event(99, 1);
        r.world_position = Some([1.5, -2.5, 100.0]);
        log.append(&r).unwrap();

        let pos = log.chain(99).unwrap()[0].world_position.unwrap();
        assert!((pos[0] - 1.5).abs() < 1e-5);
        assert!((pos[1] - (-2.5)).abs() < 1e-5);
        assert!((pos[2] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn chains_are_independent() {
        let log = EventLog::open_in_memory().unwrap();
        log.append(&make_event(1, 1)).unwrap();
        log.append(&make_event(2, 2)).unwrap();
        log.append(&make_event(1, 3)).unwrap();

        assert_eq!(log.chain(1).unwrap().len(), 2);
        assert_eq!(log.chain(2).unwrap().len(), 1);
        assert_eq!(log.len().unwrap(), 3);
    }

    #[test]
    fn from_terminal_converts_packet() {
        let mut packet = SemanticPacket::new(
            5, 5,
            PacketData::Text("What is the structure of thought?".into()),
            "dworld://test/".into(),
        );
        packet.push_hop(
            ts(),
            "dworld://council.local/identities/PHILOSOPHER".into(),
            None,
            Some([10.0, 20.0, 30.0]),
        );
        packet.score_last_hop(0.9);

        let record = EventRecord::from_terminal(
            &packet,
            "Thought is recursive structure.",
            ts() + 100,
        );

        assert_eq!(record.record_type, RecordType::Event);
        assert_eq!(record.chain_id, 5);
        assert_eq!(record.packet_id, 5);
        assert_eq!(record.hop_count, 1);
        assert_eq!(record.identity, "dworld://council.local/identities/PHILOSOPHER");
        assert_eq!(record.output, "Thought is recursive structure.");
        assert_eq!(record.world_position, Some([10.0, 20.0, 30.0]));
        let q = record.quality.unwrap();
        assert!((q - 0.9).abs() < 1e-5, "quality={q}");
    }

    #[test]
    fn signal_records_stored_and_retrieved() {
        let log = EventLog::open_in_memory().unwrap();
        let signal = EventRecord {
            record_type:    RecordType::Signal,
            timestamp_ms:   ts(),
            chain_id:       7,
            packet_id:      0,
            hop_count:      0,
            identity:       "dworld://council.local/identities/PHILOSOPHER".into(),
            output:         "drift_alert: PHILOSOPHER → SYNTHESIZER".into(),
            world_position: None,
            quality:        Some(0.3),
        };
        log.append(&signal).unwrap();

        let chain = log.chain(7).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].record_type, RecordType::Signal);
        assert_eq!(chain[0].output, "drift_alert: PHILOSOPHER → SYNTHESIZER");
    }
}
