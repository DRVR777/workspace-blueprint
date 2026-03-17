#!/usr/bin/env python3
"""
watcher.py — Workspace Metacognition Engine

Real-time observer for workspace-blueprint. Detects what the 3 active agents
are doing, measures fractal architectural health, and surfaces optimization
signals so the workspace can shape itself.

Usage:
    python watcher.py [workspace_root]
    python watcher.py --no-dashboard      # log-only mode

Output (written to programs/watcher/output/):
    LIVE_STATE.md    — agent-readable live state (refreshes every 30s)
    PATTERNS.md      — detected architectural patterns (every 5m)
    META_COGNITION.md — workspace self-model narrative (every 10m)
    HEALTH.json      — machine-readable metrics (every 30s)
"""

import sys
import json
import time
import threading
import argparse
from pathlib import Path
from datetime import datetime, timezone
from dataclasses import dataclass
from collections import deque, Counter, defaultdict
from typing import Optional

try:
    from watchdog.observers import Observer
    from watchdog.events import FileSystemEventHandler
except ImportError:
    print("ERROR: watchdog not installed. Run: pip install watchdog rich")
    sys.exit(1)

try:
    from rich.console import Console
    from rich.live import Live
    from rich.layout import Layout
    from rich.panel import Panel
    from rich.table import Table
    from rich.text import Text
    from rich import box
except ImportError:
    print("ERROR: rich not installed. Run: pip install watchdog rich")
    sys.exit(1)


# ──────────────────────────────────────────────────────────────────────────────
# CONSTANTS
# ──────────────────────────────────────────────────────────────────────────────

NAVIGATION_TRINITY = {"MANIFEST.md", "CLAUDE.md", "CONTEXT.md"}

# Map agent label → folder path prefix (relative to workspace root)
AGENT_DOMAINS = {
    "oracle-agent":     "programs/oracle",
    "game-agent":       "programs/game_engine",
    "kg-agent":         "programs/knowledge-graph",
    "meta-agent":       "_meta",
    "workspace-agent":  "programs/workspace-builder",
}

# Directories to skip entirely during tree walks
SKIP_DIRS = {
    "__pycache__", ".git", "node_modules", ".pytest_cache",
    ".mypy_cache", ".venv", "venv", ".tox",
}

# File path fragments to ignore for event recording (watcher's own output)
IGNORE_PATH_FRAGMENTS = {
    "programs/watcher/output",
    ".watcher_state.json",
    "programs\\watcher\\output",  # windows backslash variant
}

MAX_EVENTS        = 300     # ring buffer size
LIVE_INTERVAL     = 2.0     # dashboard refresh seconds
STATE_INTERVAL    = 30      # LIVE_STATE.md write cadence seconds
PATTERN_INTERVAL  = 300     # PATTERNS.md write cadence seconds
META_INTERVAL     = 600     # META_COGNITION.md write cadence seconds
SCAN_INTERVAL     = 90      # fractal re-scan cadence seconds


# ──────────────────────────────────────────────────────────────────────────────
# DATA MODELS
# ──────────────────────────────────────────────────────────────────────────────

@dataclass
class ChangeEvent:
    ts:         str    # ISO-8601 timestamp
    event_type: str    # created | modified | deleted | moved
    rel_path:   str    # path relative to workspace root (forward slashes)
    agent:      str    # inferred owning agent
    file_type:  str    # manifest | claude | context | code | planning | gap | schema | doc | other
    dest_path:  str = ""  # populated for moved events


@dataclass
class FractalNode:
    path:         str    # relative path (forward slashes)
    depth:        int
    has_manifest: bool
    has_claude:   bool
    has_context:  bool
    child_count:  int    # number of sub-directories

    @property
    def completeness(self) -> float:
        return sum([self.has_manifest, self.has_claude, self.has_context]) / 3.0

    @property
    def is_complete(self) -> bool:
        return self.has_manifest and self.has_claude and self.has_context


# ──────────────────────────────────────────────────────────────────────────────
# EVENT HANDLER
# ──────────────────────────────────────────────────────────────────────────────

