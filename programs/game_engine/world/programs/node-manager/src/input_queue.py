"""
Input queue for the NEXUS node-manager.

Buffers incoming player commands (PLAYER_ACTION messages) from the WebSocket
receive threads and drains them into the tick loop each tick. One queue per
connected client, aggregated by the InputQueueManager.

The queue is thread-safe via asyncio.Queue — WebSocket handlers put() from
their coroutine context, and the tick loop drains with get_nowait() during
Phase A.

References:
  - MANIFEST.md §TICK LOOP Phase A: drain action_queue
  - CONTEXT.md Step 2: thread-safe queue, one per client
  - player_action.fbs: input message schema
"""

from __future__ import annotations

import asyncio
import logging
from dataclasses import dataclass, field

from stubs.simulation_stub import ChangeRequest, CHANGE_TYPE_MOVE
from codec import PlayerAction, ACTION_MOVE

logger = logging.getLogger("input_queue")


@dataclass
class ClientInputQueue:
    """Per-client input queue. One instance per connected client."""
    entity_id: int
    queue: asyncio.Queue = field(default_factory=asyncio.Queue)
    total_enqueued: int = 0
    total_processed: int = 0

    async def put(self, action: PlayerAction) -> None:
        """Enqueue a player action. Called from WebSocket receive handler."""
        await self.queue.put(action)
        self.total_enqueued += 1

    def drain(self) -> list[PlayerAction]:
        """
        Drain all pending actions from this queue (non-blocking).
        Called once per tick during Phase A.
        """
        actions: list[PlayerAction] = []
        while not self.queue.empty():
            try:
                action = self.queue.get_nowait()
                actions.append(action)
                self.total_processed += 1
            except asyncio.QueueEmpty:
                break
        return actions

    @property
    def depth(self) -> int:
        """Current queue depth (items waiting to be processed)."""
        return self.queue.qsize()


class InputQueueManager:
    """
    Aggregates per-client input queues and converts raw PlayerAction
    messages into simulation ChangeRequests during the tick drain.
    """

    def __init__(self) -> None:
        self._queues: dict[int, ClientInputQueue] = {}

    def register_client(self, entity_id: int) -> ClientInputQueue:
        """Create a new input queue for a client. Returns the queue handle."""
        q = ClientInputQueue(entity_id=entity_id)
        self._queues[entity_id] = q
        logger.debug("Registered input queue for entity %d", entity_id)
        return q

    def unregister_client(self, entity_id: int) -> None:
        """Remove a client's input queue on disconnect."""
        self._queues.pop(entity_id, None)
        logger.debug("Unregistered input queue for entity %d", entity_id)

    def get_queue(self, entity_id: int) -> ClientInputQueue | None:
        """Get a client's input queue by entity ID."""
        return self._queues.get(entity_id)

    def drain_all(self) -> list[ChangeRequest]:
        """
        Drain all client queues and convert actions to ChangeRequests.

        Called once per tick during Phase A. Returns a flat list of
        ChangeRequests ready for the simulation.
        """
        inputs: list[ChangeRequest] = []
        for entity_id, client_queue in self._queues.items():
            for action in client_queue.drain():
                change = self._action_to_change_request(entity_id, action)
                if change is not None:
                    inputs.append(change)
        return inputs

    @property
    def total_depth(self) -> int:
        """Total items across all client queues."""
        return sum(q.depth for q in self._queues.values())

    @property
    def client_count(self) -> int:
        """Number of registered client queues."""
        return len(self._queues)

    # ------------------------------------------------------------------
    # Internal
    # ------------------------------------------------------------------

    @staticmethod
    def _action_to_change_request(
        entity_id: int, action: PlayerAction
    ) -> ChangeRequest | None:
        """
        Convert a PlayerAction into a simulation ChangeRequest.

        Phase 0 only supports ACTION_MOVE; other action types are logged
        and dropped.
        """
        if action.action_type == ACTION_MOVE:
            return ChangeRequest(
                source=entity_id,
                type=CHANGE_TYPE_MOVE,
                object_id=entity_id,
                sequence_number=action.sequence_number,
                requires_ack=action.requires_ack,
                payload=action.payload,
            )

        logger.debug(
            "Unsupported action type 0x%04x from entity %d — dropped",
            action.action_type, entity_id,
        )
        return None
