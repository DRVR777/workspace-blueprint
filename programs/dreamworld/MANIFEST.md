---
name: dreamworld
type: project
status: reference
version: 1.0
parent: programs/
---

# Dreamworld — Vision PRD

**What this is**: The maximalist vision document for Dreamworld — a 3D spatial internet platform
where users inhabit persistent worlds, interact with AI agents, build through natural language,
and connect through a universal graph-based multiverse. This is the requirements source for
`programs/game_engine/`.

**What it contains**:

| File | Purpose |
|------|---------|
| `prd.txt` | Full product requirements document — v1.0, February 2026, Roan Curtis |
| `implementationplans/implementationMaster.txt` | Week-by-week build guide, 4 phases, soul check rule |

## Routing

| You want to... | Go to |
|----------------|-------|
| Understand the full product vision | `prd.txt` |
| See the phased build sequence | `implementationplans/implementationMaster.txt` |
| Map requirements to game_engine | Cross-reference `prd.txt` with `programs/game_engine/PRD.md` |

## Key Concepts

- **Packet protocol** — universal message format; every system communicates through Packets
- **Four phases**: Phase 1 = single player foundation, Phase 2 = AI-native, Phase 3 = multiplayer, Phase 4 = the world
- **Soul check rule**: every component must pass "would you be proud to show this to a stranger?" before moving forward
- **Stack**: Three.js/WebGPU (browser), Node.js gateway + world processes, PostgreSQL spatial, WebSocket multiplayer