class WorkspaceEventHandler(FileSystemEventHandler):
    """Intercepts watchdog events, classifies them, stores in ring buffer."""

    def __init__(self, root: Path):
        super().__init__()
        self.root = root
        self.events: deque[ChangeEvent] = deque(maxlen=MAX_EVENTS)
        self.event_count  = Counter()   # event_type → total
        self.agent_counts = Counter()   # agent      → total
        self.file_type_counts = Counter()
        self._lock = threading.Lock()
        self._start = datetime.now(timezone.utc)

    # ── classification helpers ─────────────────────────────────────────────

    def _should_ignore(self, rel: str) -> bool:
        parts = Path(rel).parts
        for part in parts:
            if part in SKIP_DIRS or part.startswith('.'):
                return True
        # ignore watcher's own output
        norm = rel.replace("\\", "/")
        for frag in IGNORE_PATH_FRAGMENTS:
            if frag.replace("\\", "/") in norm:
                return True
        return False

    def _classify_file(self, rel: str) -> str:
        name = Path(rel).name
        if name == "MANIFEST.md":               return "manifest"
        if name == "CLAUDE.md":                 return "claude"
        if name == "CONTEXT.md":                return "context"
        if name == "pending.txt":               return "inference-log"
        if "/gaps/" in rel or "gap-" in name:   return "gap"
        if "/_planning/" in rel or "/adr/" in rel: return "planning"
        if name.endswith((".py", ".ts", ".rs", ".go", ".js", ".cpp", ".c", ".h")):
            return "code"
        if name.endswith((".json", ".fbs", ".proto", ".yaml", ".toml")):
            return "schema"
        if name.endswith(".md"):
            return "doc"
        return "other"

    def _infer_agent(self, rel: str) -> str:
        norm = rel.replace("\\", "/")
        for agent, domain in AGENT_DOMAINS.items():
            if norm.startswith(domain):
                return agent
        if norm.startswith("_meta") or norm.startswith("_registry") or norm.startswith("_core"):
            return "meta-agent"
        if norm.startswith("_intake"):
            return "intake-agent"
        return "unknown-agent"

    # ── event construction ─────────────────────────────────────────────────

    def _build_event(self, event_type: str, src: str, dest: str = "") -> Optional[ChangeEvent]:
        try:
            rel = Path(src).relative_to(self.root).as_posix()
        except ValueError:
            return None
        if self._should_ignore(rel):
            return None
        return ChangeEvent(
            ts=datetime.now(timezone.utc).isoformat(timespec="seconds"),
            event_type=event_type,
            rel_path=rel,
            agent=self._infer_agent(rel),
            file_type=self._classify_file(rel),
            dest_path=dest,
        )

    def _record(self, e: ChangeEvent):
        with self._lock:
            self.events.append(e)
            self.event_count[e.event_type] += 1
            self.agent_counts[e.agent] += 1
            self.file_type_counts[e.file_type] += 1

    # ── watchdog callbacks ─────────────────────────────────────────────────

    def on_created(self, event):
        if event.is_directory: return
        e = self._build_event("created", event.src_path)
        if e: self._record(e)

    def on_modified(self, event):
        if event.is_directory: return
        e = self._build_event("modified", event.src_path)
        if e: self._record(e)

    def on_deleted(self, event):
        if event.is_directory: return
        e = self._build_event("deleted", event.src_path)
        if e: self._record(e)

    def on_moved(self, event):
        if event.is_directory: return
        try:
            dest_rel = Path(event.dest_path).relative_to(self.root).as_posix()
        except ValueError:
            dest_rel = event.dest_path
        e = self._build_event("moved", event.src_path, dest_rel)
        if e: self._record(e)

    # ── accessors ─────────────────────────────────────────────────────────

    def recent(self, n: int = 25) -> list[ChangeEvent]:
        with self._lock:
            return list(reversed(list(self.events)))[:n]

    def uptime_seconds(self) -> float:
        return (datetime.now(timezone.utc) - self._start).total_seconds()

    def total_events(self) -> int:
        return sum(self.event_count.values())


# ──────────────────────────────────────────────────────────────────────────────
# FRACTAL ANALYZER
# ──────────────────────────────────────────────────────────────────────────────

