"""
NEXUS Node Manager — Phase 0 implementation.

Owns:
  - Node lifecycle (starting → active → draining → stopped)
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
"""

import asyncio
import logging
import time
from dataclasses import dataclass, field
from typing import Optional

import websockets
import websockets.exceptions

import codec
from codec import (
    FRAME_SIZE, peek_msg_type,
    MSG_HANDSHAKE, MSG_PLAYER_ACTION,
    HS_ACCEPTED, HS_REJECTED,
    PL_DISCONNECT,
    ACTION_MOVE,
    HandshakeResponse, PlayerAction,
    PlayerJoinedMsg, EntityState,
    decode_handshake, encode_handshake_response,
    decode_player_action,
    encode_move_payload, encode_tick_sync,
    encode_entity_position_update,
    encode_player_joined, encode_player_left,
)
from stubs.spatial_stub      import SpatialStub
from stubs.simulation_stub   import (SimulationStub, WorldSnapshot,
                                      SimpleEntity, ChangeRequest,
                                      CHANGE_TYPE_MOVE, EVT_POSITION_CHANGED)
from stubs.session_stub      import SessionStub
from stubs.node_registry_stub import NodeRegistryStub
from stubs.ticker_log_stub   import TickerLogStub, TickerEntry, SRC_PLAYER

logger = logging.getLogger("node_manager")

TARGET_TICK_DURATION  = 0.020   # 50 Hz
MAX_TICK_DT           = 0.050   # cap dt at 50ms — MANIFEST.md §TICK LOOP
HIGH_LOAD_THRESHOLD   = 0.020   # 20ms — log warning but no split in Phase 0
DEFAULT_VISIBILITY_R  = 200.0   # units — Phase 0 broadcasts all-to-all


@dataclass
class ConnectedClient:
    websocket: object                   # websockets.WebSocketServerProtocol
    entity_id: int
    player_id: int
    display_name: str
    position: list[float]              # [x, y, z] — mutated each tick
    visibility_radius: float
    action_queue: asyncio.Queue = field(default_factory=asyncio.Queue)


