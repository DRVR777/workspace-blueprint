"""
NEXUS spatial/ — In-memory octree spatial index.

Phase 0 scope (CONTEXT.md): insert, remove, move, query_radius, serialize, deserialize.
Phase 1 scope (deferred):   query_nearest_k, query_ray, update_bounds, query_box.

Key parameters (MANIFEST.md):
  MAX_OBJECTS_PER_LEAF = 32   — subdivide leaf when exceeded
  MIN_OBJECTS_PER_LEAF = 8    — minimum before parent considers merging
  MERGE_THRESHOLD      = 24   — parent merges subtree → leaf when subtree count < this
  MAX_TREE_DEPTH       = 16   — safeguard against degenerate co-located insertions

Thread safety:
  RLock around all reads and writes (Phase 0 simplicity).
  Phase 1 upgrade: replace with a readers-writer lock (shared reads, exclusive writes).
"""

import struct
import threading
from io import BytesIO

# Named constants — not magic numbers (CONTEXT.md audit requirement)
MAX_OBJECTS_PER_LEAF: int = 32
MIN_OBJECTS_PER_LEAF: int = 8
MERGE_THRESHOLD: int      = 24
MAX_TREE_DEPTH: int       = 16

# Entry tuple column indices — (object_id, px, py, pz, bbx, bby, bbz)
_OID = 0; _PX = 1; _PY = 2; _PZ = 3

# Serialization constants
_NODE_LEAF     = 0
_NODE_INTERNAL = 1
_ENTRY_FMT     = ">Qffffff"                    # uint64 + 6 × float32 = 32 bytes
_ENTRY_SIZE    = struct.calcsize(_ENTRY_FMT)   # 32
_BOUNDS_FMT    = ">6d"                          # 6 × float64 = 48 bytes
_BOUNDS_SIZE   = struct.calcsize(_BOUNDS_FMT)  # 48


# ---------------------------------------------------------------------------
# Octree node
# ---------------------------------------------------------------------------

class _Node:
    """
    Single octree node — either a leaf or an internal node.

    Leaf:     is_leaf=True,  entries=[tuple…], children=None
    Internal: is_leaf=False, entries=[],       children=[_Node|None]×8

    `count` tracks the total entries in this node's subtree, maintained
    incrementally so merge threshold checks are O(1) per ancestor level.
    """
    __slots__ = (
        "min_x", "min_y", "min_z",
        "max_x", "max_y", "max_z",
        "depth", "is_leaf", "count",
        "entries",   # list[tuple]  — active only when is_leaf=True
        "children",  # list[_Node|None]×8 — active only when is_leaf=False
    )

    def __init__(self, min_x: float, min_y: float, min_z: float,
                  max_x: float, max_y: float, max_z: float,
                  depth: int) -> None:
        self.min_x = min_x;  self.min_y = min_y;  self.min_z = min_z
        self.max_x = max_x;  self.max_y = max_y;  self.max_z = max_z
        self.depth    = depth
        self.is_leaf  = True
        self.count    = 0
        self.entries  = []
        self.children = None

    def collect_entries(self) -> list:
        """Recursively collect all entry tuples from this subtree (used on merge)."""
        if self.is_leaf:
            return list(self.entries)
        out: list = []
        for child in self.children:
            if child is not None:
                out.extend(child.collect_entries())
        return out


# ---------------------------------------------------------------------------
# Module-level helpers — outside the class to avoid attribute-lookup overhead
# on the query hot path
# ---------------------------------------------------------------------------

def _octant(cx: float, cy: float, cz: float,
            px: float, py: float, pz: float) -> int:
    """Return 0–7 octant index.  Bit layout: bit0=x≥cx, bit1=y≥cy, bit2=z≥cz."""
    return (1 if px >= cx else 0) | (2 if py >= cy else 0) | (4 if pz >= cz else 0)


