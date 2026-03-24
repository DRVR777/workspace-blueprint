/**
 * useNetworkState — WebSocket client that connects to the NEXUS Rust server.
 *
 * Drop-in replacement for worldStateStub.ts.
 * Connects to ws://SERVER:PORT, handles HANDSHAKE, receives ENTITY_POSITION_UPDATE,
 * sends PLAYER_ACTION for local movement.
 *
 * Wire protocol (from shared/schemas/README.md):
 *   [2 bytes] message_type
 *   [2 bytes] message_version
 *   [4 bytes] sequence_number
 *   [4 bytes] timestamp_ms
 *   [4 bytes] payload_length
 *   [4 bytes] schema_id      ← self-describing field; 0 = untyped/legacy
 *   [N bytes] payload
 *
 * Message types:
 *   0x0100 HANDSHAKE (C→S)
 *   0x0101 HANDSHAKE_RESPONSE (S→C)
 *   0x0001 ENTITY_POSITION_UPDATE (S→C)
 *   0x0200 PLAYER_ACTION (C→S)
 *   0x0004 TICK_SYNC (S→C)
 *   0x0005 PLAYER_JOINED (S→C)
 *   0x0006 PLAYER_LEFT (S→C)
 */

import type { WorldSnapshot, EntityState, SpatialManifest, AgentTask } from '../types/world'
import {
  MSG_SCHEMA_QUERY,
  MSG_SCHEMA_RESPONSE,
  MSG_SCHEMA_NOT_FOUND,
  decodeSchemaResponse,
} from './schema'

// ============================================================================
// Config
// ============================================================================

const DEFAULT_SERVER = 'ws://localhost:9001'
const HEADER_SIZE = 20 // bytes (matches PacketHeader in nexus-core/src/types.rs)
const RECONNECT_DELAY = 2000 // ms
const MAX_RECONNECT_ATTEMPTS = 10

// Message type codes (from shared/schemas/README.md)
const MSG_ENTITY_POSITION_UPDATE = 0x0001 // Legacy — still handled for compat
const MSG_PHYSICS_DELTA = 0x0002          // Newton delta: non-inertial + overdue bodies
const MSG_FULL_SYNC = 0x0003              // Full state: all dynamic bodies
const MSG_TICK_SYNC = 0x0004
const MSG_PLAYER_JOINED = 0x0005
const MSG_PLAYER_LEFT = 0x0006
const MSG_HANDSHAKE = 0x0100
const MSG_HANDSHAKE_RESPONSE = 0x0101
const MSG_PLAYER_ACTION = 0x0200
const MSG_ENTER = 0x0300            // C→S: request world manifest
const MSG_SPATIAL_MANIFEST = 0x0301 // S→C: world surface descriptor
// MSG_AGENT_TASK = 0x0400 reserved for agent system (not used in Phase 0)
const MSG_AGENT_BROADCAST = 0x0401  // S→C: agent task broadcast to all clients
// MSG_SCHEMA_QUERY/RESPONSE/NOT_FOUND imported from schema.ts above

const MOTION_STATE_INERTIAL = 0
// MOTION_STATE_ACCELERATING = 1 — received but unused until Phase 1 prediction reset
const MOTION_STATE_COLLISION = 2

// ============================================================================
// Network State (module-level, not React state — avoids re-renders)
// ============================================================================

let _socket: WebSocket | null = null
let _connected = false
let _playerId = 0
let _tick = 0
let _sequenceNumber = 0
let _reconnectAttempts = 0
let _serverUrl = DEFAULT_SERVER

// Entity map: id → latest state (updated from server, read each frame)
const _entities: Map<number, EntityState> = new Map()

// ============================================================================
// Input Buffer — client-side reconciliation (Gap 1 fix)
// ============================================================================
//
// Records player inputs sent to the server. When the server sends a Collision
// or Accelerating correction for the local player's entity, we:
//   1. Snap to server state (authoritative)
//   2. Replay unacknowledged inputs from the buffer (Phase 0 approximation)
//
// This eliminates rubber-banding without requiring full local physics.
// Phase 3 will replace this with full deterministic physics replay.

