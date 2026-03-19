"""
Unit tests for the entity_manager module.

Tests entity lifecycle: spawn, update, destroy, capacity limits,
and snapshot generation.
"""

import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

import config
from entity_manager import EntityManager, Entity, EntityState


class TestEntitySpawn:
    """Tests for entity spawn lifecycle."""

    def test_spawn_creates_entity(self):
        """Spawn should create an entity with correct attributes."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        entity = mgr.spawn(
            entity_id=eid,
            entity_type="player",
            owner_player_id=42,
            display_name="TestPlayer",
            position=(100.0, 0.0, 200.0),
            spawn_tick=5,
        )
        assert entity.entity_id == eid
        assert entity.entity_type == "player"
        assert entity.owner_player_id == 42
        assert entity.display_name == "TestPlayer"
        assert entity.pos_x == 100.0
        assert entity.pos_y == 0.0
        assert entity.pos_z == 200.0
        assert entity.state == EntityState.ACTIVE
        assert entity.spawn_tick == 5

    def test_spawn_allocates_unique_ids(self):
        """Each allocate_id call should return a unique ID."""
        mgr = EntityManager()
        ids = [mgr.allocate_id() for _ in range(100)]
        assert len(set(ids)) == 100

    def test_spawn_duplicate_id_raises(self):
        """Spawning with an existing entity_id should raise ValueError."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        mgr.spawn(entity_id=eid)
        with pytest.raises(ValueError, match="already exists"):
            mgr.spawn(entity_id=eid)

    def test_spawn_at_capacity_raises(self):
        """Spawning beyond MAX_ENTITIES should raise ValueError."""
        mgr = EntityManager()
        original = config.MAX_ENTITIES
        try:
            config.MAX_ENTITIES = 3
            for i in range(3):
                mgr.spawn(entity_id=mgr.allocate_id())
            with pytest.raises(ValueError, match="Max entities"):
                mgr.spawn(entity_id=mgr.allocate_id())
        finally:
            config.MAX_ENTITIES = original

    def test_spawn_default_position(self):
        """Spawn with no position should use DEFAULT_SPAWN_POSITION."""
        mgr = EntityManager()
        entity = mgr.spawn(entity_id=mgr.allocate_id())
        assert entity.pos_x == config.DEFAULT_SPAWN_POSITION[0]
        assert entity.pos_y == config.DEFAULT_SPAWN_POSITION[1]
        assert entity.pos_z == config.DEFAULT_SPAWN_POSITION[2]


class TestEntityUpdate:
    """Tests for entity position updates."""

    def test_update_position(self):
        """update_position should change the entity's coordinates."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        mgr.spawn(entity_id=eid, position=(0.0, 0.0, 0.0))
        result = mgr.update_position(eid, (50.0, 10.0, 75.0), tick=3)
        assert result is not None
        assert result.pos_x == 50.0
        assert result.pos_y == 10.0
        assert result.pos_z == 75.0
        assert result.last_update_tick == 3

    def test_update_nonexistent_returns_none(self):
        """Updating a nonexistent entity should return None."""
        mgr = EntityManager()
        assert mgr.update_position(9999, (0.0, 0.0, 0.0)) is None

    def test_update_destroying_entity_returns_none(self):
        """Updating a destroying entity should return None."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        mgr.spawn(entity_id=eid)
        mgr.mark_for_destroy(eid)
        assert mgr.update_position(eid, (1.0, 1.0, 1.0)) is None


class TestEntityDestroy:
    """Tests for entity destruction lifecycle."""

    def test_destroy_removes_entity(self):
        """destroy() should remove the entity from the registry."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        mgr.spawn(entity_id=eid)
        assert mgr.count == 1
        removed = mgr.destroy(eid)
        assert removed is not None
        assert removed.entity_id == eid
        assert mgr.count == 0

    def test_destroy_nonexistent_returns_none(self):
        """Destroying a nonexistent entity should return None."""
        mgr = EntityManager()
        assert mgr.destroy(9999) is None

    def test_mark_for_destroy_sets_state(self):
        """mark_for_destroy should set state to DESTROYING."""
        mgr = EntityManager()
        eid = mgr.allocate_id()
        mgr.spawn(entity_id=eid)
        assert mgr.mark_for_destroy(eid) is True
        entity = mgr.get(eid)
        assert entity.state == EntityState.DESTROYING

    def test_flush_destroy_queue(self):
        """flush_destroy_queue should remove all marked entities."""
        mgr = EntityManager()
        ids = [mgr.allocate_id() for _ in range(5)]
        for eid in ids:
            mgr.spawn(entity_id=eid)

        mgr.mark_for_destroy(ids[1])
        mgr.mark_for_destroy(ids[3])
        destroyed = mgr.flush_destroy_queue()

        assert len(destroyed) == 2
        assert mgr.count == 3
        assert mgr.get(ids[1]) is None
        assert mgr.get(ids[3]) is None
        assert mgr.get(ids[0]) is not None


class TestEntitySnapshot:
    """Tests for snapshot generation."""

    def test_snapshot_positions(self):
        """snapshot_positions should return position data for all active entities."""
        mgr = EntityManager()
        eid1 = mgr.allocate_id()
        eid2 = mgr.allocate_id()
        mgr.spawn(entity_id=eid1, position=(10.0, 0.0, 20.0))
        mgr.spawn(entity_id=eid2, position=(30.0, 0.0, 40.0))

        snap = mgr.snapshot_positions()
        assert len(snap) == 2
        ids = {s["entity_id"] for s in snap}
        assert eid1 in ids
        assert eid2 in ids

    def test_snapshot_excludes_destroying(self):
        """Entities marked for destruction should not appear in snapshots."""
        mgr = EntityManager()
        eid1 = mgr.allocate_id()
        eid2 = mgr.allocate_id()
        mgr.spawn(entity_id=eid1)
        mgr.spawn(entity_id=eid2)
        mgr.mark_for_destroy(eid1)

        snap = mgr.snapshot_positions()
        assert len(snap) == 1
        assert snap[0]["entity_id"] == eid2

    def test_get_all_returns_active_only(self):
        """get_all should return only ACTIVE entities."""
        mgr = EntityManager()
        eid1 = mgr.allocate_id()
        eid2 = mgr.allocate_id()
        mgr.spawn(entity_id=eid1)
        mgr.spawn(entity_id=eid2)
        mgr.mark_for_destroy(eid2)

        active = mgr.get_all()
        assert len(active) == 1
        assert active[0].entity_id == eid1


class TestEntityPositionProperty:
    """Tests for the Entity.position property."""

    def test_position_getter(self):
        """position property should return [x, y, z]."""
        e = Entity(entity_id=1, pos_x=10.0, pos_y=20.0, pos_z=30.0)
        assert e.position == [10.0, 20.0, 30.0]

    def test_position_setter(self):
        """position setter should update pos_x, pos_y, pos_z."""
        e = Entity(entity_id=1)
        e.position = (5.0, 15.0, 25.0)
        assert e.pos_x == 5.0
        assert e.pos_y == 15.0
        assert e.pos_z == 25.0
