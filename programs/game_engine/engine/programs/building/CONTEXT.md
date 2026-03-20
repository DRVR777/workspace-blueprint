# building — Build Contract (Phase 1)

Read MANIFEST.md for the full specification. This file defines the build contract.

---

## Inputs

| File | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: primitives, grid, placement, materials, validation, network |
| `../../shared/contracts/world-state-contract.md` | object_record, change_request (CREATE/DESTROY/PROPERTY_CHANGE) |
| `../../shared/contracts/simulation-contract.md` | Server-side validation of building placements |
| `../renderer/MANIFEST.md` | Rendering pipeline — building pieces render as static meshes |
| `../../ELEV8-source/components/scene/DraggableObject.tsx` | Reference: refs-based drag with zero re-renders |

Do NOT load: world/ Rust code, simulation internals, network schemas.

---

## Process

1. **Create building primitive types** `src/primitives.ts`:
   - Define all 9 piece types with default dimensions
   - `createPieceGeometry(piece: BuildingPiece)` → THREE.BufferGeometry
   - Wedge geometry generator for ramps/roofs
   - Door/window frame as multi-box composition

2. **Create grid system** `src/grid.ts`:
   - `snapToGrid(position: Vec3, gridSize: number)` → snapped position
   - `snapRotation(angle: number, snapDeg: number)` → snapped angle
   - Grid visualization mesh (translucent plane with lines)

3. **Create ghost preview** `src/components/GhostPreview.tsx`:
   - Translucent preview mesh following cursor (raycast onto terrain/pieces)
   - Color: green (valid) / red (invalid)
   - Updates in useFrame() — no React re-renders
   - Responds to Q/E rotation, R tilt

4. **Create placement validation** `src/validation.ts`:
   - `isValidPlacement(piece, existingPieces, playerPos)` → { valid, reason }
   - Overlap detection (AABB intersection)
   - Ground connectivity check (BFS through connected pieces)
   - Build distance check
   - Piece limit check

5. **Create build mode component** `src/components/BuildMode.tsx`:
   - B key toggles build mode
   - Renders: BuildGrid, GhostPreview, BuildModeUI
   - Manages piece selection (1-9 keys)
   - Handles click-to-place → sends CREATE action via network
   - Handles delete → sends DESTROY action
   - Undo/redo stack (Ctrl+Z/Y)

6. **Create material system** `src/materials.ts`:
   - 16 material definitions with MeshStandardMaterial params
   - Material picker UI component
   - Per-piece material application

7. **Create placed pieces renderer** `src/components/PlacedPieces.tsx`:
   - Instanced rendering for each material type (one InstancedMesh per material)
   - Updates when pieces are added/removed
   - Receives building data from world state snapshot

8. **Wire into NexusScene**: Add `<BuildMode />` component

9. **Write tests** (all 12 from MANIFEST.md testing requirements)

---

## Checkpoints

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 2 | Grid visualization renders on terrain | approve / revise grid size |
| Step 5 | Place walls and floors, ghost preview works | approve → materials / revise |
| Step 7 | Full build mode with materials, other players see placed pieces | approve → tests / revise |

---

## Outputs

| Output | Location |
|--------|----------|
| Building system | `src/` |
| Tests | `src/__tests__/` |
| Phase 1 completion summary | `output/phase1-complete.md` |