interface BufferedInput {
  seq: number   // monotonically increasing sequence number sent to server
  dx: number
  dy: number
  dz: number
}

// Approximate speed used for input replay (matches renderer PlayerController).
// Replace with physics-accurate value in Phase 3.
const PLAYER_APPROX_SPEED = 5 // m/s

const INPUT_BUFFER_MAX = 50 // ~1 second at 50 Hz send rate
const _inputBuffer: BufferedInput[] = []
let _inputSeq = 0 // monotonically increasing counter — server acks via ack_seq in PHYSICS_DELTA

// ============================================================================
// Connection
// ============================================================================

export function connect(serverUrl?: string): void {
  _serverUrl = serverUrl ?? DEFAULT_SERVER
  _reconnectAttempts = 0
  _connectInternal()
}

function _connectInternal(): void {
  if (_socket && (_socket.readyState === WebSocket.CONNECTING || _socket.readyState === WebSocket.OPEN)) {
    return
  }

  console.log(`[NEXUS Net] Connecting to ${_serverUrl}...`)
  _socket = new WebSocket(_serverUrl)
  _socket.binaryType = 'arraybuffer'

  _socket.onopen = () => {
    console.log(`[NEXUS Net] Connected to ${_serverUrl}`)
    console.log(`[NEXUS Net] WebSocket readyState: ${_socket?.readyState}`)
    _connected = true
    _reconnectAttempts = 0
    _totalBytesReceived = 0
    _totalBytesSent = 0
    _sendHandshake()
  }

  _socket.onmessage = (event: MessageEvent) => {
    if (event.data instanceof ArrayBuffer) {
      _handleBinaryMessage(event.data)
    }
  }

  _socket.onclose = (event) => {
    console.log(`[NEXUS Net] Disconnected — code: ${event.code}, reason: ${event.reason || 'none'}, clean: ${event.wasClean}`)
    console.log(`[NEXUS Net] Session stats: ${(_totalBytesReceived / 1024).toFixed(1)}KB received, ${(_totalBytesSent / 1024).toFixed(1)}KB sent, ${_entities.size} entities tracked`)
    _connected = false
    _socket = null
    _attemptReconnect()
  }

  _socket.onerror = (err) => {
    console.warn('[NEXUS Net] Error:', err)
  }
}

function _attemptReconnect(): void {
  if (_reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
    console.error(`[NEXUS Net] Failed to reconnect after ${MAX_RECONNECT_ATTEMPTS} attempts`)
    return
  }
  _reconnectAttempts++
  console.log(`[NEXUS Net] Reconnecting (attempt ${_reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})...`)
  setTimeout(_connectInternal, RECONNECT_DELAY)
}

export function disconnect(): void {
  if (_socket) {
    _socket.close()
    _socket = null
  }
  _connected = false
  _entities.clear()
}

export function isConnected(): boolean {
  return _connected
}

// ============================================================================
// Wire Protocol — Encode/Decode
// ============================================================================

function _encodeHeader(type: number, payloadLength: number, schemaId = 0): ArrayBuffer {
  const header = new ArrayBuffer(HEADER_SIZE)
  const view = new DataView(header)
  view.setUint16(0, type, true)                       // [0..2]  message_type
  view.setUint16(2, 1, true)                          // [2..4]  message_version
  view.setUint32(4, _sequenceNumber++, true)          // [4..8]  sequence_number
  view.setUint32(8, Date.now() & 0xFFFFFFFF, true)    // [8..12] timestamp_ms (lower 32 bits)
  view.setUint32(12, payloadLength, true)              // [12..16] payload_length
  view.setUint32(16, schemaId, true)                  // [16..20] schema_id
  return header
}

