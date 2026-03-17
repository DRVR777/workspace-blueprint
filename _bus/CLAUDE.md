# CLAUDE.md — _bus

<!-- Always-loaded map. Depth-1 only. -->

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `PROTOCOL.md` | Read this first — defines message format and session rules |
| `broadcast.md` | Shared channel — read last 10 messages at every session start |
| `convention_violations.md` | Active violations — fix any in your domain before working |
| `agents/oracle-agent/` | Oracle agent inbox + status |
| `agents/game-agent/` | Game engine agent inbox + status |
| `agents/kg-agent/` | Knowledge-graph agent inbox + status |
| `coordinator.py` | Coordinator daemon — `python _bus/coordinator.py --once` |
| `convention_checker.py` | Convention watcher — `python _bus/convention_checker.py` |

## Agent Session Checklist

At session start (in order):
1. `cat _bus/broadcast.md | tail -40` — last 10 messages
2. `cat _bus/agents/[your-name]/inbox.md` — your inbox
3. `cat _bus/convention_violations.md` — fix any in your domain
4. Overwrite `_bus/agents/[your-name]/status.md` with current state
5. Append session-open message to `_bus/broadcast.md`

At session end:
1. Update your `status.md`
2. Post session-close to `broadcast.md`
3. Any unresolved questions → append to `_bus/agents/coordinator/inbox.md`
