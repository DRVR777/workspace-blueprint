"""Entry point for the NEXUS node-manager (Phase 0)."""

import asyncio
import logging
import argparse

from node_manager import NodeManager

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-8s %(name)s: %(message)s",
    datefmt="%H:%M:%S",
)


def main() -> None:
    parser = argparse.ArgumentParser(description="NEXUS node-manager (Phase 0)")
    parser.add_argument("--host", default="localhost", help="Bind host")
    parser.add_argument("--port", type=int, default=9000, help="Bind port")
    parser.add_argument("--log-ticks", action="store_true",
                        help="Print tick stats on exit")
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
