"""
Tick metrics collector and logger for the NEXUS node-manager.

Records per-tick performance data (duration, entity count, input queue depth)
and periodically flushes to a JSONL log file. This data feeds the self-monitor
system (Phase F in the tick loop).

Phase 0: writes to a local file. Phase 2+: feeds into the orchestration
controller for load-based node splitting decisions.

References:
  - MANIFEST.md §TICK LOOP Phase F: self-monitor
  - CONTEXT.md Step 3 Phase F: record tick duration, log warning if > 20ms
"""

from __future__ import annotations

import json
import logging
import os
import time
from dataclasses import dataclass, field, asdict

import config

logger = logging.getLogger("tick_metrics")


@dataclass
class TickSample:
    """One tick's worth of performance data."""
    tick_number: int
    duration_ms: float
    entity_count: int
    client_count: int
    input_queue_depth: int
    state_changes_count: int
    timestamp_ms: int = 0

    def __post_init__(self) -> None:
        if self.timestamp_ms == 0:
            self.timestamp_ms = int(time.time() * 1000)


class TickMetrics:
    """
    Collects tick performance samples and flushes to disk periodically.

    Usage in tick loop:
        metrics.record(tick_number, duration_ms, entity_count, ...)
        # Automatically flushes every METRICS_FLUSH_INTERVAL_TICKS
    """

    def __init__(
        self,
        log_path: str = config.TICK_METRICS_LOG_PATH,
        flush_interval: int = config.METRICS_FLUSH_INTERVAL_TICKS,
    ) -> None:
        self._log_path = log_path
        self._flush_interval = flush_interval
        self._buffer: list[TickSample] = []
        self._all_durations: list[float] = []  # kept in memory for stats
        self._over_budget_count: int = 0
        self._consecutive_over_budget: int = 0

        # Ensure log directory exists
        log_dir = os.path.dirname(os.path.abspath(log_path))
        if log_dir:
            os.makedirs(log_dir, exist_ok=True)

    def record(
        self,
        tick_number: int,
        duration_ms: float,
        entity_count: int = 0,
        client_count: int = 0,
        input_queue_depth: int = 0,
        state_changes_count: int = 0,
    ) -> None:
        """
        Record a single tick's metrics. Logs a warning if over budget.
        Automatically flushes to disk at the configured interval.
        """
        sample = TickSample(
            tick_number=tick_number,
            duration_ms=duration_ms,
            entity_count=entity_count,
            client_count=client_count,
            input_queue_depth=input_queue_depth,
            state_changes_count=state_changes_count,
        )
        self._buffer.append(sample)
        self._all_durations.append(duration_ms)

        # Phase F — self-monitor: log warning if over budget
        if duration_ms > config.HIGH_LOAD_THRESHOLD_MS:
            self._over_budget_count += 1
            self._consecutive_over_budget += 1
            logger.warning(
                "Tick %d over budget: %.2fms (threshold: %.0fms, "
                "consecutive: %d)",
                tick_number, duration_ms, config.HIGH_LOAD_THRESHOLD_MS,
                self._consecutive_over_budget,
            )
            # Phase 0: log only. Phase 2+: request split after grace period
            if self._consecutive_over_budget > config.LOAD_GRACE_TICKS:
                logger.error(
                    "Tick %d: %d consecutive over-budget ticks — "
                    "split would be requested in Phase 2+",
                    tick_number, self._consecutive_over_budget,
                )
        else:
            self._consecutive_over_budget = 0

        # Periodic flush
        if len(self._buffer) >= self._flush_interval:
            self.flush()

    def flush(self) -> None:
        """Write buffered samples to the log file and clear the buffer."""
        if not self._buffer:
            return
        try:
            with open(self._log_path, "a") as f:
                for sample in self._buffer:
                    f.write(json.dumps(asdict(sample)) + "\n")
            self._buffer.clear()
        except OSError as exc:
            logger.error("Failed to flush tick metrics: %s", exc)

    def get_stats(self) -> dict:
        """
        Return aggregate tick statistics.
        Used by get_tick_stats() on NodeManager and for the phase0-complete report.
        """
        if not self._all_durations:
            return {
                "tick_count": 0,
                "avg_ms": 0.0,
                "max_ms": 0.0,
                "min_ms": 0.0,
                "over_budget": 0,
            }
        return {
            "tick_count": len(self._all_durations),
            "avg_ms": sum(self._all_durations) / len(self._all_durations),
            "max_ms": max(self._all_durations),
            "min_ms": min(self._all_durations),
            "over_budget": self._over_budget_count,
        }

    def reset(self) -> None:
        """Reset all metrics (for testing)."""
        self._buffer.clear()
        self._all_durations.clear()
        self._over_budget_count = 0
        self._consecutive_over_budget = 0
