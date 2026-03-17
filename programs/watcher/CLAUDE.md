# CLAUDE.md — watcher

<!-- Always-loaded depth-1 map. Names + one-line purposes only. -->

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `watcher.py` | Main program — run this to start the live dashboard + file output |
| `fractal_complete.py` | Scans workspace for missing nav files; generates them with `--apply` |
| `requirements.txt` | Python deps: `watchdog`, `rich` |
| `output/` | Auto-generated state files — agents read these for situational awareness |

## To Start The Watcher

```
cd programs/watcher
pip install -r requirements.txt
python watcher.py                   # Rich live dashboard
python watcher.py --no-dashboard    # headless / log-only
```

## To Complete The Fractal

```
python fractal_complete.py          # dry run: see what's missing
python fractal_complete.py --apply  # write all missing nav files
```

## Output Files Agents Should Read

- `output/LIVE_STATE.md`     — what changed in the last 30 seconds
- `output/META_COGNITION.md` — workspace self-model, start-of-session orientation
- `output/PATTERNS.md`       — detected meta-architecture patterns
- `output/HEALTH.json`       — machine-readable fractal health metrics