function _decodeHeader(data: ArrayBuffer): { type: number; version: number; sequence: number; timestamp: number; payloadLength: number; schemaId: number } {
  const view = new DataView(data, 0, HEADER_SIZE)
  return {
    type:          view.getUint16(0, true),
    version:       view.getUint16(2, true),
    sequence:      view.getUint32(4, true),
    timestamp:     view.getUint32(8, true),
    payloadLength: view.getUint32(12, true),
    schemaId:      view.getUint32(16, true),
  }
}

function _sendBinary(type: number, payload: ArrayBuffer): void {
  if (!_socket || _socket.readyState !== WebSocket.OPEN) return

  const header = _encodeHeader(type, payload.byteLength)
  const message = new Uint8Array(HEADER_SIZE + payload.byteLength)
  message.set(new Uint8Array(header), 0)
  message.set(new Uint8Array(payload), HEADER_SIZE)
  _socket.send(message.buffer)
  _totalBytesSent += message.byteLength
}

// ============================================================================
// HANDSHAKE
// ============================================================================

function _sendHandshake(): void {
  // Phase 0: minimal handshake — just send player ID as payload
  const payload = new ArrayBuffer(8)
  const view = new DataView(payload)
  view.setFloat64(0, _playerId, true)
  _sendBinary(MSG_HANDSHAKE, payload)
  console.log('[NEXUS Net] Sent HANDSHAKE')
}

// ============================================================================
// PLAYER_ACTION — movement input
// ============================================================================

/**
 * Send a movement action to the server.
 * Throttled to server tick rate (50Hz / 20ms) — prevents movement speed
 * from depending on client frame rate.
 */
const SEND_INTERVAL_MS = 20 // match server tick rate
let _lastSendTime = 0
let _pendingDir: [number, number, number] | null = null

export function sendMoveAction(
  dirX: number, dirY: number, dirZ: number,
  vehicleMode?: number,
  qx?: number, qy?: number, qz?: number, qw?: number,
  posX?: number, posY?: number, posZ?: number,
): void {
  _pendingDir = [dirX, dirY, dirZ]

  const now = performance.now()
  if (now - _lastSendTime < SEND_INTERVAL_MS) return
  _lastSendTime = now

  const seq = ++_inputSeq

  _inputBuffer.push({ seq, dx: _pendingDir[0], dy: _pendingDir[1], dz: _pendingDir[2] })
  if (_inputBuffer.length > INPUT_BUFFER_MAX) _inputBuffer.shift()

  const hasVehicle = vehicleMode !== undefined && vehicleMode !== 0
  if (hasVehicle) {
    // Extended PLAYER_ACTION payload (48 bytes):
    //   [4] dx, [4] dy, [4] dz, [4] seq
    //   [1] vehicle_mode, [3] padding
    //   [4] qx, [4] qy, [4] qz, [4] qw
    //   [4] pos_x, [4] pos_y, [4] pos_z  ← client-authoritative position
    const payload = new ArrayBuffer(48)
    const view = new DataView(payload)
    view.setFloat32(0, _pendingDir[0], true)
    view.setFloat32(4, _pendingDir[1], true)
    view.setFloat32(8, _pendingDir[2], true)
    view.setUint32(12, seq, true)
    view.setUint8(16, vehicleMode ?? 0)
    // bytes 17-19: padding (zero)
    view.setFloat32(20, qx ?? 0, true)
    view.setFloat32(24, qy ?? 0, true)
    view.setFloat32(28, qz ?? 0, true)
    view.setFloat32(32, qw ?? 1, true)
    view.setFloat32(36, posX ?? 0, true)
    view.setFloat32(40, posY ?? 0, true)
    view.setFloat32(44, posZ ?? 0, true)
    _sendBinary(MSG_PLAYER_ACTION, payload)
  } else {
    // Base PLAYER_ACTION payload (16 bytes): dx, dy, dz, seq
    const payload = new ArrayBuffer(16)
    const view = new DataView(payload)
    view.setFloat32(0, _pendingDir[0], true)
    view.setFloat32(4, _pendingDir[1], true)
    view.setFloat32(8, _pendingDir[2], true)
    view.setUint32(12, seq, true)
    _sendBinary(MSG_PLAYER_ACTION, payload)
  }
  _pendingDir = null
}

