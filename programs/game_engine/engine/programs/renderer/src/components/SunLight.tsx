/**
 * SunLight + Sky — lighting and atmosphere.
 *
 * Performance notes:
 *   - Shadow map 1024x1024 (not 2048 — saves 4x shadow fill)
 *   - Shadow camera tightened to island size (60 units, not 200)
 *   - Fog is linear (cheaper than exponential)
 *   - Sky dome is a single BackSide sphere (1 draw call)
 */
import { useEffect } from 'react'
import { useThree } from '@react-three/fiber'
import * as THREE from 'three'

export function SunLight() {
  const { scene } = useThree()

  useEffect(() => {
    scene.background = new THREE.Color(0x87CEEB)
    scene.fog = new THREE.Fog(0xc8d8e8, 80, 400) // linear fog (cheaper than FogExp2)
    return () => { scene.background = null; scene.fog = null }
  }, [scene])

  return (
    <>
      {/* Hemisphere light: sky + ground bounce */}
      <hemisphereLight color={0x87CEEB} groundColor={0x4a3728} intensity={0.6} />

      {/* Sun — shadows at 1024 resolution, tight camera */}
      <directionalLight
        position={[100, 200, 80]}
        intensity={1.5}
        color={0xfff4e0}
        castShadow
        shadow-mapSize-width={1024}
        shadow-mapSize-height={1024}
        shadow-camera-left={-60}
        shadow-camera-right={60}
        shadow-camera-top={60}
        shadow-camera-bottom={-60}
        shadow-camera-near={1}
        shadow-camera-far={400}
        shadow-bias={-0.001}
      />

      {/* Fill light — no shadows */}
      <directionalLight position={[-50, 80, -100]} intensity={0.3} color={0xb0c8e8} />

      {/* Sky dome */}
      <mesh scale={[400, 400, 400]}>
        <sphereGeometry args={[1, 16, 8]} />
        <meshBasicMaterial color={0x87CEEB} side={THREE.BackSide} fog={false} />
      </mesh>
    </>
  )
}
