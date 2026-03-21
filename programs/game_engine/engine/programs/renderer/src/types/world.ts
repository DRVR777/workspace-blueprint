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

/**
 * World surface descriptor — received from server in response to sendEnter().
 * Carried by MSG_SPATIAL_MANIFEST (0x0301) with schema_id = SCHEMA_SPATIAL_MANIFEST (2).
 * This is what a world is: its address, its 3D asset, what you can do there, who governs it.
 */
export interface SpatialManifest {
  /** Canonical address of the world — dworld:// URI */
  worldId: string
  /** URL or IPFS hash of primary 3D geometry, if any */
  geometry: string | null
  /** Named actions available at this world: "move", "build", "talk", etc. */
  surface: string[]
  /** HTTPS endpoint of the governing AI agent, if any */
  agent: string | null
  /** Payment address for access or actions that have a cost, if any */
  payment: string | null
}

/**
 * An intent packet emitted by an AI agent — received via MSG_AGENT_BROADCAST (0x0401).
 * Carried with schema_id = SCHEMA_AGENT_TASK (3).
 * The agent has read the SpatialManifest and decided what to do.
 */
export interface AgentTask {
  taskId:    number
  originId:  number          // entity ID of the agent (0 = anonymous)
  intent:    string          // natural language: what and why
  action:    string          // one item from the world's surface vocabulary
  context:   number[]        // object IDs the agent is acting on
  deadlineMs: number | null  // null = no deadline
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