class NodeManager:
    def __init__(
        self,
        host: str = "localhost",
        port: int = 9000,
        spatial:       Optional[SpatialStub]       = None,
        simulation:    Optional[SimulationStub]    = None,
        session:       Optional[SessionStub]       = None,
        node_registry: Optional[NodeRegistryStub]  = None,
        ticker_log:    Optional[TickerLogStub]     = None,
    ) -> None:
        self.host = host
        self.port = port

        # Injected services — stubs for Phase 0, real impls for Phase 1+
        self._spatial       = spatial       or SpatialStub()
        self._simulation    = simulation    or SimulationStub()
        self._session       = session       or SessionStub()
        self._node_registry = node_registry or NodeRegistryStub()
        self._ticker_log    = ticker_log    or TickerLogStub()

        self._clients: dict[int, ConnectedClient] = {}   # entity_id → client
        self._tick_number    = 0
        self._next_eid       = 1
        self._running        = False
        self._tick_durations: list[float] = []            # ms

        self._domain_min, self._domain_max = self._node_registry.get_domain()

    # -----------------------------------------------------------------------
    # Lifecycle
    # -----------------------------------------------------------------------

    async def run(self) -> None:
        """Start WebSocket server and tick loop. Runs until stop() is called."""
        self._running = True
        logger.info("Node starting — domain %s → %s  port %d",
                    self._domain_min, self._domain_max, self.port)

        self._node_registry.register_node(
            self._node_registry.NODE_ID,
            (self._domain_min, self._domain_max),
            f"{self.host}:{self.port}",
        )

        server = await websockets.serve(
            self._handle_client, self.host, self.port,
            ping_interval=None,
        )
        logger.info("WebSocket server listening on ws://%s:%d", self.host, self.port)

        tick_task = asyncio.create_task(self._tick_loop(), name="tick_loop")
        try:
            await asyncio.gather(server.wait_closed(), tick_task)
        except asyncio.CancelledError:
            pass
        finally:
            self._running = False
            server.close()
            await server.wait_closed()

    async def stop(self) -> None:
        self._running = False

    # -----------------------------------------------------------------------
    # Tick loop — MANIFEST.md §TICK LOOP
    # -----------------------------------------------------------------------

    async def _tick_loop(self) -> None:
        logger.info("Tick loop started — target %.0fms (%.0f Hz)",
                    TARGET_TICK_DURATION * 1000, 1.0 / TARGET_TICK_DURATION)
        last_tick_time = time.perf_counter()

        while self._running:
            tick_start = time.perf_counter()
            dt = min(tick_start - last_tick_time, MAX_TICK_DT)
            last_tick_time = tick_start

            await self._run_tick(dt)

            elapsed    = time.perf_counter() - tick_start
            tick_ms    = elapsed * 1000.0
            self._tick_durations.append(tick_ms)

            # Phase F — self-monitor (no split requested in Phase 0)
            if tick_ms > HIGH_LOAD_THRESHOLD * 1000:
                logger.warning("Tick %d over budget: %.2fms", self._tick_number, tick_ms)

            self._tick_number += 1

            # Phase G — sleep until next tick
            sleep = max(0.0, TARGET_TICK_DURATION - elapsed)
            await asyncio.sleep(sleep)

        logger.info("Tick loop stopped at tick %d", self._tick_number)

    async def _run_tick(self, dt: float) -> None:
        # Phase A — Drain action queues
        inputs: list[ChangeRequest] = []
        for client in list(self._clients.values()):
            while not client.action_queue.empty():
                try:
                    action: PlayerAction = client.action_queue.get_nowait()
                    if action.action_type == ACTION_MOVE:
                        inputs.append(ChangeRequest(
                            source=client.entity_id,
                            type=CHANGE_TYPE_MOVE,
                            object_id=client.entity_id,
                            sequence_number=action.sequence_number,
                            requires_ack=action.requires_ack,
                            payload=action.payload,
                        ))
                except asyncio.QueueEmpty:
                    break

        # Phase B — Run simulation (pure function: same inputs → same outputs)
        snapshot    = self._build_snapshot()
        tick_result = self._simulation.run_tick(snapshot, inputs, dt)

        # Phase C — Apply results to local snapshot; enqueue world graph writes
        for change in tick_result.state_changes:
            client = self._clients.get(change.object_id)
            if client and change.change_type == EVT_POSITION_CHANGED:
                client.position[0], client.position[1], client.position[2] = change.new_pos
                self._spatial.move(change.object_id, change.new_pos)

        # Phase D — Broadcast ENTITY_POSITION_UPDATE to all connected clients
        if self._clients:
            all_states = [
                EntityState(
                    entity_id=c.entity_id,
                    pos_x=c.position[0],
                    pos_y=c.position[1],
                    pos_z=c.position[2],
                )
                for c in self._clients.values()
            ]
            epu_frame = encode_entity_position_update(all_states,
                                                       seq=self._tick_number)
            dead: list[int] = []
            for client in list(self._clients.values()):
                try:
                    await client.websocket.send(epu_frame)
                except Exception:
                    dead.append(client.entity_id)
            for eid in dead:
                self._clients.pop(eid, None)

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

        # Phase F timing + Phase G sleep are handled by _tick_loop

    def _build_snapshot(self) -> WorldSnapshot:
        return WorldSnapshot(
            tick_number=self._tick_number,
            entities=[
                SimpleEntity(
                    entity_id=c.entity_id,
                    pos_x=c.position[0],
                    pos_y=c.position[1],
                    pos_z=c.position[2],
                )
                for c in self._clients.values()
            ],
        )

    # -----------------------------------------------------------------------
    # Client connection handling
    # -----------------------------------------------------------------------

    async def _handle_client(self, websocket) -> None:
        entity_id: Optional[int] = None
        try:
            entity_id = await self._do_handshake(websocket)
            if entity_id is not None:
                await self._recv_loop(websocket, entity_id)
        except websockets.exceptions.ConnectionClosed:
            pass
        except Exception as exc:
            logger.error("Client error (entity=%s): %s", entity_id, exc, exc_info=True)
        finally:
            if entity_id is not None:
                await self._on_disconnect(entity_id)

    async def _do_handshake(self, websocket) -> Optional[int]:
        """
        Read HANDSHAKE → validate session → send HANDSHAKE_RESPONSE.
        Returns entity_id on success, None on rejection.
        """
        try:
            raw = await asyncio.wait_for(websocket.recv(), timeout=10.0)
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

        entity_id = self._alloc_entity_id()
        spawn     = session.last_position

        await websocket.send(encode_handshake_response(HandshakeResponse(
            status=HS_ACCEPTED,
            entity_id=entity_id,
            pos_x=spawn[0], pos_y=spawn[1], pos_z=spawn[2],
        )))

        client = ConnectedClient(
            websocket=websocket,
            entity_id=entity_id,
            player_id=session.player_id,
            display_name=session.display_name,
            position=list(spawn),
            visibility_radius=DEFAULT_VISIBILITY_R,
        )
        self._clients[entity_id] = client
        self._spatial.insert(entity_id, tuple(spawn), (0.5, 1.0, 0.5))

        # Send TICK_SYNC so client can calibrate its local clock
        await websocket.send(encode_tick_sync(self._tick_number))

        # Tell new client about every player already in the node
        for other in list(self._clients.values()):
            if other.entity_id == entity_id:
                continue
            await websocket.send(encode_player_joined(PlayerJoinedMsg(
                entity_id=other.entity_id,
                player_id=other.player_id,
                display_name=other.display_name,
                pos_x=other.position[0],
                pos_y=other.position[1],
                pos_z=other.position[2],
            )))

        # Tell every existing player about the new arrival
        new_pj_frame = encode_player_joined(PlayerJoinedMsg(
            entity_id=entity_id,
            player_id=session.player_id,
            display_name=session.display_name,
            pos_x=spawn[0], pos_y=spawn[1], pos_z=spawn[2],
        ))
        for other in list(self._clients.values()):
            if other.entity_id == entity_id:
                continue
            try:
                await other.websocket.send(new_pj_frame)
            except Exception:
                pass

        logger.info("Connected: entity=%d player=%d (%s) at %s",
                    entity_id, session.player_id, session.display_name, spawn)
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
                    await client.action_queue.put(action)
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

        self._spatial.remove(entity_id)
        self._session.update_last_position(
            client.player_id, tuple(client.position)
        )

        left_frame = encode_player_left(entity_id, PL_DISCONNECT)
        for other in list(self._clients.values()):
            try:
                await other.websocket.send(left_frame)
            except Exception:
                pass

        logger.info("Disconnected: entity=%d player=%d (%s)",
                    entity_id, client.player_id, client.display_name)

    # -----------------------------------------------------------------------
    # Helpers
    # -----------------------------------------------------------------------

    def _alloc_entity_id(self) -> int:
        eid = self._next_eid
        self._next_eid += 1
        return eid

    def get_tick_stats(self) -> dict:
        if not self._tick_durations:
            return {"tick_count": 0, "avg_ms": 0.0, "max_ms": 0.0, "over_budget": 0}
        return {
            "tick_count":   len(self._tick_durations),
            "avg_ms":       sum(self._tick_durations) / len(self._tick_durations),
            "max_ms":       max(self._tick_durations),
            "over_budget":  sum(1 for d in self._tick_durations if d > 20.0),
        }