class FractalAnalyzer:
    """
    Measures how completely the workspace describes itself as a fractal.

    The workspace IS a fractal: every routing directory repeats the same
    3-file navigation trinity (MANIFEST / CLAUDE / CONTEXT). Completeness
    of this pattern determines how well agents can navigate without confusion.

    Leaf directories with no subdirectories and < 4 files are exempt from
    the CLAUDE.md requirement (they have nothing to map).
    """

    def __init__(self, root: Path):
        self.root = root
        self._nodes: list[FractalNode] = []
        self._last_scan = 0.0
        self._lock = threading.Lock()

    def scan(self, force: bool = False) -> list[FractalNode]:
        now = time.time()
        if not force and (now - self._last_scan) < SCAN_INTERVAL:
            return self._nodes

        nodes = []
        for dirpath in sorted(self.root.rglob("*")):
            try:
                if not dirpath.is_dir():
                    continue
            except OSError:
                continue
            # skip ignored dirs
            if any(part in SKIP_DIRS or part.startswith('.') for part in dirpath.parts):
                continue
            try:
                rel = dirpath.relative_to(self.root)
            except ValueError:
                continue

            contents    = list(dirpath.iterdir()) if dirpath.exists() else []
            file_names  = {f.name for f in contents if f.is_file()}
            child_dirs  = sum(1 for f in contents if f.is_dir() and f.name not in SKIP_DIRS)

            nodes.append(FractalNode(
                path=rel.as_posix(),
                depth=len(rel.parts),
                has_manifest="MANIFEST.md" in file_names,
                has_claude="CLAUDE.md" in file_names,
                has_context="CONTEXT.md" in file_names,
                child_count=child_dirs,
            ))

        with self._lock:
            self._nodes = nodes
            self._last_scan = now
        return nodes

    def health(self) -> dict:
        nodes = self.scan()
        if not nodes:
            return {"score": 0.0, "total": 0, "complete": 0,
                    "missing_manifest": [], "missing_claude": [], "missing_context": []}

        # Routing nodes: have children OR are shallow (depth ≤ 2)
        routing = [n for n in nodes if n.child_count > 0 or n.depth <= 2]
        if not routing:
            return {"score": 1.0, "total": 0, "complete": 0,
                    "missing_manifest": [], "missing_claude": [], "missing_context": []}

        complete         = sum(1 for n in routing if n.is_complete)
        missing_manifest = [n.path for n in routing if not n.has_manifest]
        missing_claude   = [n.path for n in routing if n.has_manifest and not n.has_claude]
        missing_context  = [n.path for n in routing if n.has_manifest and not n.has_context]

        return {
            "score":            complete / len(routing),
            "total":            len(routing),
            "complete":         complete,
            "missing_manifest": missing_manifest[:15],
            "missing_claude":   missing_claude[:15],
            "missing_context":  missing_context[:15],
        }

    def optimization_hints(self) -> list[str]:
        h = self.health()
        hints = []
        for p in h.get("missing_manifest", [])[:4]:
            hints.append(f"ADD MANIFEST.md  →  {p}")
        for p in h.get("missing_claude", [])[:4]:
            hints.append(f"ADD CLAUDE.md    →  {p}  (agents need depth-1 map here)")
        for p in h.get("missing_context", [])[:4]:
            hints.append(f"ADD CONTEXT.md   →  {p}  (task routing missing)")
        score = h.get("score", 1.0)
        if score < 0.6:
            hints.insert(0, f"CRITICAL: fractal coverage {score:.0%} — agents navigating blind")
        elif score < 0.85:
            hints.insert(0, f"WARN: fractal coverage {score:.0%} — significant navigation gaps")
        return hints


# ──────────────────────────────────────────────────────────────────────────────
# META-COGNITION ENGINE
# ──────────────────────────────────────────────────────────────────────────────

