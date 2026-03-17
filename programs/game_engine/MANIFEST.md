---
name: game_engine
type: program
status: scaffold
version: 0.1
campaign: campaigns/game_engine
---

# NEXUS Game Engine

**What this is**: A spatial computing substrate — a game engine purpose-built for an infinite, persistent, massively multiplayer 3D universe. Not a general-purpose engine. Built specifically to run Dreamworld-class experiences.

**What it contains**:

| Folder | Purpose |
|--------|---------|
| `world/` | Spatial partitioning, octree, domain management, world generation |
| `engine/` | Local rendering loop, LOD system, asset pipeline, client state |
| `shared/` | Contracts and message schemas between all subsystems |
| `_planning/` | ADRs (7 accepted, 7 open) and build roadmap |
| `_meta/` | Gap tracker |

*Two active program folders (world/, engine/). Max 9 total. Next folders: network/, data/ when Phase 1-2 PRD sections mature.*

**Status**: scaffold — PRD complete, 7/14 ADRs accepted, Phase 0 sub-programs specced, schemas pending (GAP-011)

**Blocking ADRs**: ADR-001 through ADR-005 must be resolved before Phase 0 begins

**See also**:
- `PRD.md` — complete product requirements
- `_planning/roadmap.md` — build order
- `_planning/adr/` — open architectural decisions
- `shared/contracts/` — subsystem interfaces
- `campaigns/game_engine/` — campaign tracker and progress documents
