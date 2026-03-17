# spatial — Build Contract (Phase 0)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: octree algorithm, contract surface, performance targets |
| `../../shared/contracts/world-state-contract.md` | `Vec3f64`, `Vec3i32`, `AABB64` types |
| `../../_planning/adr/ADR-002-spatial-index.md` | Confirms: octree in memory, R-tree in DB |

Do NOT load: network schemas, simulation-contract, engine/ files.

---

## Process

1. Create `src/` directory. Implement the `SpatialIndex` class/struct with the exact interface from MANIFEST.md §"Contract it publishes":
   - `insert(object_id, position, bounding_box)`
   - `remove(object_id)`
   - `move(object_id, new_position)`
   - `query_radius(center, radius) → list of object_ids`
   - `serialize() → bytes` / `deserialize(bytes)`
   - Skip `query_nearest_k`, `query_ray`, `update_bounds`, `query_box` — Phase 1

2. Implement the octree node structure:
   - Leaf node: stores up to `MAX_OBJECTS_PER_LEAF = 32` (object_id, position, bounding_box) entries
   - Internal node: 8 child pointers + split planes
   - On `insert`: if leaf exceeds MAX_OBJECTS_PER_LEAF, subdivide. Stop subdivision at `MAX_TREE_DEPTH = 16`
   - On `remove`: if parent total drops below `MERGE_THRESHOLD = 24`, merge children back to leaf

3. Implement `query_radius`: traverse tree, prune branches whose bounding box is entirely outside the sphere, collect leaf entries within radius. Return object_id list.

4. Implement `serialize` / `deserialize`: depth-first traversal, write (node_type, split_planes | entries) to byte buffer. `deserialize` rebuilds the tree from that buffer exactly.

5. Write tests:
   - Insert 100,000 objects at random positions, verify `query_radius` returns correct set at 10 test radii
   - Verify `serialize` → `deserialize` produces identical query results
   - Time 10,000 radius queries — must complete in under 1 second total
   - Verify concurrent reads do not corrupt state (multiple read threads, one write thread)

6. Write `output/phase0-complete.md` summarizing: lines of code, test results (pass/fail + timing), any design deviations from MANIFEST.md spec.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 2 | Octree node structure and subdivision logic (code review) | approve / revise structure |
| Step 5 | Test results with timing numbers | approve → write output / fix failures first |

---

## Audit

Before writing to `output/`:
- [ ] All 6 Phase 0 contract methods are implemented and callable
- [ ] `query_radius` returns correct results at all 10 test radii (verified by test suite)
- [ ] 10,000 queries complete in < 1 second (timing logged in output doc)
- [ ] `serialize` → `deserialize` round-trip produces identical results (verified by test)
- [ ] No dynamic allocation inside `query_radius` hot path
- [ ] `MAX_OBJECTS_PER_LEAF`, `MAX_TREE_DEPTH`, `MERGE_THRESHOLD` are named constants, not magic numbers

---

## Outputs

| Output | Location |
|--------|----------|
| Spatial index implementation | `src/spatial_index.*` |
| Test suite | `src/tests/` |
| Phase 0 completion summary | `output/phase0-complete.md` |
