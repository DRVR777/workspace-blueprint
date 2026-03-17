# Left Off Here — 2026-03-14

**This file is always current. Overwritten at end of every session via `wrap up`.**
**Start every new session by reading this file first. Nothing else.**

---

## Session Title
Multi-agent session — ORACLE fully specced + game_engine Phase 0 ALL SPECCED + knowledge-graph build in progress

## Last Thing Touched (this agent — game_engine)
`programs/game_engine/_planning/roadmap.md` — all 4 spec reviews PASSED, all Phase 0 sub-programs status → specced, Phase 0 build begins with node-manager

## Last Thing Touched (other agent — ORACLE)
`programs/oracle/programs/operator-dashboard/MANIFEST.md` — final spec-review transition, status → specced

## Last Thing Touched (other agent — knowledge-graph)
`programs/knowledge-graph/programs/context-builder/src/` — context_builder.py in progress at session close

---

## The Exact Next Steps (by project)

### game_engine — begin Phase 0 implementation
All 4 spec reviews PASSED 2026-03-14. All sub-programs are `specced`. Build order per roadmap.md:
1. `world/programs/node-manager/` ← **start here** — node lifecycle, tick loop skeleton
2. `world/programs/spatial/` — in-memory octree (node-manager calls this)
3. `engine/programs/renderer/` — renders flat plane at 60 FPS
4. `engine/programs/local-simulation/` — client-side prediction (position only)

For each: open its `CONTEXT.md`, follow the steps, hit checkpoints. Phase 0 done when one player moves server-authoritatively, client-predicted, 50 ticks server, 60 FPS client.

### oracle — start building Phase 1

**Pre-build infrastructure — all three now exist (created this session):**
- `oracle-shared/` ✅ — Pydantic package with all 10 contract models. Run `make install-shared` to install.
- `docker-compose.yml` ✅ — Redis with AOF persistence. Run `make up` to start.
- `.env.example` ✅ — Copy to `.env` and fill in keys. All 14 vars documented with comments.
- `Makefile` ✅ — `make up`, `make install`, `make [program-name]` to run any program.

**To start a build session:**
```
cp programs/oracle/.env.example programs/oracle/.env   # fill in your keys
cd programs/oracle && make up                          # start Redis
make install                                           # install oracle-shared + all programs
```

**Then build signal-ingestion:**
Open `programs/oracle/programs/signal-ingestion/CONTEXT.md` — follow build sequence row by row:
1. Polymarket REST adapter
2. Polymarket WebSocket adapter
3. Polygon on-chain listener (Alchemy WS)
4a/4b/4c. OSINT adapters (NewsAPI, Wikipedia, Reddit)
5. Birdeye price adapter
6. AI opinion poller

Each program lives at `programs/oracle/programs/[name]/` and gets its own `pyproject.toml` with `uv pip install -e ../../oracle-shared`.

**3 inferences logged during spec-review — check pending.txt before building:**
- `osint-fusion` skips `polygon_clob` and `birdeye` signals (no text) — confirm this is intentional
- `semantic_state_summary` in OSFE uses `claude-haiku-4-5-20251001` not Sonnet — cost decision, verify
- Copy-trade approval channel `oracle:copy_trade_approved` now has a contract (`copy-trade-approval.md`) — SIL execution path must subscribe to this channel to trigger copy trades

### knowledge-graph — resume context-builder
Check `programs/knowledge-graph/programs/context-builder/src/context_builder.py` for completion status, then build the final program (whichever is next after context-builder).

---

## Questions Needing Your Answer

None blocking for ORACLE or knowledge-graph. game_engine spec review is agent-executable (no human decisions required).

---

## What Was Completed This Session

### game_engine (this agent)

**GAP-011 closed** — 16 schema files in `shared/schemas/`:
- 10 Flatbuffers: entity_position_update, object_state_change, world_event, tick_sync, player_joined, player_left, player_action, asset_request, asset_chunk, asset_complete
- 6 Protobuf: handshake (+ gpu_caps ADR-005), handshake_response, node_transfer, action_acknowledgment, error, chat_message

