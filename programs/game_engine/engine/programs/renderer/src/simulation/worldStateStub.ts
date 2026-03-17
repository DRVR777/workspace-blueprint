/**
 * Phase 0 world state stub.
 *
 * Generates 50 entities orbiting at varying radii and speeds to exercise
 * the instanced renderer without a live server connection.
 * Drop-in replacement: swap this module for the real WebSocket client
 * when node-manager integration begins.
 */
import type { WorldSnapshot, EntityState } from '../types/world'

const ENTITY_COUNT = 50
const PLAYER_ID = 0

interface OrbitParams {
  radius: number
  speed: number   // radians per second
  phase: number   // starting angle
  height: number  // Y offset
}

// Pre-compute orbit params once — deterministic, no per-frame allocation
const ORBITS: OrbitParams[] = Array.from({ length: ENTITY_COUNT }, (_, i) => ({
  radius: 5 + (i % 10) * 8,
  speed: 0.3 + (i % 5) * 0.15,
  phase: (i / ENTITY_COUNT) * Math.PI * 2,
  height: 0,
}))

// Reuse a single snapshot object — shallow copy via spread in snapshot()
const _entities: EntityState[] = Array.from({ length: ENTITY_COUNT }, (_, i) => ({
  id: i,
  x: 0,
  y: 0,
  z: 0,
  yaw: 0,
}))

let _tick = 0

/**
 * Update all entity positions for the given elapsed time.
 * Call once per frame before snapshot().
 */
export function stepWorldState(elapsedSeconds: number): void {
  _tick++
  for (let i = 0; i < ENTITY_COUNT; i++) {
    const o = ORBITS[i]
    const angle = o.phase + elapsedSeconds * o.speed
    _entities[i].x = Math.cos(angle) * o.radius
    _entities[i].y = o.height
    _entities[i].z = Math.sin(angle) * o.radius
    _entities[i].yaw = angle + Math.PI
  }
}

/**
 * Snapshot current world state. Returns a shallow copy — caller owns this object.
 */
export function snapshotWorldState(): WorldSnapshot {
  return {
    player_entity_id: PLAYER_ID,
    nearby_entities: _entities.map(e => ({ ...e })),
    tick: _tick,
  }
}
