"""Phase 0 binary codec for NEXUS network protocol.

DEVIATION NOTE: This codec manually encodes all message types using big-endian
struct layout rather than compiled codegen. Production build should replace with:
  - Compiled Protobuf Python classes (protoc --python_out) for 0x0100, 0x0101
  - Compiled FlatBuffers Python classes (flatc --python) for 0x0001, 0x0004,
    0x0005, 0x0006, 0x0200

Wire frame layout (20 bytes, big-endian, per PRD §8.2):
  message_type:     uint16  — identifies the message
  message_version:  uint16  — codec version check
  sequence_number:  uint32  — message counter or 0
  timestamp_ms:     uint64  — Unix ms at encoding time
  payload_length:   uint32  — byte length of payload that follows
"""

import struct
import time
from dataclasses import dataclass

# ---------------------------------------------------------------------------
# Wire frame
# ---------------------------------------------------------------------------

FRAME_FMT  = ">HHIQI"
FRAME_SIZE = struct.calcsize(FRAME_FMT)  # 20 bytes

# Message type constants
MSG_ENTITY_POSITION_UPDATE = 0x0001
MSG_TICK_SYNC              = 0x0004
MSG_PLAYER_JOINED          = 0x0005
MSG_PLAYER_LEFT            = 0x0006
MSG_HANDSHAKE              = 0x0100
MSG_HANDSHAKE_RESPONSE     = 0x0101
MSG_PLAYER_ACTION          = 0x0200

# Handshake status codes
HS_ACCEPTED         = 0
HS_REJECTED         = 1
HS_VERSION_MISMATCH = 2

# Player-left reason codes
PL_DISCONNECT    = 0
PL_NODE_TRANSFER = 1
PL_DEATH         = 2

# Action type codes (ActionType enum from player_action.fbs)
ACTION_MOVE      = 0x0000
ACTION_INTERACT  = 0x0001
ACTION_BUILD     = 0x0002
ACTION_DESTROY   = 0x0003

SERVER_VERSION = 1
WORLD_SEED     = 0xDEADBEEFCAFEBABE


def _now_ms() -> int:
    return int(time.time() * 1000)


def _frame(msg_type: int, msg_version: int, seq: int, payload: bytes) -> bytes:
    hdr = struct.pack(FRAME_FMT, msg_type, msg_version, seq, _now_ms(), len(payload))
    return hdr + payload


def _unframe(data: bytes) -> tuple[int, int, int, int, bytes]:
    """Returns (msg_type, msg_version, seq, timestamp_ms, payload)."""
    if len(data) < FRAME_SIZE:
        raise ValueError(f"Frame too short: {len(data)} bytes (need {FRAME_SIZE})")
    msg_type, msg_version, seq, ts_ms, plen = struct.unpack_from(FRAME_FMT, data)
    return msg_type, msg_version, seq, ts_ms, data[FRAME_SIZE: FRAME_SIZE + plen]


def peek_msg_type(data: bytes) -> int:
    """Quick peek at message type without full unframe."""
    if len(data) < 2:
        raise ValueError("Data too short to peek")
    return struct.unpack_from(">H", data)[0]


# ---------------------------------------------------------------------------
# HANDSHAKE  (0x0100)
# Matches: handshake.proto fields client_version, player_id, auth_token, gpu_caps
# ---------------------------------------------------------------------------

_HS_FMT  = ">IQ32sI"
_HS_SIZE = struct.calcsize(_HS_FMT)  # 4+8+32+4 = 48


@dataclass
class Handshake:
    client_version: int
    player_id: int
    auth_token: bytes   # exactly 32 bytes on wire
    gpu_caps: int


def encode_handshake(hs: Handshake) -> bytes:
    token = (hs.auth_token + b"\x00" * 32)[:32]
    payload = struct.pack(_HS_FMT, hs.client_version, hs.player_id, token, hs.gpu_caps)
    return _frame(MSG_HANDSHAKE, 1, 0, payload)


