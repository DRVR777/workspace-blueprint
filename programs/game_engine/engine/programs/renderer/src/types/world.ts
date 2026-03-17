/** Matches the entity_record shape in world-state-contract.md */
export interface EntityState {
  id: number
  x: number
  y: number
  z: number
  /** Rotation around Y axis in radians */
  yaw: number
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
