/**
 * Terrain — circular island with grid overlay.
 *
 * The world is a flat circular platform (not an infinite plane).
 * Grid lines on the surface for building reference (0.3m snap grid shown as 3m major lines).
 * Edge of the island has a slight rim/lip.
 */
import { useMemo } from 'react'
import * as THREE from 'three'

const ISLAND_RADIUS = 50    // meters — explorable area
const GRID_SPACING = 3.0    // meters — major grid lines (10x the 0.3m snap grid)
const GRID_COLOR = 0x5a8c4f // slightly lighter than terrain
const TERRAIN_COLOR = 0x4a7c3f

export function Terrain() {
  // Circular island
  const circleGeo = useMemo(
    () => new THREE.CircleGeometry(ISLAND_RADIUS, 64),
    []
  )

  // Grid lines on the surface
  const gridLines = useMemo(() => {
    const points: THREE.Vector3[] = []
    const half = ISLAND_RADIUS

    // Lines along X axis
    for (let z = -half; z <= half; z += GRID_SPACING) {
      // Clip line to circle
      const maxX = Math.sqrt(Math.max(0, ISLAND_RADIUS * ISLAND_RADIUS - z * z))
      if (maxX > 0) {
        points.push(new THREE.Vector3(-maxX, 0.01, z))
        points.push(new THREE.Vector3(maxX, 0.01, z))
      }
    }

    // Lines along Z axis
    for (let x = -half; x <= half; x += GRID_SPACING) {
      const maxZ = Math.sqrt(Math.max(0, ISLAND_RADIUS * ISLAND_RADIUS - x * x))
      if (maxZ > 0) {
        points.push(new THREE.Vector3(x, 0.01, -maxZ))
        points.push(new THREE.Vector3(x, 0.01, maxZ))
      }
    }

    const geo = new THREE.BufferGeometry().setFromPoints(points)
    return geo
  }, [])

  // Rim/edge ring around the island
  const rimGeo = useMemo(() => {
    const shape = new THREE.TorusGeometry(ISLAND_RADIUS, 0.3, 8, 64)
    return shape
  }, [])

  return (
    <group>
      {/* Main circular terrain surface */}
      <mesh
        geometry={circleGeo}
        rotation={[-Math.PI / 2, 0, 0]}
        position={[0, 0, 0]}
        receiveShadow
      >
        <meshStandardMaterial
          color={TERRAIN_COLOR}
          roughness={0.9}
          metalness={0.0}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Grid lines */}
      <lineSegments geometry={gridLines}>
        <lineBasicMaterial color={GRID_COLOR} opacity={0.3} transparent />
      </lineSegments>

      {/* Rim around island edge */}
      <mesh
        geometry={rimGeo}
        rotation={[-Math.PI / 2, 0, 0]}
        position={[0, 0, 0]}
      >
        <meshStandardMaterial color={0x3a6c2f} roughness={0.8} />
      </mesh>

      {/* Underside — dark hemisphere so the island looks solid from below */}
      <mesh position={[0, -0.5, 0]}>
        <sphereGeometry args={[ISLAND_RADIUS, 32, 16, 0, Math.PI * 2, Math.PI / 2, Math.PI / 2]} />
        <meshStandardMaterial color={0x2a1a0a} roughness={1.0} side={THREE.BackSide} />
      </mesh>
    </group>
  )
}
