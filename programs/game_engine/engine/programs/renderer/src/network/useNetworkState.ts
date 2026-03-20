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

import type { WorldSnapshot, EntityState } from '../types/world'

// ============================================================================
// Config
// ============================================================================

const DEFAULT_SERVER = 'ws://localhost:9001'
const HEADER_SIZE = 16 // bytes
const RECONNECT_DELAY = 2000 // ms
const MAX_RECONNECT_ATTEMPTS = 10

// Message type codes (from shared/schemas/README.md)
const MSG_ENTITY_POSITION_UPDATE = 0x0001
const MSG_TICK_SYNC = 0x0004
const MSG_PLAYER_JOINED = 0x0005
const MSG_PLAYER_LEFT = 0x0006
const MSG_HANDSHAKE = 0x0100
const MSG_HANDSHAKE_RESPONSE = 0x0101
const MSG_PLAYER_ACTION = 0x0200

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

// Pending actions to send (drained each frame)
const _pendingActions: ArrayBuffer[] = []

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
    console.log('[NEXUS Net] Connected')
    _connected = true
    _reconnectAttempts = 0
    _sendHandshake()
  }

  _socket.onmessage = (event: MessageEvent) => {
    if (event.data instanceof ArrayBuffer) {
      _handleBinaryMessage(event.data)
    }
  }

  _socket.onclose = () => {
    console.log('[NEXUS Net] Disconnected')
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

function _encodeHeader(type: number, payloadLength: number): ArrayBuffer {
  const header = new ArrayBuffer(HEADER_SIZE)
  const view = new DataView(header)
  view.setUint16(0, type, true)           // message_type
  view.setUint16(2, 1, true)              // message_version
  view.setUint32(4, _sequenceNumber++, true) // sequence_number
  view.setUint32(8, Date.now() & 0xFFFFFFFF, true) // timestamp_ms (lower 32 bits)
  view.setUint32(12, payloadLength, true)  // payload_length
  return header
}

function _decodeHeader(data: ArrayBuffer): { type: number; version: number; sequence: number; timestamp: number; payloadLength: number } {
  const view = new DataView(data, 0, HEADER_SIZE)
  return {
    type: view.getUint16(0, true),
    version: view.getUint16(2, true),
    sequence: view.getUint32(4, true),
    timestamp: view.getUint32(8, true),
    payloadLength: view.getUint32(12, true),
  }
}

function _sendBinary(type: number, payload: ArrayBuffer): void {
  if (!_socket || _socket.readyState !== WebSocket.OPEN) return

  const header = _encodeHeader(type, payload.byteLength)
  const message = new Uint8Array(HEADER_SIZE + payload.byteLength)
  message.set(new Uint8Array(header), 0)
  message.set(new Uint8Array(payload), HEADER_SIZE)
  _socket.send(message.buffer)
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
 * Queue a movement action to be sent to the server.
 * direction: normalized Vec3 (x, y, z) — the direction the player wants to move.
 * Called by the character controller each frame the player is moving.
 */
export function sendMoveAction(dirX: number, dirY: number, dirZ: number): void {
  // Payload: 12 bytes (3x float32 LE) — matches simulation validate.rs decode_vec3f32
  const payload = new ArrayBuffer(12)
  const view = new DataView(payload)
  view.setFloat32(0, dirX, true)
  view.setFloat32(4, dirY, true)
  view.setFloat32(8, dirZ, true)
  _sendBinary(MSG_PLAYER_ACTION, payload)
}

// ============================================================================
// Message Handler
// ============================================================================

function _handleBinaryMessage(data: ArrayBuffer): void {
  if (data.byteLength < HEADER_SIZE) return

  const header = _decodeHeader(data)
  const payload = data.slice(HEADER_SIZE)

  switch (header.type) {
    case MSG_HANDSHAKE_RESPONSE:
      _handleHandshakeResponse(payload)
      break

    case MSG_ENTITY_POSITION_UPDATE:
      _handlePositionUpdate(payload)
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

    default:
      // Unknown message type — ignore (forward compatibility)
      break
  }
}

function _handleHandshakeResponse(payload: ArrayBuffer): void {
  if (payload.byteLength < 8) return
  const view = new DataView(payload)
  _playerId = view.getFloat64(0, true)
  console.log(`[NEXUS Net] HANDSHAKE accepted — player ID: ${_playerId}`)
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
  console.log(`[NEXUS Net] Player ${id} joined`)
  _entities.set(id, { id, x: 0, y: 0, z: 0, yaw: 0 })
}

function _handlePlayerLeft(payload: ArrayBuffer): void {
  if (payload.byteLength < 4) return
  const view = new DataView(payload)
  const id = view.getUint32(0, true)
  console.log(`[NEXUS Net] Player ${id} left`)
  _entities.delete(id)
}

// ============================================================================
// Public API — matches worldStateStub interface
// ============================================================================

/**
 * Step the network state. Call once per frame.
 * Flushes pending actions and updates internal state.
 * (elapsedSeconds not used for network — included for interface compatibility)
 */
export function stepWorldState(_elapsedSeconds: number): void {
  // Network state is event-driven (onmessage updates _entities).
  // No per-frame work needed here — actions are sent immediately by sendMoveAction.
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
