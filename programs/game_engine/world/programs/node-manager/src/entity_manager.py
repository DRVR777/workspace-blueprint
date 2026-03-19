"""
Entity lifecycle manager for the NEXUS node-manager.

Handles spawn, update, destroy operations on the authoritative entity registry.
The entity manager is the single source of truth for entity state within a node.

Entity lifecycle: SPAWNING -> ACTIVE -> DESTROYING -> removed
All mutations go through this module — node_manager.py never writes entity
state directly.

References:
  - world-state-contract.md: entity_record shape
  - entity_update.fbs: serialization schema
  - MANIFEST.md §TICK LOOP Phase C: apply results to local snapshot
"""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional

import config

logger = logging.getLogger("entity_manager")


class EntityState(IntEnum):
    """Entity lifecycle states."""
    SPAWNING = 0
    ACTIVE = 1
    DESTROYING = 2


@dataclass
class Entity:
    """
    Authoritative entity record for a single entity in this node's domain.

    Matches the entity_record shape from world-state-contract.md:
      entity_id, entity_type, position, velocity, orientation, health,
      owner_player_id, spawn_tick, last_update_tick.
    """
    entity_id: int
    entity_type: str = "player"
    owner_player_id: int = 0
    display_name: str = ""

    # Position (mutable each tick)
    pos_x: float = 0.0
    pos_y: float = 0.0
    pos_z: float = 0.0

    # Velocity (Phase 0: not used by simulation stub, but tracked)
    vel_x: float = 0.0
    vel_y: float = 0.0
    vel_z: float = 0.0

    # Orientation quaternion (w, x, y, z)
    orient_w: float = 1.0
    orient_x: float = 0.0
    orient_y: float = 0.0
    orient_z: float = 0.0

    # Lifecycle
    state: EntityState = EntityState.ACTIVE
    spawn_tick: int = 0
    last_update_tick: int = 0
    created_at_ms: int = 0

    def __post_init__(self) -> None:
        if self.created_at_ms == 0:
            self.created_at_ms = int(time.time() * 1000)

    @property
    def position(self) -> list[float]:
        """Mutable position as [x, y, z]."""
        return [self.pos_x, self.pos_y, self.pos_z]

    @position.setter
    def position(self, value: list[float] | tuple[float, float, float]) -> None:
        self.pos_x, self.pos_y, self.pos_z = value[0], value[1], value[2]


class EntityManager:
    """
    Manages the authoritative entity registry for a single node.

    Thread-safe for read access from the tick loop; mutations happen
    only during tick phases A-C (single-threaded within the tick).
    """

    def __init__(self) -> None:
        self._entities: dict[int, Entity] = {}
        self._next_entity_id: int = 1
        self._destroy_queue: list[int] = []

    # ------------------------------------------------------------------
    # ID allocation
    # ------------------------------------------------------------------

    def allocate_id(self) -> int:
        """Allocate a unique entity ID. Monotonically increasing."""
        eid = self._next_entity_id
        self._next_entity_id += 1
        return eid

    # ------------------------------------------------------------------
    # Spawn
    # ------------------------------------------------------------------

    def spawn(
        self,
        entity_id: int,
        entity_type: str = "player",
        owner_player_id: int = 0,
        display_name: str = "",
        position: tuple[float, float, float] = config.DEFAULT_SPAWN_POSITION,
        spawn_tick: int = 0,
    ) -> Entity:
        """
        Spawn a new entity into the world.

        Raises ValueError if entity_id already exists or max entities exceeded.
        """
        if entity_id in self._entities:
            raise ValueError(f"Entity {entity_id} already exists")
        if len(self._entities) >= config.MAX_ENTITIES:
            raise ValueError(
                f"Max entities ({config.MAX_ENTITIES}) reached — cannot spawn"
            )

        entity = Entity(
            entity_id=entity_id,
            entity_type=entity_type,
            owner_player_id=owner_player_id,
            display_name=display_name,
            pos_x=position[0],
            pos_y=position[1],
            pos_z=position[2],
            state=EntityState.ACTIVE,
            spawn_tick=spawn_tick,
            last_update_tick=spawn_tick,
        )
        self._entities[entity_id] = entity
        logger.debug(
            "Spawned entity %d (%s) at (%.1f, %.1f, %.1f)",
            entity_id, entity_type, position[0], position[1], position[2],
        )
        return entity

    # ------------------------------------------------------------------
    # Update
    # ------------------------------------------------------------------

    def update_position(
        self,
        entity_id: int,
        new_position: tuple[float, float, float],
        tick: int = 0,
    ) -> Optional[Entity]:
        """
        Update an entity's position. Returns the entity if found, None otherwise.
        """
        entity = self._entities.get(entity_id)
        if entity is None or entity.state != EntityState.ACTIVE:
            return None
        entity.pos_x, entity.pos_y, entity.pos_z = new_position
        entity.last_update_tick = tick
        return entity

    def get(self, entity_id: int) -> Optional[Entity]:
        """Get an entity by ID, or None if not found."""
        return self._entities.get(entity_id)

    def get_all(self) -> list[Entity]:
        """Return all active entities."""
        return [
            e for e in self._entities.values()
            if e.state == EntityState.ACTIVE
        ]

    def get_all_dict(self) -> dict[int, Entity]:
        """Return the full entity dict (read-only access pattern)."""
        return self._entities

    @property
    def count(self) -> int:
        """Number of entities currently managed."""
        return len(self._entities)

    # ------------------------------------------------------------------
    # Destroy
    # ------------------------------------------------------------------

    def mark_for_destroy(self, entity_id: int) -> bool:
        """
        Mark an entity for destruction. It will be removed at the end
        of the current tick (after broadcast).

        Returns True if the entity was found and marked.
        """
        entity = self._entities.get(entity_id)
        if entity is None:
            return False
        entity.state = EntityState.DESTROYING
        self._destroy_queue.append(entity_id)
        logger.debug("Marked entity %d for destruction", entity_id)
        return True

    def destroy(self, entity_id: int) -> Optional[Entity]:
        """
        Immediately remove an entity from the registry.
        Returns the removed entity, or None if not found.
        """
        entity = self._entities.pop(entity_id, None)
        if entity is not None:
            logger.debug("Destroyed entity %d (%s)", entity_id, entity.entity_type)
        return entity

    def flush_destroy_queue(self) -> list[Entity]:
        """
        Remove all entities marked for destruction.
        Called at end of tick after broadcasts are sent.
        Returns the list of destroyed entities.
        """
        destroyed = []
        for eid in self._destroy_queue:
            entity = self._entities.pop(eid, None)
            if entity is not None:
                destroyed.append(entity)
        self._destroy_queue.clear()
        return destroyed

    # ------------------------------------------------------------------
    # Snapshot
    # ------------------------------------------------------------------

    def snapshot_positions(self) -> list[dict]:
        """
        Return a lightweight position snapshot for all active entities.
        Used by the state serializer for EPU encoding.
        """
        return [
            {
                "entity_id": e.entity_id,
                "pos_x": e.pos_x,
                "pos_y": e.pos_y,
                "pos_z": e.pos_z,
                "vel_x": e.vel_x,
                "vel_y": e.vel_y,
                "vel_z": e.vel_z,
                "orient_w": e.orient_w,
                "orient_x": e.orient_x,
                "orient_y": e.orient_y,
                "orient_z": e.orient_z,
            }
            for e in self._entities.values()
            if e.state == EntityState.ACTIVE
        ]
