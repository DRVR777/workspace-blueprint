# game_engine — Depth-1 Map

*Read this to understand what is where. Do not read beyond depth-1 without a reason.*

| Path | What it is |
|------|-----------|
| `PRD.md` | The requirements document — thousands of lines, pseudocode, no real code yet |
| `MANIFEST.md` | Program identity and status |
| `CONTEXT.md` | Task router — your task → go here |
| `world/` | Server-side: spatial partitioning, nodes, world graph, simulation |
| `engine/` | Client-side: rendering, LOD, asset cache, local physics |
| `_planning/` | ADRs and build roadmap |
| `_meta/` | Gaps registry |
| `shared/` | Contracts between subsystems |

**Rule**: This file has depth-1 only. If you need depth-2, navigate into the subdirectory and read its MANIFEST.md.
