/**
 * Constellation — nearby worlds rendered as floating semi-sphere islands.
 *
 * Each world is a hemisphere (flat on top, rounded underneath) with:
 *   - Green top surface (terrain)
 *   - Dark brown underside (rock)
 *   - Glowing atmosphere halo
 *   - Name label (Phase 2)
 *
 * Highways between worlds are thick glowing tubes.
 * Portal connections are thin lines.
 *
 * Phase 0: static test data. Phase 1: real WORLD_INFO from server.
 */
import { useRef, useEffect } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import * as THREE from 'three'

// ============================================================================
// Types
// ============================================================================

export interface ConstellationNode {
  worldId: number
  name: string
  position: THREE.Vector3
  radius: number
  color: number
  playerCount: number
  hasAtmosphere: boolean
}

export interface ConstellationEdge {
  fromWorldId: number
  toWorldId: number
  edgeType: number // 0=portal, 1=adjacency, 2=highway
}

// ============================================================================
// Test data — replace with real world-graph data in Phase 1
// ============================================================================

const TEST_NODES: ConstellationNode[] = [
  { worldId: 1, name: "Starter Island", position: new THREE.Vector3(0, 0, 0), radius: 50, color: 0x4a7c3f, playerCount: 1, hasAtmosphere: true },
  { worldId: 2, name: "PvP Arena", position: new THREE.Vector3(250, 30, -180), radius: 35, color: 0x8b3a2a, playerCount: 0, hasAtmosphere: true },
  { worldId: 3, name: "The Market", position: new THREE.Vector3(-200, -20, 150), radius: 40, color: 0x4a8c6f, playerCount: 3, hasAtmosphere: true },
  { worldId: 4, name: "Cloud Garden", position: new THREE.Vector3(180, 80, 200), radius: 25, color: 0x6a9c4f, playerCount: 0, hasAtmosphere: true },
  { worldId: 5, name: "Deep Caves", position: new THREE.Vector3(-150, -50, -200), radius: 20, color: 0x4a4a6a, playerCount: 0, hasAtmosphere: false },
]

const TEST_EDGES: ConstellationEdge[] = [
  { fromWorldId: 1, toWorldId: 2, edgeType: 2 },  // highway
  { fromWorldId: 1, toWorldId: 3, edgeType: 2 },  // highway
  { fromWorldId: 3, toWorldId: 4, edgeType: 0 },  // portal
  { fromWorldId: 1, toWorldId: 5, edgeType: 0 },  // portal
]

// ============================================================================
// Component
// ============================================================================

export function Constellation() {
  const { scene } = useThree()
  const groupRef = useRef<THREE.Group | null>(null)

  // Build world island meshes
  useEffect(() => {
    const group = new THREE.Group()
    groupRef.current = group
    scene.add(group)

    // Create each world as a semi-sphere island
    for (const node of TEST_NODES) {
      // Skip the player's current world (worldId=1, already rendered as Terrain)
      if (node.worldId === 1) continue

      const island = createIslandMesh(node)
      group.add(island)
    }

    // Create edges (highways = thick tubes, portals = thin lines)
    for (const edge of TEST_EDGES) {
      const from = TEST_NODES.find(n => n.worldId === edge.fromWorldId)
      const to = TEST_NODES.find(n => n.worldId === edge.toWorldId)
      if (!from || !to) continue

      if (edge.edgeType === 2) {
        // Highway — thick tube
        const highway = createHighway(from.position, to.position)
        group.add(highway)
      } else {
        // Portal — thin glowing line
        const line = createPortalLine(from.position, to.position)
        group.add(line)
      }
    }

    return () => {
      scene.remove(group)
      group.traverse(child => {
        if (child instanceof THREE.Mesh || child instanceof THREE.Line) {
          child.geometry.dispose()
          const mat = child.material
          if (Array.isArray(mat)) mat.forEach(m => m.dispose())
          else (mat as THREE.Material).dispose()
        }
      })
    }
  }, [scene])

  // Slow rotation on distant islands for visual interest
  useFrame((_, delta) => {
    if (!groupRef.current) return
    groupRef.current.children.forEach(child => {
      if (child.userData.isIsland) {
        child.rotation.y += delta * 0.05 // very slow rotation
      }
    })
  })

  return null // all meshes added imperatively
}

// ============================================================================
// Mesh builders
// ============================================================================

function createIslandMesh(node: ConstellationNode): THREE.Group {
  const group = new THREE.Group()
  group.position.copy(node.position)
  group.userData.isIsland = true
  group.userData.worldId = node.worldId

  const r = node.radius

  // Top surface — flat circle (terrain) — 16 segments (was 32)
  const topGeo = new THREE.CircleGeometry(r, 16)
  const topMat = new THREE.MeshLambertMaterial({ color: node.color }) // Lambert cheaper than Standard
  const top = new THREE.Mesh(topGeo, topMat)
  top.rotation.x = -Math.PI / 2
  group.add(top)

  // Bottom — hemisphere (rock underside) — 12x6 segments (was 32x16)
  const bottomGeo = new THREE.SphereGeometry(r, 12, 6, 0, Math.PI * 2, Math.PI / 2, Math.PI / 2)
  const bottomMat = new THREE.MeshLambertMaterial({ color: 0x3a2a1a, side: THREE.BackSide })
  group.add(new THREE.Mesh(bottomGeo, bottomMat))

  // Atmosphere glow — only a point light, no glow sphere (saves 1 draw call per island)
  if (node.hasAtmosphere) {
    const light = new THREE.PointLight(node.color, 0.5, r * 4)
    light.position.y = r * 0.5
    group.add(light)
  }

  return group
}

function createHighway(from: THREE.Vector3, to: THREE.Vector3): THREE.Mesh {
  const curve = new THREE.LineCurve3(from, to)
  // Reduced segments: 8 along path × 4 radial (was 32×8 = 256 faces → now 32 faces)
  const tubeGeo = new THREE.TubeGeometry(curve, 8, 1.5, 4, false)
  const tubeMat = new THREE.MeshLambertMaterial({
    color: 0xffaa44,
    emissive: 0xff8800,
    emissiveIntensity: 0.3,
    transparent: true,
    opacity: 0.7,
  })

  const highway = new THREE.Mesh(tubeGeo, tubeMat)
  highway.userData.isHighway = true
  return highway
}

function createPortalLine(from: THREE.Vector3, to: THREE.Vector3): THREE.Line {
  const geo = new THREE.BufferGeometry().setFromPoints([from, to])
  const mat = new THREE.LineBasicMaterial({
    color: 0x8888ff,
    opacity: 0.25,
    transparent: true,
  })
  return new THREE.Line(geo, mat)
}
