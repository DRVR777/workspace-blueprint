//! NEXUS Spatial — Layer 2
//!
//! In-memory octree spatial index. Provides insert, remove, move, and
//! spatial queries (radius, box). Used by the simulation layer for
//! broad-phase acceleration and by the node-manager for interest management.
//!
//! Spec: world/programs/spatial/MANIFEST.md

pub mod octree;

// Re-export the primary interface
pub use octree::SpatialIndex;
