# node-manager — Build Contract (Phase 0)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: tick loop, startup, drain, domain split |
| `../../shared/contracts/world-state-contract.md` | object_record, entity_record, change_request shapes |
| `../../shared/contracts/simulation-contract.md` | run_tick signature, tick_result, world_state_snapshot |
| `../../shared/contracts/node-registry-contract.md` | find_node, register_node, update_node_load |
| `../../shared/contracts/ticker-log-contract.md` | append, append_batch |
| `../../shared/contracts/player-session-contract.md` | validate_token, session_record |
| `../../shared/schemas/handshake.proto` | HANDSHAKE message structure (client connection) |
| `../../shared/schemas/handshake_response.proto` | HANDSHAKE_RESPONSE |
| `../../shared/schemas/entity_position_update.fbs` | broadcast message shape |
| `../../shared/schemas/object_state_change.fbs` | broadcast message shape |
| `../../shared/schemas/tick_sync.fbs` | clock sync broadcast |
| `../../shared/schemas/player_action.fbs` | incoming action message |

Do NOT load: engine/ files, renderer, lod, asset schemas.

---

## Process

1. Implement startup sequence (MANIFEST.md §"STARTUP"):
   - Read domain from node-registry (stub: hard-code a 1000×1000×1000 unit domain for Phase 0)
   - Instantiate the spatial index (spatial/ contract)
   - Open WebSocket server on configured port
   - Register with node-registry, status = active

2. Implement the action queue: thread-safe queue that accepts PLAYER_ACTION messages from the WebSocket receive thread and drains into the tick loop each tick. One queue per connected client.

3. Implement the tick loop (MANIFEST.md §"TICK LOOP") for Phase 0 scope:
   - Phase A: Drain action queues
   - Phase B: Call `simulation.run_tick(snapshot, inputs, dt)`
   - Phase C: Apply results to local snapshot + enqueue world graph writes
   - Phase D: Broadcast ENTITY_POSITION_UPDATE to all connected clients
   - Phase E: Flush ticker log (stub for Phase 0: write to local file, not distributed log)
   - Phase F: Record tick duration; log warning if > 20ms but do NOT request split in Phase 0
   - Phase G: Sleep until next tick

4. Implement client connection handling:
   - On new connection: read HANDSHAKE, call `session.validate_token(auth_token)`, send HANDSHAKE_RESPONSE
   - On player action: parse PLAYER_ACTION, enqueue in action queue
   - On disconnect: remove from connected_clients, update last position in session layer, send PLAYER_LEFT to other clients

5. Implement the PLAYER_JOINED / PLAYER_LEFT broadcast: when a client connects (accepted), broadcast PLAYER_JOINED with their entity_id to all other clients; on disconnect, broadcast PLAYER_LEFT.

6. Write integration test:
   - Start node
   - Connect 2 test clients
   - Verify both receive each other's PLAYER_JOINED
   - Move client A — verify client B receives ENTITY_POSITION_UPDATE with correct position
   - Disconnect client A — verify client B receives PLAYER_LEFT
   - Verify tick duration stays under 20ms during the test

7. Write `output/phase0-complete.md`: tick rate achieved, client count tested, max tick duration observed.

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 3 | Tick loop skeleton (no networking yet) with timing output | approve / revise loop structure |
| Step 5 | Single client connects and receives HANDSHAKE_RESPONSE + TICK_SYNC | approve → add multi-client / revise |
| Step 6 | Integration test results | approve → write output / fix failures |

---

## Audit

Before writing to `output/`:
- [ ] Tick loop runs at 50Hz (20ms target) — verified by timing log
- [ ] Tick loop does NOT block on world graph writes (async queue)
- [ ] HANDSHAKE validation calls `session.validate_token` — no session is accepted without it
- [ ] Player position is persisted on disconnect via `session.update_last_position`
- [ ] PLAYER_JOINED is broadcast to all clients when a new player connects
- [ ] PLAYER_LEFT is broadcast to all clients when a player disconnects
- [ ] Integration test passes: 2-client scenario as described in Step 6

---

## Outputs

| Output | Location |
|--------|----------|
| Node-manager implementation | `src/` |
| Integration test | `src/tests/` |
| Phase 0 completion summary | `output/phase0-complete.md` |
