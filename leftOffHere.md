# Left Off Here — 2026-03-19

**This file is always current. Overwritten at end of every session via `wrap up`.**
**Start every new session by reading this file first. Nothing else.**

---

## Session Title
NEXUS engine — specs complete, Rust server compiles, context docs added, deploying to VPS next

## Last Thing Touched
`programs/game_engine/_planning/roadmap.md` — updated to reflect build progress (4 Rust crates done, R3F client running)

---

## The Exact Next Steps

### game_engine — wire WebSocket and deploy to VPS

**What's done (2026-03-19):**
- ADR-015 accepted: TS/R3F client, Rust server, Rapier physics, 5-layer arch
- Simulation sub-program specced (5-stage Rapier pipeline)
- Visibility sub-program specced (frustum culling + LOD)
- World-graph contract written (portals, subworlds, constellation)
- Rust server compiles: 4 crates (nexus-core, nexus-spatial, nexus-simulation, nexus-node), 19 tests pass
- R3F client runs at 60+ FPS with instanced rendering
- ELEV8-source and personalWebsite cloned as reference for character controller + camera controls
- Context docs added across entire workspace (32 gaps identified and fixed)

**What's next (in order):**
1. **Deploy Rust server to VPS** — clone repo, `cargo build --release`, run on port 9001
2. **Wire WebSocket bridge** — replace `worldStateStub.ts` with real `useNetworkState.ts` connecting to `ws://VPS_IP:9001`
3. **Implement HANDSHAKE protocol** — client sends auth, server responds with world state snapshot
4. **Port ELEV8 AvatarController** — WASD movement, pointer lock, wall collision → sends PLAYER_ACTION to server
5. **Server broadcasts ENTITY_POSITION_UPDATE** — after each tick, send positions to all clients
6. **Two-player test** — open two browser tabs, see each other move

**Key reference files for next session:**
- VPS setup: `programs/game_engine/VPS_DEPLOY.md`
- Rust crates guide: `programs/game_engine/world/crates/README.md`
- Character controller reference: `programs/ELEV8-source/components/scene/AvatarController.tsx`
- Client renderer: `programs/game_engine/engine/programs/renderer/src/`
- Handshake schema: `programs/game_engine/shared/schemas/handshake.proto`
- World state stub to replace: `programs/game_engine/engine/programs/renderer/src/simulation/worldStateStub.ts`

### oracle — start building Phase 1
Same as before. Open `programs/oracle/programs/signal-ingestion/CONTEXT.md`.

### knowledge-graph — resume context-builder
Check `programs/knowledge-graph/programs/context-builder/src/` for completion status.

---

## Project Roadmap State

| Project | Status | Next action |
|---------|--------|-------------|
| game_engine | 🔄 building | Deploy to VPS → wire WebSocket → first playable |
| oracle | ✅ specced | Build Phase 1: signal-ingestion |
| knowledge-graph | 🔄 building | context-builder → then final program |
| workspace-builder | ✅ complete | — |

---

## Resume Prompt

```
RESUME — 2026-03-19

Working directory:
C:\Users\Quandale Dingle\yearTwo777\workspace-blueprint\workspace-blueprint\

Read leftOffHere.md first. It has the full state.

Primary project: game_engine (NEXUS)
- Rust server compiles (4 crates, 19 tests). Deploy to VPS first.
- Then wire WebSocket: replace worldStateStub.ts with real network hook
- Reference: ELEV8-source/components/scene/AvatarController.tsx for character controller
- See VPS_DEPLOY.md for server setup instructions
```
