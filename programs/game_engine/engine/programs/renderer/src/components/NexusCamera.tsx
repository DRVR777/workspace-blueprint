/**
 * NexusCamera — implements the camera spec from CONTEXT.md Step 2.
 *
 * FoV: 90°  near: 0.1  far: 5000
 *
 * Phase 0: OrbitControls enabled for visual verification.
 * Phase 1: Replace OrbitControls with player-tracking camera
 *   (camera.position = player_position + offset, look at player).
 */
import { useEffect } from 'react'
import { useThree } from '@react-three/fiber'
import { OrbitControls } from '@react-three/drei'

const FOV = 90
const NEAR = 0.1
const FAR = 5000
const INITIAL_POSITION: [number, number, number] = [0, 80, 120]

export function NexusCamera() {
  const { camera } = useThree()

  useEffect(() => {
    // Apply spec values — R3F default camera has fov=75, near=0.1, far=1000
    if ('fov' in camera) {
      (camera as THREE.PerspectiveCamera).fov = FOV;
      (camera as THREE.PerspectiveCamera).near = NEAR;
      (camera as THREE.PerspectiveCamera).far = FAR;
      camera.position.set(...INITIAL_POSITION)
      camera.updateProjectionMatrix()
    }
  }, [camera])

  // OrbitControls: Phase 0 visual verification. Phase 1: remove this.
  return <OrbitControls makeDefault enableDamping dampingFactor={0.05} />
}

// THREE import needed for type cast above — pulled from global r3f context
import * as THREE from 'three'
