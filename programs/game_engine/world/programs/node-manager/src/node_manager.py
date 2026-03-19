"""
NEXUS Node Manager — Phase 0 implementation.

Owns:
  - Node lifecycle (starting -> active -> draining -> stopped)
  - Simulation tick loop at TARGET_TICK_DURATION (50 Hz / 20ms)
  - WebSocket server: client accept, HANDSHAKE, action receive
  - State broadcasting: ENTITY_POSITION_UPDATE each tick to all clients
  - PLAYER_JOINED / PLAYER_LEFT broadcast on connect / disconnect
  - World graph write-back stub: ticker log flush each tick
  - Self-monitoring: log warning if tick > 20ms; Phase 0 does NOT request split

Does NOT own:
  - Physics math (simulation/)
  - Spatial index internals (spatial/)
  - World graph database
  - Client-side rendering (engine/)

Component delegation:
  - config.py: all tunable constants
  - entity_manager.py: entity spawn/update/destroy lifecycle
  - input_queue.py: per-client action queue aggregation
  - state_serializer.py: world state -> wire format conversion
  - tick_metrics.py: tick performance logging
"""

import asyncio
import logging
import time
from dataclasses import dataclass, field
from typing import Optional

import websockets
import websockets.exceptions

import config
import codec
from codec import (
    FRAME_SIZE, peek_msg_type,
    MSG_HANDSHAKE, MSG_PLAYER_ACTION,
    HS_ACCEPTED, HS_REJECTED,
    HandshakeResponse,
    decode_handshake, encode_handshake_response,
    decode_player_action,
)
from entity_manager import EntityManager, Entity
from input_queue import InputQueueManager, ClientInputQueue
from state_serializer import StateSerializer
from tick_metrics import TickMetrics
from stubs.spatial_stub import SpatialStub
from stubs.simulation_stub import (
    SimulationStub, WorldSnapshot, SimpleEntity,
    EVT_POSITION_CHANGED,
)
from stubs.session_stub import SessionStub
from stubs.node_registry_stub import NodeRegistryStub
from stubs.ticker_log_stub import TickerLogStub, TickerEntry, SRC_PLAYER

logger = logging.getLogger("node_manager")


@dataclass
class ConnectedClient:
    """Represents a WebSocket-connected client and their associated entity."""
    websocket: object                   # websockets.WebSocketServerProtocol
    entity_id: int
    player_id: int
    display_name: str
    input_queue: ClientInputQueue
    visibility_radius: float = config.DEFAULT_VISIBILITY_RADIUS


