"""
State serializer for the NEXUS node-manager.

Converts authoritative world state into Flatbuffers-compatible wire format
for broadcasting to connected clients. Phase 0 uses the manual struct codec
(codec.py) as a stand-in for compiled Flatbuffers/Protobuf; this module
provides the high-level API that the tick loop calls.

References:
  - entity_position_update.fbs: EPU wire schema
  - object_state_change.fbs: state change wire schema
  - tick_sync.fbs: clock sync wire schema
  - MANIFEST.md §TICK LOOP Phase D: broadcast to clients
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

from codec import (
    EntityState as WireEntityState,
    encode_entity_position_update,
    encode_tick_sync,
    encode_player_joined,
    encode_player_left,
    PlayerJoinedMsg,
    PL_DISCONNECT,
)

if TYPE_CHECKING:
    from entity_manager import Entity

logger = logging.getLogger("state_serializer")


class StateSerializer:
    """
    Converts world state to wire-format bytes for client broadcast.

    All encode methods return raw bytes ready to send over WebSocket.
    Phase 0: uses manual struct codec. Phase 1+: swap to compiled
    Flatbuffers/Protobuf without changing this interface.
    """

    @staticmethod
    def encode_position_update(
        entities: list[Entity],
        tick_number: int,
    ) -> bytes:
        """
        Encode an ENTITY_POSITION_UPDATE frame containing all entity positions.

        This is the main per-tick broadcast message (Phase D).
        Matches entity_position_update.fbs EntityEntry layout.
        """
        wire_states = [
            WireEntityState(
                entity_id=e.entity_id,
                pos_x=e.pos_x,
                pos_y=e.pos_y,
                pos_z=e.pos_z,
                orient_w=e.orient_w,
                orient_x=e.orient_x,
                orient_y=e.orient_y,
                orient_z=e.orient_z,
                vel_x=e.vel_x,
                vel_y=e.vel_y,
                vel_z=e.vel_z,
            )
            for e in entities
        ]
        return encode_entity_position_update(wire_states, seq=tick_number)

    @staticmethod
    def encode_clock_sync(tick_number: int) -> bytes:
        """
        Encode a TICK_SYNC frame for client clock calibration.
        Sent once on client connection.
        """
        return encode_tick_sync(tick_number)

    @staticmethod
    def encode_player_join(
        entity_id: int,
        player_id: int,
        display_name: str,
        position: tuple[float, float, float],
    ) -> bytes:
        """Encode a PLAYER_JOINED broadcast frame."""
        return encode_player_joined(PlayerJoinedMsg(
            entity_id=entity_id,
            player_id=player_id,
            display_name=display_name,
            pos_x=position[0],
            pos_y=position[1],
            pos_z=position[2],
        ))

    @staticmethod
    def encode_player_leave(
        entity_id: int,
        reason: int = PL_DISCONNECT,
    ) -> bytes:
        """Encode a PLAYER_LEFT broadcast frame."""
        return encode_player_left(entity_id, reason)