def _child_bounds(node: "_Node", octant: int) -> tuple:
    """Return (min_x, min_y, min_z, max_x, max_y, max_z) for the child at `octant`."""
    cx = (node.min_x + node.max_x) * 0.5
    cy = (node.min_y + node.max_y) * 0.5
    cz = (node.min_z + node.max_z) * 0.5
    x0, x1 = (cx, node.max_x) if (octant & 1) else (node.min_x, cx)
    y0, y1 = (cy, node.max_y) if (octant & 2) else (node.min_y, cy)
    z0, z1 = (cz, node.max_z) if (octant & 4) else (node.min_z, cz)
    return x0, y0, z0, x1, y1, z1


def _sphere_aabb(cx: float, cy: float, cz: float, r2: float,
                  min_x: float, min_y: float, min_z: float,
                  max_x: float, max_y: float, max_z: float) -> bool:
    """True if the sphere (center, radius=√r2) intersects the AABB."""
    dx = max(min_x - cx, 0.0, cx - max_x)
    dy = max(min_y - cy, 0.0, cy - max_y)
    dz = max(min_z - cz, 0.0, cz - max_z)
    return dx * dx + dy * dy + dz * dz <= r2


# ---------------------------------------------------------------------------
# SpatialIndex — public interface per spatial/MANIFEST.md
# ---------------------------------------------------------------------------

