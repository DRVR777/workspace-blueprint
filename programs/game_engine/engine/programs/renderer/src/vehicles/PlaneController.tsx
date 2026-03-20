/**
 * PlaneController — exact port of 3dGraphUniverse/src/controls.js plane mode.
 *
 * This is a line-for-line adaptation of the final working plane physics from
 * the 3dGraphUniverse project (commit: plane-mode branch, latest).
 *
 * Implements VehicleController interface for the modular vehicle system.
 *
 * Physics:
 *   - Virtual stick with dead zone + drift-back-to-center
 *   - Combined single-axis rotation (no pitch/roll asymmetry)
 *   - Auto-level when stick is centered
 *   - Bank-to-turn (bankTurnRate = 0.025)
 *   - Speed-dependent lift vs gravity
 *   - Camera tilt blend (0 = level, 0.7 default, 1.0 = cockpit)
 *
 * Source: 3dGraphUniverse/src/controls.js lines 314-486
 */
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

// ============================================================================
// Constants (exact values from 3dGraphUniverse)
// ============================================================================

const MAX_SPEED = 0.4
const MIN_SPEED = 0.05
const SPEED_DAMPING = 0.02
const GRAVITY = 0.004
const PITCH_SPEED = 0.0012
const ROLL_SPEED = 0.0015
const BANK_TURN_RATE = 0.025
const AIR_ROLL_SPEED = 0.08
const AUTO_LEVEL_RATE = 0.02
const STICK_INPUT_SCALE = 0.5
const STICK_MAX_RADIUS = 100
const STICK_DRIFT_BACK = 0.94
const STICK_DEAD_ZONE = 8
const ROTATION_ANGLE_SCALE = 0.25
const CAMERA_OFFSET_BACK = 16
const CAMERA_OFFSET_UP = 5
const CAMERA_LERP_POS = 0.04
const CAMERA_LERP_LOOK = 0.08
const CAMERA_TILT_DEFAULT = 0.7

// ============================================================================
// Mouse input (module-level — shared across instances)
// ============================================================================

let _mouseDX = 0
let _mouseDY = 0
let _mouseListenerAdded = false

function ensureMouseListener(): void {
  if (_mouseListenerAdded) return
  _mouseListenerAdded = true
  window.addEventListener('mousemove', (e) => {
    if (!document.pointerLockElement) return
    _mouseDX += e.movementX
    _mouseDY += e.movementY
  })
}

