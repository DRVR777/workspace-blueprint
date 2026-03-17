# Campaign: game_engine
## Clarification Document — What Was Meant, What Must Be Done

*This document is a living interpretation engine. It is updated every session. It translates the raw intent of the original prompt into structured tasks, resolves ambiguity, and maps work to the workspace-blueprint folder architecture.*

---

## ADMIN BYPASS DECLARATION

> **BYPASS ACTIVE**: This campaign has full autonomous authority to create folders, write documents, expand the program structure, add subprograms, create ADRs, and extend the workspace-blueprint architecture anywhere within `programs/game_engine/`. No permission is needed to act. The only constraint is: NO CODE until the plan is declared COMPLETE by the campaign runner.

---

## 1. What The Prompt Actually Means

The user gave a dense, stream-of-consciousness prompt. Decoded, it contains the following distinct instructions, in order of intent:

### 1.1 Immediate Structural Tasks (Do These First)
- [x] Create `campaigns/` folder in workspace-blueprint root
- [x] Create `campaigns/game_engine/` folder
- [x] Copy the exact original prompt to `campaigns/game_engine/prompt.md`
- [x] Create this clarification document at `campaigns/game_engine/clarification.md`
- [x] Create `campaigns/game_engine/progress/` folder
- [x] Create `campaigns/game_engine/progress/perfect_final_output.md`
- [x] Create `programs/game_engine/` program folder (follows workspace-blueprint _template pattern)
- [x] Create `programs/game_engine/PRD.md` — multi-thousands of lines, pseudocode, no real code

### 1.2 Research Required Before Writing PRD
- Analyze `programs/ELEV8` — understand its architectural failures
- Read `programs/knowledge-graph` — understand its spatial/graph concepts (DO NOT MODIFY)
- Read `programs/dreamworld` — understand the vision being served
- Research LOD (Level of Detail) systems in spatial engines
- Research spatial partitioning (octrees, BSP trees, spatial hashing)
- Research distributed game server architectures
- Research WebSocket binary protocols for game state
- Research Docker orchestration for spatial game servers

### 1.3 The Core Technical Vision (Decoded)

The user is describing a **distributed spatial computing platform** — essentially a game engine designed from first principles to run a game like Dreamworld: an infinite, persistent, multiplayer 3D universe. The key technical concepts extracted from the prompt:

**Concept 1: The Player As A 3D Point**
Every player exists as a coordinate in a three-dimensional universe. Reality around that point is not statically loaded — it is dynamically queried. A "radius of perception" determines what is real to the player. Everything outside that radius either does not exist for them or exists in a simplified form.

**Concept 2: Nodes Are World Processes**
A "node" is not a database node or a network node — it is a running server process that owns a spatial region of the 3D universe. Nodes run the same `game_engine` program. They communicate with each other through shared databases and direct messaging. One VPS can host multiple nodes. One node can cover a small or large region depending on player density.

**Concept 3: The VPS As Dual-Purpose (Metadata + Assets)**
The VPS that a player connects to via WebSocket serves two things:
1. **Continuous metadata stream**: positions, states, events, relationships — everything happening in the nearby world, compressed and sent many times per second
2. **Occasional asset delivery**: 3D mesh files, textures, object definitions — served once per object and then cached locally. After that, the server only needs to tell the local machine WHERE and WHAT STATE the object is in, not what it looks like.

This is the key insight for infinite worlds: serve geometry once, stream only state forever.

**Concept 4: The World Graph**
There is a shared graph database that all nodes write to and read from. Every object in the world is a node in this graph. Every relationship between objects is an edge. The graph is spatial — objects know their position and their neighbors. This directly maps to the `knowledge-graph` program's architecture: 5D positioning, neighbor discovery, self-describing nodes.

**Concept 5: LOD — Level of Detail**
Objects far from the player are rendered with less geometric complexity. A mountain seen from 1km away is 4 triangles. Seen from 10 meters, it is 40,000 triangles. This is a continuous spectrum. The LOD system decides, per-object, per-frame, what resolution of geometry to request and render.

**Concept 6: Farming and Building**
The game supports emergent world-building. Players can create objects in the world. Objects can be functional — including computers that run programs within the game world. This implies a simulation layer on top of the rendering layer.

**Concept 7: Modularity With Layers**
The user explicitly requested:
- At least 3 layers of abstraction for most systems
- Maximum 9 folders in the root of the game engine program
- Start with 2, then expand deliberately
- Each expansion justified by need

**Concept 8: Scale Requirements**
- Millions of concurrent players
- Thousands of Docker containers
- An effectively infinite world
- All players sharing one consistent world state

---

## 2. Mapping to Workspace Blueprint Architecture

The workspace-blueprint uses this hierarchy at every level:
- `MANIFEST.md` — what this directory is
- `CLAUDE.md` — depth-1 map of sub-directories
- `CONTEXT.md` — task router (your task → go here)
- `_planning/` — ADRs + roadmap
- `_meta/` — gaps + runtime metadata
- `shared/` — contracts between sub-programs
- `programs/` — runnable sub-components

