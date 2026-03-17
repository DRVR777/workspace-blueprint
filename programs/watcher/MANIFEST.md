# MANIFEST — watcher

## Envelope

| Field   | Value                                |
|---------|--------------------------------------|
| name    | watcher                              |
| type    | meta-infrastructure / observability  |
| depth   | 3                                    |
| status  | active                               |
| path    | programs/watcher                     |

## Purpose

Real-time metacognition engine for the workspace. Three active agents are
simultaneously working in different parts of this workspace; the watcher
observes every filesystem change, infers agent phases, measures fractal
architectural completeness, and surfaces optimization signals — all written
to `output/` as live markdown files that any agent can read.

The watcher also provides `fractal_complete.py`: a tool that auto-generates
missing navigation trinity files (MANIFEST / CLAUDE / CONTEXT) so the
workspace can complete its own self-description fractal without manual effort.

## Contents

- `watcher.py`          — main program (Rich dashboard + watchdog observer)
- `fractal_complete.py` — auto-generates missing navigation files
- `requirements.txt`    — Python dependencies
- `output/`             — auto-generated state files (read by agents)
  - `LIVE_STATE.md`       — current agent activity (refreshes every 30s)
  - `PATTERNS.md`         — detected architectural patterns (every 5m)
  - `META_COGNITION.md`   — workspace self-model narrative (every 10m)
  - `HEALTH.json`         — machine-readable metrics (every 30s)

## Needs

- Python 3.10+
- `pip install watchdog rich`
- Read access to entire workspace tree

## Returns

- Real-time terminal dashboard showing all 3 agents' activity
- `output/LIVE_STATE.md`   — agent-readable live feed of changes
- `output/PATTERNS.md`     — emerging meta-architecture patterns
- `output/META_COGNITION.md` — workspace self-model (what is the workspace doing?)
- `output/HEALTH.json`     — fractal completeness metrics for tooling

## Gap Status

- open: none
- closed: none
