//! Agent registry — distributed agent directory for the dworld:// network.
//!
//! Any Council that connects to this NEXUS node registers its agents here.
//! Other agents discover peers via GET /.dworld/agents.
//! Messages are routed by NEXUS: find the Council that owns the agent, forward.
//!
//! NEXUS is the router. It does not process messages — it finds the agent's
//! home Council and forwards the message there. The Council does the LLM call.
//!
//! Persistence: JSON file on disk. Loaded at startup. Written on every change.
//! In-memory for reads. No lock contention on the hot path.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

// ─── Agent record ─────────────────────────────────────────────────────────────

/// One agent registered in the distributed network.
///
/// The `address` is its canonical dworld:// identity.
/// The `council_url` is where NEXUS forwards messages (the agent's home Council).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Short name, e.g. "PHILOSOPHER". Unique within the registry.
    pub name: String,
    /// dworld:// address, e.g. "dworld://vps/agents/PHILOSOPHER"
    pub address: String,
    /// What this agent does — injected into discovery results.
    pub description: String,
    /// Capabilities this agent offers, e.g. ["synthesis", "challenge", "decompose"]
    pub capabilities: Vec<String>,
    /// HTTP URL of the home Council. NEXUS forwards messages here.
    /// Format: "http://host:port"
    pub council_url: String,
    /// Last registration timestamp (ms since UNIX epoch).
    pub last_seen: u64,
}

// ─── AgentRegistry ────────────────────────────────────────────────────────────

/// In-memory agent registry with optional disk persistence.
///
/// Thread-safe via RwLock. Reads are concurrent. Writes are serialized.
/// Each write flushes to disk immediately (small files, infrequent writes).
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, AgentRecord>>,
    persist_path: Option<PathBuf>,
}

impl AgentRegistry {
    /// Create a new registry. Loads from `persist_path` if it exists.
    pub fn new(persist_path: Option<PathBuf>) -> Self {
        let agents = persist_path.as_deref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Vec<AgentRecord>>(&s).ok())
            .map(|records| records.into_iter().map(|r| (r.name.clone(), r)).collect())
            .unwrap_or_default();

        Self {
            agents: RwLock::new(agents),
            persist_path,
        }
    }

    /// Register or update an agent. Persists immediately.
    pub fn register(&self, record: AgentRecord) {
        let name = record.name.clone();
        {
            let mut guard = self.agents.write().unwrap();
            guard.insert(name.clone(), record);
        }
        self.flush();
        tracing::debug!("agent registered: {name}");
    }

    /// Look up an agent by name.
    pub fn get(&self, name: &str) -> Option<AgentRecord> {
        self.agents.read().unwrap().get(name).cloned()
    }

    /// All registered agents, in arbitrary order.
    pub fn all(&self) -> Vec<AgentRecord> {
        self.agents.read().unwrap().values().cloned().collect()
    }

    fn flush(&self) {
        if let Some(ref path) = self.persist_path {
            let records: Vec<AgentRecord> = self.agents.read().unwrap().values().cloned().collect();
            if let Ok(json) = serde_json::to_string_pretty(&records) {
                let _ = std::fs::write(path, json);
            }
        }
    }
}