def decode_handshake(data: bytes) -> Handshake:
    t, _, _, _, payload = _unframe(data)
    if t != MSG_HANDSHAKE:
        raise ValueError(f"Not a HANDSHAKE: 0x{t:04x}")
    cv, pid, tok, gc = struct.unpack_from(_HS_FMT, payload)
    return Handshake(client_version=cv, player_id=pid, auth_token=tok, gpu_caps=gc)


# ---------------------------------------------------------------------------
# HANDSHAKE_RESPONSE  (0x0101)
# Matches: handshake_response.proto fields status, assigned_entity_id,
#          initial_position_*, server_version, world_seed
# ---------------------------------------------------------------------------

_HSR_FMT  = ">BIfffIQ"
_HSR_SIZE = struct.calcsize(_HSR_FMT)  # 1+4+12+4+8 = 29


@dataclass
class HandshakeResponse:
    status: int
    entity_id: int      = 0
    pos_x: float        = 0.0
    pos_y: float        = 0.0
    pos_z: float        = 0.0
    server_version: int = SERVER_VERSION
    world_seed: int     = WORLD_SEED


def encode_handshake_response(r: HandshakeResponse) -> bytes:
    payload = struct.pack(_HSR_FMT, r.status, r.entity_id,
                          r.pos_x, r.pos_y, r.pos_z,
                          r.server_version, r.world_seed)
    return _frame(MSG_HANDSHAKE_RESPONSE, 1, 0, payload)


def decode_handshake_response(data: bytes) -> HandshakeResponse:
    t, _, _, _, payload = _unframe(data)
    if t != MSG_HANDSHAKE_RESPONSE:
        raise ValueError(f"Not a HANDSHAKE_RESPONSE: 0x{t:04x}")
    st, eid, px, py, pz, sv, ws = struct.unpack_from(_HSR_FMT, payload)
    return HandshakeResponse(status=st, entity_id=eid,
                              pos_x=px, pos_y=py, pos_z=pz,
                              server_version=sv, world_seed=ws)


# ---------------------------------------------------------------------------
# PLAYER_ACTION  (0x0200)
# Matches: player_action.fbs fields message_version, action_type,
#          sequence_number, requires_ack, payload
# ---------------------------------------------------------------------------

# Inner payload header: message_version H, action_type H,
#                       sequence_number I, requires_ack B, payload_length H
_PA_HDR_FMT  = ">HHIBH"
_PA_HDR_SIZE = struct.calcsize(_PA_HDR_FMT)  # 2+2+4+1+2 = 11

_MOVE_FMT = ">fff"   # MOVE action payload: target_x, target_y, target_z


@dataclass
class PlayerAction:
    action_type: int
    sequence_number: int
    requires_ack: bool
    payload: bytes       = b""
    message_version: int = 1


def encode_player_action(pa: PlayerAction) -> bytes:
    inner = (struct.pack(_PA_HDR_FMT,
                         pa.message_version, pa.action_type,
                         pa.sequence_number, int(pa.requires_ack),
                         len(pa.payload))
             + pa.payload)
    return _frame(MSG_PLAYER_ACTION, 1, pa.sequence_number, inner)


def decode_player_action(data: bytes) -> PlayerAction:
    t, _, _, _, inner = _unframe(data)
    if t != MSG_PLAYER_ACTION:
        raise ValueError(f"Not a PLAYER_ACTION: 0x{t:04x}")
    mv, at, sn, ack, plen = struct.unpack_from(_PA_HDR_FMT, inner)
    payload = inner[_PA_HDR_SIZE: _PA_HDR_SIZE + plen]
    return PlayerAction(action_type=at, sequence_number=sn,
                        requires_ack=bool(ack), payload=payload,
                        message_version=mv)


def encode_move_payload(x: float, y: float, z: float) -> bytes:
    return struct.pack(_MOVE_FMT, x, y, z)


def decode_move_payload(payload: bytes) -> tuple[float, float, float]:
    return struct.unpack_from(_MOVE_FMT, payload)


# ---------------------------------------------------------------------------
# TICK_SYNC  (0x0004)
# Matches: tick_sync.fbs fields message_version, server_tick, server_time_ms
# ---------------------------------------------------------------------------

