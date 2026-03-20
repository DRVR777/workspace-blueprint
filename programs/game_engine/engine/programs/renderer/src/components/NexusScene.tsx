/**
 * NexusScene — the render loop implementation.
 *
 * Maps the MANIFEST.md "EACH FRAME" pseudocode to R3F's component model:
 *
 *   Step 1: snapshot  →  useWorldState() (stub or network, fills snapshotRef)
 *   Step 2: camera    →  NexusCamera (sets FOV/near/far)
 *   Step 3: controls  →  PlayerController (WASD + pointer lock + network actions)
 *   Step 4: cull      →  skipped Phase 0 (EntityField uses frustumCulled=false)
 *   Step 5: batches   →  EntityField (InstancedMesh handles batching internally)
 *   Step 6: terrain   →  Terrain
 *   Step 7: draw      →  Three.js render pass (R3F manages this)
 */
import { NexusCamera } from './NexusCamera'
import { PlayerController } from './PlayerController'
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
      {/* Step 2: camera (FOV, near, far) */}
      <NexusCamera />

      {/* Step 3: first-person controls + network action sending */}
      <PlayerController />

      {/* Lighting — sun + ambient diffuse */}
      <SunLight />

      {/* Step 6: terrain */}
      <Terrain />

      {/* Steps 5 + 7: entity batch + draw (single instanced call) */}
      <EntityField snapshotRef={snapshotRef} />

      {/* Frame time measurement */}
      <FrameMetrics />
    </>
  )
}
