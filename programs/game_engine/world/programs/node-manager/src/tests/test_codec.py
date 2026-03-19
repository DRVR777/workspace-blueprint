"""
Unit tests for the codec module.

Tests encode/decode round-trips for all message types to ensure
the wire protocol is consistent.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

import codec


class TestHandshakeCodec:
    """Tests for HANDSHAKE encode/decode."""

    def test_round_trip(self):
        """Encoding then decoding should reproduce the original."""
        original = codec.Handshake(
            client_version=1, player_id=42,
            auth_token=b"token_123456789012345678901234",
            gpu_caps=0xFF,
        )
        data = codec.encode_handshake(original)
        decoded = codec.decode_handshake(data)
        assert decoded.client_version == original.client_version
        assert decoded.player_id == original.player_id
        assert decoded.gpu_caps == original.gpu_caps


class TestHandshakeResponseCodec:
    """Tests for HANDSHAKE_RESPONSE encode/decode."""

    def test_accepted_round_trip(self):
        """Accepted response should round-trip correctly."""
        original = codec.HandshakeResponse(
            status=codec.HS_ACCEPTED, entity_id=7,
            pos_x=100.0, pos_y=0.0, pos_z=200.0,
        )
        data = codec.encode_handshake_response(original)
        decoded = codec.decode_handshake_response(data)
        assert decoded.status == codec.HS_ACCEPTED
        assert decoded.entity_id == 7
        assert abs(decoded.pos_x - 100.0) < 0.01
        assert abs(decoded.pos_z - 200.0) < 0.01

    def test_rejected_round_trip(self):
        """Rejected response should round-trip correctly."""
        original = codec.HandshakeResponse(status=codec.HS_REJECTED)
        data = codec.encode_handshake_response(original)
        decoded = codec.decode_handshake_response(data)
        assert decoded.status == codec.HS_REJECTED


class TestPlayerActionCodec:
    """Tests for PLAYER_ACTION encode/decode."""

    def test_move_round_trip(self):
        """MOVE action should round-trip with correct payload."""
        payload = codec.encode_move_payload(50.0, 10.0, 75.0)
        original = codec.PlayerAction(
            action_type=codec.ACTION_MOVE,
            sequence_number=42,
            requires_ack=True,
            payload=payload,
        )
        data = codec.encode_player_action(original)
        decoded = codec.decode_player_action(data)
        assert decoded.action_type == codec.ACTION_MOVE
        assert decoded.sequence_number == 42
        assert decoded.requires_ack is True

        x, y, z = codec.decode_move_payload(decoded.payload)
        assert abs(x - 50.0) < 0.01
        assert abs(y - 10.0) < 0.01
        assert abs(z - 75.0) < 0.01


class TestEntityPositionUpdateCodec:
    """Tests for ENTITY_POSITION_UPDATE encode/decode."""

    def test_single_entity_round_trip(self):
        """Single entity EPU should round-trip with full-precision position."""
        entities = [codec.EntityState(
            entity_id=1, pos_x=100.0, pos_y=50.0, pos_z=200.0,
        )]
        data = codec.encode_entity_position_update(entities, seq=10)
        decoded = codec.decode_entity_position_update(data)
        assert len(decoded) == 1
        assert decoded[0].entity_id == 1
        assert abs(decoded[0].pos_x - 100.0) < 0.01
        assert abs(decoded[0].pos_y - 50.0) < 0.01
        assert abs(decoded[0].pos_z - 200.0) < 0.01

    def test_multiple_entities(self):
        """Multiple entity EPU should preserve all entries."""
        entities = [
            codec.EntityState(entity_id=1, pos_x=10.0, pos_y=0.0, pos_z=20.0),
            codec.EntityState(entity_id=2, pos_x=30.0, pos_y=0.0, pos_z=40.0),
            codec.EntityState(entity_id=3, pos_x=50.0, pos_y=0.0, pos_z=60.0),
        ]
        data = codec.encode_entity_position_update(entities)
        decoded = codec.decode_entity_position_update(data)
        assert len(decoded) == 3
        assert {e.entity_id for e in decoded} == {1, 2, 3}

    def test_empty_update(self):
        """Empty EPU should decode to empty list."""
        data = codec.encode_entity_position_update([])
        decoded = codec.decode_entity_position_update(data)
        assert decoded == []


class TestPlayerJoinedCodec:
    """Tests for PLAYER_JOINED encode/decode."""

    def test_round_trip(self):
        """PLAYER_JOINED should round-trip with all fields."""
        original = codec.PlayerJoinedMsg(
            entity_id=5, player_id=123,
            display_name="TestPlayer",
            pos_x=100.0, pos_y=0.0, pos_z=200.0,
        )
        data = codec.encode_player_joined(original)
        decoded = codec.decode_player_joined(data)
        assert decoded.entity_id == 5
        assert decoded.player_id == 123
        assert decoded.display_name == "TestPlayer"
        assert abs(decoded.pos_x - 100.0) < 0.01


class TestPlayerLeftCodec:
    """Tests for PLAYER_LEFT encode/decode."""

    def test_disconnect_round_trip(self):
        """PLAYER_LEFT with DISCONNECT reason should round-trip."""
        data = codec.encode_player_left(7, codec.PL_DISCONNECT)
        _, eid, reason = codec.decode_player_left(data)
        assert eid == 7
        assert reason == codec.PL_DISCONNECT


class TestPeekMsgType:
    """Tests for peek_msg_type utility."""

    def test_peek_handshake(self):
        """peek_msg_type should identify HANDSHAKE messages."""
        hs = codec.Handshake(1, 0, b"x" * 32, 0)
        data = codec.encode_handshake(hs)
        assert codec.peek_msg_type(data) == codec.MSG_HANDSHAKE

    def test_peek_too_short_raises(self):
        """peek_msg_type should raise on data shorter than 2 bytes."""
        import pytest
        with pytest.raises(ValueError, match="too short"):
            codec.peek_msg_type(b"\x00")