_TS_FMT  = ">HQQ"
_TS_SIZE = struct.calcsize(_TS_FMT)  # 2+8+8 = 18


def encode_tick_sync(server_tick: int, seq: int = 0) -> bytes:
    payload = struct.pack(_TS_FMT, 1, server_tick, _now_ms())
    return _frame(MSG_TICK_SYNC, 1, seq, payload)


def decode_tick_sync(data: bytes) -> tuple[int, int, int]:
    """Returns (message_version, server_tick, server_time_ms)."""
    t, _, _, _, payload = _unframe(data)
    if t != MSG_TICK_SYNC:
        raise ValueError(f"Not a TICK_SYNC: 0x{t:04x}")
    return struct.unpack_from(_TS_FMT, payload)


# ---------------------------------------------------------------------------
# ENTITY_POSITION_UPDATE  (0x0001)
# Matches: entity_position_update.fbs EntityEntry struct layout.
# orientation_* and velocity_* are half-float encoded as uint16 (per .fbs comment).
# ---------------------------------------------------------------------------

_EPU_HDR_FMT  = ">HH"   # message_version, entity_count
_EPU_HDR_SIZE = struct.calcsize(_EPU_HDR_FMT)  # 4

# EntityEntry struct (fixed-size, per fbs):
#   entity_id    uint32   4
#   pos x/y/z    float32  12
#   orient w/x/y/z uint16  8   (half-float)
#   vel x/y/z    uint16   6   (half-float)
#   total: 30 bytes
_EE_FMT  = ">Ifff4H3H"
_EE_SIZE = struct.calcsize(_EE_FMT)  # 30


@dataclass
class EntityState:
    entity_id: int
    pos_x: float
    pos_y: float
    pos_z: float
    orient_w: float = 1.0
    orient_x: float = 0.0
    orient_y: float = 0.0
    orient_z: float = 0.0
    vel_x: float    = 0.0
    vel_y: float    = 0.0
    vel_z: float    = 0.0


def _f32_to_f16(v: float) -> int:
    """IEEE 754 float32 → half-float (uint16)."""
    b    = struct.pack(">f", v)
    bits = struct.unpack(">I", b)[0]
    sign     = (bits >> 31) & 0x1
    exp      = (bits >> 23) & 0xFF
    mantissa =  bits        & 0x7FFFFF
    if exp == 0:
        return sign << 15                            # zero / denormal → zero
    if exp == 255:
        return (sign << 15) | 0x7C00 | (1 if mantissa else 0)  # inf / NaN
    exp -= 127
    if exp < -14:
        return sign << 15                            # underflow → zero
    if exp > 15:
        return (sign << 15) | 0x7C00                # overflow → inf
    return (sign << 15) | ((exp + 15) << 10) | (mantissa >> 13)


def _f16_to_f32(h: int) -> float:
    """Half-float (uint16) → IEEE 754 float32."""
    sign     = (h >> 15) & 0x1
    exp      = (h >> 10) & 0x1F
    mantissa =  h        & 0x3FF
    if exp == 0:
        f = mantissa / 1024.0
    elif exp == 31:
        f = float("inf") if mantissa == 0 else float("nan")
    else:
        f = (1.0 + mantissa / 1024.0) * (2.0 ** (exp - 15))
    return -f if sign else f


def encode_entity_position_update(entities: list[EntityState], seq: int = 0) -> bytes:
    header = struct.pack(_EPU_HDR_FMT, 1, len(entities))
    body = b"".join(
        struct.pack(_EE_FMT,
                    e.entity_id, e.pos_x, e.pos_y, e.pos_z,
                    _f32_to_f16(e.orient_w), _f32_to_f16(e.orient_x),
                    _f32_to_f16(e.orient_y), _f32_to_f16(e.orient_z),
                    _f32_to_f16(e.vel_x),    _f32_to_f16(e.vel_y),
                    _f32_to_f16(e.vel_z))
        for e in entities
    )
    return _frame(MSG_ENTITY_POSITION_UPDATE, 1, seq, header + body)


