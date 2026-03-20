/**
 * SunLight + Sky — lighting and atmosphere.
 *
 * Directional sun with shadows, hemisphere light (sky/ground),
 * gradient sky dome, and fog for depth.
 */
import { useEffect } from 'react'
import { useThree } from '@react-three/fiber'
import * as THREE from 'three'

export function SunLight() {
  const { scene } = useThree()

  // Set scene background to sky gradient and add fog
  useEffect(() => {
    // Sky gradient via a large sphere with gradient material
    scene.background = new THREE.Color(0x87CEEB) // sky blue fallback
    scene.fog = new THREE.FogExp2(0xc8d8e8, 0.002) // soft atmospheric fog

    return () => {
      scene.background = null
      scene.fog = null
    }
  }, [scene])

  return (
    <>
      {/* Hemisphere light: sky blue from above, ground brown from below */}
      <hemisphereLight
        color={0x87CEEB}     // sky
        groundColor={0x4a3728} // earth
        intensity={0.6}
      />

      {/* Sun — warm directional light with shadows */}
      <directionalLight
        position={[100, 200, 80]}
        intensity={1.5}
        color={0xfff4e0}
        castShadow
        shadow-mapSize-width={2048}
        shadow-mapSize-height={2048}
        shadow-camera-left={-60}
        shadow-camera-right={60}
        shadow-camera-top={60}
        shadow-camera-bottom={-60}
        shadow-camera-near={0.5}
        shadow-camera-far={500}
      />

      {/* Fill light — subtle blue from opposite side (simulates sky bounce) */}
      <directionalLight
        position={[-50, 80, -100]}
        intensity={0.3}
        color={0xb0c8e8}
      />

      {/* Sky dome — large inverted sphere with gradient */}
      <SkyDome />
    </>
  )
}

/** Procedural sky dome — gradient from horizon to zenith. */
function SkyDome() {
  return (
    <mesh scale={[500, 500, 500]}>
      <sphereGeometry args={[1, 32, 16]} />
      <meshBasicMaterial
        color={0x87CEEB}
        side={THREE.BackSide}
        fog={false}
      />
    </mesh>
  )
}
