/**
 * Vehicle System — modular vehicle controller interface.
 *
 * Any vehicle (plane, submarine, car, drone, boat) implements VehicleController.
 * The system manages which vehicle is active and routes input/updates to it.
 * PlayerController checks isAnyVehicleActive() and yields when true.
 *
 * Adding a new vehicle:
 *   1. Create a file (e.g., SubmarineController.ts) that implements VehicleController
 *   2. Call registerVehicle('submarine', new SubmarineController())
 *   3. Add activation trigger (key, proximity, interaction)
 *
 * Each vehicle owns:
 *   - Its own physics model (plane has lift/drag, submarine has buoyancy, car has friction)
 *   - Its own camera behavior (chase cam distance, angle, damping)
 *   - Its own input mapping (same keys can mean different things)
 *   - Its own mesh/model
 *
 * The system provides:
 *   - Vehicle switching (enter/exit)
 *   - Active vehicle tracking
 *   - Position handoff (player position → vehicle position → player position)
 */

import * as THREE from 'three'

// ============================================================================
// Vehicle Interface
// ============================================================================

export interface VehicleController {
  /** Unique vehicle type name (e.g., 'plane', 'submarine', 'car'). */
  readonly type: string

  /** Is this vehicle currently active (player is controlling it)? */
  isActive(): boolean

  /** Enter the vehicle at a given position + orientation. */
  enter(position: THREE.Vector3, orientation?: THREE.Quaternion): void

  /** Exit the vehicle. Returns the player's exit position. */
  exit(): THREE.Vector3

  /**
   * Update the vehicle physics + camera. Called every frame when active.
   * @param camera — the Three.js camera to control
   * @param scene — for adding/removing meshes
   * @param delta — frame delta time in seconds
   */
  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, delta: number): void

  /**
   * Get the vehicle's current position (for network sync).
   * The server needs to know where the vehicle is.
   */
  getPosition(): THREE.Vector3

  /**
   * Get the vehicle's forward velocity vector (for network sync).
   */
  getVelocity(): THREE.Vector3

  /**
   * Clean up meshes, lights, etc. when vehicle is destroyed.
   */
  dispose(scene: THREE.Scene): void
}

// ============================================================================
// Vehicle Registry
// ============================================================================

const _vehicles: Map<string, VehicleController> = new Map()
let _activeVehicle: VehicleController | null = null

/** Register a vehicle type. */
export function registerVehicle(controller: VehicleController): void {
  _vehicles.set(controller.type, controller)
}

/** Unregister a vehicle type. */
export function unregisterVehicle(type: string, scene: THREE.Scene): void {
  const v = _vehicles.get(type)
  if (v) {
    if (v.isActive()) v.exit()
    v.dispose(scene)
    _vehicles.delete(type)
  }
}

/** Enter a vehicle by type. Returns false if vehicle type not found. */
export function enterVehicle(type: string, position: THREE.Vector3, orientation?: THREE.Quaternion): boolean {
  const v = _vehicles.get(type)
  if (!v) {
    console.warn(`[Vehicle] Unknown type: ${type}`)
    return false
  }

  // Exit current vehicle if any
  if (_activeVehicle && _activeVehicle.isActive()) {
    _activeVehicle.exit()
  }

  v.enter(position, orientation)
  _activeVehicle = v
  console.log(`[Vehicle] Entered ${type}`)
  return true
}

/** Exit the current vehicle. Returns exit position, or null if no vehicle active. */
export function exitVehicle(): THREE.Vector3 | null {
  if (!_activeVehicle) return null
  const pos = _activeVehicle.exit()
  console.log(`[Vehicle] Exited ${_activeVehicle.type}`)
  _activeVehicle = null
  return pos
}

/** Is any vehicle currently active? PlayerController checks this. */
export function isAnyVehicleActive(): boolean {
  return _activeVehicle !== null && _activeVehicle.isActive()
}

/** Get the active vehicle (if any). */
export function getActiveVehicle(): VehicleController | null {
  return _activeVehicle
}

/** Update the active vehicle. Called from useFrame. */
export function updateActiveVehicle(camera: THREE.PerspectiveCamera, scene: THREE.Scene, delta: number): void {
  if (_activeVehicle && _activeVehicle.isActive()) {
    _activeVehicle.update(camera, scene, delta)
  }
}

/** Get all registered vehicle types. */
export function getVehicleTypes(): string[] {
  return Array.from(_vehicles.keys())
}
