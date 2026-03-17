---
name: spatial
parent: world
type: sub-program
status: active
phase: 0
layer: 3 (engine primitives)
---

# spatial — In-Memory Spatial Index

**What this sub-program is**: The in-memory octree and all spatial query functions used by the node during simulation. This is the lowest-level spatial primitive in the `world/` subsystem. Everything that needs to ask "what is near X" uses this.

**What it owns**:
- The octree data structure (construction, insertion, deletion, re-balancing)
- All spatial query functions (radius, box, nearest-K, ray-cast)
- Domain subdivision helpers (find split plane, redistribute objects after split)
- Serialization of the octree to/from world graph format (for node startup and backup)

**What it does NOT own**:
- The world graph database (that is external infrastructure)
- Any simulation logic — this is pure spatial data structure code
- Networking — no awareness of nodes, clients, or messages

**Contract it publishes** (internal to world/):
```
SPATIAL INDEX CONTRACT:

  insert(object_id, position, bounding_box) → void
  remove(object_id) → void
  move(object_id, new_position) → void
  update_bounds(object_id, new_bounding_box) → void

  query_radius(center, radius) → list of object_ids
  query_box(min, max) → list of object_ids
  query_nearest_k(position, k) → list of (object_id, distance) sorted by distance
  query_ray(origin, direction, max_distance) → list of (object_id, hit_distance) sorted by hit_distance

  get_position(object_id) → position | NOT_FOUND
  get_count() → total number of objects indexed

  serialize() → binary blob (for world graph persistence)
  deserialize(binary_blob) → void (rebuilds octree from persisted form)
```

**Key parameters** (stored in world/configuration):
```
MAX_OBJECTS_PER_LEAF = 32
  -- When a leaf node exceeds this, it subdivides
  -- Higher value = less tree depth but slower leaf queries
  -- Lower value = more tree depth but faster leaf queries

MIN_OBJECTS_PER_LEAF = 8
  -- When a leaf drops below this AND its parent total < MERGE_THRESHOLD, merge
  -- Prevents thrashing on insert/delete near a leaf threshold

MERGE_THRESHOLD = 24
  -- A parent merges 8 children into a leaf if total objects < this

MAX_TREE_DEPTH = 16
  -- Safeguard against degenerate cases (many objects at exactly the same position)
```

**Performance requirements** (from PRD Part XVI):
- `insert` / `remove` / `move`: amortized O(log N)
- `query_radius` with radius R: O(log N + K) where K = result count
- `query_nearest_k`: O(K log N)
- All operations lock-free for reads (multiple readers, one writer at a time)

**Phase 0 scope**: Build insert, remove, move, query_radius, serialize, deserialize. The ray-cast and nearest-K queries are Phase 1 (needed for AI and player targeting respectively).

**Testing requirement** (before Phase 0 gate):
- Insert 100,000 objects at random positions
- Verify radius query returns correct set at 10 different test radii
- Verify performance: 10,000 queries in under 1 second total
- Verify serialize → deserialize produces identical results
