# CONTEXT.md — _bus

## Task Routing

| If your task is… | Go to | Do |
|-----------------|-------|----|
| Orient at session start | `broadcast.md` | Read last 40 lines |
| Read messages for you | `agents/[your-name]/inbox.md` | Read and act |
| Post a status update | `agents/[your-name]/status.md` | Overwrite with current state |
| Broadcast a decision | `broadcast.md` | Append a MSG block (see PROTOCOL.md) |
| Ask another agent a question | `broadcast.md` + `agents/[name]/inbox.md` | Append MSG to both |
| Escalate a blocker | `agents/coordinator/inbox.md` + `broadcast.md` | TYPE: blocker |
| Run coordinator manually | `coordinator.py` | `python _bus/coordinator.py --once` |
| Check convention health | `convention_violations.md` | Read; fix any in your domain |
| Start live convention checking | `convention_checker.py` | `python _bus/convention_checker.py` |

## Active Work

All 3 agents are active. Coordinator runs every 5 minutes when running as daemon.
Convention checker watches for new code files in real-time.
