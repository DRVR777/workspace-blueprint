/**
 * SunLight — simple directional sun + ambient.
 * Phase 0 spec: directional lighting only, no shadows.
 */
export function SunLight() {
  return (
    <>
      {/* Ambient: fills shadows so the terrain is never pure black */}
      <ambientLight intensity={0.35} color="#b0c8e8" />
      {/* Sun: angled from upper-right, matches typical outdoor scenes */}
      <directionalLight
        position={[200, 400, 150]}
        intensity={1.8}
        color="#fff8e7"
      />
    </>
  )
}
