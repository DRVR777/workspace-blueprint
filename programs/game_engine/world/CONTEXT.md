# world — Task Router

*Your task within the world subsystem → go here*

| If you want to... | Go to |
|-------------------|-------|
| Understand what world/ owns | `MANIFEST.md` |
| Work on the octree + spatial queries | `programs/spatial/MANIFEST.md` |
| Work on node lifecycle (tick, split, merge) | `programs/node-manager/MANIFEST.md` |
| Work on physics simulation | `programs/simulation/MANIFEST.md` |
| Work on procedural terrain generation | `programs/world-generator/MANIFEST.md` |
| Read the spatial index decision | `../_planning/adr/ADR-002-spatial-index.md` |
| Read the physics decision | `../_planning/adr/ADR-003-physics-integrator.md` |
| Read the world seed decision | `../_planning/adr/ADR-014-world-seed-algorithm.md` |
| Understand the world-state contract | `../shared/contracts/world-state-contract.md` |
| Understand the simulation contract | `../shared/contracts/simulation-contract.md` |

**Build order within world/:**
1. `programs/spatial/` — must exist before node-manager can build an octree
2. `programs/node-manager/` — must exist before simulation runs
3. `programs/simulation/` — built in Phase 0 (physics) and Phase 1 (entity AI)
4. `programs/world-generator/` — built in Phase 1

**Dependency rule**: spatial/ has no dependencies within world/. node-manager/ depends on spatial/. simulation/ depends on both. world-generator/ depends only on the world graph client (external to world/).
