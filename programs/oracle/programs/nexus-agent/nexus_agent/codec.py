"""Binary codec for the NEXUS wire protocol.

Header layout (20 bytes, little-endian):
  [0..2]  msg_type    (u16)
  [2..4]  version     (u16)
  [4..8]  sequence    (u32)
  [8..12] timestamp   (u32, Unix ms lower 32 bits)
  [12..16] payload_len (u32)
  [16..20] schema_id  (u32)

Mirrors nexus-core/src/types.rs PacketHeader exactly.
"""
from __future__ import annotations

import struct
import time
from dataclasses import dataclass, field

# ── Header ────────────────────────────────────────────────────────────────────

HEADER_FMT  = "<HHIIII"
HEADER_SIZE = struct.calcsize(HEADER_FMT)  # 20 bytes

# Message type codes (matches nexus-node/src/protocol.rs)
MSG_HANDSHAKE          = 0x0100
MSG_HANDSHAKE_RESPONSE = 0x0101
MSG_PLAYER_ACTION      = 0x0200
MSG_ENTER              = 0x0300
MSG_SPATIAL_MANIFEST   = 0x0301
MSG_AGENT_TASK         = 0x0400
MSG_AGENT_BROADCAST    = 0x0401

# Bootstrap schema IDs — hardcoded forever (nexus-core/src/types.rs)
SCHEMA_UNTYPED  = 0  # legacy / unknown
SCHEMA_REGISTRY = 1  # schema discovery

# All other schema IDs are FNV-1a 32-bit hashes of "<name>@<version>".
# Mirrors nexus_schema::schema_id() in Rust and schemaId() in TypeScript.

def _fnv32(s: str) -> int:
    """FNV-1a 32-bit hash."""
    h = 0x811c9dc5
    for b in s.encode("utf-8"):
        h ^= b
        h = (h * 0x01000193) & 0xFFFFFFFF
    return h

def schema_id(name: str, version: str) -> int:
    """Compute the stable schema_id for a name+version pair."""
    return _fnv32(f"{name}@{version}")

# Pre-computed IDs for known schemas — same values as Rust/TS, no coordination needed
SCHEMA_PHYSICS_BODY     = schema_id("physics_body",     "1.0")
SCHEMA_SPATIAL_MANIFEST = schema_id("spatial_manifest", "1.0")
SCHEMA_AGENT_TASK       = schema_id("agent_task",       "1.0")
SCHEMA_COMPUTER         = schema_id("computer",         "1.0")
SCHEMA_DISPLAY_FRAME    = schema_id("display_frame",    "1.0")
SCHEMA_FILE             = schema_id("file",             "1.0")

_seq = 0

def _next_seq() -> int:
    global _seq
    _seq += 1
    return _seq

def _now_ms() -> int:
    return int(time.time() * 1000) & 0xFFFFFFFF


def encode_frame(msg_type: int, payload: bytes, schema_id: int = SCHEMA_UNTYPED) -> bytes:
    header = struct.pack(
        HEADER_FMT,
        msg_type,
        1,           # version
        _next_seq(),
        _now_ms(),
        len(payload),
        schema_id,
    )
    return header + payload


@dataclass
class Frame:
    msg_type:   int
    version:    int
    sequence:   int
    timestamp:  int
    schema_id:  int
    payload:    bytes


def decode_frame(data: bytes) -> Frame | None:
    if len(data) < HEADER_SIZE:
        return None
    msg_type, version, seq, ts, payload_len, schema_id = struct.unpack_from(HEADER_FMT, data)
    if len(data) < HEADER_SIZE + payload_len:
        return None
    payload = data[HEADER_SIZE: HEADER_SIZE + payload_len]
    return Frame(msg_type, version, seq, ts, schema_id, payload)


# ── HANDSHAKE (C→S) ───────────────────────────────────────────────────────────

def encode_handshake(player_id: int = 0) -> bytes:
    """Minimal handshake: 8-byte f64 player_id."""
    payload = struct.pack("<d", float(player_id))
    return encode_frame(MSG_HANDSHAKE, payload)


def decode_handshake_response(payload: bytes) -> int | None:
    """Returns the assigned entity_id (f64 on wire → int)."""
    if len(payload) < 8:
        return None
    (entity_id_f,) = struct.unpack_from("<d", payload)
    return int(entity_id_f)


# ── ENTER (C→S) ───────────────────────────────────────────────────────────────

def encode_enter(world_id: str = "") -> bytes:
    """Request the spatial manifest for world_id. Empty = default world."""
    b = world_id.encode("utf-8")
    payload = struct.pack("<H", len(b)) + b
    return encode_frame(MSG_ENTER, payload)


# ── SPATIAL_MANIFEST (S→C) ────────────────────────────────────────────────────

@dataclass
class SpatialManifest:
    world_id:  str
    geometry:  str | None
    surface:   list[str]
    agent:     str | None
    payment:   str | None


def _read_str(data: bytes, pos: int) -> tuple[str, int]:
    if pos + 2 > len(data):
        raise ValueError(f"truncated at {pos}")
    length = struct.unpack_from("<H", data, pos)[0]
    pos += 2
    s = data[pos: pos + length].decode("utf-8")
    return s, pos + length


def _read_opt_str(data: bytes, pos: int) -> tuple[str | None, int]:
    s, pos = _read_str(data, pos)
    return (None if s == "" else s), pos


def decode_spatial_manifest(payload: bytes) -> SpatialManifest | None:
    try:
        pos = 0
        world_id, pos = _read_str(payload, pos)
        geometry, pos = _read_opt_str(payload, pos)
        surface_count = payload[pos]; pos += 1
        surface = []
        for _ in range(surface_count):
            s, pos = _read_str(payload, pos)
            surface.append(s)
        agent, pos   = _read_opt_str(payload, pos)
        payment, pos = _read_opt_str(payload, pos)
        return SpatialManifest(world_id=world_id, geometry=geometry,
                                surface=surface, agent=agent, payment=payment)
    except Exception:
        return None


# ── AGENT_TASK (C→S) ──────────────────────────────────────────────────────────

def encode_agent_task(
    task_id:    int,
    origin_id:  int,
    intent:     str,
    action:     str,
    context:    list[int] | None = None,
    deadline_ms: int = 0,
) -> bytes:
    """Encode an AgentTask packet with schema_id = SCHEMA_AGENT_TASK."""
    context = context or []
    intent_b = intent.encode("utf-8")
    action_b = action.encode("utf-8")
    payload  = struct.pack("<QQ", task_id, origin_id)
    payload += struct.pack("<H", len(intent_b)) + intent_b
    payload += struct.pack("<H", len(action_b)) + action_b
    payload += struct.pack("<B", min(len(context), 255))
    for obj_id in context[:255]:
        payload += struct.pack("<Q", obj_id)
    payload += struct.pack("<I", deadline_ms)
    return encode_frame(MSG_AGENT_TASK, payload, schema_id=SCHEMA_AGENT_TASK)
