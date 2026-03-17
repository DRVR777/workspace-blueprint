"""
Phase 0 integration test — 2-client scenario.

Test plan (CONTEXT.md Step 6):
  1. Start node
  2. Connect clients A and B
  3. Verify both receive PLAYER_JOINED for each other
  4. Move client A → verify B receives EPU with A's new position
  5. Disconnect A → verify B receives PLAYER_LEFT with A's entity_id
  6. Verify all tick durations are < 20ms during the test

Run from src/ directory:
  pip install -r ../requirements.txt
  pytest
"""

import asyncio
import os
import random
import sys
import time

import pytest
import websockets

# Make src/ importable when pytest runs from src/
sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

import codec
from node_manager import NodeManager
from stubs.session_stub    import SessionStub
from stubs.ticker_log_stub import TickerLogStub

TEST_HOST = "localhost"
TEST_PORT = 9901   # distinct from default 9000 to avoid conflicts

# Stable 32-byte auth tokens — same token always produces the same player_id
TOKEN_A = b"test_player_A_token_000000000000"
TOKEN_B = b"test_player_B_token_000000000000"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

async def connect_and_handshake(token: bytes) -> tuple:
    """
    Open WebSocket, send HANDSHAKE, receive HANDSHAKE_RESPONSE.
    Returns (websocket, HandshakeResponse).
    """
    ws = await websockets.connect(
        f"ws://{TEST_HOST}:{TEST_PORT}", ping_interval=None
    )
    hs = codec.Handshake(client_version=1, player_id=0,
                          auth_token=token, gpu_caps=0)
    await ws.send(codec.encode_handshake(hs))
    raw = await asyncio.wait_for(ws.recv(), timeout=5.0)
    resp = codec.decode_handshake_response(raw)
    return ws, resp


async def recv_type(ws, target_type: int, timeout: float = 3.0) -> bytes:
    """
    Drain messages until one matching target_type arrives.
    Skips all other message types (EPU, TICK_SYNC, etc.).
    """
    deadline = time.perf_counter() + timeout
    while time.perf_counter() < deadline:
        remaining = max(0.01, deadline - time.perf_counter())
        try:
            raw = await asyncio.wait_for(ws.recv(), timeout=remaining)
        except asyncio.TimeoutError:
            break
        if isinstance(raw, bytes) and len(raw) >= codec.FRAME_SIZE:
            if codec.peek_msg_type(raw) == target_type:
                return raw
    raise AssertionError(
        f"Did not receive message type 0x{target_type:04x} within {timeout}s"
    )


# ---------------------------------------------------------------------------
# Fixture: per-test NodeManager on TEST_PORT
# ---------------------------------------------------------------------------

@pytest.fixture
async def node():
    session = SessionStub()
    log     = TickerLogStub(log_path="output/test_ticker.jsonl")
    nm      = NodeManager(host=TEST_HOST, port=TEST_PORT,
                          session=session, ticker_log=log)
    task    = asyncio.create_task(nm.run())
    await asyncio.sleep(0.15)   # let websockets.serve() finish binding
    yield nm
    nm._running = False
    task.cancel()
    try:
        await asyncio.wait_for(task, timeout=1.0)
    except (asyncio.CancelledError, asyncio.TimeoutError):
        pass


# ---------------------------------------------------------------------------
# Test 1 — both clients get ACCEPTED handshake with distinct entity IDs
# ---------------------------------------------------------------------------

async def test_handshake_accepted(node):
    ws_a, resp_a = await connect_and_handshake(TOKEN_A)
    ws_b, resp_b = await connect_and_handshake(TOKEN_B)
    try:
        assert resp_a.status == codec.HS_ACCEPTED, \
            f"Client A not ACCEPTED (status={resp_a.status})"
        assert resp_b.status == codec.HS_ACCEPTED, \
            f"Client B not ACCEPTED (status={resp_b.status})"
        assert resp_a.entity_id != 0, "A got entity_id=0"
        assert resp_b.entity_id != 0, "B got entity_id=0"
        assert resp_a.entity_id != resp_b.entity_id, \
            "A and B share the same entity_id"
    finally:
        await ws_a.close()
        await ws_b.close()


# ---------------------------------------------------------------------------
# Test 2 — PLAYER_JOINED broadcast in both directions
# ---------------------------------------------------------------------------

async def test_player_joined_broadcast(node):
    """
    A connects first. B connects second.
    B should receive PLAYER_JOINED for A (existing player list sent at handshake).
    A should receive PLAYER_JOINED for B (broadcast from new connection).
    """
    ws_a, resp_a = await connect_and_handshake(TOKEN_A)
    await asyncio.sleep(0.05)   # ensure A is fully registered before B
    ws_b, resp_b = await connect_and_handshake(TOKEN_B)

    try:
        # B receives existing players: expects A's entity_id
        raw_b = await recv_type(ws_b, codec.MSG_PLAYER_JOINED)
        msg_b = codec.decode_player_joined(raw_b)
        assert msg_b.entity_id == resp_a.entity_id, \
            (f"B's PLAYER_JOINED should be for A "
             f"(expected eid={resp_a.entity_id}, got {msg_b.entity_id})")

        # A receives broadcast of new arrival: expects B's entity_id
        raw_a = await recv_type(ws_a, codec.MSG_PLAYER_JOINED)
        msg_a = codec.decode_player_joined(raw_a)
        assert msg_a.entity_id == resp_b.entity_id, \
            (f"A's PLAYER_JOINED should be for B "
             f"(expected eid={resp_b.entity_id}, got {msg_a.entity_id})")
    finally:
        await ws_a.close()
        await ws_b.close()


