# CONTEXT.md — watcher

<!-- Task router for this directory. -->

## Task Routing

| If your task is… | Go to | Do |
|-----------------|-------|----|
| Start the live watcher | `watcher.py` | `python watcher.py` |
| Run headless (no UI) | `watcher.py` | `python watcher.py --no-dashboard` |
| See what files are missing from the fractal | `fractal_complete.py` | `python fractal_complete.py` (dry run) |
| Auto-generate all missing nav files | `fractal_complete.py` | `python fractal_complete.py --apply` |
| Read current agent activity | `output/LIVE_STATE.md` | Open / read the file |
| Understand what workspace is doing | `output/META_COGNITION.md` | Read at session start |
| See architectural patterns | `output/PATTERNS.md` | Read for meta-level insights |
| Get machine-readable health metrics | `output/HEALTH.json` | Parse JSON |

## Active Work

- Watcher is monitoring: `programs/oracle`, `programs/game_engine`, `programs/knowledge-graph`
- Output files refresh automatically while watcher is running

## Fix-First Rule

If a generated output file has stale data (watcher was stopped), re-run the
watcher and the files will refresh within 30 seconds.
