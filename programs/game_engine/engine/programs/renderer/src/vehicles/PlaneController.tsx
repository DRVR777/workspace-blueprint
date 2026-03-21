/**
 * PlaneController — simplified flight controls.
 *
 * Mouse Y  → pitch (nose up / down)
 * A / D    → roll (bank left / right)
 * W / S    → throttle up / down
 * Bank-to-turn handles yaw automatically — no rudder input needed.
 * Auto-level gently flattens roll when A/D are released.
 */
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active    = false
  private _camera: THREE.PerspectiveCamera | null = null
  private euler     = new THREE.Euler(0, 0, 0, 'YXZ')
  private keys: Record<string, boolean> = {}
  private mouseDY   = 0

  // ── fly mode (free-look, entered first via Tab) ───────────────────────────
  private moveSpeed       = 0.15
  private lookSpeed       = 0.001
  private boostMultiplier = 3
  private mouseDX         = 0
  private flyTarget:  THREE.Vector3 | null = null
  private flyLookAt:  THREE.Vector3 | null = null

  // ── plane mode (Tab again) ────────────────────────────────────────────────
  private planeMode        = false
  private planePosition    = new THREE.Vector3()
  private planeEuler       = new THREE.Euler(0, 0, 0, 'YXZ')
  private planeOrientation = new THREE.Quaternion()

  private throttle     = 0.5
  private planeSpeed   = 0
  private maxSpeed     = 0.4
  private minSpeed     = 0.05
  private speedDamping = 0.02
  private gravity      = 0.004
  private bankTurnRate = 0.015

  // Input rates (radians per frame)
  private readonly PITCH_SENS = 0.003   // mouse pixels → pitch radians
  private readonly ROLL_RATE  = 0.025   // radians/frame while A or D held
  private readonly AUTO_LEVEL = 0.03    // max auto-level correction per frame

  private cameraOffset    = new THREE.Vector3(0, 5, 16)
  private cameraLerpSpeed = 0.04
  private cameraTilt      = 0.7

  private modelRotationOffset = new THREE.Quaternion(0, 1, 0, 0)

  private placeholderPlane: THREE.Group | null = null
  private planeModel:       THREE.Group | null = null
  private planeLoaded = false

  // ── constructor ───────────────────────────────────────────────────────────
  constructor() {
    if (typeof window === 'undefined') return

    window.addEventListener('keydown', e => {
      this.keys[e.code] = true
      if (e.code === 'Tab' && this.active) {
        e.preventDefault()
        this._togglePlaneMode()
      }
    })

    window.addEventListener('keyup', e => { this.keys[e.code] = false })

    window.addEventListener('mousemove', e => {
      if (!document.pointerLockElement) return
      this.mouseDX += e.movementX
      this.mouseDY += e.movementY
    })
  }

  // ── VehicleController interface ───────────────────────────────────────────
  isActive(): boolean { return this.active }

  enter(_pos: THREE.Vector3, orientation?: THREE.Quaternion): void {
    this.euler.setFromQuaternion(orientation ?? new THREE.Quaternion())
    this.planeMode = false
    this.active    = true
  }

  exit(): THREE.Vector3 {
    this.active    = false
    this.planeMode = false
    if (this.placeholderPlane) this.placeholderPlane.visible = false
    if (this.planeModel)       this.planeModel.visible       = false
    sendMoveAction(0, 0, 0, 0, 0, 0, 0, 1)
    return this.planePosition.clone()
  }

  getPosition(): THREE.Vector3 {
    return this.planeMode ? this.planePosition.clone() : new THREE.Vector3()
  }

  getVelocity(): THREE.Vector3 {
    if (!this.planeMode) return new THREE.Vector3()
    return new THREE.Vector3(0, 0, -1)
      .applyQuaternion(this.planeOrientation)
      .multiplyScalar(this.planeSpeed)
  }

  dispose(scene: THREE.Scene): void {
    if (this.placeholderPlane) scene.remove(this.placeholderPlane)
    if (this.planeModel)       scene.remove(this.planeModel)
  }

  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, _delta: number): void {
    if (!this.active) return
    this._camera = camera
    this._ensureMeshes(scene)
    if (this.planeMode) this._updatePlaneMode(camera)
    else                this._updateFlyMode(camera)
  }

  // ── fly mode ──────────────────────────────────────────────────────────────
  private _updateFlyMode(camera: THREE.PerspectiveCamera): void {
    if (this.flyTarget) {
      camera.position.lerp(this.flyTarget, 0.06)
      if (camera.position.distanceTo(this.flyTarget) < 0.3) {
        camera.position.copy(this.flyTarget)
        this.flyTarget = null
      }
      if (this.flyLookAt) {
        const m = new THREE.Matrix4().lookAt(camera.position, this.flyLookAt, new THREE.Vector3(0,1,0))
        const q = new THREE.Quaternion().setFromRotationMatrix(m)
        const te = new THREE.Euler().setFromQuaternion(q, 'YXZ')
        this.euler.x += (te.x - this.euler.x) * 0.06
        this.euler.y += (te.y - this.euler.y) * 0.06
        camera.quaternion.setFromEuler(this.euler)
        if (!this.flyTarget) this.flyLookAt = null
      }
      this.mouseDX = 0; this.mouseDY = 0
      return
    }

    if (document.pointerLockElement) {
      this.euler.y -= this.mouseDX * this.lookSpeed
      this.euler.x -= this.mouseDY * this.lookSpeed
      this.euler.x  = Math.max(-Math.PI * 0.47, Math.min(Math.PI * 0.47, this.euler.x))
      camera.quaternion.setFromEuler(this.euler)
    }
    this.mouseDX = 0; this.mouseDY = 0

    const speed = this.moveSpeed *
      ((this.keys['ShiftLeft'] || this.keys['ShiftRight']) ? this.boostMultiplier : 1)
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(camera.quaternion)
    const rgt = new THREE.Vector3(1, 0, 0).applyQuaternion(camera.quaternion)
    if (this.keys['KeyW']) camera.position.addScaledVector(fwd,  speed)
    if (this.keys['KeyS']) camera.position.addScaledVector(fwd, -speed)
    if (this.keys['KeyA']) camera.position.addScaledVector(rgt, -speed)
    if (this.keys['KeyD']) camera.position.addScaledVector(rgt,  speed)
    if (this.keys['Space'])                                    camera.position.y += speed
    if (this.keys['ControlLeft'] || this.keys['ControlRight']) camera.position.y -= speed
  }

  // ── plane mode ────────────────────────────────────────────────────────────
  private _updatePlaneMode(camera: THREE.PerspectiveCamera): void {
    const plane = this._getActivePlane()
    if (!plane) return

    // Throttle
    if (this.keys['KeyW']) this.throttle = Math.min(1.0, this.throttle + 0.008)
    if (this.keys['KeyS']) this.throttle = Math.max(0.0, this.throttle - 0.006)
    const targetSpeed = this.minSpeed + this.throttle * (this.maxSpeed - this.minSpeed)
    this.planeSpeed  += (targetSpeed - this.planeSpeed) * this.speedDamping

    // Pitch — mouse Y only
    const pitchRate = this.mouseDY * this.PITCH_SENS
    if (Math.abs(pitchRate) > 0.00001) {
      this.planeOrientation.multiply(
        new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), pitchRate)
      )
    }
    this.mouseDX = 0
    this.mouseDY = 0

    // Roll — A/D keys only
    const rollInput = (this.keys['KeyA'] ? 1 : 0) - (this.keys['KeyD'] ? 1 : 0)
    if (rollInput !== 0) {
      this.planeOrientation.multiply(
        new THREE.Quaternion().setFromAxisAngle(
          new THREE.Vector3(0, 0, 1), rollInput * this.ROLL_RATE
        )
      )
    } else {
      // Auto-level: gently flatten roll when A/D released
      const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.planeOrientation)
      const planeUp    = new THREE.Vector3(0, 1, 0).applyQuaternion(this.planeOrientation)
      let bankAngle    = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
      if (planeUp.y < 0) bankAngle = Math.sign(bankAngle || 1) * Math.PI - bankAngle
      if (Math.abs(bankAngle) > 0.01) {
        const correction = THREE.MathUtils.clamp(bankAngle, -this.AUTO_LEVEL, this.AUTO_LEVEL)
        this.planeOrientation.multiply(
          new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), -correction)
        )
      }
    }

    // Bank-to-turn: yaw proportional to bank angle × speed
    const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.planeOrientation)
    const bankAmount = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
    const bankYaw    = bankAmount * this.bankTurnRate * (this.planeSpeed / this.maxSpeed)
    if (Math.abs(bankYaw) > 0.00001) {
      this.planeOrientation.premultiply(
        new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), bankYaw)
      )
    }

    this.planeOrientation.normalize()

    // Movement
    const noseDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.planeOrientation)
    this.planePosition.addScaledVector(noseDir, this.planeSpeed)

    // Gravity + lift
    const planeUp  = new THREE.Vector3(0, 1, 0).applyQuaternion(this.planeOrientation)
    const liftUp   = Math.max(0, planeUp.y)
    const netGravity = this.gravity * (1.0 - liftUp * Math.min(this.planeSpeed / this.minSpeed, 1.0))
    this.planePosition.y -= netGravity

    // Apply to mesh
    plane.quaternion.copy(this.planeOrientation).multiply(this.modelRotationOffset)
    plane.position.copy(this.planePosition)

    // Camera follows behind/above
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(this.planeOrientation)
    const up  = new THREE.Vector3(0, 1, 0).applyQuaternion(this.planeOrientation)
    const camTarget = this.planePosition.clone()
      .addScaledVector(fwd, -this.cameraOffset.z)
      .addScaledVector(up,   this.cameraOffset.y)
    camera.position.lerp(camTarget, this.cameraLerpSpeed)

    const t       = this.cameraTilt
    const camUp   = up.clone().multiplyScalar(t).add(new THREE.Vector3(0, 1 - t, 0)).normalize()
    const lookTarget = this.planePosition.clone().addScaledVector(fwd, 10)
    const lookM   = new THREE.Matrix4().lookAt(camera.position, lookTarget, camUp)
    const lookQ   = new THREE.Quaternion().setFromRotationMatrix(lookM)
    camera.quaternion.slerp(lookQ, 0.08)
    this.euler.setFromQuaternion(camera.quaternion, 'YXZ')

    // Network
    const vel = this.getVelocity()
    const q   = this.planeOrientation
    const pos = this.planePosition
    sendMoveAction(vel.x, vel.y, vel.z, 1, q.x, q.y, q.z, q.w, pos.x, pos.y, pos.z)
  }

  // ── toggle ────────────────────────────────────────────────────────────────
  private _togglePlaneMode(): void {
    this.planeMode = !this.planeMode
    const plane = this._getActivePlane()
    const cam   = this._camera

    if (this.planeMode) {
      if (cam) this.planePosition.copy(cam.position)
      this.planeEuler.set(0, this.euler.y, 0, 'YXZ')
      this.planeOrientation.setFromEuler(new THREE.Euler(0, this.euler.y, 0, 'YXZ'))
      this.throttle   = 0.5
      this.planeSpeed = this.minSpeed + 0.5 * (this.maxSpeed - this.minSpeed)
      if (plane) { plane.visible = true; plane.position.copy(this.planePosition) }
    } else {
      if (cam) {
        cam.position.copy(this.planePosition)
        this.euler.copy(this.planeEuler)
        cam.quaternion.setFromEuler(this.euler)
      }
      if (plane) plane.visible = false
    }
  }

  // ── helpers ───────────────────────────────────────────────────────────────
  flyTo(position: THREE.Vector3, lookAt?: THREE.Vector3): void {
    this.flyTarget = position.clone()
    this.flyLookAt = lookAt ? lookAt.clone() : null
  }

  private _getActivePlane(): THREE.Group | null {
    return (this.planeLoaded && this.planeModel) ? this.planeModel : this.placeholderPlane
  }

  private _ensureMeshes(scene: THREE.Scene): void {
    if (this.placeholderPlane) return

    const group   = new THREE.Group()
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
    group.visible = false
    scene.add(group)
    this.placeholderPlane = group

    new GLTFLoader().load(
      (import.meta as unknown as { env: { BASE_URL: string } }).env.BASE_URL + 'models/plane.glb',
      (gltf) => {
        this.planeModel = gltf.scene
        this.planeModel.scale.setScalar(3.4)
        this.planeModel.visible = false
        scene.add(this.planeModel)
        this.planeLoaded = true
      },
      undefined,
      () => { this.planeLoaded = false },
    )
  }
}