**GAP-012 closed** — `gpu_caps` bitmask in handshake.proto

**GAP-002 closed** — all 6 shared contracts accepted:
- `world-state-contract.md` v0.2 — full shapes from PRD §9.2
- `simulation-contract.md` v0.2 — physics_body (3 categories), tick_result, constants

**GAP-003 closed** — CONTEXT.md build contracts for all 4 Phase 0 sub-programs:
- `world/programs/spatial/CONTEXT.md`
- `world/programs/node-manager/CONTEXT.md`
- `engine/programs/renderer/CONTEXT.md`
- `engine/programs/local-simulation/CONTEXT.md`

**GAP-014 closed** — Spec review PASS on all 4 Phase 0 sub-programs (2026-03-14):
- `world/programs/spatial/` → status: specced ✅
- `world/programs/node-manager/` → status: specced ✅
- `engine/programs/renderer/` → status: specced ✅
- `engine/programs/local-simulation/` → status: specced ✅

### ORACLE (other agent)
- Full scaffold from PRD intake
- 23 ADRs accepted (language: Python 3.11 / asyncio; bus: Redis; AI: Claude)
- 10 contracts with full Pydantic shapes
- 12 blocking gaps closed
- Spec-review PASS on all 7 programs → all specced

### knowledge-graph (other agent)
- 4 ADRs accepted (ADR-004, 005, 007, 008)
- data-store ✅, file-selector ✅, indexer ✅, context-builder in progress

### workspace-builder (this agent, earlier)
- Priorities 1–10 complete (improvement-engine, graph-engine, registry, guards, CONVENTIONS)

---

## Project Roadmap State

| Project | Status | Next action |
|---------|--------|-------------|
| workspace-builder | ✅ complete | — |
| oracle | ✅ specced | Build Phase 1: signal-ingestion |
| knowledge-graph | 🔄 building | context-builder → then final program |
| game_engine | ✅ specced | Begin Phase 0 build: node-manager first |

---

## ORACLE Stack (locked)

| Concern | Decision |
|---------|---------|
| Language | Python 3.11+, asyncio, uv |
| Event bus | Redis pub/sub, 8 channels |
| Shared state | Redis hash `oracle:state:`, AOF persistence |
| Embeddings | OpenAI text-embedding-3-small, 512 dims |
| Vector store | ChromaDB local, 2 collections |
| Polygon RPC | Alchemy WebSocket, web3.py |
| Solana | Birdeye (price) + Jupiter v6 (execution) |
| Polymarket | py-clob-client, env-var keypair |
| AI (RE + KBPM) | claude-sonnet-4-6 |
| AI (OSFE state) | claude-haiku-4-5-20251001 |
| Dashboard | FastAPI + vanilla JS, localhost:8080 |
| Alerts | Telegram bot, python-telegram-bot |

---

## Resume Prompt

```
RESUME — 2026-03-14

Working directory:
C:\Users\Quandale Dingle\yearTwo777\workspace-blueprint\workspace-blueprint\

Read leftOffHere.md first. It has the full state.

Three active projects:

1. game_engine — ALL SPECCED. Start Phase 0 build. Open programs/game_engine/world/programs/node-manager/CONTEXT.md. Build in order: node-manager → spatial → renderer → local-simulation.

2. oracle — fully specced, ready to build.
   BEFORE touching any program: create oracle-shared package + docker-compose.yml + .env.example (all specified in leftOffHere.md oracle section above).
   Then: open programs/oracle/programs/signal-ingestion/CONTEXT.md. Row 1 = Polymarket REST adapter.
   Contract shapes are in programs/oracle/shared/contracts/*.md — transcribe to Pydantic in oracle-shared.

3. knowledge-graph — context-builder in progress. Check src/ for completion status, continue or move to next program.
```
