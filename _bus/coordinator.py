#!/usr/bin/env python3
"""
coordinator.py — Claude-Powered Agent Coordinator

Reads the current workspace state (watcher output + agent statuses + broadcast),
calls Claude claude-sonnet-4-6 to synthesize coordination advice, and posts
per-agent guidance to their inbox.md files.

Usage:
    python _bus/coordinator.py              # run as daemon (every 5 minutes)
    python _bus/coordinator.py --once       # run once and exit
    python _bus/coordinator.py --dry-run    # print advice without writing

Requires:
    pip install anthropic
    ANTHROPIC_API_KEY set in environment or .env file

If ANTHROPIC_API_KEY is not set, runs in OFFLINE mode:
    - Reads statuses and recent messages
    - Generates rule-based (non-AI) advice from known workspace state
    - Still writes to inboxes
"""

import os
import sys
import time
import argparse
import re
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).parent.parent
BUS  = ROOT / "_bus"
OUT  = ROOT / "programs" / "watcher" / "output"

AGENTS      = ["oracle-agent", "game-agent", "kg-agent", "meta-agent"]
INTERVAL_S  = 300   # 5 minutes between coordinator runs
MAX_BROADCAST_LINES = 80  # lines of broadcast to feed to Claude


# ──────────────────────────────────────────────────────────────────────────────
# HELPERS
# ──────────────────────────────────────────────────────────────────────────────

def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")

def _read(path: Path, default: str = "") -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (FileNotFoundError, OSError):
        return default

def _append_message(path: Path, from_: str, to: str, type_: str, content: str):
    """Append a properly formatted message to a channel file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    ts  = now_iso()
    msg = f"\n<!-- MSG {ts} | FROM: {from_} | TO: {to} | TYPE: {type_} -->\n{content.strip()}\n<!-- /MSG -->\n"
    with open(path, "a", encoding="utf-8") as f:
        f.write(msg)

def _read_recent_broadcast(n_lines: int = MAX_BROADCAST_LINES) -> str:
    text  = _read(BUS / "broadcast.md", "")
    lines = text.splitlines()
    return "\n".join(lines[-n_lines:])

def _read_agent_status(agent: str) -> str:
    return _read(BUS / "agents" / agent / "status.md", f"(no status yet for {agent})")

def _read_agent_inbox(agent: str) -> str:
    """Read last 30 lines of inbox."""
    text  = _read(BUS / "agents" / agent / "inbox.md", "")
    lines = text.splitlines()
    return "\n".join(lines[-30:])

def _read_violations() -> str:
    return _read(BUS / "convention_violations.md", "(none)")

def _read_meta_cognition() -> str:
    return _read(OUT / "META_COGNITION.md", "(watcher not running)")

def _read_conventions_summary() -> str:
    """Read key patterns from CONVENTIONS.md — just the index table."""
    text  = _read(ROOT / "_core" / "CONVENTIONS.md", "")
    lines = text.splitlines()
    # grab the index table (first ~35 lines after the header)
    return "\n".join(lines[:50])


# ──────────────────────────────────────────────────────────────────────────────
# OFFLINE COORDINATOR (no API key — rule-based)
# ──────────────────────────────────────────────────────────────────────────────

def offline_coordinate(dry_run: bool = False) -> dict[str, str]:
    """
    Rule-based coordination when no API key is available.
    Reads agent statuses and returns per-agent advice strings.
    """
    advice: dict[str, str] = {}

    violations = _read_violations()
    has_violations = "no violations" not in violations.lower() and "(none)" not in violations

    for agent in AGENTS:
        status_text = _read_agent_status(agent)
        lines = []

        # extract phase and blockers from status
        phase = "idle"
        for line in status_text.splitlines():
            if "**Phase:**" in line:
                phase = line.split("**Phase:**")[-1].strip().lower()
            if "**Blocked on:**" in line:
                blocked = line.split("**Blocked on:**")[-1].strip()
                non_blocked = {"nothing", "—", "", "nothing — unblocked, ready to build",
                               "nothing — system is live, needs real content"}
                if blocked.lower() not in non_blocked and not blocked.lower().startswith("nothing"):
                    lines.append(f"**BLOCKER DETECTED:** {blocked}")
                    lines.append("Resolve this before any other work.")

        # agent-specific rules
        if agent == "oracle-agent":
            lines += [
                "Build signal-ingestion: start at `programs/oracle/programs/signal-ingestion/CONTEXT.md`",
                "Implement adapters in order — polymarket_clob first (simplest, synchronous)",
                "Each adapter: post Signal to Redis `signals.raw`, log to ticker.log",
                "Use contracts from `oracle-shared/oracle_shared/contracts/signal.py`",
            ]
        elif agent == "game-agent":
            lines += [
                "Close GAP-011 first: write .fbs schema files in `shared/schemas/`",
                "Derive from existing contracts in `shared/contracts/`",
                "6 Flatbuffers + 6 Protobuf files — see MANIFEST gap notes",
                "Once schemas exist, start node-manager tick loop implementation",
            ]
        elif agent == "kg-agent":
            lines += [
                "Load real workspace docs — target CONVENTIONS.md first (most referenced)",
                "Run: kg_index_batch on 5 docs at a time",
                "After 20 docs: post to broadcast.md so other agents know graph is ready",
                "Current: 5 synthetic files loaded, need ~20 real for useful proximity queries",
            ]
        elif agent == "meta-agent":
            lines += [
                "Run fractal_complete.py --apply to close 89 missing nav files",
                "After apply: re-run watcher to confirm fractal score rises above 80%",
                "Then process open gaps in _meta/gaps/CONTEXT.md",
            ]

        if has_violations:
            lines.append(f"⚠ Convention violations detected — check `_bus/convention_violations.md`")

        advice[agent] = "\n".join(f"- {l}" for l in lines)

    return advice


# ──────────────────────────────────────────────────────────────────────────────
# CLAUDE COORDINATOR (with API key)
# ──────────────────────────────────────────────────────────────────────────────

SYSTEM_PROMPT = """\
You are the COORDINATOR for a multi-agent software development workspace.

