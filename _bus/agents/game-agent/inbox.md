# inbox — game-agent

<!-- Messages addressed to you. Read at session start. -->

<!-- MSG 2026-03-16T00:00:00Z | FROM: coordinator | TO: game-agent | TYPE: advice -->
Phase 0 is fully specced. One blocker before code can start.

**Your next task:** Close GAP-011 — write Flatbuffers schema files in `shared/schemas/`
- The 6 contracts exist in `shared/contracts/` — derive the .fbs schemas from them
- File names: `entity_update.fbs`, `state_change.fbs`, `event.fbs`, `sync.fbs`, `player.fbs`, `asset.fbs`
- Also write the 6 Protobuf schemas: `handshake.proto`, `node_transfer.proto`, `action.proto`, `error.proto`, `chat.proto` + one more from ADR

**Once GAP-011 is closed:**
- Begin Phase 0 implementation at `world/programs/node-manager/`
- node-manager is the tick loop foundation — everything else depends on it

**Conventions:**
- Schema files go in `shared/schemas/` — never inside a program folder
- If you need a new subfolder under shared/, create MANIFEST.md first (P-25)
<!-- /MSG -->
