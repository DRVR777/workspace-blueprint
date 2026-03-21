/** Matches the entity_record shape in world-state-contract.md */
export interface EntityState {
  id: number
  x: number
  y: number
  z: number
  /** Rotation around Y axis in radians (legacy field, derived from quaternion when present) */
  yaw: number
  /** Velocity components (m/s) — present when received via PHYSICS_DELTA or FULL_SYNC */
  vx?: number
  vy?: number
  vz?: number
  /** Newton motion state — 0=inertial, 1=accelerating, 2=collision */
  motionState?: number
  /** Vehicle the entity is piloting — 0=on foot, 1=plane/fly */
  vehicleMode?: number
  /** Orientation quaternion components (decoded from i16 wire format) */
  qx?: number
  qy?: number
  qz?: number
  qw?: number
}

/** Shallow snapshot of client world state — produced once per frame before rendering */
export interface WorldSnapshot {
  /** Entity ID of the local player */
  player_entity_id: number
  /** All entities in the local player's visibility radius */
  nearby_entities: EntityState[]
  /** Server tick number at time of snapshot */
  tick: number
}