# ---------------------------------------------------------------------------
# Test 3 — MOVE action results in EPU with updated position at B
# ---------------------------------------------------------------------------

async def test_entity_position_update_after_move(node):
    """
    A sends MOVE to (100, 0, 200).
    B must receive an ENTITY_POSITION_UPDATE that contains A's entity_id
    at position (100, 0, 200) within the next few ticks.
    """
    ws_a, resp_a = await connect_and_handshake(TOKEN_A)
    await asyncio.sleep(0.05)
    ws_b, resp_b = await connect_and_handshake(TOKEN_B)
    await asyncio.sleep(0.1)   # drain initial PLAYER_JOINED / TICK_SYNC messages

    target_x, target_y, target_z = 100.0, 0.0, 200.0
    move = codec.PlayerAction(
        action_type=codec.ACTION_MOVE,
        sequence_number=1,
        requires_ack=False,
        payload=codec.encode_move_payload(target_x, target_y, target_z),
    )

    try:
        await ws_a.send(codec.encode_player_action(move))

        # Scan EPUs arriving at B for A's entity at the new position
        deadline = time.perf_counter() + 3.0
        found = False
        while time.perf_counter() < deadline and not found:
            remaining = max(0.01, deadline - time.perf_counter())
            try:
                raw = await asyncio.wait_for(ws_b.recv(), timeout=remaining)
            except asyncio.TimeoutError:
                break
            if (isinstance(raw, bytes)
                    and len(raw) >= codec.FRAME_SIZE
                    and codec.peek_msg_type(raw) == codec.MSG_ENTITY_POSITION_UPDATE):
                for e in codec.decode_entity_position_update(raw):
                    if e.entity_id == resp_a.entity_id:
                        if (abs(e.pos_x - target_x) < 0.01
                                and abs(e.pos_z - target_z) < 0.01):
                            found = True
                            break

        assert found, (
            f"B never saw EPU with A (entity_id={resp_a.entity_id}) "
            f"at ({target_x}, {target_y}, {target_z})"
        )
    finally:
        await ws_a.close()
        await ws_b.close()


# ---------------------------------------------------------------------------
# Test 4 — PLAYER_LEFT broadcast when A disconnects
# ---------------------------------------------------------------------------

async def test_player_left_on_disconnect(node):
    ws_a, resp_a = await connect_and_handshake(TOKEN_A)
    await asyncio.sleep(0.05)
    ws_b, resp_b = await connect_and_handshake(TOKEN_B)
    await asyncio.sleep(0.1)   # let both clients settle

    try:
        await ws_a.close()

        raw = await recv_type(ws_b, codec.MSG_PLAYER_LEFT, timeout=3.0)
        _, eid, reason = codec.decode_player_left(raw)

        assert eid == resp_a.entity_id, \
            (f"PLAYER_LEFT entity_id mismatch: "
             f"expected {resp_a.entity_id}, got {eid}")
        assert reason == codec.PL_DISCONNECT, \
            f"Expected PL_DISCONNECT (0), got {reason}"
    finally:
        await ws_b.close()


# ---------------------------------------------------------------------------
# Test 5 — tick duration stays under 20ms with 2 active clients
# ---------------------------------------------------------------------------

async def test_tick_duration_under_budget(node):
    """
    Two clients send MOVE actions for ~0.5s at ~50 Hz.
    All recorded tick durations must be < 20ms.
    """
    ws_a, _ = await connect_and_handshake(TOKEN_A)
    ws_b, _ = await connect_and_handshake(TOKEN_B)
    await asyncio.sleep(0.1)

    seq = 0

    async def send_moves(ws: object) -> None:
        nonlocal seq
        for _ in range(25):
            seq += 1
            x = random.uniform(0.0, 1000.0)
            z = random.uniform(0.0, 1000.0)
            move = codec.PlayerAction(
                action_type=codec.ACTION_MOVE,
                sequence_number=seq,
                requires_ack=False,
                payload=codec.encode_move_payload(x, 0.0, z),
            )
            await ws.send(codec.encode_player_action(move))
            await asyncio.sleep(TARGET_TICK_DURATION)

    try:
        await asyncio.gather(send_moves(ws_a), send_moves(ws_b))
        await asyncio.sleep(0.1)   # let final ticks flush

        stats = node.get_tick_stats()
        assert stats["tick_count"] > 0, "No ticks recorded"
        assert stats["over_budget"] == 0, (
            f"{stats['over_budget']} ticks exceeded 20ms "
            f"(max={stats['max_ms']:.2f}ms avg={stats['avg_ms']:.2f}ms)"
        )
        print(f"\n  Tick stats: count={stats['tick_count']} "
              f"avg={stats['avg_ms']:.2f}ms max={stats['max_ms']:.2f}ms")
    finally:
        await ws_a.close()
        await ws_b.close()


# Make TARGET_TICK_DURATION accessible inside the test function
TARGET_TICK_DURATION = 0.020
