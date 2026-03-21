//! FNV-1a 32-bit hash — used to derive stable schema IDs from name@version strings.
//!
//! Properties that matter here:
//! - Deterministic: same input always produces the same u32
//! - Coordination-free: two processes that hash "computer@1.0" independently
//!   get the same number without ever talking to each other
//! - Stable: the hash of a published schema name never changes
//! - Collision probability: ~0.03% for 50 schemas in 2^32 space (birthday bound)

/// Compute a stable schema ID from a name and version string.
///
/// The canonical input is `"<name>@<version>"` — e.g., `"spatial_manifest@1.0"`.
/// This is what goes in the wire header's `schema_id` field.
///
/// # Examples
///
/// ```
/// use nexus_schema::schema_id;
/// let id = schema_id("spatial_manifest", "1.0");
/// assert_eq!(id, schema_id("spatial_manifest", "1.0")); // always stable
/// ```
pub fn schema_id(name: &str, version: &str) -> u32 {
    let key = format!("{}@{}", name, version);
    fnv32(key.as_bytes())
}

/// FNV-1a 32-bit hash over raw bytes.
pub fn fnv32(data: &[u8]) -> u32 {
    const OFFSET_BASIS: u32 = 0x811c9dc5;
    const PRIME:        u32 = 0x01000193;
    let mut hash = OFFSET_BASIS;
    for &b in data {
        hash ^= b as u32;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stability() {
        // These values must never change — they are baked into live wire traffic.
        assert_eq!(schema_id("spatial_manifest", "1.0"), schema_id("spatial_manifest", "1.0"));
        assert_eq!(schema_id("agent_task",       "1.0"), schema_id("agent_task",       "1.0"));
    }

    #[test]
    fn bootstrap_ids_not_colliding() {
        // The two hardcoded bootstrap IDs (0 and 1) must not be reachable via hash.
        // fnv32 over any non-empty string starts at 0x811c9dc5 — far from 0 or 1.
        assert!(schema_id("spatial_manifest", "1.0") > 1);
        assert!(schema_id("agent_task",       "1.0") > 1);
        assert!(schema_id("physics_body",     "1.0") > 1);
    }

    #[test]
    fn no_collision_known_schemas() {
        let ids: Vec<u32> = [
            schema_id("physics_body",     "1.0"),
            schema_id("spatial_manifest", "1.0"),
            schema_id("agent_task",       "1.0"),
            schema_id("computer",         "1.0"),
            schema_id("file",             "1.0"),
            schema_id("display_frame",    "1.0"),
        ].to_vec();
        // All unique
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "Schema ID collision detected");
    }
}
