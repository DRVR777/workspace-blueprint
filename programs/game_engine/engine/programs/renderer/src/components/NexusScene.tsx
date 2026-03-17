/**
 * NexusScene — the render loop implementation.
 *
 * Maps the MANIFEST.md "EACH FRAME" pseudocode to R3F's component model:
 *
 *   Step 1: snapshot  →  useWorldState() (runs in useFrame, fills snapshotRef)
 *   Step 2: camera    →  NexusCamera (reads snapshotRef.player position in Phase 1)
 *   Step 3: cull      →  skipped Phase 0 (EntityField uses frustumCulled=false)
 *   Step 4: cull ents →  skipped Phase 0
 *   Step 5: batches   →  EntityField (InstancedMesh handles batching internally)
 *   Step 6: terrain   →  Terrain
 *   Step 7: draw      →  Three.js render pass (R3F manages this)
 *   Step 8: entities  →  EntityField (instanced, single draw call)
 *   Step 9: post      →  skipped Phase 0
 *   Step 10: present  →  R3F Canvas manages swap
 */
import { NexusCamera } from './NexusCamera'
import { SunLight } from './SunLight'
import { Terrain } from './Terrain'
import { EntityField } from './EntityField'
import { FrameMetrics } from './FrameMetrics'
import { useWorldState } from '../hooks/useWorldState'

export function NexusScene() {
  // Step 1: snapshot — updated every frame inside useWorldState
  const snapshotRef = useWorldState()

  return (
    <>
      {/* Step 2: camera */}
      <NexusCamera />

      {/* Lighting — sun + ambient diffuse */}
      <SunLight />

      {/* Step 6: terrain */}
      <Terrain />

      {/* Steps 5 + 8: entity batch + draw (single instanced call) */}
      <EntityField snapshotRef={snapshotRef} />

      {/* Frame time measurement */}
      <FrameMetrics />
    </>
  )
}
