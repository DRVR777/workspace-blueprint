# engine — Task Router

*Your task within the engine subsystem → go here*

| If you want to... | Go to |
|-------------------|-------|
| Understand what engine/ owns | `MANIFEST.md` |
| Work on the rendering loop / frame production | `programs/renderer/MANIFEST.md` |
| Work on LOD tier assignment + blending | `programs/lod/MANIFEST.md` |
| Work on asset cache + streaming consumer | `programs/asset-pipeline/MANIFEST.md` |
| Work on client-side physics prediction | `programs/local-simulation/MANIFEST.md` |
| Work on frustum culling + visibility | `programs/visibility/MANIFEST.md` |
| Read the LOD decisions | `../_planning/adr/ADR-002-spatial-index.md` (for octree) |
| Read the texture format decision | `../_planning/adr/ADR-005-texture-format.md` |
| Understand the LOD system contract | `../shared/contracts/lod-system-contract.md` |
| Understand asset store contract | `../shared/contracts/asset-store-contract.md` |

**Build order within engine/:**
1. `programs/renderer/` — minimal skeleton (Phase 0: render a flat plane at 60fps)
2. `programs/local-simulation/` — client prediction (Phase 0)
3. `programs/visibility/` — frustum culling (Phase 0, needed for any real scene)
4. `programs/lod/` — LOD tier system (Phase 1, requires real object types)
5. `programs/asset-pipeline/` — streaming consumer + cache (Phase 1)

**Dependency rule**: renderer/ depends on visibility/ for the culled object list. renderer/ depends on lod/ for tier selection. lod/ depends on asset-pipeline/ for geometry availability. local-simulation/ is independent of all others except it reads from the client world state that the renderer also reads.
