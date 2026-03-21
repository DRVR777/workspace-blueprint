//! SchemaRegistry — loads all `world/schemas/*.json` files at startup.
//!
//! After loading, any encoder can call `registry.get(schema_id)` to get the
//! descriptor, and any server can respond to MSG_SCHEMA_QUERY by returning
//! the descriptor as JSON bytes.

use std::collections::HashMap;
use std::path::Path;

use crate::descriptor::SchemaDescriptor;

/// Loaded schema registry. Built once at server startup; immutable at runtime.
#[derive(Debug, Default)]
pub struct SchemaRegistry {
    /// Map from schema_id (FNV32 hash) to descriptor.
    by_id:   HashMap<u32, SchemaDescriptor>,
    /// Map from "name@version" to schema_id — for lookup by name.
    by_name: HashMap<String, u32>,
}

impl SchemaRegistry {
    /// Load all `*.json` files from `schemas_dir`. Logs and skips malformed files.
    pub fn load_from_dir(schemas_dir: &Path) -> Self {
        let mut registry = SchemaRegistry::default();

        let entries = match std::fs::read_dir(schemas_dir) {
            Ok(e)  => e,
            Err(e) => {
                eprintln!("[nexus-schema] Cannot read schemas dir {:?}: {}", schemas_dir, e);
                return registry;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            match std::fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str::<SchemaDescriptor>(&json) {
                    Ok(desc) => {
                        let id  = desc.schema_id();
                        let key = format!("{}@{}", desc.name, desc.version);
                        registry.by_id.insert(id, desc);
                        registry.by_name.insert(key, id);
                    }
                    Err(e) => eprintln!("[nexus-schema] Malformed {:?}: {}", path, e),
                },
                Err(e) => eprintln!("[nexus-schema] Cannot read {:?}: {}", path, e),
            }
        }

        eprintln!(
            "[nexus-schema] Loaded {} schemas from {:?}",
            registry.by_id.len(),
            schemas_dir,
        );
        registry
    }

    /// Look up a schema by its computed ID.
    pub fn get(&self, id: u32) -> Option<&SchemaDescriptor> {
        self.by_id.get(&id)
    }

    /// Look up a schema by name and version.
    pub fn get_by_name(&self, name: &str, version: &str) -> Option<&SchemaDescriptor> {
        let key = format!("{}@{}", name, version);
        self.by_name.get(&key).and_then(|id| self.by_id.get(id))
    }

    /// Serialize a descriptor to JSON bytes for a MSG_SCHEMA_RESPONSE payload.
    pub fn to_json_bytes(&self, id: u32) -> Option<Vec<u8>> {
        self.by_id.get(&id)
            .and_then(|d| serde_json::to_vec(d).ok())
    }

    pub fn schema_count(&self) -> usize {
        self.by_id.len()
    }
}
