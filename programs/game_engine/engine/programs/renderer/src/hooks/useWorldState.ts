/**
 * useWorldState — produces a fresh WorldSnapshot every frame.
 *
 * Calls stepWorldState + snapshotWorldState from the simulation stub.
 * In Phase 1, replace the stub import with the real network client.
 * The hook interface stays the same.
 */
import { useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import { stepWorldState, snapshotWorldState } from '../simulation/worldStateStub'
import type { WorldSnapshot } from '../types/world'

export function useWorldState(): React.MutableRefObject<WorldSnapshot | null> {
  const snapshotRef = useRef<WorldSnapshot | null>(null)

  useFrame(({ clock }) => {
    stepWorldState(clock.elapsedTime)
    snapshotRef.current = snapshotWorldState()
  })

  return snapshotRef
}
