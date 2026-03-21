/**
 * Local simulation — client-side physics prediction.
 *
 * Implements Newton's 1st Law for network optimization:
 * - When the server tags an entity as Inertial (ΣF≈0), its velocity is constant.
 * - The client can predict future positions locally: pos += vel * dt.
 * - Only when the server sends Accelerating or Collision state does the client
 *   need to snap/blend to server state.
 *
 * This eliminates perceived latency for straight-line motion (cruising players,
 * vehicles in transit, falling objects) at zero bandwidth cost.
 *
 * Phase 0: velocity extrapolation only.
 * Phase 1: add gravity + damping to the local prediction loop.
 * Phase 3: full input prediction + server reconciliation.
 */

export const MOTION_STATE_INERTIAL = 0
export const MOTION_STATE_ACCELERATING = 1
export const MOTION_STATE_COLLISION = 2

/** Minimum position error (meters) before triggering a hard snap on COLLISION state. */
const SNAP_THRESHOLD = 0.5

/** Blend factor for smooth error correction on ACCELERATING state (0-1). */
const BLEND_FACTOR = 0.2

export interface EntityPrediction {
  id: number
  x: number
  y: number
  z: number
  yaw: number
  vx: number
  vy: number
  vz: number
  motionState: number
  /** Timestamp (performance.now()) when this state was last set from a server update. */
  lastServerUpdateMs: number
}

/** Predicted state for all tracked entities. Keyed by entity ID. */
const _predictions = new Map<number, EntityPrediction>()

/**
 * Apply a server state update for one entity.
 *
 * On Inertial: store position + velocity for local extrapolation.
 * On Accelerating: blend predicted position toward server state, update velocity.
 * On Collision: hard snap to server state (velocity changed discontinuously).
 */
export function onServerUpdate(
  id: number,
  serverX: number,
  serverY: number,
  serverZ: number,
  serverYaw: number,
  serverVx: number,
  serverVy: number,
  serverVz: number,
  motionState: number,
): void {
  const now = performance.now()
  const existing = _predictions.get(id)

  if (!existing || motionState === MOTION_STATE_COLLISION) {
    // Hard snap: no prior state, or velocity changed discontinuously
    _predictions.set(id, {
      id,
      x: serverX, y: serverY, z: serverZ, yaw: serverYaw,
      vx: serverVx, vy: serverVy, vz: serverVz,
      motionState,
      lastServerUpdateMs: now,
    })
    return
  }

  if (motionState === MOTION_STATE_INERTIAL) {
    // Server confirms inertial motion — update velocity, keep predicted position
    // (prevents snap on minor drift; velocity is authoritative)
    existing.vx = serverVx
    existing.vy = serverVy
    existing.vz = serverVz
    existing.yaw = serverYaw
    existing.motionState = MOTION_STATE_INERTIAL
    existing.lastServerUpdateMs = now
    // Snap position only if drift exceeds threshold
    const dx = serverX - existing.x
    const dy = serverY - existing.y
    const dz = serverZ - existing.z
    const drift = Math.sqrt(dx * dx + dy * dy + dz * dz)
    if (drift > SNAP_THRESHOLD) {
      existing.x = serverX
      existing.y = serverY
      existing.z = serverZ
    }
  } else {
    // Accelerating: blend toward server state
    existing.x += (serverX - existing.x) * BLEND_FACTOR
    existing.y += (serverY - existing.y) * BLEND_FACTOR
    existing.z += (serverZ - existing.z) * BLEND_FACTOR
    existing.vx = serverVx
    existing.vy = serverVy
    existing.vz = serverVz
    existing.yaw = serverYaw
    existing.motionState = MOTION_STATE_ACCELERATING
    existing.lastServerUpdateMs = now
  }
}

/**
 * Advance all tracked entities by dt seconds using local physics.
 *
 * Inertial entities: pos += vel * dt  (Newton's 1st Law — constant velocity)
 * Non-inertial entities: no local advance (wait for server update)
 *
 * Call once per frame before reading predicted positions.
 */
export function step(dt: number): void {
  if (dt <= 0 || dt > 0.1) return // guard against bad deltas

  _predictions.forEach((p) => {
    if (p.motionState === MOTION_STATE_INERTIAL) {
      p.x += p.vx * dt
      p.y += p.vy * dt
      p.z += p.vz * dt
    }
  })
}

/** Get the current predicted state for an entity, or null if not tracked. */
export function getPrediction(id: number): EntityPrediction | null {
  return _predictions.get(id) ?? null
}

/** Get all tracked predictions. */
export function getAllPredictions(): EntityPrediction[] {
  return Array.from(_predictions.values())
}

/** Remove an entity from tracking (e.g. on PLAYER_LEFT). */
export function removeEntity(id: number): void {
  _predictions.delete(id)
}

/** Clear all tracked entities. */
export function clear(): void {
  _predictions.clear()
}