// ============================================================================
// Schema discovery — send MSG_SCHEMA_QUERY for any unknown schema_id
// ============================================================================

const _queriedSchemas = new Set<number>() // avoid re-querying the same ID

function _sendSchemaQuery(schemaId: number): void {
  if (_queriedSchemas.has(schemaId)) return
  _queriedSchemas.add(schemaId)
  const payload = new ArrayBuffer(4)
  new DataView(payload).setUint32(0, schemaId, true)
  _sendBinary(MSG_SCHEMA_QUERY, payload)
}

// ============================================================================
// ENTER — request the spatial manifest for a world
// ============================================================================

/**
 * Send an ENTER request to the server. Server responds with MSG_SPATIAL_MANIFEST.
 * @param worldId - dworld:// URI of the world to enter. Empty string = default world.
 */
export function sendEnter(worldId = ''): void {
  const encoded = new TextEncoder().encode(worldId)
  const payload = new ArrayBuffer(2 + encoded.byteLength)
  const view = new DataView(payload)
  view.setUint16(0, encoded.byteLength, true) // [2] uri_len
  new Uint8Array(payload, 2).set(encoded)      // [N] utf-8 bytes
  _sendBinary(MSG_ENTER, payload)
}

// ============================================================================
// SpatialManifest — callback registry
// ============================================================================

type ManifestCallback = (manifest: SpatialManifest) => void
const _manifestCallbacks: ManifestCallback[] = []

/** Register a callback invoked when the server sends a SpatialManifest. */
export function onSpatialManifest(cb: ManifestCallback): () => void {
  _manifestCallbacks.push(cb)
  return () => {
    const i = _manifestCallbacks.indexOf(cb)
    if (i !== -1) _manifestCallbacks.splice(i, 1)
  }
}

// ============================================================================
// AgentTask — callback registry + decoder
// ============================================================================

type AgentTaskCallback = (task: AgentTask) => void
const _agentTaskCallbacks: AgentTaskCallback[] = []

/** Register a callback invoked when the server broadcasts an AgentTask. */
export function onAgentTask(cb: AgentTaskCallback): () => void {
  _agentTaskCallbacks.push(cb)
  return () => {
    const i = _agentTaskCallbacks.indexOf(cb)
    if (i !== -1) _agentTaskCallbacks.splice(i, 1)
  }
}

function _decodeAgentTask(payload: ArrayBuffer): AgentTask | null {
  const bytes = new Uint8Array(payload)
  const view  = new DataView(payload)
  if (bytes.length < 16) return null

  // task_id and origin_id are u64 — JS can't represent full u64 but the IDs
  // we use in practice are small, so reading as two u32s and combining is fine.
  const taskId   = view.getUint32(0, true) + view.getUint32(4, true) * 0x100000000
  const originId = view.getUint32(8, true) + view.getUint32(12, true) * 0x100000000
  let pos = 16

  function readStr(): string | null {
    if (pos + 2 > bytes.length) return null
    const len = bytes[pos] | (bytes[pos + 1] << 8)
    pos += 2
    if (pos + len > bytes.length) return null
    const s = new TextDecoder().decode(bytes.slice(pos, pos + len))
    pos += len
    return s
  }

  const intent = readStr()
  const action = readStr()
  if (intent === null || action === null) return null

  const contextCount = pos < bytes.length ? bytes[pos++] : 0
  const context: number[] = []
  for (let i = 0; i < contextCount; i++) {
    if (pos + 8 > bytes.length) break
    context.push(view.getUint32(pos, true))  // lower 32 bits sufficient for entity IDs
    pos += 8
  }
  const deadlineMs = pos + 4 <= bytes.length ? view.getUint32(pos, true) : 0

  return { taskId, originId, intent, action, context, deadlineMs: deadlineMs || null }
}