The game_engine program follows this exactly. The PRD lives at the root of the program. The two starting folders are `world/` and `engine/`, matching the two foundational concepts.

---

## 3. What Makes A Valid Session (Continuous Re-Interpretation Protocol)

At the start of every session working on game_engine, the campaign runner (the AI) should:

1. Read this file first
2. Read `progress/perfect_final_output.md` — the north star
3. Read `programs/game_engine/PRD.md` — current state of requirements
4. Check `programs/game_engine/_meta/gaps/pending.txt` — open questions
5. Read the highest-priority gap
6. Execute the gap
7. Close the gap and log what was learned
8. If new gaps were discovered, add them to pending.txt

### Questions This Document Must Answer Over Time:

| Question | Current Answer | Confidence |
|----------|---------------|------------|
| What is the minimum viable world? | A player, a terrain, and another player visible across distance | Low |
| How does the spatial partitioning work? | Octree with dynamic subdivision | Medium |
| How many objects per node? | Unknown — needs benchmarking | None |
| How does cross-node entity handoff work? | When player crosses node boundary, both nodes negotiate transfer | Medium |
| What is the binary WebSocket protocol shape? | TBD in network PRD section | None |
| How does the LOD system request geometry? | Distance thresholds trigger tier transitions | Medium |
| How does the world graph handle conflict? | Last-write-wins with vector clocks? | Low |
| What is the asset cache invalidation strategy? | Object version hash — if hash same, don't re-serve | Medium |
| How does a computer in the world work? | TBD — simulation layer above physical layer | None |
| What is the agent (AI entity) lifecycle? | TBD | None |

---

## 4. What ELEV8 Got Wrong (Must Not Repeat)

*Full analysis in `programs/game_engine/PRD.md` Appendix A*

Summary of fatal mistakes to never repeat in game_engine:

1. **Three separate tech stacks** — game_engine has ONE architecture. One data model. One protocol. Everything speaks the same language structurally.

2. **Custom physics from scratch under deadline** — game_engine uses proven spatial data structures (octrees, BVH trees) rather than reinventing them. The physics concepts are specified abstractly first; only then is a library chosen.

3. **No centralized state** — game_engine has a single world state model. The world graph IS the state. Everything derives from it.

4. **Brittle WebSocket sync** — game_engine's network protocol is typed, versioned, and has explicit acknowledgment. Malformed messages trigger error recovery, not silent failure.

5. **Hand-written SQL with no spatial indexing** — game_engine's object database has spatial indexes as a first-class requirement, not an afterthought.

6. **No separation between rendering and state** — game_engine strictly separates: world state (server-authoritative), local prediction (client-side), and rendering (purely visual). These three layers never mix.

---

## 5. What Knowledge-Graph Got Right (Must Incorporate)

1. **5D semantic positioning** → In game_engine, objects have 3D spatial position PLUS semantic dimensions (mass, energy, information-density, age). This allows non-spatial queries like "find all objects with high information density within 100 units."

2. **Embedded prompts on nodes** → World objects can contain embedded behavioral instructions (for AI agents). An object knows what to do when an agent reads it.

3. **Ticker/audit log** → The world has an event log. Every state change is a ticker entry. This makes replay, debugging, and conflict resolution tractable.

4. **Strict program boundaries** → The game_engine programs (world, engine, network, data, simulation) have hard contracts. They do not reach into each other's internals. They communicate through shared contracts.

5. **Self-navigating architecture** → Nodes in the game world discover their neighbors automatically by querying the world graph. A newly spawned node finds what is adjacent to it without central coordination.

---

## 6. Current Campaign Status

| Item | Status | Next Action |
|------|---------|-------------|
| Campaign folder structure | DONE | — |
| perfect_final_output.md | DONE | Review and expand |
| PRD.md scaffold | DONE | Expand each section |
| world/ folder | DONE | Add MANIFEST, CONTEXT, _planning |
| engine/ folder | DONE | Add MANIFEST, CONTEXT, _planning |
| network/ folder | NOT YET | Add when PRD network section is complete |
| data/ folder | NOT YET | Add when PRD data section is complete |
| simulation/ folder | NOT YET | Add when simulation section is designed |
| Shared contracts | NOT YET | Define after all sections drafted |
| ADRs | NOT YET | List every open architectural decision |
| Spec review | NOT YET | Run after all ADRs resolved |
| Build order | NOT YET | After spec review passes |

---

## 7. Admin Notes

*This section records any structural decisions made by the campaign runner autonomously.*

- **2026-03-13**: Campaign created. Started with 2 root program folders (world/, engine/) per user instruction. Max 9 is the ceiling. campaigns/ folder added to workspace root per user instruction (not in original _template — this is a new concept). Admin bypass declared for autonomous operation.

- **FUTURE SESSIONS**: When the PRD is complete and all ADRs resolved, this campaign transitions to "build phase." The campaign runner will update the program's MANIFEST.md status from "scaffold" to "specced" and create the build order in `programs/game_engine/_planning/roadmap.md`.
