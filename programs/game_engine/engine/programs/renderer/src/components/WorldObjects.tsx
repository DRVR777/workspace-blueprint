/**
 * WorldObjects — loads and places 3D assets on the island.
 *
 * Phase 0: places tree.glb models at fixed positions.
 * Phase 1: objects loaded from server world state.
 */
import { useEffect, useRef } from 'react'
import { useThree } from '@react-three/fiber'
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'

// Tree positions around the island (hand-placed, inside 50m radius)
const TREE_POSITIONS: [number, number, number][] = [
  [15, 0, 10],
  [-20, 0, 15],
  [8, 0, -25],
  [-12, 0, -18],
  [30, 0, -5],
  [-35, 0, 8],
  [5, 0, 35],
  [-8, 0, -38],
  [25, 0, 25],
  [-28, 0, -12],
  [40, 0, 3],
  [-15, 0, 30],
  [18, 0, -35],
  [-40, 0, -5],
  [10, 0, -10],
]

const TREE_SCALE_MIN = 2.0
const TREE_SCALE_MAX = 4.0

export function WorldObjects() {
  const { scene } = useThree()
  const groupRef = useRef<THREE.Group | null>(null)

  useEffect(() => {
    const group = new THREE.Group()
    groupRef.current = group
    scene.add(group)

    // Load tree model
    new GLTFLoader().load('/models/tree.glb', (gltf) => {
      const treeTemplate = gltf.scene

      for (const [x, y, z] of TREE_POSITIONS) {
        // Check position is within island radius
        const dist = Math.sqrt(x * x + z * z)
        if (dist > 45) continue // leave 5m margin from edge

        const tree = treeTemplate.clone()
        const scale = TREE_SCALE_MIN + Math.random() * (TREE_SCALE_MAX - TREE_SCALE_MIN)
        tree.scale.setScalar(scale)
        tree.position.set(x, y, z)
        tree.rotation.y = Math.random() * Math.PI * 2 // random rotation
        tree.castShadow = true

        // Mark as non-collidable for now (Phase 1: add collision)
        tree.traverse(child => {
          if (child instanceof THREE.Mesh) {
            child.castShadow = true
            child.receiveShadow = true
          }
        })

        group.add(tree)
      }

      console.log(`[WorldObjects] Placed ${TREE_POSITIONS.length} trees`)
    }, undefined, (err) => {
      console.warn('[WorldObjects] Failed to load tree.glb:', err)
    })

    return () => {
      scene.remove(group)
      group.traverse(child => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose()
          const mat = child.material
          if (Array.isArray(mat)) mat.forEach(m => m.dispose())
          else (mat as THREE.Material).dispose()
        }
      })
    }
  }, [scene])

  return null
}