function _decodeSpatialManifest(payload: ArrayBuffer): SpatialManifest | null {
  const bytes = new Uint8Array(payload)
  let pos = 0

  function readStr(): string | null {
    if (pos + 2 > bytes.length) return null
    const len = bytes[pos] | (bytes[pos + 1] << 8)
    pos += 2
    if (pos + len > bytes.length) return null
    const s = new TextDecoder().decode(bytes.slice(pos, pos + len))
    pos += len
    return s
  }

  function readOptStr(): string | null {
    const s = readStr()
    return s === '' ? null : s
  }

  const worldId = readStr()
  if (worldId === null) return null
  const geometry = readOptStr()
  if (pos >= bytes.length) return null
  const surfaceCount = bytes[pos++]
  const surface: string[] = []
  for (let i = 0; i < surfaceCount; i++) {
    const s = readStr()
    if (s === null) return null
    surface.push(s)
  }
  const agent = readOptStr()
  const payment = readOptStr()
  return { worldId, geometry, surface, agent, payment }
}

// ============================================================================
// Message Handler
// ============================================================================

// Rate-limited logging (don't spam console with position updates)
let _lastPositionLogTime = 0
let _positionUpdateCount = 0
let _totalBytesReceived = 0
let _totalBytesSent = 0

function _handleBinaryMessage(data: ArrayBuffer): void {
  if (data.byteLength < HEADER_SIZE) return

  const header = _decodeHeader(data)
  const payload = data.slice(HEADER_SIZE)
  _totalBytesReceived += data.byteLength

  switch (header.type) {
    case MSG_HANDSHAKE_RESPONSE:
      _handleHandshakeResponse(payload)
      break

    case MSG_ENTITY_POSITION_UPDATE:
      _handlePositionUpdate(payload)
      _positionUpdateCount++
      // Log position updates every 5 seconds (not every frame)
      if (performance.now() - _lastPositionLogTime > 5000) {
        const entityCount = Math.floor(payload.byteLength / 24)
        console.log(
          `[NEXUS Net] Position updates: ${_positionUpdateCount} in last 5s, ` +
          `${entityCount} entities, ` +
          `${(_totalBytesReceived / 1024).toFixed(1)}KB received total, ` +
          `${(_totalBytesSent / 1024).toFixed(1)}KB sent total`
        )
        _positionUpdateCount = 0
        _lastPositionLogTime = performance.now()
      }
      break

    case MSG_PHYSICS_DELTA:
    case MSG_FULL_SYNC:
      _handlePhysicsDelta(payload)
      _positionUpdateCount++
      if (performance.now() - _lastPositionLogTime > 5000) {
        const isFull = header.type === MSG_FULL_SYNC
        const entityCount = Math.floor((payload.byteLength - 4) / 26) // -4 for ack_seq header
        console.log(
          `[NEXUS Net] ${isFull ? 'FULL_SYNC' : 'PHYSICS_DELTA'}: ${_positionUpdateCount} in last 5s, ` +
          `${entityCount} entities in batch, ` +
          `${(_totalBytesReceived / 1024).toFixed(1)}KB received total`
        )
        _positionUpdateCount = 0
        _lastPositionLogTime = performance.now()
      }
      break

    case MSG_TICK_SYNC:
      _handleTickSync(payload)
      break

    case MSG_PLAYER_JOINED:
      _handlePlayerJoined(payload)
      break

    case MSG_PLAYER_LEFT:
      _handlePlayerLeft(payload)
      break

    case MSG_AGENT_BROADCAST: {
      const task = _decodeAgentTask(payload)
      if (task) {
        console.log(`[NEXUS Net] AgentTask #${task.taskId} origin=${task.originId} action=${task.action} — ${task.intent}`)
        _agentTaskCallbacks.forEach(cb => cb(task))
      }
      break
    }

    case MSG_SPATIAL_MANIFEST: {
      const manifest = _decodeSpatialManifest(payload)
      if (manifest) {
        console.log(`[NEXUS Net] SpatialManifest: ${manifest.worldId} surface=[${manifest.surface.join(',')}]`)
        _manifestCallbacks.forEach(cb => cb(manifest))
      }
      break
    }

    case MSG_SCHEMA_RESPONSE: {
      const result = decodeSchemaResponse(payload)
      if (result) {
        console.log(`[NEXUS Net] Schema 0x${result.id.toString(16)} resolved: ${result.descriptor.name}@${result.descriptor.version}`)
      }
      break
    }

    case MSG_SCHEMA_NOT_FOUND:
      // Server doesn't know this schema — log and move on
      if (payload.byteLength >= 4) {
        const id = new DataView(payload).getUint32(0, true)
        console.warn(`[NEXUS Net] Schema 0x${id.toString(16)} not found in server registry`)
      }
      break

    default: {
      // Unknown message type — check if schema_id is also unknown; if so, query it
      const sid = header.schemaId
      if (sid !== 0 && sid !== 1) {
        _sendSchemaQuery(sid)
      }
      console.log(`[NEXUS Net] Unknown message type: 0x${header.type.toString(16).padStart(4, '0')} schema_id=0x${sid.toString(16)} (${data.byteLength} bytes)`)
      break
    }
  }
}