def decode_entity_position_update(data: bytes) -> list[EntityState]:
    t, _, _, _, payload = _unframe(data)
    if t != MSG_ENTITY_POSITION_UPDATE:
        raise ValueError(f"Not an EPU: 0x{t:04x}")
    _, count = struct.unpack_from(_EPU_HDR_FMT, payload)
    out, off = [], _EPU_HDR_SIZE
    for _ in range(count):
        eid, px, py, pz, ow, ox, oy, oz, vx, vy, vz = struct.unpack_from(_EE_FMT, payload, off)
        out.append(EntityState(
            entity_id=eid, pos_x=px, pos_y=py, pos_z=pz,
            orient_w=_f16_to_f32(ow), orient_x=_f16_to_f32(ox),
            orient_y=_f16_to_f32(oy), orient_z=_f16_to_f32(oz),
            vel_x=_f16_to_f32(vx), vel_y=_f16_to_f32(vy), vel_z=_f16_to_f32(vz),
        ))
        off += _EE_SIZE
    return out


# ---------------------------------------------------------------------------
# PLAYER_JOINED  (0x0005)
# Matches: player_joined.fbs fields message_version, entity_id, position_*,
#          orientation_* (full float32 — sent once only), player_id, display_name
# ---------------------------------------------------------------------------

# Fixed part: message_version H, entity_id I, pos x/y/z fff,
#             orient w/x/y/z ffff, player_id Q, display_name_len B
_PJ_FIXED_FMT  = ">HIfffffffQB"
_PJ_FIXED_SIZE = struct.calcsize(_PJ_FIXED_FMT)  # 2+4+12+16+8+1 = 43


@dataclass
class PlayerJoinedMsg:
    entity_id:    int
    player_id:    int
    display_name: str
    pos_x: float    = 0.0
    pos_y: float    = 0.0
    pos_z: float    = 0.0
    orient_w: float = 1.0
    orient_x: float = 0.0
    orient_y: float = 0.0
    orient_z: float = 0.0


def encode_player_joined(m: PlayerJoinedMsg, seq: int = 0) -> bytes:
    name = m.display_name.encode("utf-8")[:32]
    fixed = struct.pack(_PJ_FIXED_FMT,
                        1, m.entity_id,
                        m.pos_x, m.pos_y, m.pos_z,
                        m.orient_w, m.orient_x, m.orient_y, m.orient_z,
                        m.player_id, len(name))
    return _frame(MSG_PLAYER_JOINED, 1, seq, fixed + name)


def decode_player_joined(data: bytes) -> PlayerJoinedMsg:
    t, _, _, _, payload = _unframe(data)
    if t != MSG_PLAYER_JOINED:
        raise ValueError(f"Not a PLAYER_JOINED: 0x{t:04x}")
    mv, eid, px, py, pz, ow, ox, oy, oz, pid, nlen = struct.unpack_from(_PJ_FIXED_FMT, payload)
    name = payload[_PJ_FIXED_SIZE: _PJ_FIXED_SIZE + nlen].decode("utf-8")
    return PlayerJoinedMsg(entity_id=eid, player_id=pid, display_name=name,
                            pos_x=px, pos_y=py, pos_z=pz,
                            orient_w=ow, orient_x=ox, orient_y=oy, orient_z=oz)


# ---------------------------------------------------------------------------
# PLAYER_LEFT  (0x0006)
# Matches: player_left.fbs fields message_version, entity_id, reason
# ---------------------------------------------------------------------------

_PL_FMT  = ">HIB"
_PL_SIZE = struct.calcsize(_PL_FMT)  # 2+4+1 = 7


def encode_player_left(entity_id: int, reason: int = PL_DISCONNECT,
                        seq: int = 0) -> bytes:
    return _frame(MSG_PLAYER_LEFT, 1, seq,
                  struct.pack(_PL_FMT, 1, entity_id, reason))


def decode_player_left(data: bytes) -> tuple[int, int, int]:
    """Returns (message_version, entity_id, reason)."""
    t, _, _, _, payload = _unframe(data)
    if t != MSG_PLAYER_LEFT:
        raise ValueError(f"Not a PLAYER_LEFT: 0x{t:04x}")
    return struct.unpack_from(_PL_FMT, payload)
