"""
Unit tests for tick loop timing and the tick metrics system.

Tests the fixed-timestep tick behavior, metric recording, and
over-budget detection without needing a live WebSocket server.
"""

import asyncio
import os
import sys
import time

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

import config
from tick_metrics import TickMetrics, TickSample
from node_manager import NodeManager
from stubs.session_stub import SessionStub
from stubs.ticker_log_stub import TickerLogStub


class TestTickMetrics:
    """Tests for the TickMetrics collector."""

    def test_record_and_stats(self):
        """Recording samples should produce correct aggregate statistics."""
        metrics = TickMetrics(log_path="test_tm.jsonl", flush_interval=1000)
        metrics.record(tick_number=0, duration_ms=5.0)
        metrics.record(tick_number=1, duration_ms=10.0)
        metrics.record(tick_number=2, duration_ms=15.0)

        stats = metrics.get_stats()
        assert stats["tick_count"] == 3
        assert abs(stats["avg_ms"] - 10.0) < 0.01
        assert stats["max_ms"] == 15.0
        assert stats["min_ms"] == 5.0
        assert stats["over_budget"] == 0

    def test_over_budget_detection(self):
        """Ticks exceeding HIGH_LOAD_THRESHOLD_MS should be counted."""
        metrics = TickMetrics(log_path="test_tm.jsonl", flush_interval=1000)
        metrics.record(tick_number=0, duration_ms=5.0)
        metrics.record(tick_number=1, duration_ms=25.0)  # over budget
        metrics.record(tick_number=2, duration_ms=30.0)  # over budget
        metrics.record(tick_number=3, duration_ms=10.0)

        stats = metrics.get_stats()
        assert stats["over_budget"] == 2

    def test_empty_stats(self):
        """Stats on a fresh metrics object should return zeros."""
        metrics = TickMetrics(log_path="test_tm.jsonl", flush_interval=1000)
        stats = metrics.get_stats()
        assert stats["tick_count"] == 0
        assert stats["avg_ms"] == 0.0

    def test_reset(self):
        """reset() should clear all accumulated data."""
        metrics = TickMetrics(log_path="test_tm.jsonl", flush_interval=1000)
        metrics.record(tick_number=0, duration_ms=10.0)
        metrics.reset()
        stats = metrics.get_stats()
        assert stats["tick_count"] == 0

    def test_flush_writes_to_file(self, tmp_path):
        """flush() should write buffered samples to disk as JSONL."""
        log_file = str(tmp_path / "metrics.jsonl")
        metrics = TickMetrics(log_path=log_file, flush_interval=1000)
        metrics.record(tick_number=0, duration_ms=5.0)
        metrics.record(tick_number=1, duration_ms=10.0)
        metrics.flush()

        with open(log_file) as f:
            lines = f.readlines()
        assert len(lines) == 2

    def test_auto_flush_at_interval(self, tmp_path):
        """Buffer should auto-flush when reaching flush_interval."""
        log_file = str(tmp_path / "metrics.jsonl")
        metrics = TickMetrics(log_path=log_file, flush_interval=3)
        metrics.record(tick_number=0, duration_ms=1.0)
        metrics.record(tick_number=1, duration_ms=2.0)
        # File should not exist or be empty yet
        metrics.record(tick_number=2, duration_ms=3.0)  # triggers flush

        with open(log_file) as f:
            lines = f.readlines()
        assert len(lines) == 3


class TestTickSample:
    """Tests for the TickSample dataclass."""

    def test_auto_timestamp(self):
        """TickSample should auto-set timestamp_ms if not provided."""
        before = int(time.time() * 1000)
        sample = TickSample(
            tick_number=0, duration_ms=5.0,
            entity_count=10, client_count=2,
            input_queue_depth=0, state_changes_count=0,
        )
        after = int(time.time() * 1000)
        assert before <= sample.timestamp_ms <= after

    def test_explicit_timestamp(self):
        """Explicit timestamp_ms should be preserved."""
        sample = TickSample(
            tick_number=0, duration_ms=5.0,
            entity_count=0, client_count=0,
            input_queue_depth=0, state_changes_count=0,
            timestamp_ms=12345,
        )
        assert sample.timestamp_ms == 12345


class TestTickLoopTiming:
    """Tests for the NodeManager tick loop timing accuracy."""

    @pytest.fixture
    async def node(self):
        """Create a NodeManager and run it briefly to measure tick timing."""
        session = SessionStub()
        log = TickerLogStub(log_path="test_ticker.jsonl")
        metrics = TickMetrics(log_path="test_tick_metrics.jsonl", flush_interval=1000)
        nm = NodeManager(
            host="localhost", port=9902,
            session=session, ticker_log=log, tick_metrics=metrics,
        )
        task = asyncio.create_task(nm.run())
        await asyncio.sleep(0.15)  # let server start
        yield nm
        nm._running = False
        task.cancel()
        try:
            await asyncio.wait_for(task, timeout=1.0)
        except (asyncio.CancelledError, asyncio.TimeoutError):
            pass

    async def test_tick_rate_approximately_50hz(self, node):
        """The tick loop should run at approximately 50 Hz (20ms per tick)."""
        # Let it run for ~0.5 seconds
        await asyncio.sleep(0.5)
        stats = node.get_tick_stats()
        tick_count = stats["tick_count"]

        # At 50 Hz, 0.5s should yield ~25 ticks. Allow +/- 30% tolerance.
        assert tick_count >= 17, (
            f"Too few ticks: {tick_count} (expected ~25 in 0.5s at 50Hz)"
        )
        assert tick_count <= 35, (
            f"Too many ticks: {tick_count} (expected ~25 in 0.5s at 50Hz)"
        )

    async def test_tick_durations_under_budget(self, node):
        """All tick durations should be under the 20ms budget (with no load)."""
        await asyncio.sleep(0.3)
        stats = node.get_tick_stats()
        assert stats["tick_count"] > 0, "No ticks recorded"
        assert stats["over_budget"] == 0, (
            f"Over-budget ticks: {stats['over_budget']} "
            f"(max={stats['max_ms']:.2f}ms)"
        )

    async def test_avg_tick_under_5ms_idle(self, node):
        """Average tick duration with no clients should be well under 5ms."""
        await asyncio.sleep(0.3)
        stats = node.get_tick_stats()
        assert stats["avg_ms"] < 5.0, (
            f"Average tick too slow: {stats['avg_ms']:.2f}ms"
        )
