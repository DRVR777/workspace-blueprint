//! nexus-schema — content-addressed schema registry.
//!
//! # Why this exists
//!
//! Every packet on the NEXUS wire carries a `schema_id: u32` in its header.
//! Without this crate, those IDs are hardcoded constants in `nexus-core`.
//! That means adding a new object type requires editing Rust, recompiling,
//! and redeploying. With this crate, adding an object type is writing a JSON
//! file in `world/schemas/`.
//!
//! # How schema IDs work
//!
//! `schema_id = fnv32("<name>@<version>")` — computed from the descriptor's
//! name and version string. Two systems that have never communicated can
//! agree on a schema ID without coordination by hashing the same string.
//! The hash IS the identity. No central registry, no enum, no code change.
//!
//! Two IDs are hardcoded forever:
//!
//! ```text
//! 0  SCHEMA_UNTYPED   — legacy / bootstrap, always decodable as physics
//! 1  SCHEMA_REGISTRY  — used by MSG_SCHEMA_QUERY/RESPONSE to discover others
//! ```
//!
//! Everything else is in `world/schemas/*.json`.
//!
//! # Adding a new schema
//!
//! 1. Write `world/schemas/<name>.json` — see `SchemaDescriptor` for the shape.
//! 2. Compute the ID: `nexus_schema::schema_id("<name>", "<version>")`.
//! 3. Use it in your encoder: `encode_message_with_schema(msg_type, payload, id)`.
//! 4. Done. No Rust code changes. No redeploy of the core crates.

pub mod descriptor;
pub mod registry;
pub mod hash;

pub use descriptor::{SchemaDescriptor, FieldDescriptor, FieldType};
pub use registry::SchemaRegistry;
pub use hash::schema_id;

// The two bootstrap IDs — hardcoded forever. Everything else is `schema_id()`.
pub const SCHEMA_UNTYPED:  u32 = 0;
pub const SCHEMA_REGISTRY: u32 = 1;

// Wire message types for schema discovery.
// Receiver gets an unknown schema_id → sends SCHEMA_QUERY → gets SCHEMA_RESPONSE.
pub const MSG_SCHEMA_QUERY:    u16 = 0x0500; // C→S: "what is schema 0xABCD1234?"
pub const MSG_SCHEMA_RESPONSE: u16 = 0x0501; // S→C: SchemaDescriptor as JSON bytes
pub const MSG_SCHEMA_NOT_FOUND: u16 = 0x0502; // S→C: schema_id unknown to this server
