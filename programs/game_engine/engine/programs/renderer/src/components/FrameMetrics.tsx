/**
 * FrameMetrics — FPS counter and frame time logger.
 *
 * Spec: "Measure frame time. Log to output/ when consistently above 16.67ms."
 *
 * - Displays FPS in top-left corner via HTML overlay
 * - Accumulates frame times; logs a warning to console when rolling
 *   average exceeds 16.67ms (below 60 FPS) for 60 consecutive frames
 *
 * Uses Drei's Stats component for the on-screen panel.
 * The console logger runs inside useFrame so it has access to Three's clock.
 */
import { useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import { Stats } from '@react-three/drei'

const FRAME_BUDGET_MS = 1000 / 60   // 16.67ms
const LOG_WINDOW = 60               // frames

export function FrameMetrics() {
  const frameTimes = useRef<number[]>([])
  const lastLogTime = useRef(0)

  useFrame(({ clock }) => {
    const now = clock.getElapsedTime() * 1000
    const dt = now - lastLogTime.current
    lastLogTime.current = now

    const buf = frameTimes.current
    buf.push(dt)
    if (buf.length > LOG_WINDOW) buf.shift()

    if (buf.length === LOG_WINDOW) {
      const avg = buf.reduce((s, v) => s + v, 0) / LOG_WINDOW
      if (avg > FRAME_BUDGET_MS) {
        // eslint-disable-next-line no-console
        console.warn(
          `[NEXUS renderer] frame budget exceeded: avg=${avg.toFixed(2)}ms ` +
          `(${(1000 / avg).toFixed(1)} FPS) over ${LOG_WINDOW} frames`
        )
      }
    }
  })

  // Stats panel: shows FPS / MS / MB in top-left
  return <Stats showPanel={0} />
}
