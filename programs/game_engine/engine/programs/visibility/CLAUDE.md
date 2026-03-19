# CLAUDE.md — visibility

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: frustum culling, cull pipeline, data shapes, R3F integration, occlusion (Phase 2+) |
| `CONTEXT.md` | Build contract — phased process (Phase 0 stub → Phase 1 full), checkpoints, outputs |
| `src/` | TypeScript implementation (pure module, NOT a React component) |
| `output/` | Phase completion reports |

## Quick Rules For This Directory

- Visibility is a **pure TypeScript module** — no React hooks, no state, no re-renders
- Called imperatively inside renderer's `useFrame()` — never triggered by React reconciliation
- Gate order matters: distance → LOD → frustum → occlusion (cheapest first)
- Local player entity is ALWAYS returned — skip all gates
- Front-to-back sort on output — enables GPU early-Z rejection
- Phase 0 is a pass-through stub with the same interface as the full implementation

## Cross-References

- `../../shared/contracts/lod-system-contract.md` — assign_tier_with_blend called per object
- `../../shared/contracts/world-state-contract.md` — object_record, entity_record shapes
- `../renderer/MANIFEST.md` — the caller (cull() invoked in render loop Step 3-4)
- `../local-simulation/MANIFEST.md` — provides predicted positions consumed by cull_entities