Three agents are simultaneously building different systems:
- oracle-agent: ORACLE trading intelligence platform (Python, asyncio, Redis, Claude API)
- game-agent: NEXUS spatial game engine (Flatbuffers, Protobuf, client-server simulation)
- kg-agent: knowledge-graph Cognitive Document System (Python, MCP server, 5D vectors)

Your job:
1. Read the current workspace state (agent statuses, recent messages, violations)
2. For each ACTIVE or BLOCKED agent: write 3-6 bullet points of specific, actionable advice
3. Flag any cross-agent conflicts or shared dependencies
4. Flag any convention violations (from _core/CONVENTIONS.md)
5. If an agent is idle and has no clear next task, give them one

RULES:
- Keep each agent's advice to 3-6 bullets MAXIMUM
- Each bullet = one specific action with file path when possible
- Do not repeat what the agent already knows from their status
- Reference actual file paths — not abstract descriptions
- If no advice needed for an agent, say "No action needed — continue current task."
- Never invent requirements not in the workspace context

OUTPUT FORMAT (exactly):
AGENT: oracle-agent
- bullet 1
- bullet 2

AGENT: game-agent
- bullet 1

AGENT: kg-agent
- bullet 1

CROSS-AGENT:
- any shared concerns, dependency conflicts, or coordination notes

Do not include any other text. Just the AGENT blocks and CROSS-AGENT block.
"""

def claude_coordinate(dry_run: bool = False) -> dict[str, str]:
    """Call Claude to synthesize coordination advice."""
    try:
        import anthropic
    except ImportError:
        print("[coordinator] anthropic not installed — falling back to offline mode")
        return offline_coordinate(dry_run)

    api_key = os.environ.get("ANTHROPIC_API_KEY", "")
    if not api_key:
        # try .env in workspace root
        env_path = ROOT / ".env"
        if env_path.exists():
            for line in env_path.read_text().splitlines():
                if line.startswith("ANTHROPIC_API_KEY="):
                    api_key = line.split("=", 1)[1].strip().strip('"')
                    break

    if not api_key:
        print("[coordinator] ANTHROPIC_API_KEY not found — falling back to offline mode")
        return offline_coordinate(dry_run)

    # build context
    statuses = "\n\n".join(
        f"=== {agent} ===\n{_read_agent_status(agent)}"
        for agent in AGENTS
    )
    broadcast = _read_recent_broadcast()
    violations = _read_violations()
    meta = _read_meta_cognition()[:1500]  # cap to avoid huge context

    user_message = f"""## Workspace Self-Model (from watcher)
{meta}

