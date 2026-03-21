/**
 * useWorldState — produces a fresh WorldSnapshot every frame.
 *
 * Connection logic:
 *   1. If VITE_NEXUS_SERVER env var is set → use that URL (dev override)
 *   2. If page is served from a remote host (not localhost) → connect to ws://same-host:9001
 *   3. Otherwise → stub mode (50 orbiting entities, no server)
 *
 * This means:
 *   - Local dev with no env var → stub mode (works offline)
 *   - Local dev with VITE_NEXUS_SERVER=ws://65.108.67.204:9001 → connects to VPS
 *   - Served from VPS (http://65.108.67.204) → auto-connects to ws://65.108.67.204:9001
 */
import { useRef, useEffect } from 'react'
import { useFrame } from '@react-three/fiber'
import type { WorldSnapshot } from '../types/world'

import {
  connect,
  disconnect,
  stepWorldState as networkStep,
  snapshotWorldState as networkSnapshot,
} from '../network/useNetworkState'

import {
  stepWorldState as stubStep,
  snapshotWorldState as stubSnapshot,
} from '../simulation/worldStateStub'

// Determine server URL
function getServerUrl(): string | null {
  // 1. Explicit env var (dev override)
  const envUrl = (typeof import.meta !== 'undefined' && (import.meta as any).env?.VITE_NEXUS_SERVER) as string | undefined
  if (envUrl) return envUrl

  // 2. Auto-detect from page URL (when served from VPS via nginx)
  // nginx proxies WSS → WS at /newworld/ws, so use that path — not the raw :9001 port.
  if (typeof window !== 'undefined') {
    const host = window.location.hostname
    if (host && host !== 'localhost' && host !== '127.0.0.1') {
      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      return `${wsProtocol}//${host}/newworld/ws`
    }
  }

  // 3. No server — stub mode
  return null
}

const SERVER_URL = getServerUrl()
const USE_NETWORK = !!SERVER_URL

export function useWorldState(): React.MutableRefObject<WorldSnapshot | null> {
  const snapshotRef = useRef<WorldSnapshot | null>(null)

  useEffect(() => {
    if (USE_NETWORK) {
      console.log(`[useWorldState] Network mode → ${SERVER_URL}`)
      connect(SERVER_URL ?? 'ws://localhost:9001')
      return () => disconnect()
    } else {
      console.log('[useWorldState] Stub mode (localhost, no VITE_NEXUS_SERVER)')
    }
  }, [])

  useFrame(({ clock }, delta) => {
    if (USE_NETWORK) {
      // Pass frame delta (not cumulative time) — stepWorldState advances positions by vel*dt
      networkStep(delta)
      snapshotRef.current = networkSnapshot()
    } else {
      // Stub uses cumulative time for orbit angle math
      stubStep(clock.elapsedTime)
      snapshotRef.current = stubSnapshot()
    }
  })

  return snapshotRef
}
