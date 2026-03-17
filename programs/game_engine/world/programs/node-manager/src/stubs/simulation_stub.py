"""
Phase 0 stub: applies MOVE actions as instant position updates.
No physics integration, no collision detection.

This stub fulfills the simulation-contract.md run_tick() signature exactly
so node_manager.py can swap in the real simulation/ program with no changes.
"""

import struct
import time
from dataclasses import dataclass, field

# Change request types (matches change_request.type in world-state-contract.md)
CHANGE_TYPE_MOVE            = 0
CHANGE_TYPE_PROPERTY_CHANGE = 1
CHANGE_TYPE_CREATE          = 2
CHANGE_TYPE_DESTROY         = 3
CHANGE_TYPE_INTERACT        = 4

# State change event types (matches state_change_event.change_type)
EVT_POSITION_CHANGED = 0x0003

_MOVE_FMT = ">fff"   # target_x, target_y, target_z


@dataclass
class ChangeRequest:
    """Mirrors change_request shape from world-state-contract.md."""
    source: int           # entity_id making the request
    type: int             # CHANGE_TYPE_*
    object_id: int        # target entity
    sequence_number: int
    requires_ack: bool
    payload: bytes


@dataclass
class SimpleEntity:
    """Minimal entity record used by the snapshot (subset of entity_record)."""
    entity_id: int
    pos_x: float
    pos_y: float
    pos_z: float


@dataclass
class WorldSnapshot:
    """Minimal world_state_snapshot for Phase 0 (position only)."""
    tick_number: int
    entities: list[SimpleEntity] = field(default_factory=list)


@dataclass
class StateChangeEvent:
    """Mirrors state_change_event shape from world-state-contract.md."""
    object_id: int
    change_type: int
    new_pos: tuple[float, float, float]
    old_pos: tuple[float, float, float] = (0.0, 0.0, 0.0)
    timestamp_ms: int = 0

    def __post_init__(self) -> None:
        if self.timestamp_ms == 0:
            self.timestamp_ms = int(time.time() * 1000)


@dataclass
class TickResult:
    """Mirrors tick_result shape from simulation-contract.md."""
    next_tick_number: int
    state_changes: list[StateChangeEvent] = field(default_factory=list)
    events: list                           = field(default_factory=list)
    rejected_requests: list                = field(default_factory=list)


class SimulationStub:
    """
    Deterministic stub: same inputs → same outputs.
    MOVE action → teleport entity to target position (no physics).
    """

    def run_tick(self,
                 snapshot: WorldSnapshot,
                 inputs: list[ChangeRequest],
                 dt: float) -> TickResult:
        changes: list[StateChangeEvent] = []
        entity_map = {e.entity_id: e for e in snapshot.entities}

        for req in inputs:
            if req.type != CHANGE_TYPE_MOVE:
                continue
            if len(req.payload) < 12:
                continue
            tx, ty, tz = struct.unpack_from(_MOVE_FMT, req.payload)
            entity = entity_map.get(req.source)
            if entity is None:
                continue
            old = (entity.pos_x, entity.pos_y, entity.pos_z)
            entity.pos_x, entity.pos_y, entity.pos_z = tx, ty, tz
            changes.append(StateChangeEvent(
                object_id=entity.entity_id,
                change_type=EVT_POSITION_CHANGED,
                old_pos=old,
                new_pos=(tx, ty, tz),
            ))

        return TickResult(
            next_tick_number=snapshot.tick_number + 1,
            state_changes=changes,
        )
