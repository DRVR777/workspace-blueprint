/**
 * NexusScene — the render loop implementation.
 *
 * Maps the MANIFEST.md "EACH FRAME" pseudocode to R3F's component model:
 *
 *   Step 1: snapshot    →  useWorldState() (stub or network)
 *   Step 2: camera      →  NexusCamera (FOV/near/far)
 *   Step 3: controls    →  PlayerController (ground) OR VehicleSystem (plane/sub/car)
 *   Step 4: cull        →  skipped Phase 0
 *   Step 5: batches     →  EntityField (InstancedMesh)
 *   Step 6: terrain     →  Terrain
 *   Step 7: vehicles    →  VehicleManager (updates active vehicle)
 *   Step 8: constellation → Constellation (world nodes visible in sky)
 */
import { useEffect } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import * as THREE from 'three'

import { NexusCamera } from './NexusCamera'
import { PlayerController } from './PlayerController'
import { SunLight } from './SunLight'
import { Terrain } from './Terrain'
import { EntityField } from './EntityField'
import { FrameMetrics } from './FrameMetrics'
import { Constellation } from './Constellation'
import { WorldObjects } from './WorldObjects'
import { useWorldState } from '../hooks/useWorldState'

// Vehicle system
import {
  registerVehicle,
  updateActiveVehicle,
  enterVehicle,
  exitVehicle,
  isAnyVehicleActive,
} from '../vehicles/VehicleSystem'
import { PlaneVehicle } from '../vehicles/PlaneController'

export function NexusScene() {
  const { camera, scene } = useThree()
  const snapshotRef = useWorldState()

  // Register vehicles on mount
  useEffect(() => {
    registerVehicle(new PlaneVehicle())
    // Future: registerVehicle(new SubmarineVehicle())
    // Future: registerVehicle(new CarVehicle())
    // Future: registerVehicle(new DroneVehicle())

    // Tab enters fly mode (vehicle handles Tab internally to switch to plane mode)
    // Escape exits vehicle entirely
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Tab') {
        e.preventDefault()
        if (!isAnyVehicleActive()) {
          enterVehicle('plane', camera.position.clone(), camera.quaternion.clone())
        }
        // Tab while active is handled inside PlaneVehicle (fly ↔ plane toggle)
      }
      if (e.key === 'Escape' && isAnyVehicleActive()) {
        const pos = exitVehicle()
        if (pos) {
          camera.position.copy(pos)
          camera.position.y = Math.max(camera.position.y, 1.6)
        }
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [camera])

  // Update active vehicle each frame.
  // Priority 100 = runs after all default-priority hooks (PlayerController,
  // EntityField, useWorldState) so the plane's camera writes are never
  // overwritten by something running later in the same frame.
  useFrame((_, delta) => {
    updateActiveVehicle(
      camera as THREE.PerspectiveCamera,
      scene,
      delta,
    )
  }, 100)

  return (
    <>
      {/* Camera config */}
      <NexusCamera />

      {/* Ground controls (yields when vehicle is active) */}
      <PlayerController />

      {/* Lighting */}
      <SunLight />

      {/* World terrain */}
      <Terrain />

      {/* World objects (trees, props) */}
      <WorldObjects />

      {/* Entities (all players) */}
      <EntityField snapshotRef={snapshotRef} />

      {/* Constellation — world nodes visible in the sky */}
      <Constellation />

      {/* Frame metrics */}
      <FrameMetrics />
    </>
  )
}
