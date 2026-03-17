/**
 * EntityField — GPU-instanced capsule rendering for all nearby entities.
 *
 * Spec:
 *   - Procedural capsule mesh, ~200 triangles (CapsuleGeometry)
 *   - One draw call for all N entities via InstancedMesh
 *   - Per-frame: reads WorldSnapshot, updates instance matrices, sets needsUpdate
 *   - Player entity rendered identically to others in Phase 0 (no special treatment)
 *
 * Hot path (inside useFrame):
 *   - No heap allocation: _dummy Object3D reused across all entities every frame
 *   - instanceMatrix.needsUpdate toggled only (not re-created)
 *   - meshRef.current null-checked once per frame, not per entity
 */
import { useRef, useMemo } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import type { WorldSnapshot } from '../types/world'

// Capsule dimensions: radius=0.3, length=1.4 → total height ~2.0 units (approx human scale)
// capSegments=4, radialSegments=8 → ~192 triangles per capsule
const CAPSULE_RADIUS = 0.3
const CAPSULE_LENGTH = 1.4
const CAP_SEGMENTS = 4
const RADIAL_SEGMENTS = 8

// Max instance buffer size — allocate once, never resize in Phase 0
const MAX_ENTITIES = 512

// Player color vs other entity color
const PLAYER_COLOR = new THREE.Color('#e8c840')
const ENTITY_COLOR = new THREE.Color('#3a8fc7')

interface Props {
  snapshotRef: React.MutableRefObject<WorldSnapshot | null>
}

// Reused across frames — avoids allocation in hot path
const _dummy = new THREE.Object3D()

export function EntityField({ snapshotRef }: Props) {
  const meshRef = useRef<THREE.InstancedMesh>(null)

  const geometry = useMemo(
    () => new THREE.CapsuleGeometry(CAPSULE_RADIUS, CAPSULE_LENGTH, CAP_SEGMENTS, RADIAL_SEGMENTS),
    []
  )

  const material = useMemo(
    () => new THREE.MeshLambertMaterial({ color: ENTITY_COLOR }),
    []
  )

  // Per-entity color array — set once, player entity gets highlight color
  const colorArray = useMemo(() => {
    const arr = new Float32Array(MAX_ENTITIES * 3)
    for (let i = 0; i < MAX_ENTITIES; i++) {
      arr[i * 3 + 0] = ENTITY_COLOR.r
      arr[i * 3 + 1] = ENTITY_COLOR.g
      arr[i * 3 + 2] = ENTITY_COLOR.b
    }
    return arr
  }, [])

  useFrame(() => {
    const mesh = meshRef.current
    if (!mesh) return

    const snapshot = snapshotRef.current
    if (!snapshot) return

    const entities = snapshot.nearby_entities
    const playerId = snapshot.player_entity_id
    const count = Math.min(entities.length, MAX_ENTITIES)

    for (let i = 0; i < count; i++) {
      const e = entities[i]
      // Capsule origin is at its centre; raise by half height so base sits on terrain
      _dummy.position.set(e.x, e.y + CAPSULE_RADIUS + CAPSULE_LENGTH * 0.5, e.z)
      _dummy.rotation.set(0, e.yaw, 0)
      _dummy.updateMatrix()
      mesh.setMatrixAt(i, _dummy.matrix)

      // Highlight player entity
      if (e.id === playerId) {
        mesh.setColorAt(i, PLAYER_COLOR)
      } else {
        mesh.setColorAt(i, ENTITY_COLOR)
      }
    }

    mesh.count = count
    mesh.instanceMatrix.needsUpdate = true
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true
  })

  return (
    <instancedMesh
      ref={meshRef}
      args={[geometry, material, MAX_ENTITIES]}
      frustumCulled={false}
      // frustumCulled=false: Phase 0 has no visibility system.
      // All entities are treated as visible (per CONTEXT.md Step 3 Phase 0 rule).
    />
  )
}
