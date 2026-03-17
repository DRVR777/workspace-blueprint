"""
signal-ingestion entry point.
Usage:  uv run python -m signal_ingestion
        (or via Makefile: make signal-ingestion)

Reads env from .env in the oracle root (loaded by python-dotenv).
Requires Redis to be running: make up
"""
from __future__ import annotations

import asyncio
import logging
import os
import signal
import sys

from dotenv import load_dotenv

# Load .env from the oracle root (two levels up from programs/signal-ingestion/)
load_dotenv(dotenv_path=os.path.join(os.path.dirname(__file__), "..", "..", "..", ".env"))

import redis.asyncio as aioredis

from signal_ingestion.adapters.polymarket_rest import PolymarketRESTAdapter

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s | %(levelname)s | %(name)s | %(message)s",
    stream=sys.stdout,
)
logger = logging.getLogger("signal_ingestion")

REDIS_URL = os.getenv("REDIS_URL", "redis://localhost:6379")
POLYMARKET_POLL_INTERVAL = int(os.getenv("POLYMARKET_REST_POLL_INTERVAL", "60"))


async def main() -> None:
    logger.info("signal-ingestion starting  redis=%s", REDIS_URL)

    redis_client = aioredis.from_url(REDIS_URL, decode_responses=True)

    # Adapter 1 — Polymarket REST
    polymarket_rest = PolymarketRESTAdapter(
        redis_client=redis_client,
        poll_interval=POLYMARKET_POLL_INTERVAL,
    )

    # Graceful shutdown on SIGINT / SIGTERM
    loop = asyncio.get_running_loop()

    def _shutdown(sig_name: str) -> None:
        logger.info("signal-ingestion received %s — shutting down", sig_name)
        asyncio.ensure_future(polymarket_rest.stop())

    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, lambda s=sig.name: _shutdown(s))
        except NotImplementedError:
            # Windows does not support add_signal_handler for all signals
            pass

    try:
        await polymarket_rest.start()
    finally:
        await redis_client.aclose()
        logger.info("signal-ingestion stopped")


if __name__ == "__main__":
    asyncio.run(main())
