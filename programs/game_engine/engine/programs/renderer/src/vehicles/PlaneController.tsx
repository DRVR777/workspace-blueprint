/**
 * PlaneController — arcade flight model implementing VehicleController.
 *
 * Ported from 3dGraphUniverse/src/controls.js (FlyControls plane mode).
 * Now implements the modular VehicleController interface so any vehicle
 * (plane, submarine, car, drone) works the same way with the system.
 *
 * Physics model (from 3dGraphUniverse):
 *   - Quaternion-based orientation (no gimbal lock)
 *   - Bank-to-turn: roll creates yaw (real aircraft feel)
 *   - Speed-dependent lift vs gravity
 *   - Auto-level when stick is centered
 *   - Chase camera: behind + above plane, smooth slerp
 *
 * Controls:
 *   W/S       — throttle up/down
 *   Mouse     — pitch and roll
 *   Q/E       — air roll (barrel roll)
 *   Space     — vertical trim up
 *   Ctrl      — vertical trim down
 *   Shift     — boost (2x speed)
 *
 * Source: 3dGraphUniverse/src/controls.js lines 27-483
 */
import * as THREE from 'three'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

// ============================================================================
// Flight Physics Constants
// ============================================================================

const MAX_SPEED = 0.4
const MIN_SPEED = 0.05
const SPEED_DAMPING = 0.02
const GRAVITY = 0.004
const PITCH_SPEED = 0.0012
const ROLL_SPEED = 0.0015
const BANK_TURN_RATE = 0.015
const AIR_ROLL_SPEED = 0.08
const AUTO_LEVEL_RATE = 0.02
const STICK_DEAD_ZONE = 8
const STICK_MAX_RADIUS = 100
const STICK_DRIFT_BACK = 0.94
const CAMERA_OFFSET_BACK = 16
const CAMERA_OFFSET_UP = 5
const CAMERA_LERP_POS = 0.04
const CAMERA_LERP_LOOK = 0.08
const CAMERA_LOOK_AHEAD = 10

// ============================================================================
// Hoisted vectors (zero allocation in hot path)
// ============================================================================

const _fwd = new THREE.Vector3()
const _up = new THREE.Vector3()
const _right = new THREE.Vector3()
const _inputQ = new THREE.Quaternion()
const _yawQ = new THREE.Quaternion()
const _levelQ = new THREE.Quaternion()
const _worldUp = new THREE.Vector3(0, 1, 0)
const _cameraTarget = new THREE.Vector3()
const _lookTarget = new THREE.Vector3()
const _camUp = new THREE.Vector3()
const _lookM = new THREE.Matrix4()
const _lookQ = new THREE.Quaternion()

// ============================================================================
// Input state
// ============================================================================

let _stickX = 0
let _stickY = 0
let _mouseListenerAdded = false

function ensureMouseListener(): void {
  if (_mouseListenerAdded) return
  _mouseListenerAdded = true
  window.addEventListener('mousemove', (e) => {
    if (!document.pointerLockElement) return
    _stickX += e.movementX
    _stickY += e.movementY
  })
}