/**
 * Handle PHYSICS_DELTA (0x0002) and FULL_SYNC (0x0003).
 *
 * Payload layout:
 *   [4] ack_seq (u32 LE) — last input seq the server processed for this client
 *   [N × 26] entity records:
 *     [4] entity_id (u32)
 *     [1] motion_state (u8: 0=inertial, 1=accelerating, 2=collision)
 *     [1] vehicle_mode (u8: 0=on foot, 1=plane/fly)
 *     [2] x (i16, × 1/32 = meters)
 *     [2] y (i16)
 *     [2] z (i16)
 *     [2] vx (i16, × 1/32 = m/s)
 *     [2] vy (i16)
 *     [2] vz (i16)
 *     [2] oqx (i16, × 1/32767 → -1..1)
 *     [2] oqy (i16)
 *     [2] oqz (i16)
 *     [2] oqw (i16)
 */
function _handlePhysicsDelta(payload: ArrayBuffer): void {
  if (payload.byteLength < 4) return

  const view = new DataView(payload)

  // First 4 bytes: ack_seq — server's confirmation of inputs processed for this client.
  // Use to prune the input buffer: discard any entry with seq ≤ ack_seq.
  const ackSeq = view.getUint32(0, true)

  const ENTITY_SIZE = 26
  const entityDataStart = 4 // ack_seq header offset
  const count = Math.floor((payload.byteLength - entityDataStart) / ENTITY_SIZE)

  for (let i = 0; i < count; i++) {
    const off = entityDataStart + i * ENTITY_SIZE
    const id = view.getUint32(off, true)
    const motionState = view.getUint8(off + 4)
    const vehicleMode = view.getUint8(off + 5)
    const x = view.getInt16(off + 6, true) / 32.0
    const y = view.getInt16(off + 8, true) / 32.0
    const z = view.getInt16(off + 10, true) / 32.0
    const vx = view.getInt16(off + 12, true) / 32.0
    const vy = view.getInt16(off + 14, true) / 32.0
    const vz = view.getInt16(off + 16, true) / 32.0
    const qx = view.getInt16(off + 18, true) / 32767.0
    const qy = view.getInt16(off + 20, true) / 32767.0
    const qz = view.getInt16(off + 22, true) / 32767.0
    const qw = view.getInt16(off + 24, true) / 32767.0
    // Derive yaw from quaternion for legacy callers
    const yaw = Math.atan2(2 * (qw * qy + qx * qz), 1 - 2 * (qy * qy + qz * qz))

    _entities.set(id, { id, x, y, z, yaw, vx, vy, vz, motionState, vehicleMode, qx, qy, qz, qw })

    // ---- Reconciliation: replay unacknowledged inputs after server correction ----
    //
    // Triggered only for the local player on Collision/Accelerating state.
    // Inertial: client prediction was correct — no correction needed.
    // Collision/Accelerating: server diverged from client prediction —
    //   snap to server state (done above), then re-apply inputs with seq > ack_seq
    //   (those are in-flight; server hasn't seen them yet).
    if (id === _playerId && motionState !== MOTION_STATE_INERTIAL) {
      const entity = _entities.get(id)
      if (entity) {
        // Prune acknowledged inputs (seq ≤ ack_seq from this message)
        let discardCount = 0
        while (discardCount < _inputBuffer.length && _inputBuffer[discardCount].seq <= ackSeq) {
          discardCount++
        }
        if (discardCount > 0) _inputBuffer.splice(0, discardCount)

        // Replay in-flight inputs (seq > ack_seq) — hide RTT latency
        const replayDt = SEND_INTERVAL_MS / 1000
        for (const input of _inputBuffer) {
          entity.x += input.dx * PLAYER_APPROX_SPEED * replayDt
          entity.y += input.dy * PLAYER_APPROX_SPEED * replayDt
          entity.z += input.dz * PLAYER_APPROX_SPEED * replayDt
        }

        // Signal PlayerController to snap camera ONLY on Collision.
        // Normal movement (Accelerating) stays local — server tick rate must not
        // affect feel of walking. Only a collision interrupt (external impulse,
        // wall contact, player bump) overrides local camera.
        if (motionState === MOTION_STATE_COLLISION) {
          _pendingCorrection = { x: entity.x, y: entity.y, z: entity.z }
        }
      }
    }
  }
}

