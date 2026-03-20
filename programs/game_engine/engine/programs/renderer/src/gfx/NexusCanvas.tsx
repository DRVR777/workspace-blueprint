/**
 * NexusCanvas — GfxContext implementation.
 *
 * This file is the ONLY place that knows about R3F's Canvas, renderer settings,
 * shadow maps, tone mapping, and color space. All components above this boundary
 * are platform-agnostic R3F JSX.
 *
 * Performance optimizations:
 *   - dpr capped at 1.5 (matches personalWebsite — saves 50%+ fill rate vs 2.0)
 *   - Shadows on but PCFSoft (cheaper than default PCF)
 *   - powerPreference: high-performance
 *   - frameloop: always (no demand-based — we want consistent FPS)
 */
import { Canvas } from '@react-three/fiber'
import * as THREE from 'three'
import type { NexusCanvasProps } from './types'

export function NexusCanvas({ children, dpr }: NexusCanvasProps) {
  return (
    <Canvas
      frameloop="always"
      dpr={dpr ?? [1, 1.5]}
      gl={{
        antialias: true,
        powerPreference: 'high-performance',
        toneMapping: THREE.ACESFilmicToneMapping,
        toneMappingExposure: 1.0,
        outputColorSpace: THREE.SRGBColorSpace,
      }}
      shadows={{ type: THREE.PCFSoftShadowMap }}
      performance={{ min: 0.5 }}
    >
      {children}
    </Canvas>
  )
}