class MetaCognitionEngine:
    """
    The workspace's self-model.

    Interprets raw event data into high-level understanding:
    - What phase is each agent currently in?
    - Where is friction occurring (high activity, no output)?
    - What is the workspace collectively doing right now?
    - What meta-patterns are emerging from agent behavior?
    """

    # File types that signal each phase
    PHASE_SIGNALS = {
        "speccing":  {"planning", "schema", "manifest", "claude", "context", "doc"},
        "building":  {"code"},
        "reviewing": {"gap", "inference-log"},
        "indexing":  {"schema", "manifest", "doc"},
    }

    def __init__(self, handler: WorkspaceEventHandler, fractal: FractalAnalyzer):
        self.handler = handler
        self.fractal = fractal

    def agent_phases(self) -> dict[str, str]:
        """Infer current phase per agent from last 60 events."""
        recent = self.handler.recent(60)
        per_agent: dict[str, Counter] = defaultdict(Counter)
        for e in recent:
            per_agent[e.agent][e.file_type] += 1

        phases = {}
        for agent, counts in per_agent.items():
            best, best_score = "active", 0
            for phase, signals in self.PHASE_SIGNALS.items():
                score = sum(counts.get(s, 0) for s in signals)
                if score > best_score:
                    best_score, best = score, phase
            phases[agent] = best if best_score > 0 else "idle"
        return phases

    def friction_zones(self) -> list[str]:
        """
        Directories with ≥3 events but zero code file creation →
        agents spending time navigating/reading but not producing.
        """
        recent = self.handler.recent(120)
        dir_activity: Counter = Counter()
        dir_code_out: Counter = Counter()

        for e in recent:
            parent = str(Path(e.rel_path).parent)
            dir_activity[parent] += 1
            if e.file_type == "code" and e.event_type == "created":
                dir_code_out[parent] += 1

        result = []
        for d, count in dir_activity.most_common(10):
            if count >= 3 and dir_code_out.get(d, 0) == 0:
                result.append(f"{d}  ({count} events, 0 code outputs → possible confusion)")
        return result[:5]

    def hot_paths(self) -> list[tuple[str, int]]:
        """Top 5 most-touched directories."""
        recent = self.handler.recent(200)
        counts: Counter = Counter()
        for e in recent:
            counts[str(Path(e.rel_path).parent)] += 1
        return counts.most_common(5)

    def workspace_narrative(self) -> str:
        """One-sentence summary of what the workspace is doing right now."""
        phases  = self.agent_phases()
        if not phases:
            return "No agent activity detected yet — watching."

        by_phase: dict[str, list[str]] = defaultdict(list)
        for agent, phase in phases.items():
            by_phase[phase].append(agent)

        parts = []
        for phase in ("building", "speccing", "reviewing", "indexing", "active"):
            agents = by_phase.get(phase, [])
            if agents:
                parts.append(f"{', '.join(agents)} {phase}")

        h     = self.fractal.health()
        score = h.get("score", 1.0)
        arch  = "architecture healthy" if score > 0.85 else f"architecture {score:.0%} complete"

        return ("; ".join(parts) + f" — {arch}.") if parts else f"Agents idle — {arch}."

    def meta_architecture_patterns(self) -> list[str]:
        """
        Detect emerging meta-patterns from agent behavior that suggest
        structural improvements to the workspace architecture.
        """
        patterns = []
        recent   = self.handler.recent(200)

        if not recent:
            return patterns

        # Pattern: multiple agents touching same directory
        dir_agents: dict[str, set] = defaultdict(set)
        for e in recent:
            dir_agents[str(Path(e.rel_path).parent)].add(e.agent)
        for d, agents in dir_agents.items():
            if len(agents) >= 2:
                patterns.append(
                    f"MULTI-AGENT CONVERGENCE: {', '.join(sorted(agents))} "
                    f"both touching `{d}` → consider a shared contract or bridge"
                )

        # Pattern: high gap creation rate
        gap_events = [e for e in recent if e.file_type == "gap"]
        if len(gap_events) >= 3:
            patterns.append(
                f"GAP ACCUMULATION: {len(gap_events)} gap events in last 200 "
                f"→ systemic spec gap, review _meta/gaps/CONTEXT.md"
            )

        # Pattern: planning files being created = spec phase active
        planning_events = [e for e in recent if e.file_type == "planning" and e.event_type == "created"]
        if len(planning_events) >= 3:
            agents = Counter(e.agent for e in planning_events)
            top = agents.most_common(1)[0]
            patterns.append(
                f"SPEC SPRINT: {top[0]} created {top[1]} planning docs "
                f"→ pre-build phase active, ensure spec-review before build"
            )

        # Pattern: schema creation = contracts being defined
        schema_events = [e for e in recent if e.file_type == "schema" and e.event_type == "created"]
        if len(schema_events) >= 2:
            patterns.append(
                f"CONTRACT DEFINITION: {len(schema_events)} schema files created "
                f"→ verify they land in shared/contracts/ not inside programs/"
            )

        return patterns[:6]


# ──────────────────────────────────────────────────────────────────────────────
# STATE WRITER  (output files agents can read)
# ──────────────────────────────────────────────────────────────────────────────