// ============================================================================
// PlaneVehicle — implements VehicleController
// ============================================================================

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active = false
  private throttle = 0.5
  private speed = 0
  private airRollTarget = 0
  private orientation = new THREE.Quaternion()
  private position = new THREE.Vector3()
  private planeMesh: THREE.Group | null = null
  private keys: Record<string, boolean> = {}

  constructor() {
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', (e) => {
        this.keys[e.key.toLowerCase()] = true
      })
      window.addEventListener('keyup', (e) => {
        this.keys[e.key.toLowerCase()] = false
      })
      ensureMouseListener()
    }
  }

  isActive(): boolean { return this.active }

  enter(position: THREE.Vector3, orientation?: THREE.Quaternion): void {
    this.position.copy(position)
    this.position.y = Math.max(this.position.y, 50)
    this.orientation.copy(orientation ?? new THREE.Quaternion())
    this.throttle = 0.5
    this.speed = MIN_SPEED
    this.active = true
    _stickX = 0
    _stickY = 0
  }

  exit(): THREE.Vector3 {
    this.active = false
    if (this.planeMesh) this.planeMesh.visible = false
    return this.position.clone()
  }

  getPosition(): THREE.Vector3 { return this.position.clone() }

  getVelocity(): THREE.Vector3 {
    _fwd.set(0, 0, -1).applyQuaternion(this.orientation)
    return _fwd.clone().multiplyScalar(this.speed)
  }

  dispose(scene: THREE.Scene): void {
    if (this.planeMesh) {
      scene.remove(this.planeMesh)
      this.planeMesh = null
    }
  }

  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, _delta: number): void {
    if (!this.active) return

    // Create mesh on first update
    if (!this.planeMesh) {
      this.planeMesh = this.createPlaneMesh()
      scene.add(this.planeMesh)
    }
    this.planeMesh.visible = true

    const keys = this.keys

    // === THROTTLE ===
    if (keys['w']) this.throttle = Math.min(1.0, this.throttle + 0.008)
    if (keys['s']) this.throttle = Math.max(0.0, this.throttle - 0.006)

    const boost = keys['shift'] ? 2.0 : 1.0
    const targetSpeed = (MIN_SPEED + this.throttle * (MAX_SPEED - MIN_SPEED)) * boost
    this.speed += (targetSpeed - this.speed) * SPEED_DAMPING

    // === MOUSE STICK ===
    const stickDist = Math.sqrt(_stickX * _stickX + _stickY * _stickY)
    if (stickDist > STICK_MAX_RADIUS) {
      const scale = STICK_MAX_RADIUS / stickDist
      _stickX *= scale
      _stickY *= scale
    }

    const effX = Math.abs(_stickX) > STICK_DEAD_ZONE ? _stickX - Math.sign(_stickX) * STICK_DEAD_ZONE : 0
    const effY = Math.abs(_stickY) > STICK_DEAD_ZONE ? _stickY - Math.sign(_stickY) * STICK_DEAD_ZONE : 0

    if (effX !== 0 || effY !== 0) {
      _inputQ.setFromEuler(new THREE.Euler(effY * PITCH_SPEED, 0, -effX * ROLL_SPEED, 'XYZ'))
      this.orientation.multiply(_inputQ)
    }

    _stickX *= STICK_DRIFT_BACK
    _stickY *= STICK_DRIFT_BACK

    // === AUTO-LEVEL ===
    if (Math.abs(effX) < 1 && Math.abs(effY) < 1) {
      _right.set(1, 0, 0).applyQuaternion(this.orientation)
      const bankAngle = Math.asin(Math.max(-1, Math.min(1, -_right.y)))
      if (Math.abs(bankAngle) > 0.01) {
        _levelQ.setFromAxisAngle(
          new THREE.Vector3(0, 0, 1).applyQuaternion(this.orientation).normalize(),
          -bankAngle * AUTO_LEVEL_RATE
        )
        this.orientation.premultiply(_levelQ)
      }
    }

    // === BANK-TO-TURN ===
    _right.set(1, 0, 0).applyQuaternion(this.orientation)
    const bankAmount = Math.asin(Math.max(-1, Math.min(1, -_right.y)))
    const bankYaw = bankAmount * BANK_TURN_RATE * (this.speed / MAX_SPEED)
    if (Math.abs(bankYaw) > 0.0001) {
      _yawQ.setFromAxisAngle(_worldUp, bankYaw)
      this.orientation.premultiply(_yawQ)
    }

    // === AIR ROLL (Q/E) ===
    if (keys['q']) this.airRollTarget = Math.PI
    if (keys['e']) this.airRollTarget = -Math.PI
    if (Math.abs(this.airRollTarget) > 0.01) {
      const rollStep = Math.sign(this.airRollTarget) * Math.min(AIR_ROLL_SPEED, Math.abs(this.airRollTarget))
      _fwd.set(0, 0, -1).applyQuaternion(this.orientation)
      _inputQ.setFromAxisAngle(_fwd, rollStep)
      this.orientation.premultiply(_inputQ)
      this.airRollTarget -= rollStep
    }

    // === FORWARD MOTION ===
    _fwd.set(0, 0, -1).applyQuaternion(this.orientation)
    this.position.addScaledVector(_fwd, this.speed)

    // === GRAVITY & LIFT ===
    _up.set(0, 1, 0).applyQuaternion(this.orientation)
    const liftUp = Math.max(0, _up.y)
    const netGravity = GRAVITY * (1.0 - liftUp * Math.min(this.speed / MIN_SPEED, 1.0))
    this.position.y -= netGravity

    // === VERTICAL TRIM ===
    if (keys[' ']) this.position.y += 0.3 * this.speed
    if (keys['control']) this.position.y -= 0.3 * this.speed
    if (this.position.y < 2) this.position.y = 2

    // === UPDATE MODEL ===
    this.orientation.normalize()
    this.planeMesh.position.copy(this.position)
    this.planeMesh.quaternion.copy(this.orientation)

    // === CHASE CAMERA ===
    _fwd.set(0, 0, -1).applyQuaternion(this.orientation)
    _up.set(0, 1, 0).applyQuaternion(this.orientation)

    _cameraTarget.copy(this.position)
      .addScaledVector(_fwd, -CAMERA_OFFSET_BACK)
      .addScaledVector(_up, CAMERA_OFFSET_UP)
    camera.position.lerp(_cameraTarget, CAMERA_LERP_POS)

    _camUp.copy(_up).multiplyScalar(0.7).add(new THREE.Vector3(0, 0.3, 0)).normalize()
    _lookTarget.copy(this.position).addScaledVector(_fwd, CAMERA_LOOK_AHEAD)
    _lookM.lookAt(camera.position, _lookTarget, _camUp)
    _lookQ.setFromRotationMatrix(_lookM)
    camera.quaternion.slerp(_lookQ, CAMERA_LERP_LOOK)

    // === NETWORK ===
    sendMoveAction(_fwd.x * this.speed, _fwd.y * this.speed, _fwd.z * this.speed)
  }

  private createPlaneMesh(): THREE.Group {
    const group = new THREE.Group()
    const bodyMat = new THREE.MeshStandardMaterial({ color: 0x6366f1, metalness: 0.8, roughness: 0.2 })
    const wingMat = new THREE.MeshStandardMaterial({ color: 0x8b5cf6, metalness: 0.7, roughness: 0.3 })

    const body = new THREE.Mesh(new THREE.ConeGeometry(0.3, 2, 8), bodyMat)
    body.rotation.x = Math.PI / 2
    group.add(body)

    const wing = new THREE.Mesh(new THREE.BoxGeometry(3, 0.05, 0.6), wingMat)
    wing.position.z = 0.3
    group.add(wing)

    const tail = new THREE.Mesh(new THREE.BoxGeometry(1, 0.05, 0.4), wingMat)
    tail.position.set(0, 0.3, 0.9)
    group.add(tail)

    const vStab = new THREE.Mesh(new THREE.BoxGeometry(0.05, 0.6, 0.4), wingMat)
    vStab.position.set(0, 0.3, 0.9)
    group.add(vStab)

    group.add(new THREE.PointLight(0x6366f1, 1, 8))
    return group
  }
}
