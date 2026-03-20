/**
 * PlaneController — direct TypeScript port of 3dGraphUniverse/src/controls.js (plane-mode branch).
 * One intentional deviation from source: dead-zone auto-level flattens roll only, not pitch.
 */
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

const DEG2 = Math.PI / 90

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  // ── shared ────────────────────────────────────────────────────────────────
  private active = false
  private _camera: THREE.PerspectiveCamera | null = null
  private euler    = new THREE.Euler(0, 0, 0, 'YXZ')
  private keys: Record<string, boolean> = {}
  private mouseDX  = 0
  private mouseDY  = 0

  // ── fly mode ──────────────────────────────────────────────────────────────
  private moveSpeed       = 0.15
  private lookSpeed       = 0.001
  private boostMultiplier = 3
  private flyTarget:  THREE.Vector3 | null = null
  private flyLookAt:  THREE.Vector3 | null = null

  // ── plane mode ────────────────────────────────────────────────────────────
  private planeMode        = false
  private planePosition    = new THREE.Vector3()
  private planeEuler       = new THREE.Euler(0, 0, 0, 'YXZ')
  private planeOrientation = new THREE.Quaternion()

  // physics — exact values from source
  private throttle    = 0.5
  private planeSpeed  = 0
  private maxSpeed    = 0.4
  private minSpeed    = 0.05
  private speedDamping = 0.02
  private gravity     = 0.004
  private pitchSpeed  = 0.0012
  private rollSpeed   = 0.0015
  private bankTurnRate = 0.015
  private cameraOffset = new THREE.Vector3(0, 5, 16) // x unused, y=up, z=back

  private cameraLerpSpeed = 0.04
  private cameraTilt      = 0.7

  // stick
  private _stickX = 0
  private _stickY = 0

  // air roll
  private airRollTarget = 0
  private airRolling    = false

  // calibration
  private calibrating          = false
  private modelRotationOffset  = new THREE.Quaternion(0, 1, 0, 0)

  // reticle smoothing
  private _reticleOffX = 0
  private _reticleOffY = 0

  // DOM
  private reticleEl:      HTMLDivElement | null = null
  private calibrationUI:  HTMLDivElement | null = null

  // meshes
  private placeholderPlane: THREE.Group | null = null
  private planeModel:       THREE.Group | null = null
  private planeLoaded = false

  // ── constructor ───────────────────────────────────────────────────────────
  constructor() {
    if (typeof window === 'undefined') return

    window.addEventListener('keydown', e => {
      this.keys[e.code] = true

      // Tab: enter vehicle → fly mode; while active Tab → toggle plane mode
      if (e.code === 'Tab' && this.active) {
        e.preventDefault()
        if (!this.calibrating) this._togglePlaneMode()
        return
      }

      // C: toggle calibration (plane mode only)
      if (e.code === 'KeyC' && this.planeMode && this.active) {
        e.preventDefault()
        this.calibrating = !this.calibrating
        if (this.calibrationUI)
          this.calibrationUI.style.display = this.calibrating ? 'block' : 'none'
        if (this.calibrating) document.exitPointerLock()
        return
      }

      // calibration arrow / U / O keys
      if (this.calibrating) {
        const q = new THREE.Quaternion()
        let rotated = false
        if (e.code === 'ArrowLeft')  { q.setFromAxisAngle(new THREE.Vector3(0,1,0),  DEG2); rotated = true }
        if (e.code === 'ArrowRight') { q.setFromAxisAngle(new THREE.Vector3(0,1,0), -DEG2); rotated = true }
        if (e.code === 'ArrowUp')    { q.setFromAxisAngle(new THREE.Vector3(1,0,0),  DEG2); rotated = true }
        if (e.code === 'ArrowDown')  { q.setFromAxisAngle(new THREE.Vector3(1,0,0), -DEG2); rotated = true }
        if (e.code === 'KeyU')       { q.setFromAxisAngle(new THREE.Vector3(0,0,1),  DEG2); rotated = true }
        if (e.code === 'KeyO')       { q.setFromAxisAngle(new THREE.Vector3(0,0,1), -DEG2); rotated = true }
        if (rotated) {
          this.modelRotationOffset.multiply(q)
          const o = this.modelRotationOffset
          console.log('MODEL_OFFSET:', JSON.stringify({
            x: +o.x.toFixed(6), y: +o.y.toFixed(6),
            z: +o.z.toFixed(6), w: +o.w.toFixed(6),
          }))
        }
        return
      }

      // Q / E air roll (plane mode only)
      if (this.planeMode && this.active) {
        if (e.code === 'KeyQ' && !this.airRolling) { this.airRollTarget =  Math.PI; this.airRolling = true }
        if (e.code === 'KeyE' && !this.airRolling) { this.airRollTarget = -Math.PI; this.airRolling = true }
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
    this._ensureDOM()
  }

  exit(): THREE.Vector3 {
    this.active    = false
    this.planeMode = false
    this.calibrating = false
    if (this.calibrationUI)   this.calibrationUI.style.display  = 'none'
    if (this.reticleEl)       this.reticleEl.style.display       = 'none'
    if (this.placeholderPlane) this.placeholderPlane.visible     = false
    if (this.planeModel)       this.planeModel.visible           = false
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
    if (this.reticleEl)        this.reticleEl.remove()
    if (this.calibrationUI)    this.calibrationUI.remove()
  }

  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, _delta: number): void {
    if (!this.active) return
    this._camera = camera
    this._ensureMeshes(scene)
    if (this.planeMode) this._updatePlaneMode(camera)
    else                this._updateFlyMode(camera)
  }

  // ── fly mode (source: _updateFlyMode) ────────────────────────────────────
  private _updateFlyMode(camera: THREE.PerspectiveCamera): void {
    // fly-to animation (used by external callers e.g. node click)
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
      return
    }

    if (document.pointerLockElement) {
      this.euler.y -= this.mouseDX * this.lookSpeed
      this.euler.x -= this.mouseDY * this.lookSpeed
      this.euler.x  = Math.max(-Math.PI * 0.47, Math.min(Math.PI * 0.47, this.euler.x))
      camera.quaternion.setFromEuler(this.euler)
    }
    this.mouseDX = 0
    this.mouseDY = 0

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

  // ── plane mode (source: _updatePlaneMode) ────────────────────────────────
  private _updatePlaneMode(camera: THREE.PerspectiveCamera): void {
    const plane = this._getActivePlane()
    if (!plane) return

    if (!this.calibrating) {
      // throttle
      const boost = (this.keys['ShiftLeft'] || this.keys['ShiftRight']) ? 2.0 : 1.0
      if (this.keys['KeyW']) this.throttle = Math.min(1.0, this.throttle + 0.008)
      if (this.keys['KeyS']) this.throttle = Math.max(0.0, this.throttle - 0.006)
      const targetSpeed = (this.minSpeed + this.throttle * (this.maxSpeed - this.minSpeed)) * boost
      this.planeSpeed  += (targetSpeed - this.planeSpeed) * this.speedDamping

      // virtual stick — accumulate mouse while pointer locked
      if (document.pointerLockElement) {
        this._stickX += this.mouseDX * 0.5
        this._stickY += this.mouseDY * 0.5
        const stickDist = Math.sqrt(this._stickX ** 2 + this._stickY ** 2)
        if (stickDist > 100) {
          this._stickX *= 100 / stickDist
          this._stickY *= 100 / stickDist
        }
      }
      this.mouseDX = 0
      this.mouseDY = 0

      // drift stick to center
      this._stickX *= 0.94
      this._stickY *= 0.94

      const sx       = this._stickX
      const sy       = this._stickY
      const stickMag = Math.sqrt(sx * sx + sy * sy)

      if (stickMag > 8) {
        // outside dead zone — rotate plane
        const effectiveMag = stickMag - 8
        const nx = sx / stickMag
        const ny = sy / stickMag
        const ex = nx * effectiveMag
        const ey = ny * effectiveMag

        const axis = new THREE.Vector3(
          -ey * this.pitchSpeed,
           0,
          -ex * this.rollSpeed,
        ).normalize()
        const angle = effectiveMag * Math.max(this.pitchSpeed, this.rollSpeed) * 0.25
        this.planeOrientation.multiply(new THREE.Quaternion().setFromAxisAngle(axis, angle))

      } else {
        // inside dead zone — flatten roll only, leave pitch untouched
        const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.planeOrientation)
        const bankAngle  = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
        if (Math.abs(bankAngle) > 0.01) {
          this.planeOrientation.multiply(
            new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), -bankAngle * 0.02)
          )
        }
      }

      // bank-to-turn
      const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.planeOrientation)
      const bankAmount = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
      const bankYaw    = bankAmount * this.bankTurnRate * (this.planeSpeed / this.maxSpeed)
      if (Math.abs(bankYaw) > 0.00001) {
        this.planeOrientation.premultiply(
          new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), bankYaw)
        )
      }

      // air roll Q / E
      if (this.airRolling) {
        const step = Math.sign(this.airRollTarget) * 0.08
        let airRollAngle: number
        if (Math.abs(this.airRollTarget) > Math.abs(step)) {
          this.airRollTarget -= step
          airRollAngle = step
        } else {
          airRollAngle       = this.airRollTarget
          this.airRollTarget = 0
          this.airRolling    = false
        }
        this.planeOrientation.multiply(
          new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), airRollAngle)
        )
      }

      this.planeOrientation.normalize()

      // movement
      const noseDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.planeOrientation)
      this.planePosition.addScaledVector(noseDir, this.planeSpeed)

      // gravity + lift
      const planeUp  = new THREE.Vector3(0, 1, 0).applyQuaternion(this.planeOrientation)
      const liftUp   = Math.max(0, planeUp.y)
      const netGravity = this.gravity * (1.0 - liftUp * Math.min(this.planeSpeed / this.minSpeed, 1.0))
      this.planePosition.y -= netGravity

      if (this.keys['Space'])                                    this.planePosition.y += this.planeSpeed * 0.3
      if (this.keys['ControlLeft'] || this.keys['ControlRight']) this.planePosition.y -= this.planeSpeed * 0.3

      this._reticleOffX = this._stickX
      this._reticleOffY = this._stickY
    } else {
      this.mouseDX = 0
      this.mouseDY = 0
    }

    // apply to mesh
    plane.quaternion.copy(this.planeOrientation).multiply(this.modelRotationOffset)
    plane.position.copy(this.planePosition)

    // camera
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(this.planeOrientation)
    const up  = new THREE.Vector3(0, 1, 0).applyQuaternion(this.planeOrientation)
    const camTarget = this.planePosition.clone()
      .addScaledVector(fwd, -this.cameraOffset.z)
      .addScaledVector(up,   this.cameraOffset.y)
    camera.position.lerp(camTarget, this.cameraLerpSpeed)

    const t      = this.cameraTilt
    const camUp  = up.clone().multiplyScalar(t).add(new THREE.Vector3(0, 1 - t, 0)).normalize()
    const lookTarget = this.planePosition.clone().addScaledVector(fwd, 10)
    const lookM  = new THREE.Matrix4().lookAt(camera.position, lookTarget, camUp)
    const lookQ  = new THREE.Quaternion().setFromRotationMatrix(lookM)
    camera.quaternion.slerp(lookQ, 0.08)
    this.euler.setFromQuaternion(camera.quaternion, 'YXZ')

    // reticle
    const offDist = Math.sqrt(this._reticleOffX ** 2 + this._reticleOffY ** 2)
    if (offDist > 80) { this._reticleOffX *= 80 / offDist; this._reticleOffY *= 80 / offDist }
    this._reticleOffX *= 0.92
    this._reticleOffY *= 0.92
    if (this.reticleEl) {
      this.reticleEl.style.left    = `${window.innerWidth  / 2 + this._reticleOffX}px`
      this.reticleEl.style.top     = `${window.innerHeight / 2 + this._reticleOffY}px`
      this.reticleEl.style.opacity = '1'
    }

    // network
    const vel = this.getVelocity()
    sendMoveAction(vel.x, vel.y, vel.z)
  }

  // ── toggle (source: togglePlaneMode) ─────────────────────────────────────
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
      this._stickX = 0; this._stickY = 0
      this.airRolling = false; this.airRollTarget = 0
      if (plane) { plane.visible = true; plane.position.copy(this.planePosition) }
      if (this.reticleEl) this.reticleEl.style.display = 'block'
    } else {
      if (cam) {
        cam.position.copy(this.planePosition)
        this.euler.copy(this.planeEuler)
        cam.quaternion.setFromEuler(this.euler)
      }
      if (plane) plane.visible = false
      this.calibrating = false
      if (this.calibrationUI) this.calibrationUI.style.display = 'none'
      if (this.reticleEl)     this.reticleEl.style.display     = 'none'
    }
  }

  // ── helpers ───────────────────────────────────────────────────────────────
  flyTo(position: THREE.Vector3, lookAt?: THREE.Vector3): void {
    this.flyTarget  = position.clone()
    this.flyLookAt  = lookAt ? lookAt.clone() : null
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

    new GLTFLoader().load('models/plane.glb', (gltf) => {
      this.planeModel = gltf.scene
      this.planeModel.scale.setScalar(3.4)
      this.planeModel.visible = false
      scene.add(this.planeModel)
      this.planeLoaded = true
    }, undefined, () => { this.planeLoaded = false })
  }

  private _ensureDOM(): void {
    if (!this.reticleEl) {
      const el  = document.createElement('div')
      el.style.cssText = `
        position:fixed; width:20px; height:20px;
        border:1.5px solid rgba(255,255,255,0.35); border-radius:50%;
        pointer-events:none; z-index:16; display:none;
        transform:translate(-50%,-50%);
      `
      const dot = document.createElement('div')
      dot.style.cssText = `
        position:absolute; top:50%; left:50%;
        width:3px; height:3px; background:rgba(255,255,255,0.6);
        border-radius:50%; transform:translate(-50%,-50%);
      `
      el.appendChild(dot)
      document.body.appendChild(el)
      this.reticleEl = el
    }

    if (!this.calibrationUI) {
      const div = document.createElement('div')
      div.style.cssText = `
        position:fixed; top:14px; right:14px;
        background:rgba(10,10,20,0.92); border:2px solid #f59e0b;
        border-radius:12px; padding:20px 28px; z-index:100;
        font-family:monospace; color:#fbbf24; text-align:center;
        display:none; pointer-events:none;
      `
      div.innerHTML = `
        <div style="font-size:1.1rem;font-weight:bold;margin-bottom:8px;">CALIBRATION MODE</div>
        <div style="font-size:0.8rem;color:rgba(255,255,255,0.6);line-height:1.6;">
          Arrows — Y/X axis &bull; U/O — Z axis &bull; C — exit
        </div>
      `
      document.body.appendChild(div)
      this.calibrationUI = div
    }
  }
}
