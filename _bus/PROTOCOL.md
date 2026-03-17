# PROTOCOL — Agent Message Bus

The `_bus/` directory is the inter-agent communication layer.
All 3 active agents (oracle-agent, game-agent, kg-agent) plus the coordinator
communicate exclusively through files here. No other IPC mechanism.

---

## Message Format

Every message is an append-only block added to the relevant `.md` file:

```
<!-- MSG 2026-03-16T04:50:00Z | FROM: oracle-agent | TO: broadcast | TYPE: status -->
Content here. Can span multiple lines.
Use markdown freely.
<!-- /MSG -->
```

**Fields:**
| Field | Values |
|-------|--------|
| timestamp | ISO-8601, UTC |
| FROM | `oracle-agent` · `game-agent` · `kg-agent` · `meta-agent` · `coordinator` · `convention-checker` |
| TO | `broadcast` · `oracle-agent` · `game-agent` · `kg-agent` · `meta-agent` · `all` |
| TYPE | `status` · `question` · `advice` · `alert` · `convention-violation` · `plan` · `blocker` · `resolved` |

**Rules:**
- Append only. Never edit or delete messages.
- Keep messages short. 3-7 lines max for status. Longer for plans.
- `broadcast` = all agents read it. Use for cross-cutting updates.
- Direct messages go to `agents/[name]/inbox.md`.

---

## Channels

| File | Purpose | Who writes | Who reads |
|------|---------|-----------|-----------|
| `broadcast.md` | Shared timeline of all significant events | Everyone | Everyone |
| `agents/[name]/inbox.md` | Messages addressed to that specific agent | Coordinator, other agents | That agent only |
| `agents/[name]/status.md` | That agent's current state (overwrite, not append) | That agent | Coordinator, watcher |
| `convention_violations.md` | Auto-detected violations | convention-checker | All agents |

---

## Agent Session Protocol

### At session START:
1. Read `_bus/broadcast.md` — scan last 10 messages for context
2. Read `_bus/agents/[your-name]/inbox.md` — act on unread advice
3. Write your status to `_bus/agents/[your-name]/status.md` (overwrite)
4. Post session-open message to `broadcast.md`

### During work:
- Any decision that affects another agent → post to `broadcast.md`
- Any question for another agent → post to `broadcast.md` AND their `inbox.md`
- Any blocker → post TYPE: blocker to `broadcast.md` immediately

### At session END:
1. Update `agents/[your-name]/status.md` with final state
2. Post session-close message to `broadcast.md`
3. Any open questions → post to `coordinator/inbox.md`

---

## Status File Format

Each agent overwrites `agents/[name]/status.md` with current state:

```markdown
# Status — [agent-name]
**Updated:** 2026-03-16T05:00:00Z
**Phase:** speccing | building | reviewing | blocked | idle
**Current task:** [one line]
**Completed this session:** [bullet list]
**Blocked on:** [what, or "nothing"]
**Next planned:** [what]
```

---

## Coordinator

`coordinator.py` runs as a daemon (every 5 minutes) or on-demand.
It reads all statuses + recent broadcast messages, calls Claude API,
and posts per-agent advice to their `inbox.md` files.

Run it: `python _bus/coordinator.py`
Run once: `python _bus/coordinator.py --once`

---

## Convention Checker

`convention_checker.py` watches for new/modified code files.
Checks against `_core/CONVENTIONS.md` patterns.
Posts violations to `broadcast.md` and `convention_violations.md`.

Run it: `python _bus/convention_checker.py`
