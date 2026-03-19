# CLAUDE.md — src

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `config.py` | Centralized configuration — all tunable constants, env-var backed |
| `node_manager.py` | NodeManager class — tick loop, WebSocket server, lifecycle orchestration |
| `entity_manager.py` | Entity lifecycle — spawn, update, destroy, snapshots |
| `input_queue.py` | Per-client action queue — buffers and drains player inputs |
| `state_serializer.py` | World state to wire format conversion (Flatbuffers-compatible) |
| `tick_metrics.py` | Tick performance collector — records durations, flushes to JSONL |
| `codec.py` | Binary wire codec — encode/decode all 7 message types |
| `main.py` | CLI entry point — argument parsing, logging setup |
| `__init__.py` | Package marker |
| `stubs/` | Phase 0 service stubs — spatial, simulation, session, registry, ticker log |
| `tests/` | Unit and integration tests (53 tests) |

## Quick Rules For This Directory

- All constants come from `config.py` — no magic numbers (P-16)
- Run tests: `cd src && python -m pytest tests/ -v`
- Entry point: `python main.py [--host HOST] [--port PORT] [--log-ticks]`

## Cross-References

- `../MANIFEST.md` — full tick loop specification
- `../CONTEXT.md` — build contract with checkpoints
