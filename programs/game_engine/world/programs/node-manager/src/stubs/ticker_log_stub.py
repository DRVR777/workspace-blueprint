"""
Phase 0 stub: writes ticker entries to a local JSONL file.

Per CONTEXT.md Step 3 Phase E: "stub for Phase 0: write to local file,
not distributed log". Interface matches ticker-log-contract.md append/append_batch.
"""

import json
import os
import time
from dataclasses import dataclass, field, asdict

# Event type codes (from ticker-log-contract.md Event Type Registry)
EVT_OBJECT_CREATED    = 0x0001
EVT_OBJECT_DESTROYED  = 0x0002
EVT_POSITION_CHANGED  = 0x0003
EVT_PROPERTY_CHANGED  = 0x0004
EVT_ENTITY_SPAWNED    = 0x0007
EVT_ENTITY_DESPAWNED  = 0x0008

# Source type codes
SRC_PLAYER = 0
SRC_NODE   = 1
SRC_SYSTEM = 2
SRC_AGENT  = 3


@dataclass
class TickerEntry:
    """Mirrors ticker_entry shape from ticker-log-contract.md."""
    sequence:      int   = 0
    timestamp_us:  int   = 0
    object_id:     int   = 0
    event_type:    int   = EVT_POSITION_CHANGED
    source_type:   int   = SRC_PLAYER
    source_id:     int   = 0
    payload:       dict  = field(default_factory=dict)

    def __post_init__(self) -> None:
        if self.timestamp_us == 0:
            self.timestamp_us = int(time.time() * 1_000_000)


class TickerLogStub:
    def __init__(self, log_path: str = "ticker.jsonl") -> None:
        self._log_path = log_path
        self._sequence = 0
        os.makedirs(os.path.dirname(os.path.abspath(log_path)), exist_ok=True)

    def append(self, entry: TickerEntry) -> int:
        self._sequence += 1
        entry.sequence = self._sequence
        with open(self._log_path, "a") as f:
            f.write(json.dumps(asdict(entry)) + "\n")
        return self._sequence

    def append_batch(self, entries: list[TickerEntry]) -> list[int]:
        return [self.append(e) for e in entries]

    def get_latest_sequence(self) -> int:
        return self._sequence
