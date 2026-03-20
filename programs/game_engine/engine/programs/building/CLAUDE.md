# CLAUDE.md — building

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: 9 primitives, 0.3m grid, placement mode, 16 materials, validation |
| `CONTEXT.md` | Build contract: 9-step process, checkpoints, outputs |
| `src/` | TypeScript implementation (R3F components + pure modules) |
| `output/` | Phase completion reports |

## Quick Rules For This Directory

- Building is Phase 1 — requires working WebSocket + renderer first
- Grid size is 0.3m (1 foot) — this is a user requirement, don't change it
- Ghost preview and grid visualization update in useFrame() — no React state
- All placements go through the server (CREATE action) — no client-only objects
- Geometry is procedural (BoxGeometry, wedge BufferGeometry) — no GLTF models in Phase 1
- Pieces must be connected to ground — no floating structures (Phase 1 validation)
- 9 primitive types: wall, floor, ramp, stair, roof, door frame, window frame, column, wedge

## Cross-References

- `../../shared/contracts/world-state-contract.md` — CREATE/DESTROY/PROPERTY_CHANGE actions
- `../renderer/MANIFEST.md` — rendering pipeline (building pieces are static meshes)
- `../../../ELEV8-source/components/scene/DraggableObject.tsx` — reference: drag with zero re-renders
