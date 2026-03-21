//! SchemaDescriptor — JSON shape of a schema file in `world/schemas/`.
//!
//! Each file describes one object type. The schema_id is not stored in the file —
//! it is derived at load time via `schema_id(name, version)`.

use serde::{Deserialize, Serialize};

/// One schema file — `world/schemas/<name>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDescriptor {
    /// Short machine name, no spaces. Used as part of the schema_id hash input.
    pub name:        String,
    /// Semver-like version string. Increment when fields change incompatibly.
    pub version:     String,
    /// Human-readable description shown in debug tools and schema browsers.
    pub description: String,
    /// Ordered field list — defines the binary wire layout.
    pub fields:      Vec<FieldDescriptor>,
    /// Wire encoding hint. Default: "le-binary" (little-endian length-prefixed).
    #[serde(default = "default_encoding")]
    pub encoding:    String,
}

impl SchemaDescriptor {
    /// Compute the stable schema_id for this descriptor.
    pub fn schema_id(&self) -> u32 {
        crate::hash::schema_id(&self.name, &self.version)
    }
}

fn default_encoding() -> String {
    "le-binary".to_string()
}

/// One field in a schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub name:        String,
    #[serde(rename = "type")]
    pub field_type:  FieldType,
    #[serde(default)]
    pub description: String,
}

/// Primitive field types supported in schema descriptors.
/// These are wire-level types — not language types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    /// UTF-8 string, length-prefixed with u16.
    String,
    /// Optional string: u16 length, 0 = absent.
    #[serde(rename = "string?")]
    StringOpt,
    /// Variable-length byte array, u32-length-prefixed.
    Bytes,
    /// URI string (same wire format as string, semantically a URI).
    Uri,
    /// Array of the inner type, u8-count-prefixed.
    #[serde(rename = "array")]
    Array(Box<FieldType>),
}
