"""
Phase 0 stub: dict-backed spatial index satisfying the spatial/ contract surface.

Replace with the real octree from world/programs/spatial/ when that program is built.
The interface here matches spatial/MANIFEST.md §"Contract it publishes" exactly so the
swap is a single import change in node_manager.py.
"""

from dataclasses import dataclass


@dataclass
class _Entry:
    position: tuple[float, float, float]
    bounding_box: tuple[float, float, float]   # half-extents x/y/z


class SpatialStub:
    """O(N) linear scan — acceptable for Phase 0 client counts (<= 50)."""

    def __init__(self) -> None:
        self._entries: dict[int, _Entry] = {}

    # -- Mutation ----------------------------------------------------------

    def insert(self, object_id: int,
               position: tuple[float, float, float],
               bounding_box: tuple[float, float, float]) -> None:
        self._entries[object_id] = _Entry(position, bounding_box)

    def remove(self, object_id: int) -> None:
        self._entries.pop(object_id, None)

    def move(self, object_id: int,
             new_position: tuple[float, float, float]) -> None:
        if object_id in self._entries:
            self._entries[object_id].position = new_position

    def update_bounds(self, object_id: int,
                      new_bounding_box: tuple[float, float, float]) -> None:
        if object_id in self._entries:
            self._entries[object_id].bounding_box = new_bounding_box

    # -- Query -------------------------------------------------------------

    def query_radius(self, center: tuple[float, float, float],
                     radius: float) -> list[int]:
        cx, cy, cz = center
        r2 = radius * radius
        return [
            oid for oid, e in self._entries.items()
            if (cx - e.position[0]) ** 2
             + (cy - e.position[1]) ** 2
             + (cz - e.position[2]) ** 2 <= r2
        ]

    def get_position(self, object_id: int) -> tuple[float, float, float] | None:
        e = self._entries.get(object_id)
        return e.position if e else None

    def get_count(self) -> int:
        return len(self._entries)

    # -- Persistence -------------------------------------------------------

    def serialize(self) -> bytes:
        import json
        data = {
            str(k): list(v.position) + list(v.bounding_box)
            for k, v in self._entries.items()
        }
        return json.dumps(data).encode()

    def deserialize(self, blob: bytes) -> None:
        import json
        data = json.loads(blob)
        self._entries = {
            int(k): _Entry(
                position=tuple(v[:3]),
                bounding_box=tuple(v[3:]),
            )
            for k, v in data.items()
        }
