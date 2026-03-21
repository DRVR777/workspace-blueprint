/**
 * EntityField — GPU-instanced rendering for all nearby entities.
 *
 * Draw strategy:
 *   - Capsule InstancedMesh for on-foot entities (vehicleMode !== 1) — one draw call
 *   - Per-entity THREE.Group for plane-mode entities (vehicleMode === 1):
 *       • Tries to use the plane.glb model (loaded once via GLTFLoader)
 *       • Falls back to the procedural placeholder geometry if GLB is absent
 *     Groups are created/destroyed as entities enter/exit plane mode and their
 *     position + quaternion are updated imperatively each frame in useFrame.
 *
 * Hot path (inside useFrame):
 *   - No heap allocation: _dummy Object3D reused across all capsule entities
 *   - Plane groups stored in a Map<id, Group> — O(1) lookup per entity
 *   - GLB loaded once, cloned (deep) per plane entity — not re-created each frame
 */
import { useRef, useMemo, useEffect } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'
import type { WorldSnapshot } from '../types/world'

// Capsule dimensions
const CAPSULE_RADIUS = 0.3
const CAPSULE_LENGTH = 1.4
const CAP_SEGMENTS   = 4
const RADIAL_SEGMENTS = 8

const MAX_ENTITIES = 512

const PLAYER_COLOR = new THREE.Color('#e8c840')
const ENTITY_COLOR = new THREE.Color('#3a8fc7')
const PLANE_COLOR  = new THREE.Color('#6366f1')

// Plane scale — must match PlaneController.planeModel.scale.setScalar(3.4)
const PLANE_GLB_SCALE = 3.4

interface Props {
  snapshotRef: React.MutableRefObject<WorldSnapshot | null>
}

// Reused across frames — avoids allocation in hot path
const _dummy = new THREE.Object3D()

export function EntityField({ snapshotRef }: Props) {
  const { scene } = useThree()

  // ── Capsule InstancedMesh (on-foot entities) ──────────────────────────────
  const capsuleRef = useRef<THREE.InstancedMesh>(null)

  const capsuleGeo = useMemo(
    () => new THREE.CapsuleGeometry(CAPSULE_RADIUS, CAPSULE_LENGTH, CAP_SEGMENTS, RADIAL_SEGMENTS),
    []
  )
  const capsuleMat = useMemo(
    () => new THREE.MeshLambertMaterial({ color: ENTITY_COLOR }),
    []
  )

  // ── Plane entities — individual Three.js Groups ───────────────────────────
  // GLB template (deep-cloned per entity)
  const glbTemplateRef = useRef<THREE.Group | null>(null)
  const glbLoadedRef   = useRef(false)

  // Procedural fallback geometry (shared across all plane instances)
  const planeGeoFallback = useMemo(() => {
    const wings    = new THREE.BoxGeometry(3, 0.08, 0.6)
    const fuselage = new THREE.BoxGeometry(0.4, 0.4, 2.0)
    const tail     = new THREE.BoxGeometry(1, 0.08, 0.4)
    tail.translate(0, 0.3, 0.9)
    return mergeGeometries([wings, fuselage, tail])
  }, [])
  const planeMatFallback = useMemo(
    () => new THREE.MeshLambertMaterial({ color: PLANE_COLOR }),
    []
  )

  // Live plane entity groups: entity id → Group in the R3F scene
  const planeGroupsRef = useRef<Map<number, THREE.Group>>(new Map())

  // Load GLB once on mount
  useEffect(() => {
    const loader = new GLTFLoader()
    loader.load(
      (import.meta as unknown as { env: { BASE_URL: string } }).env.BASE_URL + 'models/plane.glb',
      (gltf) => {
        glbTemplateRef.current = gltf.scene
        glbLoadedRef.current   = true
      },
      undefined,
      () => {
        // GLB not found — procedural fallback will be used
        glbLoadedRef.current = true
      },
    )
  }, [])

  // Cleanup all plane groups when component unmounts
  useEffect(() => {
    const groups = planeGroupsRef.current
    return () => {
      for (const group of groups.values()) scene.remove(group)
      groups.clear()
    }
  }, [scene])

  // ── Per-frame update ──────────────────────────────────────────────────────
  useFrame(() => {
    const capsule = capsuleRef.current
    if (!capsule) return

    const snapshot = snapshotRef.current
    if (!snapshot) return

    const entities = snapshot.nearby_entities
    const playerId = snapshot.player_entity_id

    let ci = 0  // capsule instance index

    // Track which plane entity IDs we saw this frame
    const seenPlaneIds = new Set<number>()

    for (let i = 0; i < entities.length && ci < MAX_ENTITIES; i++) {
      const e = entities[i]

      if (e.vehicleMode === 1) {
        // ── Plane entity ────────────────────────────────────────────────────
        // Skip the local player — they see their own smooth local plane from
        // PlaneController. Rendering the server-synced plane on top would flicker.
        if (e.id === playerId) continue

        seenPlaneIds.add(e.id)

        let group = planeGroupsRef.current.get(e.id)
        if (!group) {
          group = new THREE.Group()
          if (glbTemplateRef.current) {
            const clone = glbTemplateRef.current.clone(true)
            clone.scale.setScalar(PLANE_GLB_SCALE)
            // GLB forward axis is +Z; physics uses -Z (nose direction).
            // Rotate 180° around Y so the model faces the right way for observers.
            clone.rotation.y = Math.PI
            group.add(clone)
          } else {
            group.add(new THREE.Mesh(planeGeoFallback, planeMatFallback))
          }
          scene.add(group)
          planeGroupsRef.current.set(e.id, group)
        }

        group.position.set(e.x, e.y, e.z)
        group.quaternion.set(e.qx ?? 0, e.qy ?? 0, e.qz ?? 0, e.qw ?? 1)

      } else {
        // ── On-foot entity (capsule) ────────────────────────────────────────
        _dummy.position.set(e.x, e.y + CAPSULE_RADIUS + CAPSULE_LENGTH * 0.5, e.z)
        _dummy.rotation.set(0, e.yaw, 0)
        _dummy.updateMatrix()
        capsule.setMatrixAt(ci, _dummy.matrix)
        capsule.setColorAt(ci, e.id === playerId ? PLAYER_COLOR : ENTITY_COLOR)
        ci++
      }
    }

    // Remove groups for entities that are no longer in plane mode
    for (const [id, group] of planeGroupsRef.current) {
      if (!seenPlaneIds.has(id)) {
        scene.remove(group)
        planeGroupsRef.current.delete(id)
      }
    }

    capsule.count = ci
    capsule.instanceMatrix.needsUpdate = true
    if (capsule.instanceColor) capsule.instanceColor.needsUpdate = true
  })

  return (
    <instancedMesh
      ref={capsuleRef}
      args={[capsuleGeo, capsuleMat, MAX_ENTITIES]}
      frustumCulled={false}
    />
  )
}