class StateWriter:
    """Periodically writes the output/ markdown + JSON files."""

    def __init__(self, root: Path, handler: WorkspaceEventHandler,
                 fractal: FractalAnalyzer, meta: MetaCognitionEngine):
        self.out     = root / "programs" / "watcher" / "output"
        self.out.mkdir(parents=True, exist_ok=True)
        self.handler = handler
        self.fractal = fractal
        self.meta    = meta
        self._t_state   = 0.0
        self._t_pattern = 0.0
        self._t_meta    = 0.0

    def tick(self):
        now = time.time()
        if now - self._t_state >= STATE_INTERVAL:
            self._write_live_state()
            self._write_health_json()
            self._t_state = now
        if now - self._t_pattern >= PATTERN_INTERVAL:
            self._write_patterns()
            self._t_pattern = now
        if now - self._t_meta >= META_INTERVAL:
            self._write_meta_cognition()
            self._t_meta = now

    def force_all(self):
        self._t_state = self._t_pattern = self._t_meta = 0.0
        self.tick()

    # ── LIVE_STATE.md ─────────────────────────────────────────────────────

    def _write_live_state(self):
        now    = datetime.now(timezone.utc).isoformat(timespec="seconds")
        recent = self.handler.recent(30)
        h      = self.fractal.health()
        hints  = self.fractal.optimization_hints()

        lines = [
            "# LIVE_STATE.md",
            f"<!-- auto-generated by watcher · {now} — DO NOT EDIT -->",
            "",
            f"**Last updated:** {now}  |  "
            f"**Total events:** {self.handler.total_events()}  |  "
            f"**Fractal health:** {h.get('score', 0):.1%}",
            "",
            "## Recent Changes (last 30)",
            "",
            "| Time (UTC) | Op | Agent | File |",
            "|------------|-----|-------|------|",
        ]
        for e in recent:
            t = e.ts.replace("T", " ")
            path = e.rel_path
            if len(path) > 60:
                parts = Path(path).parts
                path = ("…/" + "/".join(parts[-2:])) if len(parts) > 2 else path
            lines.append(f"| `{t}` | {e.event_type} | {e.agent} | `{path}` |")

        lines += [
            "",
            "## Agent Activity",
            "",
            "| Agent | Events | Phase |",
            "|-------|--------|-------|",
        ]
        phases = self.meta.agent_phases()
        for agent, count in self.handler.agent_counts.most_common():
            lines.append(f"| {agent} | {count} | {phases.get(agent, 'idle')} |")

        lines += ["", f"## Fractal Health: {h.get('score', 0):.1%}", ""]
        for hint in hints[:6]:
            lines.append(f"- `{hint}`")
        if not hints:
            lines.append("- Architecture fully self-described.")

        lines += ["", "---", f"*watcher · {now}*"]
        (self.out / "LIVE_STATE.md").write_text("\n".join(lines), encoding="utf-8")

    # ── HEALTH.json ───────────────────────────────────────────────────────

    def _write_health_json(self):
        h = self.fractal.health()
        payload = {
            "ts":               datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "fractal_score":    round(h.get("score", 0), 4),
            "total_nodes":      h.get("total", 0),
            "complete_nodes":   h.get("complete", 0),
            "missing_manifest": h.get("missing_manifest", []),
            "missing_claude":   h.get("missing_claude", []),
            "missing_context":  h.get("missing_context", []),
            "agent_counts":     dict(self.handler.agent_counts),
            "file_type_counts": dict(self.handler.file_type_counts),
            "event_totals":     dict(self.handler.event_count),
            "total_events":     self.handler.total_events(),
            "uptime_seconds":   round(self.handler.uptime_seconds(), 1),
            "narrative":        self.meta.workspace_narrative(),
        }
        (self.out / "HEALTH.json").write_text(
            json.dumps(payload, indent=2), encoding="utf-8"
        )

    # ── PATTERNS.md ───────────────────────────────────────────────────────

    def _write_patterns(self):
        now    = datetime.now(timezone.utc).isoformat(timespec="seconds")
        h      = self.fractal.health()
        phases = self.meta.agent_phases()
        meta_p = self.meta.meta_architecture_patterns()
        hot    = self.meta.hot_paths()

        lines = [
            "# PATTERNS.md",
            f"<!-- auto-generated by watcher · {now} -->",
            "",
            "## Fractal Self-Description Status",
            "",
            f"Coverage: **{h.get('score', 0):.1%}**  "
            f"({h.get('complete', 0)} / {h.get('total', 0)} routing nodes complete).",
            "",
            "Every routing directory should contain the navigation trinity:",
            "- `MANIFEST.md` — what is this directory (envelope)",
            "- `CLAUDE.md`   — depth-1 map (names + purposes only, no internals)",
            "- `CONTEXT.md`  — task router (what are you doing → go here)",
            "",
            "### Incomplete Routing Nodes",
            "",
        ]
        for p in h.get("missing_manifest", [])[:6]:
            lines.append(f"- `{p}` — **missing MANIFEST.md**")
        for p in h.get("missing_claude", [])[:6]:
            lines.append(f"- `{p}` — missing CLAUDE.md")
        for p in h.get("missing_context", [])[:6]:
            lines.append(f"- `{p}` — missing CONTEXT.md")
        if not any([h.get("missing_manifest"), h.get("missing_claude"), h.get("missing_context")]):
            lines.append("- None — fractal is fully self-described.")

        lines += [
            "",
            "## Agent Activity Patterns",
            "",
            "| Agent | Phase | Events |",
            "|-------|-------|--------|",
        ]
        all_agents = set(AGENT_DOMAINS) | set(self.handler.agent_counts)
        for agent in sorted(all_agents):
            count = self.handler.agent_counts.get(agent, 0)
            phase = phases.get(agent, "idle")
            lines.append(f"| {agent} | {phase} | {count} |")

        lines += ["", "## Hot Paths", ""]
        if hot:
            for path, count in hot:
                lines.append(f"- `{path}`  ({count} events)")
        else:
            lines.append("- No activity yet.")

        lines += ["", "## Emerging Meta-Architecture Patterns", ""]
        if meta_p:
            for p in meta_p:
                lines.append(f"- {p}")
        else:
            lines.append("- No meta-patterns detected yet.")

        lines += [
            "",
            "## Friction Zones",
            "> High agent activity but no code output — possible confusion or blocked work.",
            "",
        ]
        friction = self.meta.friction_zones()
        if friction:
            for f in friction:
                lines.append(f"- `{f}`")
        else:
            lines.append("- None detected.")

        lines += ["", "---", f"*watcher · {now}*"]
        (self.out / "PATTERNS.md").write_text("\n".join(lines), encoding="utf-8")

    # ── META_COGNITION.md ─────────────────────────────────────────────────

    def _write_meta_cognition(self):
        now       = datetime.now(timezone.utc).isoformat(timespec="seconds")
        h         = self.fractal.health()
        phases    = self.meta.agent_phases()
        narrative = self.meta.workspace_narrative()
        hints     = self.fractal.optimization_hints()
        meta_p    = self.meta.meta_architecture_patterns()
        friction  = self.meta.friction_zones()
        hot       = self.meta.hot_paths()
        uptime    = self.handler.uptime_seconds()
        total     = self.handler.total_events()

        lines = [
            "# META_COGNITION.md",
            f"<!-- auto-generated by watcher · {now} — workspace self-model -->",
            "",
            "## What The Workspace Is Doing Right Now",
            "",
            f"> {narrative}",
            "",
            "## Agent Self-Model",
            "",
            "| Agent | Phase | Events | Signal Strength |",
            "|-------|-------|--------|-----------------|",
        ]
        for agent in sorted(set(AGENT_DOMAINS) | set(phases)):
            count  = self.handler.agent_counts.get(agent, 0)
            phase  = phases.get(agent, "idle")
            signal = "strong" if count > 20 else "moderate" if count > 5 else "weak" if count > 0 else "none"
            lines.append(f"| {agent} | {phase} | {count} | {signal} |")

        score = h.get("score", 0)
        lines += [
            "",
            "## Fractal Architecture Self-Assessment",
            "",
            f"The workspace fractal is **{score:.1%}** complete  "
            f"({h.get('complete', 0)} of {h.get('total', 0)} routing nodes have all 3 navigation files).",
            "",
            "### What This Means For Agents",
            "",
        ]
        if score >= 0.9:
            lines.append("The workspace is **fully self-describing**. Agents can navigate confidently "
                         "at every depth level without inference gaps.")
        elif score >= 0.7:
            lines.append("The workspace is **mostly self-describing**. A few routing nodes are missing "
                         "navigation files. Agents may encounter brief confusion in uncovered areas.")
        else:
            lines.append("The workspace has **significant self-description gaps**. Agents are likely "
                         "spending extra tokens inferring structure. Run `fractal_complete.py --apply` "
                         "to auto-generate missing files.")

        lines += ["", "### Optimization Actions Required", ""]
        if hints:
            for hint in hints:
                lines.append(f"- `{hint}`")
        else:
            lines.append("- None. Architecture is complete.")

        lines += ["", "## Meta-Architecture Patterns", ""]
        if meta_p:
            for p in meta_p:
                lines.append(f"- {p}")
        else:
            lines.append("- No patterns detected yet — need more agent activity.")

        lines += [
            "",
            "## Friction Analysis",
            "> Locations where agents are doing work but not producing code output.",
            "",
        ]
        if friction:
            for f in friction:
                lines.append(f"- `{f}`")
        else:
            lines.append("- No friction detected.")

        lines += ["", "## Hot Paths", "> Most-touched directories this session.", ""]
        if hot:
            for path, count in hot:
                lines.append(f"- `{path}` — {count} events")
        else:
            lines.append("- No data yet.")

        lines += [
            "",
            "## Session Metrics",
            "",
            f"| Metric | Value |",
            f"|--------|-------|",
            f"| Uptime | {uptime/60:.1f} minutes |",
            f"| Total events | {total} |",
            f"| Event rate | {total/max(uptime,1):.2f} events/sec |",
            f"| Fractal coverage | {score:.1%} |",
            f"| Active agents | {sum(1 for c in self.handler.agent_counts.values() if c > 0)} |",
            "",
            "---",
            f"*Generated by workspace watcher · {now}*",
            "",
            "> This file is the workspace's self-model. It describes what the workspace",
            "> understands about itself — what agents are doing, where the architecture",
            "> is complete, and where it needs to grow. Read this at the start of any",
            "> session to orient immediately.",
        ]
        (self.out / "META_COGNITION.md").write_text("\n".join(lines), encoding="utf-8")