class NodeManager:
    """
    The top-level server-side game process for a single node.

    Manages the tick loop, WebSocket connections, entity state, and
    coordinates between simulation, spatial index, session management,
    and the ticker log.

    All configuration comes from config.py — no magic numbers here.
    """

    def __init__(
        self,
        host: str = config.DEFAULT_HOST,
        port: int = config.DEFAULT_PORT,
        spatial: Optional[SpatialStub] = None,
        simulation: Optional[SimulationStub] = None,
        session: Optional[SessionStub] = None,
        node_registry: Optional[NodeRegistryStub] = None,
        ticker_log: Optional[TickerLogStub] = None,
        tick_metrics: Optional[TickMetrics] = None,
    ) -> None:
        self.host = host
        self.port = port

        # Injected services — stubs for Phase 0, real impls for Phase 1+
        self._spatial = spatial or SpatialStub()
        self._simulation = simulation or SimulationStub()
        self._session = session or SessionStub()
        self._node_registry = node_registry or NodeRegistryStub()
        self._ticker_log = ticker_log or TickerLogStub()

        # Core subsystems
        self._entities = EntityManager()
        self._input_queues = InputQueueManager()
        self._serializer = StateSerializer()
        self._metrics = tick_metrics or TickMetrics()

        # Client tracking (entity_id -> ConnectedClient)
        self._clients: dict[int, ConnectedClient] = {}

        # Tick state
        self._tick_number = 0
        self._running = False

        # Domain from registry
        self._domain_min, self._domain_max = self._node_registry.get_domain()

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def run(self) -> None:
        """Start WebSocket server and tick loop. Runs until stop() is called."""
        self._running = True
        logger.info(
            "Node starting — domain %s -> %s  port %d",
            self._domain_min, self._domain_max, self.port,
        )

        self._node_registry.register_node(
            self._node_registry.NODE_ID,
            (self._domain_min, self._domain_max),
            f"{self.host}:{self.port}",
        )

        server = await websockets.serve(
            self._handle_client, self.host, self.port,
            ping_interval=None,
        )
        logger.info(
            "WebSocket server listening on ws://%s:%d", self.host, self.port
        )

        tick_task = asyncio.create_task(self._tick_loop(), name="tick_loop")
        try:
            await asyncio.gather(server.wait_closed(), tick_task)
        except asyncio.CancelledError:
            pass
        finally:
            self._running = False
            self._metrics.flush()
            server.close()
            await server.wait_closed()

    async def stop(self) -> None:
        """Signal the tick loop and server to shut down."""
        self._running = False

    # ------------------------------------------------------------------
    # Tick loop — MANIFEST.md §TICK LOOP
    # ------------------------------------------------------------------

    async def _tick_loop(self) -> None:
        logger.info(
            "Tick loop started — target %.0fms (%.0f Hz)",
            config.TARGET_TICK_DURATION * 1000,
            config.TICK_RATE,
        )
        last_tick_time = time.perf_counter()

        while self._running:
            tick_start = time.perf_counter()
            dt = min(tick_start - last_tick_time, config.MAX_TICK_DT)
            last_tick_time = tick_start

            changes_count = await self._run_tick(dt)

            elapsed = time.perf_counter() - tick_start
            tick_ms = elapsed * 1000.0

            # Phase F — record metrics (handles warnings internally)
            self._metrics.record(
                tick_number=self._tick_number,
                duration_ms=tick_ms,
                entity_count=self._entities.count,
                client_count=len(self._clients),
                input_queue_depth=self._input_queues.total_depth,
                state_changes_count=changes_count,
            )

            self._tick_number += 1

            # Phase G — sleep until next tick
            sleep = max(0.0, config.TARGET_TICK_DURATION - elapsed)
            await asyncio.sleep(sleep)

        logger.info("Tick loop stopped at tick %d", self._tick_number)

    async def _run_tick(self, dt: float) -> int:
        """
        Execute one simulation tick. Returns the number of state changes.

        Phase A: Drain action queues
        Phase B: Run simulation
        Phase C: Apply results
        Phase D: Broadcast EPU
        Phase E: Flush ticker log
        """
        # Phase A — Drain action queues (via InputQueueManager)
        inputs = self._input_queues.drain_all()

        # Phase B — Run simulation (pure function: same inputs -> same outputs)
        snapshot = self._build_snapshot()
        tick_result = self._simulation.run_tick(snapshot, inputs, dt)

        # Phase C — Apply results to entity manager + spatial index
        for change in tick_result.state_changes:
            if change.change_type == EVT_POSITION_CHANGED:
                self._entities.update_position(
                    change.object_id, change.new_pos, self._tick_number
                )
                self._spatial.move(change.object_id, change.new_pos)

        # Phase D — Broadcast ENTITY_POSITION_UPDATE to all connected clients
        if self._clients:
            all_entities = self._entities.get_all()
            epu_frame = self._serializer.encode_position_update(
                all_entities, self._tick_number
            )
            dead: list[int] = []
            for client in list(self._clients.values()):
                try:
                    await client.websocket.send(epu_frame)
                except Exception:
                    dead.append(client.entity_id)
            for eid in dead:
                await self._on_disconnect(eid)

        # Phase E — Flush ticker log (stub: local file write)
        if tick_result.state_changes:
            entries = [
                TickerEntry(
                    object_id=ch.object_id,
                    event_type=EVT_POSITION_CHANGED,
                    source_type=SRC_PLAYER,
                    source_id=ch.object_id,
                    payload={"old": list(ch.old_pos), "new": list(ch.new_pos)},
                )
                for ch in tick_result.state_changes
            ]
            self._ticker_log.append_batch(entries)

        return len(tick_result.state_changes)

    def _build_snapshot(self) -> WorldSnapshot:
        """Build a WorldSnapshot from the entity manager for the simulation."""
        return WorldSnapshot(
            tick_number=self._tick_number,
            entities=[
                SimpleEntity(
                    entity_id=e.entity_id,
                    pos_x=e.pos_x,
                    pos_y=e.pos_y,
                    pos_z=e.pos_z,
                )
                for e in self._entities.get_all()
            ],
        )

    # ------------------------------------------------------------------
    # Client connection handling
    # ------------------------------------------------------------------

    async def _handle_client(self, websocket) -> None:
        """WebSocket connection handler — handshake then receive loop."""
        entity_id: Optional[int] = None
        try:
            entity_id = await self._do_handshake(websocket)
            if entity_id is not None:
                await self._recv_loop(websocket, entity_id)
        except websockets.exceptions.ConnectionClosed:
            pass
        except Exception as exc:
            logger.error(
                "Client error (entity=%s): %s", entity_id, exc, exc_info=True
            )
        finally:
            if entity_id is not None:
                await self._on_disconnect(entity_id)

    async def _do_handshake(self, websocket) -> Optional[int]:
        """
        Read HANDSHAKE -> validate session -> send HANDSHAKE_RESPONSE.
        Returns entity_id on success, None on rejection.
        """
        # Check capacity
        if len(self._clients) >= config.MAX_CLIENTS:
            logger.warning("Max clients (%d) reached — rejecting", config.MAX_CLIENTS)
            await websocket.send(encode_handshake_response(
                HandshakeResponse(status=HS_REJECTED)
            ))
            await websocket.close()
            return None

        try:
            raw = await asyncio.wait_for(
                websocket.recv(), timeout=config.HANDSHAKE_TIMEOUT_S
            )
        except asyncio.TimeoutError:
            logger.warning("Handshake timeout from %s", websocket.remote_address)
            return None

        try:
            hs = decode_handshake(raw)
        except Exception as exc:
            logger.warning("Malformed HANDSHAKE: %s", exc)
            return None

        session = self._session.validate_token(hs.auth_token)
        if session is None:
            await websocket.send(encode_handshake_response(
                HandshakeResponse(status=HS_REJECTED)
            ))
            await websocket.close()
            return None

        # Spawn entity via EntityManager
        entity_id = self._entities.allocate_id()
        spawn = session.last_position
        entity = self._entities.spawn(
            entity_id=entity_id,
            entity_type="player",
            owner_player_id=session.player_id,
            display_name=session.display_name,
            position=spawn,
            spawn_tick=self._tick_number,
        )

        # Send HANDSHAKE_RESPONSE
        await websocket.send(encode_handshake_response(HandshakeResponse(
            status=HS_ACCEPTED,
            entity_id=entity_id,
            pos_x=spawn[0], pos_y=spawn[1], pos_z=spawn[2],
        )))

        # Register input queue
        input_q = self._input_queues.register_client(entity_id)

        # Register client
        client = ConnectedClient(
            websocket=websocket,
            entity_id=entity_id,
            player_id=session.player_id,
            display_name=session.display_name,
            input_queue=input_q,
        )
        self._clients[entity_id] = client
        self._spatial.insert(entity_id, tuple(spawn), config.ENTITY_BOUNDING_BOX)

        # Send TICK_SYNC so client can calibrate its local clock
        await websocket.send(
            self._serializer.encode_clock_sync(self._tick_number)
        )

        # Tell new client about every player already in the node
        for other in list(self._clients.values()):
            if other.entity_id == entity_id:
                continue
            other_entity = self._entities.get(other.entity_id)
            if other_entity is None:
                continue
            await websocket.send(self._serializer.encode_player_join(
                entity_id=other.entity_id,
                player_id=other.player_id,
                display_name=other.display_name,
                position=(other_entity.pos_x, other_entity.pos_y, other_entity.pos_z),
            ))

        # Tell every existing player about the new arrival
        new_pj_frame = self._serializer.encode_player_join(
            entity_id=entity_id,
            player_id=session.player_id,
            display_name=session.display_name,
            position=spawn,
        )
        for other in list(self._clients.values()):
            if other.entity_id == entity_id:
                continue
            try:
                await other.websocket.send(new_pj_frame)
            except Exception:
                pass

        logger.info(
            "Connected: entity=%d player=%d (%s) at %s",
            entity_id, session.player_id, session.display_name, spawn,
        )
        return entity_id

    async def _recv_loop(self, websocket, entity_id: int) -> None:
        """Read PLAYER_ACTION messages and enqueue them for the tick loop."""
        client = self._clients.get(entity_id)
        if client is None:
            return
        async for raw in websocket:
            if not isinstance(raw, bytes) or len(raw) < FRAME_SIZE:
                continue
            try:
                if peek_msg_type(raw) == MSG_PLAYER_ACTION:
                    action = decode_player_action(raw)
                    await client.input_queue.put(action)
            except Exception as exc:
                logger.debug("Bad message from entity %d: %s", entity_id, exc)

    async def _on_disconnect(self, entity_id: int) -> None:
        """
        Remove client, persist last position, broadcast PLAYER_LEFT.
        Matches MANIFEST.md §DRAIN client handoff spec (Phase 0: simple drop).
        """
        client = self._clients.pop(entity_id, None)
        if client is None:
            return

        # Get final position from entity manager before destroying
        entity = self._entities.get(entity_id)
        final_pos = (entity.pos_x, entity.pos_y, entity.pos_z) if entity else (0, 0, 0)

        # Clean up entity and input queue
        self._entities.destroy(entity_id)
        self._input_queues.unregister_client(entity_id)
        self._spatial.remove(entity_id)

        # Persist last position in session layer
        self._session.update_last_position(client.player_id, final_pos)

        # Broadcast PLAYER_LEFT to all remaining clients
        left_frame = self._serializer.encode_player_leave(entity_id)
        for other in list(self._clients.values()):
            try:
                await other.websocket.send(left_frame)
            except Exception:
                pass

        logger.info(
            "Disconnected: entity=%d player=%d (%s)",
            entity_id, client.player_id, client.display_name,
        )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get_tick_stats(self) -> dict:
        """Return aggregate tick performance statistics."""
        return self._metrics.get_stats()
