//! Octree-based spatial index.
//!
//! Implements the spatial index contract from spatial/MANIFEST.md:
//!   insert, remove, move, query_radius, query_box, get_position, get_count.

use nexus_core::math::{Vec3f64, Aabb64};
use nexus_core::types::ObjectId;
use nexus_core::constants::{MAX_OBJECTS_PER_LEAF, MAX_TREE_DEPTH};
use std::collections::HashMap;

/// Entry stored in the octree.
#[derive(Debug, Clone)]
struct SpatialEntry {
    pub object_id: ObjectId,
    pub position: Vec3f64,
    pub bounding_radius: f32,
}

/// The primary spatial index. Wraps the octree with a fast ID→position lookup.
pub struct SpatialIndex {
    root: OctreeNode,
    bounds: Aabb64,
    positions: HashMap<ObjectId, Vec3f64>,
}

enum OctreeNode {
    Leaf {
        entries: Vec<SpatialEntry>,
    },
    Branch {
        children: Box<[OctreeNode; 8]>,
        center: Vec3f64,
    },
}

impl SpatialIndex {
    /// Create a new spatial index covering the given domain.
    pub fn new(bounds: Aabb64) -> Self {
        Self {
            root: OctreeNode::Leaf { entries: Vec::new() },
            bounds,
            positions: HashMap::new(),
        }
    }

    /// Insert an object at a position.
    pub fn insert(&mut self, object_id: ObjectId, position: Vec3f64, bounding_radius: f32) {
        let entry = SpatialEntry { object_id, position, bounding_radius };
        self.positions.insert(object_id, position);
        Self::insert_into(&mut self.root, entry, &self.bounds, 0);
    }

    /// Remove an object by ID.
    pub fn remove(&mut self, object_id: ObjectId) -> bool {
        if self.positions.remove(&object_id).is_some() {
            Self::remove_from(&mut self.root, object_id);
            true
        } else {
            false
        }
    }

    /// Update an object's position.
    pub fn move_object(&mut self, object_id: ObjectId, new_position: Vec3f64) {
        if let Some(old_pos) = self.positions.get(&object_id) {
            let _old = *old_pos;
            // Remove and re-insert (simple approach — optimize later with in-place update)
            if let Some(entry) = Self::remove_entry(&mut self.root, object_id) {
                let updated = SpatialEntry {
                    object_id,
                    position: new_position,
                    bounding_radius: entry.bounding_radius,
                };
                self.positions.insert(object_id, new_position);
                Self::insert_into(&mut self.root, updated, &self.bounds, 0);
            }
        }
    }

    /// Query all objects within radius of center.
    pub fn query_radius(&self, center: Vec3f64, radius: f64) -> Vec<ObjectId> {
        let mut results = Vec::new();
        let radius_sq = radius * radius;
        Self::query_radius_recursive(&self.root, &self.bounds, center, radius_sq, &mut results);
        results
    }

    /// Query all objects within an AABB.
    pub fn query_box(&self, query_bounds: &Aabb64) -> Vec<ObjectId> {
        let mut results = Vec::new();
        Self::query_box_recursive(&self.root, &self.bounds, query_bounds, &mut results);
        results
    }

    /// Get an object's current position.
    pub fn get_position(&self, object_id: ObjectId) -> Option<Vec3f64> {
        self.positions.get(&object_id).copied()
    }

