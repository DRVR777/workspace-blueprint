# MANIFEST — _bus

## Envelope

| Field  | Value                        |
|--------|------------------------------|
| name   | _bus                         |
| type   | meta-infrastructure / comms  |
| depth  | 1                            |
| status | active                       |
| path   | _bus                         |

## Purpose

Inter-agent message bus. All 3 active agents (oracle-agent, game-agent, kg-agent)
plus the coordinator communicate through files here.

The file system IS the message bus — no external infrastructure needed.
Agents append to channel files. The coordinator reads all channels and
posts per-agent advice using Claude claude-sonnet-4-6.

## Contents

| Name | Type | Purpose |
|------|------|---------|
| `PROTOCOL.md` | doc | Message format spec and agent session protocol |
| `broadcast.md` | channel | Shared timeline — all agents read/write |
| `convention_violations.md` | log | Auto-appended by convention_checker.py |
| `agents/` | dir | Per-agent inbox + status files |
| `coordinator.py` | program | Claude-powered coordinator daemon |
| `convention_checker.py` | program | Real-time convention enforcement daemon |

## Returns

- `broadcast.md` — shared situational awareness
- `agents/[name]/inbox.md` — per-agent advice and questions
- `agents/[name]/status.md` — each agent's self-reported state
- `convention_violations.md` — architecture health feed
