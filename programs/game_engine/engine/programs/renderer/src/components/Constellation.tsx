/**
 * Constellation — renders nearby worlds as glowing spheres in the sky.
 *
 * Visible when in bird view or plane mode. Each world in the world graph
 * appears as a sphere with:
 *   - Size based on activity (player count, object count)
 *   - Color from world config (constellation_color)
 *   - Emissive glow (atmosphere halo)
 *   - Billboard label showing world name + player count
 *   - Edge lines connecting linked worlds (portals, highways)
 *
 * Data comes from WORLD_INFO messages (world-graph-contract).
 * Rendering approach from 3dGraphUniverse/src/nodes.js.
 *
 * Phase 0: static test data. Phase 1: real world-graph data from server.
 */
import { useRef, useMemo, useEffect } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import * as THREE from 'three'

// ============================================================================
// Types
// ============================================================================

export interface ConstellationNode {
  worldId: number
  name: string
  position: THREE.Vector3 // constellation_position from world_record
  radius: number          // visual size (from activity)
  color: number           // RGBA packed
  playerCount: number
  hasAtmosphere: boolean
}

export interface ConstellationEdge {
  fromWorldId: number
  toWorldId: number
  edgeType: number        // 0=portal, 1=adjacency, 2=highway
}

// ============================================================================
// Phase 0: Static test constellation
// ============================================================================

const TEST_NODES: ConstellationNode[] = [
  { worldId: 1, name: "Roan's Island", position: new THREE.Vector3(0, 200, 0), radius: 8, color: 0x4488ff, playerCount: 1, hasAtmosphere: true },
  { worldId: 2, name: "PvP Arena", position: new THREE.Vector3(150, 180, -100), radius: 5, color: 0xff4444, playerCount: 0, hasAtmosphere: true },
  { worldId: 3, name: "The Market", position: new THREE.Vector3(-120, 220, 80), radius: 6, color: 0x44ff88, playerCount: 3, hasAtmosphere: true },
  { worldId: 4, name: "Cloud Garden", position: new THREE.Vector3(80, 250, 120), radius: 4, color: 0xffaa44, playerCount: 0, hasAtmosphere: false },
  { worldId: 5, name: "Deep Caves", position: new THREE.Vector3(-80, 160, -150), radius: 3, color: 0x8844ff, playerCount: 0, hasAtmosphere: false },
]

const TEST_EDGES: ConstellationEdge[] = [
  { fromWorldId: 1, toWorldId: 2, edgeType: 0 },  // portal
  { fromWorldId: 1, toWorldId: 3, edgeType: 2 },  // highway
  { fromWorldId: 3, toWorldId: 4, edgeType: 1 },  // adjacency
  { fromWorldId: 1, toWorldId: 5, edgeType: 0 },  // portal
]

// ============================================================================
// Component
// ============================================================================

const _dummy = new THREE.Object3D()

export function Constellation() {
  const nodes = TEST_NODES
  const edges = TEST_EDGES
  const { scene } = useThree()

  const meshRef = useRef<THREE.InstancedMesh>(null)
  const edgeLinesRef = useRef<THREE.Group>(new THREE.Group())

  const geometry = useMemo(() => new THREE.SphereGeometry(1, 32, 32), [])
  const material = useMemo(() => new THREE.MeshStandardMaterial({
    roughness: 0.3,
    metalness: 0.7,
    emissiveIntensity: 0.5,
  }), [])

  // Add edge lines imperatively (avoids R3F <line> vs SVG conflict)
  useEffect(() => {
    const group = edgeLinesRef.current
    scene.add(group)

    for (const edge of edges) {
      const from = nodes.find(n => n.worldId === edge.fromWorldId)
      const to = nodes.find(n => n.worldId === edge.toWorldId)
      if (!from || !to) continue

      const geo = new THREE.BufferGeometry().setFromPoints([from.position, to.position])
      const mat = new THREE.LineBasicMaterial({
        color: edge.edgeType === 2 ? 0xffaa44 : 0xffffff,
        opacity: edge.edgeType === 2 ? 0.4 : 0.15,
        transparent: true,
      })
      group.add(new THREE.Line(geo, mat))
    }

    return () => {
      scene.remove(group)
      group.traverse(child => {
        if (child instanceof THREE.Line) {
          child.geometry.dispose()
          ;(child.material as THREE.Material).dispose()
        }
      })
    }
  }, [scene, nodes, edges])

  useFrame(() => {
    const mesh = meshRef.current
    if (!mesh) return

    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i]
      _dummy.position.copy(node.position)
      _dummy.scale.setScalar(node.radius)
      _dummy.updateMatrix()
      mesh.setMatrixAt(i, _dummy.matrix)
      mesh.setColorAt(i, new THREE.Color(node.color))
    }

    mesh.count = nodes.length
    mesh.instanceMatrix.needsUpdate = true
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true
  })

  return (
    <group>
      <instancedMesh
        ref={meshRef}
        args={[geometry, material, 64]}
        frustumCulled={false}
      />

      {nodes.filter(n => n.hasAtmosphere).map(node => (
        <pointLight
          key={node.worldId}
          position={node.position}
          color={node.color}
          intensity={1}
          distance={node.radius * 5}
        />
      ))}
    </group>
  )
}