function _handleHandshakeResponse(payload: ArrayBuffer): void {
  if (payload.byteLength < 8) return
  const view = new DataView(payload)
  _playerId = view.getFloat64(0, true)
  console.log(`[NEXUS Net] HANDSHAKE accepted — player ID: ${_playerId}`)
  // Request the world manifest immediately — first non-physics packet
  sendEnter()
}

function _handlePositionUpdate(payload: ArrayBuffer): void {
  // Payload per entity: 24 bytes
  //   [4] entity_id (uint32)
  //   [4] x (float32)
  //   [4] y (float32)
  //   [4] z (float32)
  //   [4] yaw (float32)
  //   [4] padding/flags
  const ENTITY_SIZE = 24
  const count = Math.floor(payload.byteLength / ENTITY_SIZE)
  const view = new DataView(payload)

  for (let i = 0; i < count; i++) {
    const offset = i * ENTITY_SIZE
    const id = view.getUint32(offset, true)
    const x = view.getFloat32(offset + 4, true)
    const y = view.getFloat32(offset + 8, true)
    const z = view.getFloat32(offset + 12, true)
    const yaw = view.getFloat32(offset + 16, true)

    _entities.set(id, { id, x, y, z, yaw })
  }
}

function _handleTickSync(payload: ArrayBuffer): void {
  if (payload.byteLength < 8) return
  const view = new DataView(payload)
  _tick = view.getUint32(0, true)
}

function _handlePlayerJoined(payload: ArrayBuffer): void {
  if (payload.byteLength < 4) return
  const view = new DataView(payload)
  const id = view.getUint32(0, true)
  console.log(`[NEXUS Net] ★ Player ${id} joined — total entities: ${_entities.size + 1}`)
  _entities.set(id, { id, x: 0, y: 0, z: 0, yaw: 0 })
  console.log(`[NEXUS Net] _entities now contains:`, Array.from(_entities.keys()))
}

function _handlePlayerLeft(payload: ArrayBuffer): void {
  if (payload.byteLength < 4) return
  const view = new DataView(payload)
  const id = view.getUint32(0, true)
  console.log(`[NEXUS Net] Player ${id} left`)
  _entities.delete(id)
}