## Agent Statuses
{statuses}

## Recent Broadcast (last {MAX_BROADCAST_LINES} lines)
{broadcast}

## Convention Violations
{violations}

## Coordinator Inbox (escalations from agents)
{_read(_BUS_inbox_coordinator())}

Now generate the per-agent coordination update.
"""

    client = anthropic.Anthropic(api_key=api_key)
    resp = client.messages.create(
        model="claude-sonnet-4-6",
        max_tokens=1024,
        system=SYSTEM_PROMPT,
        messages=[{"role": "user", "content": user_message}],
    )
    raw = resp.content[0].text

    # parse output into per-agent advice dict
    advice: dict[str, str] = {}
    current_agent = None
    current_lines: list[str] = []
    cross_agent_lines: list[str] = []
    in_cross = False

    for line in raw.splitlines():
        if line.startswith("AGENT: "):
            if current_agent:
                advice[current_agent] = "\n".join(current_lines).strip()
            current_agent = line[7:].strip()
            current_lines = []
            in_cross = False
        elif line.strip() == "CROSS-AGENT:":
            if current_agent:
                advice[current_agent] = "\n".join(current_lines).strip()
                current_agent = None
            in_cross = True
        elif in_cross:
            cross_agent_lines.append(line)
        elif current_agent is not None:
            current_lines.append(line)

    if current_agent:
        advice[current_agent] = "\n".join(current_lines).strip()

    if cross_agent_lines:
        advice["_cross_agent"] = "\n".join(cross_agent_lines).strip()

    return advice


def _BUS_inbox_coordinator() -> Path:
    return BUS / "agents" / "coordinator" / "inbox.md"


# ──────────────────────────────────────────────────────────────────────────────
# WRITE ADVICE TO INBOXES
# ──────────────────────────────────────────────────────────────────────────────

def write_advice(advice: dict[str, str], dry_run: bool = False):
    ts = now_iso()
    for agent, content in advice.items():
        if not content or not content.strip():
            continue
        if agent == "_cross_agent":
            # post cross-agent concerns to broadcast
            if dry_run:
                print(f"\n=== BROADCAST (cross-agent) ===\n{content}\n")
            else:
                _append_message(BUS / "broadcast.md", "coordinator", "all", "advice", content)
            continue

        inbox = BUS / "agents" / agent / "inbox.md"
        if dry_run:
            print(f"\n=== {agent} ===\n{content}\n")
        else:
            _append_message(inbox, "coordinator", agent, "advice", content)
            print(f"[coordinator] → posted to {agent}/inbox.md")

    if not dry_run:
        # also post summary to broadcast
        active = [a for a in advice if a != "_cross_agent" and advice[a].strip()]
        summary = f"Coordinator update posted to: {', '.join(active)}"
        _append_message(BUS / "broadcast.md", "coordinator", "broadcast", "plan", summary)


# ──────────────────────────────────────────────────────────────────────────────
# MAIN
# ──────────────────────────────────────────────────────────────────────────────

def run_once(dry_run: bool = False):
    print(f"[coordinator] running at {now_iso()}")
    has_api = bool(os.environ.get("ANTHROPIC_API_KEY"))
    mode = "claude API" if has_api else "offline (rule-based)"
    print(f"[coordinator] mode: {mode}")

    advice = claude_coordinate(dry_run) if has_api else offline_coordinate(dry_run)
    write_advice(advice, dry_run)
    print(f"[coordinator] done.")


def main():
    parser = argparse.ArgumentParser(description="Workspace Agent Coordinator")
    parser.add_argument("--once",    action="store_true", help="Run once and exit")
    parser.add_argument("--dry-run", action="store_true", help="Print advice, don't write")
    args = parser.parse_args()

    if args.once or args.dry_run:
        run_once(dry_run=args.dry_run)
    else:
        print(f"[coordinator] daemon started — running every {INTERVAL_S//60} minutes")
        print("[coordinator] Ctrl+C to stop\n")
        while True:
            try:
                run_once()
                time.sleep(INTERVAL_S)
            except KeyboardInterrupt:
                print("\n[coordinator] stopped.")
                break
            except Exception as e:
                print(f"[coordinator] ERROR: {e} — retrying in 60s")
                time.sleep(60)


if __name__ == "__main__":
    main()