# ──────────────────────────────────────────────────────────────────────────────
# RICH DASHBOARD
# ──────────────────────────────────────────────────────────────────────────────

def _ev_color(et: str) -> str:
    return {"created": "green", "modified": "yellow", "deleted": "red", "moved": "cyan"}.get(et, "white")

def _agent_color(agent: str) -> str:
    return {
        "oracle-agent":    "bright_yellow",
        "game-agent":      "bright_blue",
        "kg-agent":        "bright_magenta",
        "meta-agent":      "bright_cyan",
        "workspace-agent": "bright_green",
        "intake-agent":    "bright_white",
    }.get(agent, "white")


class Dashboard:
    def __init__(self, handler: WorkspaceEventHandler, fractal: FractalAnalyzer,
                 meta: MetaCognitionEngine, root: Path):
        self.handler = handler
        self.fractal = fractal
        self.meta    = meta
        self.root    = root

    def build(self) -> Layout:
        layout = Layout()
        layout.split_column(
            Layout(name="header", size=3),
            Layout(name="body"),
            Layout(name="footer", size=3),
        )
        layout["body"].split_row(
            Layout(name="left", ratio=3),
            Layout(name="right", ratio=2),
        )
        layout["left"].split_column(
            Layout(name="changes", ratio=3),
            Layout(name="fractal", ratio=2),
        )
        layout["right"].split_column(
            Layout(name="agents", ratio=1),
            Layout(name="meta", ratio=1),
        )

        layout["header"].update(self._header())
        layout["changes"].update(self._changes())
        layout["fractal"].update(self._fractal())
        layout["agents"].update(self._agents())
        layout["meta"].update(self._meta_panel())
        layout["footer"].update(self._footer())
        return layout

    def _header(self) -> Panel:
        h      = self.fractal.health()
        score  = h.get("score", 0)
        sc     = "green" if score > 0.85 else "yellow" if score > 0.6 else "red"
        total  = self.handler.total_events()
        active = sum(1 for c in self.handler.agent_counts.values() if c > 0)
        up     = self.handler.uptime_seconds()
        up_s   = f"{up/60:.1f}m" if up < 3600 else f"{up/3600:.1f}h"

        t = Text()
        t.append("  WORKSPACE METACOGNITION WATCHER", style="bold white")
        t.append("   ·   fractal: ", style="dim")
        t.append(f"{score:.0%}", style=f"bold {sc}")
        t.append("   ·   agents: ", style="dim")
        t.append(f"{active} active", style="bold cyan")
        t.append("   ·   events: ", style="dim")
        t.append(f"{total}", style="bold white")
        t.append(f"   ·   up {up_s}", style="dim")
        return Panel(t, style="bold blue", box=box.HEAVY)

    def _changes(self) -> Panel:
        tbl = Table(box=box.SIMPLE, expand=True, show_header=True, header_style="bold dim")
        tbl.add_column("Time",  width=9,  style="dim")
        tbl.add_column("Op",    width=9)
        tbl.add_column("Agent", width=17)
        tbl.add_column("File")

        for e in self.handler.recent(20):
            t    = e.ts.split("T")[1]
            path = e.rel_path
            if len(path) > 52:
                parts = Path(path).parts
                path  = "…/" + "/".join(parts[-2:]) if len(parts) > 2 else path
            tbl.add_row(
                t,
                Text(e.event_type, style=_ev_color(e.event_type)),
                Text(e.agent,      style=_agent_color(e.agent)),
                path,
            )
        return Panel(tbl, title="[bold]LIVE CHANGES[/bold]", border_style="blue")

    def _fractal(self) -> Panel:
        h     = self.fractal.health()
        score = h.get("score", 0)
        sc    = "green" if score > 0.85 else "yellow" if score > 0.6 else "red"
        hints = self.fractal.optimization_hints()

        t = Text()
        t.append("Coverage: ", style="dim")
        t.append(f"{score:.1%}", style=f"bold {sc}")
        t.append(f"  ({h.get('complete',0)}/{h.get('total',0)} routing nodes)\n\n", style="dim")

        for label, key, icon in [
            ("MANIFEST", "missing_manifest", "✓"),
            ("CLAUDE  ", "missing_claude",   "✓"),
            ("CONTEXT ", "missing_context",  "✓"),
        ]:
            missing = h.get(key, [])
            ok   = "✓" if not missing else "✗"
            col  = "green" if not missing else "red"
            t.append(f"  {ok} {label}", style=col)
            if missing:
                t.append(f" ({len(missing)} missing)", style="dim red")
            t.append("\n")

        if hints:
            t.append("\nOptimization signals:\n", style="bold dim")
            for h_item in hints[:4]:
                t.append(f"  → {h_item}\n", style="yellow")
        else:
            t.append("\n✓ Architecture fully self-described.\n", style="bold green")

        return Panel(t, title="[bold]FRACTAL HEALTH[/bold]", border_style="cyan")

    def _agents(self) -> Panel:
        phases = self.meta.agent_phases()
        tbl = Table(box=box.SIMPLE, expand=True, show_header=True, header_style="bold dim")
        tbl.add_column("Agent",  width=18)
        tbl.add_column("Phase",  width=12)
        tbl.add_column("Events", width=7)

        for agent in sorted(set(AGENT_DOMAINS) | set(self.handler.agent_counts)):
            count = self.handler.agent_counts.get(agent, 0)
            phase = phases.get(agent, "idle")
            pcol  = {"building":"green","speccing":"yellow","reviewing":"cyan"}.get(phase,"dim")
            tbl.add_row(
                Text(agent, style=_agent_color(agent)),
                Text(phase, style=pcol),
                str(count),
            )
        return Panel(tbl, title="[bold]AGENT ACTIVITY[/bold]", border_style="green")

    def _meta_panel(self) -> Panel:
        narrative = self.meta.workspace_narrative()
        meta_p    = self.meta.meta_architecture_patterns()
        friction  = self.meta.friction_zones()

        t = Text()
        t.append("Narrative:\n", style="bold dim")
        t.append(f"  {narrative}\n\n", style="italic white")

        if meta_p:
            t.append("Meta-Patterns:\n", style="bold dim")
            for p in meta_p[:2]:
                short = p[:72] + "…" if len(p) > 72 else p
                t.append(f"  ⬡ {short}\n", style="cyan")
            t.append("\n")

        if friction:
            t.append("Friction:\n", style="bold dim")
            for f in friction[:2]:
                short = f[:68] + "…" if len(f) > 68 else f
                t.append(f"  ⚡ {short}\n", style="yellow")
        else:
            t.append("No friction zones.\n", style="dim green")

        return Panel(t, title="[bold]META-COGNITION[/bold]", border_style="magenta")

    def _footer(self) -> Panel:
        now = datetime.now(timezone.utc).isoformat(timespec="seconds")
        t = Text()
        t.append("  Watching: ", style="dim")
        t.append(str(self.root), style="bold white")
        t.append("   ·   Output → programs/watcher/output/", style="dim cyan")
        t.append(f"   ·   {now}", style="dim")
        t.append("   ·   Ctrl+C to stop", style="dim")
        return Panel(t, style="dim", box=box.HEAVY)