// ============================================================================
// Server correction channel — consumed by PlayerController to snap camera
// ============================================================================
//
// When the server sends a non-inertial update for the local player, it means
// the server's position diverges from what the client predicted. PlayerController
// polls this each frame and snaps the camera to the corrected position.

interface ServerCorrection {
  x: number
  y: number
  z: number
}

let _pendingCorrection: ServerCorrection | null = null

/** Consume and clear the latest server correction for the local player (if any). */
export function consumeServerCorrection(): ServerCorrection | null {
  const c = _pendingCorrection
  _pendingCorrection = null
  return c
}

// ============================================================================
// Public API — matches worldStateStub interface
// ============================================================================

/**
 * Step the network state. Call once per frame with the frame delta time.
 *
 * For entities tagged as Inertial by the server (MotionState = 0), advances
 * position locally using Newton's 1st Law: pos += vel * dt.
 * This eliminates visual stutter between server updates at no bandwidth cost.
 *
 * Non-inertial entities (Accelerating / Collision) only update on server message.
 */
export function stepWorldState(elapsedSeconds: number): void {
  if (elapsedSeconds <= 0 || elapsedSeconds > 0.1) return // guard against bad deltas

  _entities.forEach((entity) => {
    if (entity.vx === undefined || entity.vy === undefined || entity.vz === undefined) return

    if (entity.motionState === MOTION_STATE_INERTIAL) {
      // Newton's 1st Law: constant velocity until a force acts
      entity.x += entity.vx * elapsedSeconds
      entity.y += entity.vy * elapsedSeconds
      entity.z += entity.vz * elapsedSeconds
    } else if (entity.vehicleMode !== undefined && entity.vehicleMode !== 0) {
      // Vehicle entities are always Accelerating (pilot applies continuous thrust),
      // but their velocity is still a valid dead-reckoning signal between server ticks.
      // Dead-reckon by current velocity to eliminate observer snap at tick boundaries.
      entity.x += entity.vx * elapsedSeconds
      entity.y += entity.vy * elapsedSeconds
      entity.z += entity.vz * elapsedSeconds
    }
  })
}

/**
 * Snapshot current world state from network. Returns a shallow copy.
 * Drop-in replacement for worldStateStub.snapshotWorldState().
 */
export function snapshotWorldState(): WorldSnapshot {
  return {
    player_entity_id: _playerId,
    nearby_entities: Array.from(_entities.values()),
    tick: _tick,
  }
}

// ============================================================================
// Debug API — call from browser console: nexusDebug()
// ============================================================================

/** Print full network debug state. Call from console: nexusDebug() */
export function debugNetworkState(): void {
  console.log('=== NEXUS Network Debug ===')
  console.log(`Server: ${_serverUrl}`)
  console.log(`Connected: ${_connected}`)
  console.log(`Player ID: ${_playerId}`)
  console.log(`Server tick: ${_tick}`)
  console.log(`Entities tracked: ${_entities.size}`)
  console.log(`Reconnect attempts: ${_reconnectAttempts}`)
  console.log(`Bytes received: ${(_totalBytesReceived / 1024).toFixed(1)}KB`)
  console.log(`Bytes sent: ${(_totalBytesSent / 1024).toFixed(1)}KB`)
  console.log(`WebSocket state: ${_socket ? ['CONNECTING','OPEN','CLOSING','CLOSED'][_socket.readyState] : 'null'}`)
  console.log('Entities:')
  _entities.forEach((e, id) => {
    console.log(`  ${id}: pos(${e.x.toFixed(2)}, ${e.y.toFixed(2)}, ${e.z.toFixed(2)}) yaw=${e.yaw.toFixed(2)}`)
  })
  console.log('===========================')
}

// Expose to window for console access
if (typeof window !== 'undefined') {
  (window as any).nexusDebug = debugNetworkState
}