// ============================================================================
// PlaneVehicle
// ============================================================================

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active = false
  private throttle = 0.5
  private speed = 0
  private airRollTarget = 0
  private airRolling = false
  private cameraTilt = CAMERA_TILT_DEFAULT
  private orientation = new THREE.Quaternion()
  private position = new THREE.Vector3()
  private planeMesh: THREE.Group | null = null
  private planeGLB: THREE.Group | null = null
  private glbLoaded = false
  private modelRotationOffset = new THREE.Quaternion(0, 1, 0, 0) // 180° Y rotation
  private keys: Record<string, boolean> = {}

  // Virtual stick state (from 3dGraphUniverse)
  private stickX = 0
  private stickY = 0

  constructor() {
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', (e) => { this.keys[e.key.toLowerCase()] = true })
      window.addEventListener('keyup', (e) => { this.keys[e.key.toLowerCase()] = false })
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
    this.stickX = 0
    this.stickY = 0
    this.active = true
  }

  exit(): THREE.Vector3 {
    this.active = false
    if (this.planeMesh) this.planeMesh.visible = false
    return this.position.clone()
  }

  getPosition(): THREE.Vector3 { return this.position.clone() }

  getVelocity(): THREE.Vector3 {
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
    return fwd.multiplyScalar(this.speed)
  }

  dispose(scene: THREE.Scene): void {
    if (this.planeMesh) { scene.remove(this.planeMesh); this.planeMesh = null }
  }

  // ==========================================================================
  // UPDATE — exact port of _updatePlaneMode() from 3dGraphUniverse
  // ==========================================================================

  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, _delta: number): void {
    if (!this.active) return

    if (!this.planeMesh) {
      this.planeMesh = this.createPlaneMesh()
      scene.add(this.planeMesh)
    }
    this.planeMesh.visible = true

    const keys = this.keys

    // === Throttle (controls.js lines 322-327) ===
    const boost = keys['shift'] ? 2.0 : 1.0
    if (keys['w']) this.throttle = Math.min(1.0, this.throttle + 0.008)
    if (keys['s']) this.throttle = Math.max(0.0, this.throttle - 0.006)
    const targetSpeed = (MIN_SPEED + this.throttle * (MAX_SPEED - MIN_SPEED)) * boost
    this.speed += (targetSpeed - this.speed) * SPEED_DAMPING

    // === Mouse → virtual stick (controls.js lines 334-348) ===
    if (document.pointerLockElement) {
      this.stickX += _mouseDX * STICK_INPUT_SCALE
      this.stickY += _mouseDY * STICK_INPUT_SCALE

      const stickDist = Math.sqrt(this.stickX ** 2 + this.stickY ** 2)
      if (stickDist > STICK_MAX_RADIUS) {
        this.stickX *= STICK_MAX_RADIUS / stickDist
        this.stickY *= STICK_MAX_RADIUS / stickDist
      }
    }
    _mouseDX = 0
    _mouseDY = 0

    // Drift stick back to center (controls.js lines 350-352)
    this.stickX *= STICK_DRIFT_BACK
    this.stickY *= STICK_DRIFT_BACK

    // Dead zone + combined single-axis rotation (controls.js lines 354-390)
    const stickMag = Math.sqrt(this.stickX * this.stickX + this.stickY * this.stickY)

    if (stickMag > STICK_DEAD_ZONE) {
      // Subtract dead zone from magnitude
      const effectiveMag = stickMag - STICK_DEAD_ZONE
      const nx = this.stickX / stickMag
      const ny = this.stickY / stickMag
      const ex = nx * effectiveMag
      const ey = ny * effectiveMag

      // Combined axis: mouse direction → one rotation axis in local space
      const axis = new THREE.Vector3(
        -ey * PITCH_SPEED,
        0,
        -ex * ROLL_SPEED
      ).normalize()

      const angle = effectiveMag * Math.max(PITCH_SPEED, ROLL_SPEED) * ROTATION_ANGLE_SCALE
      const inputQ = new THREE.Quaternion().setFromAxisAngle(axis, angle)
      this.orientation.multiply(inputQ)
    } else {
      // Inside dead zone — auto-level the bank (controls.js lines 382-390)
      const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.orientation)
      const bankAngle = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
      if (Math.abs(bankAngle) > 0.01) {
        const levelQ = new THREE.Quaternion().setFromAxisAngle(
          new THREE.Vector3(0, 0, 1), -bankAngle * AUTO_LEVEL_RATE
        )
        this.orientation.multiply(levelQ)
      }
    }

    // === Bank-to-turn (controls.js lines 393-404) ===
    const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.orientation)
    const bankAmount = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
    const bankYaw = bankAmount * BANK_TURN_RATE * (this.speed / MAX_SPEED)
    if (Math.abs(bankYaw) > 0.00001) {
      const yawQ = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), bankYaw)
      this.orientation.premultiply(yawQ)
    }

    // === Air roll Q/E (controls.js lines 406-421) ===
    if (keys['q'] && !this.airRolling) { this.airRollTarget = Math.PI; this.airRolling = true }
    if (keys['e'] && !this.airRolling) { this.airRollTarget = -Math.PI; this.airRolling = true }
    if (this.airRolling) {
      const step = Math.sign(this.airRollTarget) * AIR_ROLL_SPEED
      let airRollAngle: number
      if (Math.abs(this.airRollTarget) > Math.abs(step)) {
        this.airRollTarget -= step
        airRollAngle = step
      } else {
        airRollAngle = this.airRollTarget
        this.airRollTarget = 0
        this.airRolling = false
      }
      this.orientation.multiply(
        new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), airRollAngle)
      )
    }

    this.orientation.normalize()

    // === Movement (controls.js lines 425-438) ===
    const noseDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
    this.position.addScaledVector(noseDir, this.speed)

    // Gravity + lift
    const planeUp = new THREE.Vector3(0, 1, 0).applyQuaternion(this.orientation)
    const liftUp = Math.max(0, planeUp.y)
    const netGravity = GRAVITY * (1.0 - liftUp * Math.min(this.speed / MIN_SPEED, 1.0))
    this.position.y -= netGravity

    // Space/Ctrl trim
    if (keys[' ']) this.position.y += this.speed * 0.3
    if (keys['control']) this.position.y -= this.speed * 0.3

    // Floor clamp
    if (this.position.y < 2) this.position.y = 2

    // === Apply to model (controls.js lines 448-450) ===
    this.planeMesh.position.copy(this.position)
    // Apply model rotation offset so GLB aligns with flight direction
    this.planeMesh.quaternion.copy(this.orientation)
    if (this.glbLoaded) {
      this.planeMesh.quaternion.multiply(this.modelRotationOffset)
    }

    // === Camera (controls.js lines 452-471) ===
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
    const up = new THREE.Vector3(0, 1, 0).applyQuaternion(this.orientation)

    const cameraTarget = this.position.clone()
      .addScaledVector(fwd, -CAMERA_OFFSET_BACK)
      .addScaledVector(up, CAMERA_OFFSET_UP)
    camera.position.lerp(cameraTarget, CAMERA_LERP_POS)

    // Camera up: blend between plane up and world up
    const t = this.cameraTilt
    const camUp = up.clone().multiplyScalar(t).add(new THREE.Vector3(0, 1 - t, 0)).normalize()
    const lookTarget = this.position.clone().addScaledVector(fwd, 10)
    const lookM = new THREE.Matrix4().lookAt(camera.position, lookTarget, camUp)
    const lookQ = new THREE.Quaternion().setFromRotationMatrix(lookM)
    camera.quaternion.slerp(lookQ, CAMERA_LERP_LOOK)

    // === Network ===
    sendMoveAction(noseDir.x * this.speed, noseDir.y * this.speed, noseDir.z * this.speed)
  }

  private createPlaneMesh(): THREE.Group {
    const group = new THREE.Group()

    // Placeholder mesh (shown until GLB loads)
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

    // Load real plane.glb model (from 3dGraphUniverse)
    // Replaces placeholder when loaded
    new GLTFLoader().load('/models/plane.glb', (gltf) => {
      this.planeGLB = gltf.scene
      this.planeGLB.scale.setScalar(3.4)
      this.glbLoaded = true

      // Hide placeholder parts, show GLB
      group.children.forEach(child => {
        if (child instanceof THREE.Mesh) child.visible = false
      })
      group.add(this.planeGLB)

      console.log('[Plane] GLB model loaded')
    }, undefined, (err) => {
      console.warn('[Plane] GLB load failed, using placeholder:', err)
    })

    return group
  }
}
