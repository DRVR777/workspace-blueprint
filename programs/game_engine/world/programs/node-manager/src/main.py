"""Entry point for the NEXUS node-manager (Phase 0).

Usage:
  python main.py [--host HOST] [--port PORT] [--log-ticks]

All defaults come from config.py and can be overridden via environment
variables (see config.py docstring for the full list).
"""

import asyncio
import logging
import argparse

import config
from node_manager import NodeManager

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-8s %(name)s: %(message)s",
    datefmt="%H:%M:%S",
)


def main() -> None:
    """Parse CLI args and start the node manager."""
    parser = argparse.ArgumentParser(description="NEXUS node-manager (Phase 0)")
    parser.add_argument(
        "--host", default=config.DEFAULT_HOST, help="Bind host"
    )
    parser.add_argument(
        "--port", type=int, default=config.DEFAULT_PORT, help="Bind port"
    )
    parser.add_argument(
        "--log-ticks", action="store_true", help="Print tick stats on exit"
    )
    args = parser.parse_args()

    node = NodeManager(host=args.host, port=args.port)
    try:
        asyncio.run(node.run())
    except KeyboardInterrupt:
        pass
    finally:
        if args.log_ticks:
            stats = node.get_tick_stats()
            print(f"\nTick stats: {stats}")


if __name__ == "__main__":
    main()