class SpatialIndex:
    """
    In-memory octree spatial index.

    Phase 0 public surface:
        insert(object_id, position, bounding_box)  → None
        remove(object_id)                           → None
        move(object_id, new_position)               → None
        query_radius(center, radius)                → list[int]
        get_position(object_id)                     → tuple | None
        get_count()                                 → int
        serialize()                                 → bytes
        deserialize(blob)                           → None

    Phase 1 (deferred): query_nearest_k, query_ray, update_bounds, query_box
    """

    def __init__(self,
                  world_min: tuple = (0.0,    0.0,    0.0),
                  world_max: tuple = (1000.0, 1000.0, 1000.0)) -> None:
        self._root      = _Node(*world_min, *world_max, depth=0)
        self._positions: dict[int, tuple] = {}   # object_id → (px, py, pz)
        self._bboxes:    dict[int, tuple] = {}   # object_id → (bbx, bby, bbz)
        self._lock       = threading.RLock()

    # ------------------------------------------------------------------
    # Mutation
    # ------------------------------------------------------------------

    def insert(self, object_id: int,
               position: tuple,
               bounding_box: tuple) -> None:
        """Insert or replace an object. If object_id already exists, it is moved."""
        px, py, pz    = position
        bbx, bby, bbz = bounding_box
        with self._lock:
            if object_id in self._positions:
                self._remove_entry(object_id)
            self._positions[object_id] = (px, py, pz)
            self._bboxes[object_id]    = (bbx, bby, bbz)
            self._insert_entry(self._root, object_id, px, py, pz, bbx, bby, bbz)

    def remove(self, object_id: int) -> None:
        """Remove an object from the index. No-op if not present."""
        with self._lock:
            if object_id not in self._positions:
                return
            self._remove_entry(object_id)
            del self._positions[object_id]
            del self._bboxes[object_id]

    def move(self, object_id: int, new_position: tuple) -> None:
        """Update an object's position. No-op if object not present."""
        px, py, pz = new_position
        with self._lock:
            if object_id not in self._positions:
                return
            bb = self._bboxes[object_id]
            self._remove_entry(object_id)
            self._positions[object_id] = (px, py, pz)
            self._insert_entry(self._root, object_id, px, py, pz, *bb)

    # ------------------------------------------------------------------
    # Query
    # ------------------------------------------------------------------

    def query_radius(self, center: tuple, radius: float) -> list[int]:
        """
        Return all object_ids within `radius` of `center`.
        O(log N + K) where K = result count.

        Iterative DFS with sphere-AABB prune.

        Hot-path optimisations (CPython):
          - AABB test inlined: avoids ~2M function-call frames for 10K queries
          - Entry positions by integer literal: avoids global name lookup for
            _PX/_PY/_PZ on every iteration
          - result.append / stack.append bound to locals: avoids attr lookup
        """
        cx, cy, cz = center
        r2 = radius * radius
        result: list[int] = []
        _append = result.append
        with self._lock:
            stack     = [self._root]
            _pop      = stack.pop
            _s_append = stack.append
            while stack:
                node = _pop()
                # Inline sphere-AABB overlap.
                _dx = max(node.min_x - cx, 0.0, cx - node.max_x)
                _dy = max(node.min_y - cy, 0.0, cy - node.max_y)
                _dz = max(node.min_z - cz, 0.0, cz - node.max_z)
                if _dx * _dx + _dy * _dy + _dz * _dz > r2:
                    continue
                if node.is_leaf:
                    for entry in node.entries:
                        # Integer literals: no global lookup per access
                        dx = cx - entry[1]
                        dy = cy - entry[2]
                        dz = cz - entry[3]
                        if dx * dx + dy * dy + dz * dz <= r2:
                            _append(entry[0])
                else:
                    for child in node.children:
                        if child is not None:
                            _s_append(child)
        return result

    def get_position(self, object_id: int) -> tuple | None:
        return self._positions.get(object_id)

    def get_count(self) -> int:
        return len(self._positions)

    # ------------------------------------------------------------------
    # Serialization
    # ------------------------------------------------------------------

    def serialize(self) -> bytes:
        """
        Depth-first binary encoding.
          [root bounds: 48 bytes (6×float64)]
          [DFS nodes:
            leaf:     0x00 | count(uint16) | entries(count×32 bytes)
            internal: 0x01 | presence_mask(uint8) | children in octant order
          ]
        Child bounds are derivable from parent bounds + octant index,
        so they are not stored — this keeps the format compact.
        """
        buf = BytesIO()
        buf.write(struct.pack(_BOUNDS_FMT,
                               self._root.min_x, self._root.min_y, self._root.min_z,
                               self._root.max_x, self._root.max_y, self._root.max_z))
        self._write_node(buf, self._root)
        return buf.getvalue()

    def _write_node(self, buf: BytesIO, node: _Node) -> None:
        if node.is_leaf:
            buf.write(bytes([_NODE_LEAF]))
            buf.write(struct.pack(">H", len(node.entries)))
            for entry in node.entries:
                buf.write(struct.pack(_ENTRY_FMT, *entry))
        else:
            buf.write(bytes([_NODE_INTERNAL]))
            presence = sum((1 << i)
                           for i, c in enumerate(node.children)
                           if c is not None)
            buf.write(bytes([presence]))
            for child in node.children:
                if child is not None:
                    self._write_node(buf, child)

    def deserialize(self, blob: bytes) -> None:
        """Rebuild tree from bytes produced by serialize(). Replaces current state."""
        buf = BytesIO(blob)
        min_x, min_y, min_z, max_x, max_y, max_z = struct.unpack(_BOUNDS_FMT,
                                                                    buf.read(_BOUNDS_SIZE))
        self._root      = _Node(min_x, min_y, min_z, max_x, max_y, max_z, depth=0)
        self._positions = {}
        self._bboxes    = {}
        self._root.count = self._read_node(buf, self._root)

    def _read_node(self, buf: BytesIO, node: _Node) -> int:
        """
        Read one node from buf, populate it, and return its subtree entry count.
        The returned count is used by the parent to set its own count field.
        """
        node_type = buf.read(1)[0]
        if node_type == _NODE_LEAF:
            count = struct.unpack(">H", buf.read(2))[0]
            for _ in range(count):
                entry = struct.unpack(_ENTRY_FMT, buf.read(_ENTRY_SIZE))
                node.entries.append(entry)
                oid, px, py, pz, bbx, bby, bbz = entry
                self._positions[oid] = (px, py, pz)
                self._bboxes[oid]    = (bbx, bby, bbz)
            node.count = count
            return count
        else:
            presence = buf.read(1)[0]
            node.is_leaf  = False
            node.entries  = []
            node.children = [None] * 8
            total = 0
            for i in range(8):
                if presence & (1 << i):
                    child = _Node(*_child_bounds(node, i), node.depth + 1)
                    node.children[i] = child
                    total += self._read_node(buf, child)
            node.count = total
            return total

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _insert_entry(self, node: _Node,
                       oid: int,
                       px: float, py: float, pz: float,
                       bbx: float, bby: float, bbz: float) -> None:
        """Recursively insert, incrementing count at every ancestor along the path."""
        node.count += 1
        if node.is_leaf:
            node.entries.append((oid, px, py, pz, bbx, bby, bbz))
            if (len(node.entries) > MAX_OBJECTS_PER_LEAF
                    and node.depth < MAX_TREE_DEPTH):
                self._subdivide(node)
        else:
            cx = (node.min_x + node.max_x) * 0.5
            cy = (node.min_y + node.max_y) * 0.5
            cz = (node.min_z + node.max_z) * 0.5
            idx = _octant(cx, cy, cz, px, py, pz)
            if node.children[idx] is None:
                node.children[idx] = _Node(*_child_bounds(node, idx),
                                            node.depth + 1)
            self._insert_entry(node.children[idx], oid, px, py, pz, bbx, bby, bbz)

    def _subdivide(self, node: _Node) -> None:
        """Convert a full leaf into an internal node, redistributing its entries."""
        entries       = node.entries
        node.entries  = []
        node.children = [None] * 8
        node.is_leaf  = False
        # node.count is unchanged — it was already set to len(entries)
        cx = (node.min_x + node.max_x) * 0.5
        cy = (node.min_y + node.max_y) * 0.5
        cz = (node.min_z + node.max_z) * 0.5
        for entry in entries:
            idx = _octant(cx, cy, cz, entry[_PX], entry[_PY], entry[_PZ])
            if node.children[idx] is None:
                node.children[idx] = _Node(*_child_bounds(node, idx),
                                            node.depth + 1)
            node.children[idx].entries.append(entry)
            node.children[idx].count += 1

    def _remove_entry(self, object_id: int) -> None:
        """
        Two-pass remove:
          Pass 1 — navigate to the containing leaf using stored position; remove entry.
          Pass 2 — walk path back to root: decrement count at every ancestor,
                   then merge any ancestor whose count drops below MERGE_THRESHOLD.

        Separating the two concerns means all ancestor counts are always correct,
        whether or not a merge happens at any level.
        """
        px, py, pz = self._positions[object_id]
        path: list[_Node] = []   # ancestors root→parent, for the upward pass
        node = self._root

        # Pass 1: navigate to the containing leaf
        while not node.is_leaf:
            cx = (node.min_x + node.max_x) * 0.5
            cy = (node.min_y + node.max_y) * 0.5
            cz = (node.min_z + node.max_z) * 0.5
            idx = _octant(cx, cy, cz, px, py, pz)
            child = node.children[idx]
            if child is None:
                return   # structural gap — should not occur; silently skip
            path.append(node)
            node = child

        # Remove from leaf
        before       = len(node.entries)
        node.entries = [e for e in node.entries if e[_OID] != object_id]
        if len(node.entries) == before:
            return   # entry not found (guard against double-remove)
        node.count -= 1

        # Pass 2: update every ancestor — always decrement, then check merge
        for parent in reversed(path):
            parent.count -= 1
            if parent.is_leaf:
                continue   # already merged by a deeper iteration
            if parent.count < MERGE_THRESHOLD:
                # Collapse subtree into a leaf; count is already correct
                parent.entries  = parent.collect_entries()
                parent.children = None
                parent.is_leaf  = True
