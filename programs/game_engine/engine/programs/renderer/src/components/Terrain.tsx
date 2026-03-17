/**
 * Terrain — flat 1000×1000 terrain quad (Phase 0).
 *
 * Spec: PlaneGeometry 1000×1000 units, simple diffuse material.
 * The plane is rotated -90° on X so it lies flat (Three.js planes
 * face +Z by default). Uploaded once as a static geometry.
 *
 * Phase 1: replace with chunked terrain from TerrainChunk contract.
 */
import { useMemo } from 'react'
import * as THREE from 'three'

const TERRAIN_SIZE = 1000
// 100×100 grid = 10,000 quads = 20,000 triangles; fine for Phase 0
const TERRAIN_SEGMENTS = 100

export function Terrain() {
  const geometry = useMemo(
    () => new THREE.PlaneGeometry(TERRAIN_SIZE, TERRAIN_SIZE, TERRAIN_SEGMENTS, TERRAIN_SEGMENTS),
    []
  )

  return (
    <mesh
      geometry={geometry}
      rotation={[-Math.PI / 2, 0, 0]}
      receiveShadow={false}
    >
      {/* MeshLambertMaterial: diffuse-only — no specular, GPU cheap */}
      <meshLambertMaterial color="#4a7c3f" side={THREE.FrontSide} />
    </mesh>
  )
}