    /// Total number of indexed objects.
    pub fn get_count(&self) -> usize {
        self.positions.len()
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn insert_into(node: &mut OctreeNode, entry: SpatialEntry, bounds: &Aabb64, depth: usize) {
        match node {
            OctreeNode::Leaf { entries } => {
                entries.push(entry);
                // Subdivide if over capacity and not at max depth
                if entries.len() > MAX_OBJECTS_PER_LEAF && depth < MAX_TREE_DEPTH {
                    let center = bounds.center();
                    let old_entries = std::mem::take(entries);
                    let children = Box::new(std::array::from_fn(|_| {
                        OctreeNode::Leaf { entries: Vec::new() }
                    }));
                    *node = OctreeNode::Branch { children, center };
                    // Re-insert all entries
                    for e in old_entries {
                        Self::insert_into(node, e, bounds, depth);
                    }
                }
            }
            OctreeNode::Branch { children, center } => {
                let idx = Self::child_index(*center, entry.position);
                let child_bounds = Self::child_bounds(bounds, *center, idx);
                Self::insert_into(&mut children[idx], entry, &child_bounds, depth + 1);
            }
        }
    }

    fn remove_from(node: &mut OctreeNode, object_id: ObjectId) -> bool {
        match node {
            OctreeNode::Leaf { entries } => {
                if let Some(pos) = entries.iter().position(|e| e.object_id == object_id) {
                    entries.swap_remove(pos);
                    return true;
                }
                false
            }
            OctreeNode::Branch { children, .. } => {
                for child in children.iter_mut() {
                    if Self::remove_from(child, object_id) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn remove_entry(node: &mut OctreeNode, object_id: ObjectId) -> Option<SpatialEntry> {
        match node {
            OctreeNode::Leaf { entries } => {
                if let Some(pos) = entries.iter().position(|e| e.object_id == object_id) {
                    Some(entries.swap_remove(pos))
                } else {
                    None
                }
            }
            OctreeNode::Branch { children, .. } => {
                for child in children.iter_mut() {
                    if let Some(entry) = Self::remove_entry(child, object_id) {
                        return Some(entry);
                    }
                }
                None
            }
        }
    }

    fn query_radius_recursive(
        node: &OctreeNode,
        bounds: &Aabb64,
        center: Vec3f64,
        radius_sq: f64,
        results: &mut Vec<ObjectId>,
    ) {
        // Early out: if the AABB doesn't intersect the query sphere, skip
        if !Self::aabb_intersects_sphere(bounds, center, radius_sq) {
            return;
        }

        match node {
            OctreeNode::Leaf { entries } => {
                for entry in entries {
                    let dist_sq = center.distance_to(entry.position).powi(2);
                    if dist_sq <= radius_sq {
                        results.push(entry.object_id);
                    }
                }
            }
            OctreeNode::Branch { children, center: node_center } => {
                for (i, child) in children.iter().enumerate() {
                    let child_bounds = Self::child_bounds(bounds, *node_center, i);
                    Self::query_radius_recursive(child, &child_bounds, center, radius_sq, results);
                }
            }
        }
    }

    fn query_box_recursive(
        node: &OctreeNode,
        bounds: &Aabb64,
        query: &Aabb64,
        results: &mut Vec<ObjectId>,
    ) {
        if !bounds.intersects(query) {
            return;
        }

        match node {
            OctreeNode::Leaf { entries } => {
                for entry in entries {
                    if query.contains_point(entry.position) {
                        results.push(entry.object_id);
                    }
                }
            }
            OctreeNode::Branch { children, center } => {
                for (i, child) in children.iter().enumerate() {
                    let child_bounds = Self::child_bounds(bounds, *center, i);
                    Self::query_box_recursive(child, &child_bounds, query, results);
                }
            }
        }
    }

    /// Determine which octant a position falls into relative to a center point.
    /// Returns 0-7 based on which side of each axis the position is on.
    fn child_index(center: Vec3f64, position: Vec3f64) -> usize {
        let mut idx = 0;
        if position.x >= center.x { idx |= 1; }
        if position.y >= center.y { idx |= 2; }
        if position.z >= center.z { idx |= 4; }
        idx
    }

    /// Compute the AABB for a child octant.
    fn child_bounds(parent: &Aabb64, center: Vec3f64, index: usize) -> Aabb64 {
        let min = Vec3f64::new(
            if index & 1 != 0 { center.x } else { parent.min.x },
            if index & 2 != 0 { center.y } else { parent.min.y },
            if index & 4 != 0 { center.z } else { parent.min.z },
        );
        let max = Vec3f64::new(
            if index & 1 != 0 { parent.max.x } else { center.x },
            if index & 2 != 0 { parent.max.y } else { center.y },
            if index & 4 != 0 { parent.max.z } else { center.z },
        );
        Aabb64::new(min, max)
    }

    /// Check if an AABB intersects a sphere (center + radius_squared).
    fn aabb_intersects_sphere(aabb: &Aabb64, center: Vec3f64, radius_sq: f64) -> bool {
        // Find closest point on AABB to sphere center
        let closest = Vec3f64::new(
            center.x.clamp(aabb.min.x, aabb.max.x),
            center.y.clamp(aabb.min.y, aabb.max.y),
            center.z.clamp(aabb.min.z, aabb.max.z),
        );
        let dist_sq = center.distance_to(closest).powi(2);
        dist_sq <= radius_sq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bounds() -> Aabb64 {
        Aabb64::new(Vec3f64::ZERO, Vec3f64::new(1000.0, 1000.0, 1000.0))
    }

    #[test]
    fn insert_and_count() {
        let mut index = SpatialIndex::new(test_bounds());
        index.insert(1, Vec3f64::new(50.0, 50.0, 50.0), 1.0);
        index.insert(2, Vec3f64::new(100.0, 100.0, 100.0), 1.0);
        assert_eq!(index.get_count(), 2);
    }

    #[test]
    fn remove_object() {
        let mut index = SpatialIndex::new(test_bounds());
        index.insert(1, Vec3f64::new(50.0, 50.0, 50.0), 1.0);
        assert!(index.remove(1));
        assert_eq!(index.get_count(), 0);
        assert!(!index.remove(1)); // double remove
    }

    #[test]
    fn query_radius_finds_nearby() {
        let mut index = SpatialIndex::new(test_bounds());
        index.insert(1, Vec3f64::new(50.0, 50.0, 50.0), 1.0);
        index.insert(2, Vec3f64::new(55.0, 50.0, 50.0), 1.0);
        index.insert(3, Vec3f64::new(900.0, 900.0, 900.0), 1.0);

        let results = index.query_radius(Vec3f64::new(50.0, 50.0, 50.0), 10.0);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(!results.contains(&3));
    }

    #[test]
    fn query_box_finds_contained() {
        let mut index = SpatialIndex::new(test_bounds());
        index.insert(1, Vec3f64::new(50.0, 50.0, 50.0), 1.0);
        index.insert(2, Vec3f64::new(500.0, 500.0, 500.0), 1.0);

        let query = Aabb64::new(Vec3f64::ZERO, Vec3f64::new(100.0, 100.0, 100.0));
        let results = index.query_box(&query);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));
    }

    #[test]
    fn move_object_updates_position() {
        let mut index = SpatialIndex::new(test_bounds());
        index.insert(1, Vec3f64::new(50.0, 50.0, 50.0), 1.0);
        index.move_object(1, Vec3f64::new(900.0, 900.0, 900.0));

        assert_eq!(index.get_position(1), Some(Vec3f64::new(900.0, 900.0, 900.0)));

        // Should no longer be near origin
        let results = index.query_radius(Vec3f64::new(50.0, 50.0, 50.0), 10.0);
        assert!(results.is_empty());

        // Should be near new position
        let results = index.query_radius(Vec3f64::new(900.0, 900.0, 900.0), 10.0);
        assert!(results.contains(&1));
    }

    #[test]
    fn handles_many_objects() {
        let mut index = SpatialIndex::new(test_bounds());
        // Insert more than MAX_OBJECTS_PER_LEAF to trigger subdivision
        for i in 0..100 {
            let pos = Vec3f64::new(
                (i as f64 * 10.0) % 1000.0,
                (i as f64 * 7.0) % 1000.0,
                (i as f64 * 13.0) % 1000.0,
            );
            index.insert(i, pos, 1.0);
        }
        assert_eq!(index.get_count(), 100);

        // All should be findable with a large radius
        let results = index.query_radius(Vec3f64::new(500.0, 500.0, 500.0), 1500.0);
        assert_eq!(results.len(), 100);
    }
}
