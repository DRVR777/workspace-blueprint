"""nexus-agent — AI agent that lives inside the NEXUS world.

Lifecycle:
  1. Connect to NEXUS WebSocket
  2. Send HANDSHAKE → receive entity_id
  3. Send ENTER → receive SpatialManifest (schema_id=2)
  4. Call LLM with the manifest surface vocabulary
  5. Send AgentTask (schema_id=3) — server broadcasts it to all clients
  6. Wait TASK_COOLDOWN_S, then repeat from step 3 if the world changes
  7. On disconnect, wait RECONNECT_DELAY_S and reconnect

Run:
    python -m nexus_agent
"""
from __future__ import annotations

import asyncio
import logging
import time
import signal
import sys

import websockets

from . import config
from .codec import (
    decode_frame,
    decode_handshake_response,
    decode_spatial_manifest,
    encode_handshake,
    encode_enter,
    encode_agent_task,
    MSG_HANDSHAKE_RESPONSE,
    MSG_SPATIAL_MANIFEST,
    MSG_AGENT_BROADCAST,
)
from .manifest_handler import manifest_to_task
from oracle_shared.providers import get_llm

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [nexus-agent] %(levelname)s %(message)s",
)
logger = logging.getLogger(__name__)

_task_counter = 0
_shutdown = asyncio.Event()


def _next_task_id() -> int:
    global _task_counter
    _task_counter += 1
    return _task_counter


async def _run_session(llm) -> None:
    """Single WebSocket session: connect → manifest → task → repeat."""
    logger.info("Connecting to %s", config.NEXUS_WS_URL)
    async with websockets.connect(config.NEXUS_WS_URL) as ws:
        logger.info("Connected")

        # 1. HANDSHAKE
        await ws.send(encode_handshake(player_id=0))

        entity_id: int = 0
        last_task_at: float = 0.0
        current_world: str = ""

        async for raw in ws:
            if _shutdown.is_set():
                break

            frame = decode_frame(raw)
            if frame is None:
                continue

            # ── HANDSHAKE_RESPONSE ────────────────────────────────────────────
            if frame.msg_type == MSG_HANDSHAKE_RESPONSE:
                eid = decode_handshake_response(frame.payload)
                if eid is not None:
                    entity_id = eid
                    logger.info("Assigned entity_id=%d", entity_id)
                # Request the manifest immediately
                await ws.send(encode_enter(config.NEXUS_WORLD_ID))
                logger.info("Sent ENTER world_id=%r", config.NEXUS_WORLD_ID)

            # ── SPATIAL_MANIFEST ──────────────────────────────────────────────
            elif frame.msg_type == MSG_SPATIAL_MANIFEST:
                manifest = decode_spatial_manifest(frame.payload)
                if manifest is None:
                    logger.warning("Failed to decode SpatialManifest")
                    continue

                logger.info(
                    "SpatialManifest received: world=%s surface=%s",
                    manifest.world_id, manifest.surface,
                )

                now = time.monotonic()
                world_changed = manifest.world_id != current_world
                cooled_down   = (now - last_task_at) >= config.TASK_COOLDOWN_S

                if not (world_changed or cooled_down):
                    logger.debug("Skipping task — cooldown active")
                    continue

                # 4. Ask the LLM what to do
                logger.info("Consulting LLM for action...")
                decision = await manifest_to_task(manifest, llm)
                intent = decision.get("intent", "Explore this world")
                action = decision.get("action", manifest.surface[0] if manifest.surface else "observe")

                logger.info("Decision — action=%r intent=%r", action, intent)

                # 5. Send AgentTask to NEXUS server
                task_id = _next_task_id()
                packet  = encode_agent_task(
                    task_id=task_id,
                    origin_id=entity_id,
                    intent=intent,
                    action=action,
                )
                await ws.send(packet)
                logger.info("Sent AgentTask task_id=%d", task_id)

                last_task_at  = now
                current_world = manifest.world_id

            # ── AGENT_BROADCAST ───────────────────────────────────────────────
            # The server echoes our task back to all clients (including us).
            # Log it so operators can see the full round-trip.
            elif frame.msg_type == MSG_AGENT_BROADCAST:
                logger.info("AgentBroadcast received (round-trip confirmed, %dB)", len(frame.payload))

            # All other messages (physics ticks, player joined/left) are ignored.
            # The agent is not a player — it doesn't move.


async def _main() -> None:
    llm = get_llm(config.LLM_PROVIDER or None)
    logger.info("LLM provider: %s", type(llm).__name__)

    while not _shutdown.is_set():
        try:
            await _run_session(llm)
        except (websockets.ConnectionClosed, OSError, ConnectionRefusedError) as exc:
            logger.warning("Connection lost: %s — reconnecting in %.0fs", exc, config.RECONNECT_DELAY_S)
        except Exception as exc:
            logger.error("Unexpected error: %s", exc, exc_info=True)

        if _shutdown.is_set():
            break
        await asyncio.sleep(config.RECONNECT_DELAY_S)

    logger.info("Shut down cleanly")


def _handle_signal(sig, _frame):
    logger.info("Received signal %s — shutting down", sig)
    _shutdown.set()


if __name__ == "__main__":
    signal.signal(signal.SIGINT,  _handle_signal)
    signal.signal(signal.SIGTERM, _handle_signal)
    try:
        asyncio.run(_main())
    except KeyboardInterrupt:
        pass
    sys.exit(0)
