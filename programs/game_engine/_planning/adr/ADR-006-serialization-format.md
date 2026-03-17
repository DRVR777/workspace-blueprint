# ADR-006: Binary Message Serialization Format
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
All game state messages over the WebSocket connection are binary. The serialization format determines:
- How fast messages are encoded and decoded (happens every tick, per client)
- How compact messages are (bandwidth budget is tight — see Part XVI)
- How safe the protocol is against schema drift (adding a field breaks clients)
- How easy it is to define message schemas (developer experience)

This is not text serialization (JSON). The ELEV8 post-mortem (Failure A-03) showed that untyped JSON messages led to silent failures when schemas diverged. The binary format must have explicit schemas that are validated at decode time.

## Options Considered

**Custom binary format**
- Maximum compactness, zero overhead
- Schema is in the code — drift is silent and catastrophic
- No tooling, no debugging support
- Verdict: The ELEV8 failure pattern. Rejected.

**Protocol Buffers (protobuf)**
- Industry standard, excellent tooling
- Generates code from `.proto` schema files — schemas are explicit
- Good compactness, some allocation overhead on decode
- Slightly verbose encoding (field tags on every field)
- Version compatible (can add optional fields without breaking old decoders)
- Verdict: Strong option but allocates on decode

**Flatbuffers**
- Google's answer to "protobuf but zero-copy"
- Schema files (`.fbs`) generate accessors, not objects — no deserialization step
- Reading a Flatbuffer message does NOT allocate — it reads directly from the wire buffer
- Schemas are explicit and validated
- Compact binary encoding
- Disadvantage: Less mature tooling than protobuf, generated API is less ergonomic
- Verdict: Best performance for high-frequency game messages

**Cap'n Proto**
- Even faster than Flatbuffers for some operations
- More complex schema language
- Less ecosystem support
- Verdict: Complexity not justified over Flatbuffers

**MessagePack**
- Binary JSON — schemas are implicit (same problem as JSON)
- Verdict: Rejected for same reason as custom binary

## Decision

**Flatbuffers** for all game state messages (entity positions, object state changes, world events, player actions).

Rationale: The position update message is sent 50 times per second per client. At 200 connected clients per node, that's 10,000 decode operations per second just for position updates. Flatbuffers' zero-copy reads eliminate allocation and decompression overhead entirely — the receive buffer IS the decoded message.

**Exception**: Control messages (HANDSHAKE, ERROR, NODE_TRANSFER) use protobuf. These are rare, correctness-critical, and benefit from protobuf's more ergonomic generated API and better tooling support.

## Schema Discipline

- Every message type has a `.fbs` or `.proto` schema file in `shared/schemas/`
- Schema files are the canonical definition — if code and schema disagree, fix the code
- Schema files are versioned with the message type
- Breaking schema changes (removing a field, changing a field type) require a new message type, not a version bump
- Additive changes (adding an optional field) are allowed with a version bump

## Consequences

- `shared/schemas/` folder is created (new — add to MANIFEST)
- Schema files must be compiled before any implementation can run
- The network layer (when implemented) takes a dependency on the Flatbuffers runtime
- All message type specifications in PRD Part VIII must be transcribed into `.fbs` schema files before Phase 0 implementation begins
- The HANDSHAKE message (which uses protobuf) needs a `.proto` schema file
