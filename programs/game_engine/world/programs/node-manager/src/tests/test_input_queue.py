"""
Unit tests for the input_queue module.

Tests per-client queue operations, the drain-all aggregation, and
action-to-change-request conversion.
"""

import asyncio
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from codec import PlayerAction, ACTION_MOVE, ACTION_INTERACT, encode_move_payload
from input_queue import InputQueueManager, ClientInputQueue
from stubs.simulation_stub import CHANGE_TYPE_MOVE


class TestClientInputQueue:
    """Tests for a single client's input queue."""

    async def test_put_and_drain(self):
        """Items put into the queue should be returned by drain()."""
        q = ClientInputQueue(entity_id=1)
        action = PlayerAction(
            action_type=ACTION_MOVE,
            sequence_number=1,
            requires_ack=False,
            payload=encode_move_payload(10.0, 0.0, 20.0),
        )
        await q.put(action)
        await q.put(action)

        drained = q.drain()
        assert len(drained) == 2
        assert q.total_enqueued == 2
        assert q.total_processed == 2

    async def test_drain_empty_queue(self):
        """Draining an empty queue should return an empty list."""
        q = ClientInputQueue(entity_id=1)
        assert q.drain() == []

    async def test_depth_tracking(self):
        """depth property should reflect current queue size."""
        q = ClientInputQueue(entity_id=1)
        action = PlayerAction(
            action_type=ACTION_MOVE, sequence_number=1,
            requires_ack=False, payload=b"",
        )
        assert q.depth == 0
        await q.put(action)
        assert q.depth == 1
        q.drain()
        assert q.depth == 0


class TestInputQueueManager:
    """Tests for the InputQueueManager aggregator."""

    def test_register_and_unregister(self):
        """Registering and unregistering clients should work correctly."""
        mgr = InputQueueManager()
        q = mgr.register_client(entity_id=1)
        assert isinstance(q, ClientInputQueue)
        assert mgr.client_count == 1

        mgr.unregister_client(entity_id=1)
        assert mgr.client_count == 0

    def test_get_queue(self):
        """get_queue should return the registered queue or None."""
        mgr = InputQueueManager()
        mgr.register_client(entity_id=1)
        assert mgr.get_queue(1) is not None
        assert mgr.get_queue(999) is None

    async def test_drain_all_converts_to_change_requests(self):
        """drain_all should convert PlayerActions to ChangeRequests."""
        mgr = InputQueueManager()
        q = mgr.register_client(entity_id=42)

        action = PlayerAction(
            action_type=ACTION_MOVE,
            sequence_number=7,
            requires_ack=False,
            payload=encode_move_payload(100.0, 0.0, 200.0),
        )
        await q.put(action)

        inputs = mgr.drain_all()
        assert len(inputs) == 1
        cr = inputs[0]
        assert cr.source == 42
        assert cr.type == CHANGE_TYPE_MOVE
        assert cr.object_id == 42
        assert cr.sequence_number == 7

    async def test_drain_all_drops_unsupported_actions(self):
        """Non-MOVE actions should be dropped in Phase 0."""
        mgr = InputQueueManager()
        q = mgr.register_client(entity_id=1)

        await q.put(PlayerAction(
            action_type=ACTION_INTERACT, sequence_number=1,
            requires_ack=False, payload=b"",
        ))
        inputs = mgr.drain_all()
        assert len(inputs) == 0

    async def test_drain_all_multiple_clients(self):
        """drain_all should aggregate inputs from all client queues."""
        mgr = InputQueueManager()
        q1 = mgr.register_client(entity_id=1)
        q2 = mgr.register_client(entity_id=2)

        await q1.put(PlayerAction(
            action_type=ACTION_MOVE, sequence_number=1,
            requires_ack=False, payload=encode_move_payload(1, 0, 1),
        ))
        await q2.put(PlayerAction(
            action_type=ACTION_MOVE, sequence_number=2,
            requires_ack=False, payload=encode_move_payload(2, 0, 2),
        ))

        inputs = mgr.drain_all()
        assert len(inputs) == 2
        sources = {cr.source for cr in inputs}
        assert sources == {1, 2}

    def test_total_depth(self):
        """total_depth should sum all client queue depths."""
        mgr = InputQueueManager()
        mgr.register_client(entity_id=1)
        mgr.register_client(entity_id=2)
        assert mgr.total_depth == 0
