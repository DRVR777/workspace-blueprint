/**
 * NexusCamera — sets camera parameters from spec.
 *
 * FoV: 75°  near: 0.1  far: 5000
 * Starting position: (0, 1.6, 5) — eye height, facing forward
 *
 * Controls are handled by PlayerController (PointerLockControls).
 * This component only configures the camera — it does not add controls.
 */
import { useEffect } from 'react'
import { useThree } from '@react-three/fiber'
import * as THREE from 'three'

const FOV = 75
const NEAR = 0.1
const FAR = 5000
const INITIAL_POSITION: [number, number, number] = [0, 1.6, 5]

export function NexusCamera() {
  const { camera } = useThree()

  useEffect(() => {
    if ('fov' in camera) {
      const pc = camera as THREE.PerspectiveCamera
      pc.fov = FOV
      pc.near = NEAR
      pc.far = FAR
      camera.position.set(...INITIAL_POSITION)
      camera.updateProjectionMatrix()
    }
  }, [camera])

  return null
}
