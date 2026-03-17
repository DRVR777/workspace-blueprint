# shared/schemas — Message Schema Definitions

*Required by ADR-006 (Flatbuffers for game state, Protobuf for control messages).*

This folder contains the canonical schema files for all network messages. The schema files are the source of truth. Generated code derives from these files. If code and schema disagree, fix the code.

**GAP-011: CLOSED** — all schemas written 2026-03-14.
**GAP-012: CLOSED** — `gpu_caps` field added to `handshake.proto` per ADR-005.

## File Naming

```
[message_type_name].fbs          — Flatbuffers schema (game state messages)
[message_type_name].proto        — Protobuf schema (control messages)
```

## Wire Framing

Every message has a common wire header prepended by the transport layer (not part of the schema payload). See PRD.md §8.2:

```
[2 bytes] message_type
[2 bytes] message_version
[4 bytes] sequence_number
[4 bytes] timestamp_ms
[4 bytes] payload_length
[N bytes] payload     ← this is what each schema file defines
```

Schema files define the **payload** only.

## float16 Note

FlatBuffers has no native `float16` type. Fields marked as half-float in the PRD (orientation and velocity components in `entity_position_update.fbs`) are stored as `uint16` using IEEE 754 half-precision encoding. Use the `half_float::to_float()` / `half_float::from_float()` utilities in the network runtime when reading these fields.

## Schema Files

### Flatbuffers (.fbs) — Game State Messages

| Schema file | Message type | Hex | Direction | Status |
|------------|-------------|-----|-----------|--------|
| `entity_position_update.fbs` | ENTITY_POSITION_UPDATE | 0x0001 | S→C | ✅ defined |
| `object_state_change.fbs` | OBJECT_STATE_CHANGE | 0x0002 | S→C | ✅ defined |
| `world_event.fbs` | WORLD_EVENT | 0x0003 | S→C | ✅ defined |
| `tick_sync.fbs` | TICK_SYNC | 0x0004 | S→C | ✅ defined |
| `player_joined.fbs` | PLAYER_JOINED | 0x0005 | S→C | ✅ defined |
| `player_left.fbs` | PLAYER_LEFT | 0x0006 | S→C | ✅ defined |
| `player_action.fbs` | PLAYER_ACTION | 0x0200 | C→S | ✅ defined |
| `asset_request.fbs` | ASSET_REQUEST | 0x0201 | C→S | ✅ defined |
| `asset_chunk.fbs` | ASSET_CHUNK | 0x0300 | S→C | ✅ defined |
| `asset_complete.fbs` | ASSET_COMPLETE | 0x0301 | S→C | ✅ defined |

### Protobuf (.proto) — Control Messages

| Schema file | Message type | Hex | Direction | Status |
|------------|-------------|-----|-----------|--------|
| `handshake.proto` | HANDSHAKE | 0x0100 | C→S | ✅ defined (gpu_caps added ADR-005) |
| `handshake_response.proto` | HANDSHAKE_RESPONSE | 0x0101 | S→C | ✅ defined |
| `node_transfer.proto` | NODE_TRANSFER | 0x0102 | S→C | ✅ defined |
| `action_acknowledgment.proto` | ACTION_ACKNOWLEDGMENT | 0x0103 | S→C | ✅ defined |
| `error.proto` | ERROR | 0x01FF | S→C | ✅ defined |
| `chat_message.proto` | CHAT_MESSAGE | 0x0202 | C→S | ✅ defined |

## Generation Process

```
INPUT:  .fbs schema file
TOOL:   flatc (FlatBuffers compiler) — specific invocation decided at Phase 0 start
OUTPUT: generated accessor code for the target runtime

INPUT:  .proto schema file
TOOL:   protoc (Protocol Buffers compiler)
OUTPUT: generated message class code for the target runtime
```

Compiler invocations are not scripted here — they will be added to the build system when Phase 0 implementation begins.

## Schema Discipline (from ADR-006)

- Schema files are the canonical definition — if code and schema disagree, fix the code
- Breaking changes (removing a field, changing a field type) require a **new message type**, not a version bump
- Additive changes (adding an optional field) are allowed with a version bump
- The `message_version` field in each schema must match the wire frame version at runtime
