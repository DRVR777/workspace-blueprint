"""
Centralized configuration for the NEXUS node-manager (Phase 0).

All tunable constants live here. No magic numbers in other modules.
Values come from environment variables with sensible defaults matching
the MANIFEST.md performance targets and ADR decisions.

Environment variable overrides:
  NEXUS_TICK_RATE          -> ticks per second (default: 50)
  NEXUS_MAX_TICK_DT        -> max dt cap in seconds (default: 0.050)
  NEXUS_HIGH_LOAD_MS       -> tick budget warning threshold in ms (default: 20)
  NEXUS_LOAD_GRACE_TICKS   -> consecutive over-budget ticks before split request (default: 50)
  NEXUS_DEFAULT_HOST       -> bind host (default: localhost)
  NEXUS_DEFAULT_PORT       -> bind port (default: 9000)
  NEXUS_MAX_ENTITIES       -> max simultaneous entities per node (default: 10000)
  NEXUS_MAX_CLIENTS        -> max simultaneous clients per node (default: 200)
  NEXUS_VISIBILITY_RADIUS  -> default client visibility radius (default: 200.0)
  NEXUS_HANDSHAKE_TIMEOUT  -> seconds to wait for handshake (default: 10.0)
  NEXUS_WORLD_SEED         -> world seed (default: 0xDEADBEEFCAFEBABE)
  NEXUS_TICKER_LOG_PATH    -> path for ticker log file (default: ticker.jsonl)
  NEXUS_TICK_METRICS_PATH  -> path for tick metrics log (default: tick_metrics.jsonl)
"""

import os


def _env_int(key: str, default: int) -> int:
    """Read an integer from environment, falling back to default."""
    val = os.environ.get(key)
    return int(val) if val is not None else default


def _env_float(key: str, default: float) -> float:
    """Read a float from environment, falling back to default."""
    val = os.environ.get(key)
    return float(val) if val is not None else default


def _env_str(key: str, default: str) -> str:
    """Read a string from environment, falling back to default."""
    return os.environ.get(key, default)


# ---------------------------------------------------------------------------
# Tick loop timing — MANIFEST.md §TICK LOOP
# ---------------------------------------------------------------------------

TICK_RATE: int = _env_int("NEXUS_TICK_RATE", 50)
"""Target ticks per second (50 Hz = 20ms per tick)."""

TARGET_TICK_DURATION: float = 1.0 / TICK_RATE
"""Seconds per tick (derived from TICK_RATE)."""

MAX_TICK_DT: float = _env_float("NEXUS_MAX_TICK_DT", 0.050)
"""Maximum delta-time cap in seconds — prevents spiral of death."""

HIGH_LOAD_THRESHOLD_MS: float = _env_float("NEXUS_HIGH_LOAD_MS", 20.0)
"""Tick duration (ms) above which a warning is logged."""

LOAD_GRACE_TICKS: int = _env_int("NEXUS_LOAD_GRACE_TICKS", 50)
"""Consecutive over-budget ticks before requesting a node split (Phase 2+)."""

# ---------------------------------------------------------------------------
# Network
# ---------------------------------------------------------------------------

DEFAULT_HOST: str = _env_str("NEXUS_DEFAULT_HOST", "localhost")
"""Default WebSocket bind host."""

DEFAULT_PORT: int = _env_int("NEXUS_DEFAULT_PORT", 9000)
"""Default WebSocket bind port."""

HANDSHAKE_TIMEOUT_S: float = _env_float("NEXUS_HANDSHAKE_TIMEOUT", 10.0)
"""Seconds to wait for client HANDSHAKE before disconnecting."""

# ---------------------------------------------------------------------------
# Capacity
# ---------------------------------------------------------------------------

MAX_ENTITIES: int = _env_int("NEXUS_MAX_ENTITIES", 10_000)
"""Maximum simultaneous entities per node (hard cap)."""

MAX_CLIENTS: int = _env_int("NEXUS_MAX_CLIENTS", 200)
"""Maximum simultaneous client connections per node."""

# ---------------------------------------------------------------------------
# Visibility
# ---------------------------------------------------------------------------

DEFAULT_VISIBILITY_RADIUS: float = _env_float("NEXUS_VISIBILITY_RADIUS", 200.0)
"""Default radius (world units) for EPU broadcast filtering.
Phase 0 broadcasts all-to-all; this is used in Phase 1+."""

# ---------------------------------------------------------------------------
# World
# ---------------------------------------------------------------------------

WORLD_SEED: int = _env_int("NEXUS_WORLD_SEED", 0xDEADBEEFCAFEBABE)
"""Deterministic seed for world generation."""

SERVER_VERSION: int = 1
"""Protocol version reported in HANDSHAKE_RESPONSE."""

# ---------------------------------------------------------------------------
# Domain — Phase 0 hardcoded (CONTEXT.md Step 1)
# ---------------------------------------------------------------------------

DOMAIN_SIZE: float = 1000.0
"""Side length of the Phase 0 domain cube."""

DOMAIN_MIN: tuple[float, float, float] = (0.0, 0.0, 0.0)
"""Minimum corner of the node's spatial domain."""

DOMAIN_MAX: tuple[float, float, float] = (DOMAIN_SIZE, DOMAIN_SIZE, DOMAIN_SIZE)
"""Maximum corner of the node's spatial domain."""

# ---------------------------------------------------------------------------
# Entity defaults
# ---------------------------------------------------------------------------

DEFAULT_SPAWN_POSITION: tuple[float, float, float] = (500.0, 0.0, 500.0)
"""Default spawn position for new players (center of domain)."""

ENTITY_BOUNDING_BOX: tuple[float, float, float] = (0.5, 1.0, 0.5)
"""Default half-extents for a player entity's bounding box."""

# ---------------------------------------------------------------------------
# Logging / metrics
# ---------------------------------------------------------------------------

TICKER_LOG_PATH: str = _env_str("NEXUS_TICKER_LOG_PATH", "ticker.jsonl")
"""Path for the ticker event log (JSONL format)."""

TICK_METRICS_LOG_PATH: str = _env_str("NEXUS_TICK_METRICS_PATH", "tick_metrics.jsonl")
"""Path for tick performance metrics log (JSONL format)."""

METRICS_FLUSH_INTERVAL_TICKS: int = _env_int("NEXUS_METRICS_FLUSH_INTERVAL", 50)
"""Flush tick metrics to disk every N ticks."""
