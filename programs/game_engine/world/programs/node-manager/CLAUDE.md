# CLAUDE.md — node-manager

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `MANIFEST.md` | Full specification: tick loop, startup, drain, domain split |
| `CONTEXT.md` | Build contract — inputs, process, checkpoints, outputs |
| `src/` | Phase 0 implementation: tick loop, WebSocket server, entity management |
| `output/` | Phase completion reports |
| `requirements.txt` | Python dependencies (websockets, pytest, pytest-asyncio) |

## Quick Rules For This Directory

- Follow patterns in `_core/CONVENTIONS.md`
- All configuration in `src/config.py` — no magic numbers in other modules (P-16)
- Entity lifecycle through `entity_manager.py` — never mutate entity state directly
- Input processing through `input_queue.py` — never drain queues inline
- Wire encoding through `state_serializer.py` / `codec.py`
- Stubs in `src/stubs/` match contract interfaces exactly — swap for real impls with import change only

## Cross-References

- `../../shared/contracts/` — world-state, simulation, node-registry, ticker-log, player-session contracts
- `../../shared/schemas/` — Flatbuffers (.fbs) and Protobuf (.proto) wire schemas
- `../../_planning/adr/` — ADR-006 (serialization format), ADR-001 (sector size)