# ──────────────────────────────────────────────────────────────────────────────
# MAIN
# ──────────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Workspace Metacognition Watcher")
    parser.add_argument(
        "root", nargs="?",
        default=str(Path(__file__).parent.parent.parent),
        help="Workspace root (default: 3 levels up from this script)",
    )
    parser.add_argument("--no-dashboard", action="store_true",
                        help="Run in headless mode (no Rich UI, writes files only)")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    if not root.exists():
        print(f"ERROR: workspace root not found: {root}", file=sys.stderr)
        sys.exit(1)

    print(f"[watcher] root       : {root}")
    print(f"[watcher] output     : {root / 'programs' / 'watcher' / 'output'}")

    # init
    handler = WorkspaceEventHandler(root)
    fractal = FractalAnalyzer(root)
    meta    = MetaCognitionEngine(handler, fractal)
    writer  = StateWriter(root, handler, fractal, meta)

    # initial fractal scan
    print("[watcher] scanning fractal structure …")
    fractal.scan(force=True)
    h = fractal.health()
    print(f"[watcher] fractal    : {h['score']:.1%}  ({h['complete']}/{h['total']} nodes complete)")

    # write initial state files
    writer.force_all()
    print("[watcher] output files written.")

    # start filesystem observer
    observer = Observer()
    observer.schedule(handler, str(root), recursive=True)
    observer.start()
    print(f"[watcher] watching …  (Ctrl+C to stop)\n")

    try:
        if args.no_dashboard:
            while True:
                time.sleep(5)
                fractal.scan()
                writer.tick()
        else:
            dashboard = Dashboard(handler, fractal, meta, root)
            console   = Console()
            with Live(dashboard.build(), refresh_per_second=1 / LIVE_INTERVAL,
                      console=console, screen=True) as live:
                while True:
                    time.sleep(LIVE_INTERVAL)
                    fractal.scan()
                    writer.tick()
                    live.update(dashboard.build())
    except KeyboardInterrupt:
        pass

    observer.stop()
    observer.join()

    # final write on exit
    writer.force_all()
    print("\n[watcher] stopped. Final state written.")


if __name__ == "__main__":
    main()
