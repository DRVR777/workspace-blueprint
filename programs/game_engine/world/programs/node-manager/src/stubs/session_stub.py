"""
Phase 0 stub: accepts all auth tokens with length >= 4.

Derives a stable player_id from the token bytes so the same token always
produces the same session. Replace with real auth service call when built.

Interface matches player-session-contract.md exactly.
"""

import struct
from dataclasses import dataclass


@dataclass
class SessionRecord:
    """Mirrors session_record shape from player-session-contract.md."""
    player_id: int
    display_name: str
    last_position: tuple[float, float, float]
    auth_level: int = 0   # 0=player, 1=moderator, 2=admin


class SessionStub:
    def __init__(self) -> None:
        self._sessions: dict[int, SessionRecord] = {}
        self._positions: dict[int, tuple[float, float, float]] = {}
        self._counter = 1

    def validate_token(self, auth_token: bytes) -> SessionRecord | None:
        """Returns session_record on success, None on REJECTED."""
        if len(auth_token) < 4:
            return None
        # Derive stable player_id from first 8 bytes of token
        padded = (auth_token + b"\x00" * 8)[:8]
        player_id = struct.unpack(">Q", padded)[0]
        if player_id == 0:
            player_id = self._counter
            self._counter += 1

        if player_id not in self._sessions:
            self._sessions[player_id] = SessionRecord(
                player_id=player_id,
                display_name=f"Player_{player_id & 0xFFFF:04X}",
                last_position=self._positions.get(player_id, (500.0, 0.0, 500.0)),
            )
        else:
            # Restore persisted position if available
            if player_id in self._positions:
                self._sessions[player_id].last_position = self._positions[player_id]

        return self._sessions[player_id]

    def update_last_position(self, player_id: int,
                              position: tuple[float, float, float]) -> None:
        """Called by node on disconnect or shutdown."""
        self._positions[player_id] = position
        if player_id in self._sessions:
            self._sessions[player_id].last_position = position

    def get_session(self, player_id: int) -> SessionRecord | None:
        return self._sessions.get(player_id)
