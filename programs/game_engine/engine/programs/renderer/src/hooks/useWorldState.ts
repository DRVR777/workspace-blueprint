/**
 * useWorldState — produces a fresh WorldSnapshot every frame.
 *
 * Selects between stub (offline) and network (connected) mode:
 *   - If NEXUS_SERVER env var is set → network mode (real WebSocket)
 *   - Otherwise → stub mode (50 orbiting entities, no server needed)
 *
 * The hook interface is the same either way.
 */
import { useRef, useEffect } from 'react'
import { useFrame } from '@react-three/fiber'
import type { WorldSnapshot } from '../types/world'

// Network imports
import {
  connect,
  disconnect,
  stepWorldState as networkStep,
  snapshotWorldState as networkSnapshot,
  isConnected,
} from '../network/useNetworkState'

// Stub imports (fallback when no server)
import {
  stepWorldState as stubStep,
  snapshotWorldState as stubSnapshot,
} from '../simulation/worldStateStub'

// Detect server URL from env or global
const SERVER_URL: string | undefined =
  (typeof import.meta !== 'undefined' && (import.meta as any).env?.VITE_NEXUS_SERVER) ||
  undefined

const USE_NETWORK = !!SERVER_URL

export function useWorldState(): React.MutableRefObject<WorldSnapshot | null> {
  const snapshotRef = useRef<WorldSnapshot | null>(null)

  // Connect to server on mount (if network mode)
  useEffect(() => {
    if (USE_NETWORK) {
      console.log(`[useWorldState] Network mode → ${SERVER_URL}`)
      connect(SERVER_URL)
      return () => disconnect()
    } else {
      console.log('[useWorldState] Stub mode (no VITE_NEXUS_SERVER set)')
    }
  }, [])

  useFrame(({ clock }) => {
    if (USE_NETWORK) {
      networkStep(clock.elapsedTime)
      snapshotRef.current = networkSnapshot()
    } else {
      stubStep(clock.elapsedTime)
      snapshotRef.current = stubSnapshot()
    }
  })

  return snapshotRef
}
