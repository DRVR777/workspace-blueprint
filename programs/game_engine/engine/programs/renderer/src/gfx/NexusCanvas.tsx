/**
 * NexusCanvas — GfxContext implementation.
 *
 * This file is the ONLY place that knows about R3F's Canvas, renderer settings,
 * shadow maps, tone mapping, and color space. All components above this boundary
 * are platform-agnostic R3F JSX.
 */
import { Canvas } from '@react-three/fiber'
import * as THREE from 'three'
import type { NexusCanvasProps } from './types'

export function NexusCanvas({ children, dpr }: NexusCanvasProps) {
  return (
    <Canvas
      frameloop="always"
      dpr={dpr ?? [1, 2]}
      gl={{
        antialias: true,
        powerPreference: 'high-performance',
        // Tone mapping matches diffuse-only Phase 0 lighting
        toneMapping: THREE.ACESFilmicToneMapping,
        toneMappingExposure: 1.0,
        outputColorSpace: THREE.SRGBColorSpace,
      }}
      shadows
    >
      {children}
    </Canvas>
  )
}
