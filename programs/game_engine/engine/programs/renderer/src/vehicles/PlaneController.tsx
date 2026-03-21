/**
 * PlaneController — exact port of FlyControls._updatePlaneMode() from
 * https://github.com/DRVR777/3dGraphUniverse (branch: plane-mode, src/controls.js)
 *
 * Physics order every frame:
 *   1. Throttle (R/F keys; Shift = 2× boost)
 *   2. Mouse → combined pitch+roll as ONE setFromAxisAngle (not two separate multiplies)
 *   3. WASD pitch/roll at 0.015 rad/frame
 *   4. Bank-to-turn: -bankAmount * rate, dead zone 0.09, multiply() (local space)
 *   5. Air roll Q/E (180° barrel roll)
 *   6. Normalize
 *   7. Move along nose dir + gravity/lift + Space/Ctrl trim
 *   8. Apply to model mesh
 *   9. Camera follow
 *
 * Game-engine additions kept:
 *   - VehicleController interface
 *   - sendMoveAction() with client-authoritative position (server teleports body)
 *   - GLTFLoader with BASE_URL prefix
 *   - TypeScript types
 */

import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

const DEG2 = Math.PI / 90

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active = false
  private _camera: THREE.PerspectiveCamera | null = null
  private euler = new THREE.Euler(0, 0, 0, 'YXZ')
  private keys: Record<string, boolean> = {}
  private mouseDX = 0
  private mouseDY = 0

  // ── fly mode ──────────────────────────────────────────────────────────────
  private moveSpeed = 0.15
  private lookSpeed = 0.001
  private boostMultiplier = 3
  private flyTarget: THREE.Vector3 | null = null
  private flyLookAt: THREE.Vector3 | null = null

  // ── plane mode ────────────────────────────────────────────────────────────
  private planeMode = false
  private planePosition = new THREE.Vector3()
  private planeEuler = new THREE.Euler(0, 0, 0, 'YXZ')

  // Single quaternion — no euler decomposition
  private planeOrientation = new THREE.Quaternion()

  // Physics (exact values from controls.js)
  private throttle = 0.5
  private planeSpeed = 0
  private maxSpeed = 0.4
  private minSpeed = 0.05
  private speedDamping = 0.02
  private gravity = 0.004

  // Mouse sensitivity (exact values from controls.js)
  private pitchSpeed = 0.0012
  private rollSpeed = 0.0015
  private bankTurnRate = 0.015

  // Camera (exact values from controls.js)
  private cameraOffset = new THREE.Vector3(0, 5, 16)
  private cameraLerpSpeed = 0.04
  private cameraTilt = 0.7

  // Model correction (180° Y — depends on GLB baked rotation)
  private modelRotationOffset = new THREE.Quaternion(0, 1, 0, 0)

  // Calibration
  private calibrating = false

  // Air roll (Q/E)
  private airRollTarget = 0
  private airRolling = false

  // Reticle stick tracking
  private _stickX = 0
  private _stickY = 0

  // Hoisted temporaries — zero allocations in hot path
  private _tmpAxis  = new THREE.Vector3()
  private _tmpQ     = new THREE.Quaternion()
  private _planeRight = new THREE.Vector3()
  private _noseDir  = new THREE.Vector3()
  private _planeUp  = new THREE.Vector3()
  private _fwd      = new THREE.Vector3()
  private _up       = new THREE.Vector3()
  private _camTarget  = new THREE.Vector3()
  private _lookTarget = new THREE.Vector3()
  private _lookM    = new THREE.Matrix4()
  private _lookQ    = new THREE.Quaternion()

  // Meshes
  private placeholderPlane: THREE.Group | null = null
  private planeModel: THREE.Group | null = null
  private planeLoaded = false

  // DOM elements (created in _ensureMeshes, destroyed in dispose)
  private calibrationUI: HTMLElement | null = null
  private debugHUD: HTMLElement | null = null
  private debugLog: HTMLElement | null = null
  private _statsEl: HTMLElement | null = null
  private _debugLogLines: string[] = []
  private reticleEl: HTMLElement | null = null

  // ── constructor ───────────────────────────────────────────────────────────
  constructor() {
    if (typeof window === 'undefined') return

    window.addEventListener('keydown', e => {
      this.keys[e.code] = true

      if (e.code === 'Tab' && this.active) {
        e.preventDefault()
        if (!this.calibrating) this._togglePlaneMode()
        return
      }

      // Calibration toggle (C key in plane mode)
      if (e.code === 'KeyC' && this.planeMode && this.active) {
        e.preventDefault()
        this.calibrating = !this.calibrating
        if (this.calibrationUI) {
          this.calibrationUI.style.display = this.calibrating ? 'block' : 'none'
        }
        if (this.calibrating) {
          document.exitPointerLock()
        } else {
          document.querySelector('canvas')?.requestPointerLock()
        }
        return
      }

      // Calibration arrow keys
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
            x: +o.x.toFixed(6), y: +o.y.toFixed(6), z: +o.z.toFixed(6), w: +o.w.toFixed(6)
          }))
        }
        return
      }

      // Air roll Q/E
      if (this.planeMode && this.active && !this.calibrating) {
        if (e.code === 'KeyQ' && !this.airRolling) { this.airRollTarget = Math.PI;  this.airRolling = true }
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
    this.active = true
  }

  exit(): THREE.Vector3 {
    this.active = false
    this.planeMode = false
    this.calibrating = false
    if (this.calibrationUI) this.calibrationUI.style.display = 'none'
    if (this.debugHUD) this.debugHUD.style.display = 'none'
    if (this.reticleEl) this.reticleEl.style.display = 'none'
    if (this.placeholderPlane) this.placeholderPlane.visible = false
    if (this.planeModel) this.planeModel.visible = false
    sendMoveAction(0, 0, 0, 0, 0, 0, 0, 1)
    return this.planePosition.clone()
  }

  getPosition(): THREE.Vector3 {
    return this.planeMode ? this.planePosition.clone() : new THREE.Vector3()
  }

  getVelocity(): THREE.Vector3 {
    if (!this.planeMode) return this._noseDir.set(0, 0, 0)
    return this._noseDir.set(0, 0, -1)
      .applyQuaternion(this.planeOrientation)
      .multiplyScalar(this.planeSpeed)
  }

  dispose(scene: THREE.Scene): void {
    if (this.placeholderPlane) scene.remove(this.placeholderPlane)
    if (this.planeModel) scene.remove(this.planeModel)
    this.calibrationUI?.remove()
    this.debugHUD?.remove()
    this.reticleEl?.remove()
  }

  update(camera: THREE.PerspectiveCamera, scene: THREE.Scene, delta: number): void {
    if (!this.active) return
    this._camera = camera
    this._ensureMeshes(scene)
    if (this.planeMode) this._updatePlaneMode(camera, delta)
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
      this.euler.x = Math.max(-Math.PI * 0.47, Math.min(Math.PI * 0.47, this.euler.x))
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

  // ── plane mode — exact port of controls.js _updatePlaneMode() ─────────────
  private _updatePlaneMode(camera: THREE.PerspectiveCamera, delta: number): void {
    const plane = this._getActivePlane()
    if (!plane) return

    if (!this.calibrating) {
      // 1. THROTTLE — R/F keys; Shift = 2× boost
      const boost = (this.keys['ShiftLeft'] || this.keys['ShiftRight']) ? 2.0 : 1.0
      if (this.keys['KeyR']) this.throttle = Math.min(1.0, this.throttle + 0.008)
      if (this.keys['KeyF']) this.throttle = Math.max(0.0, this.throttle - 0.006)
      const targetSpeed = (this.minSpeed + this.throttle * (this.maxSpeed - this.minSpeed)) * boost
      this.planeSpeed += (targetSpeed - this.planeSpeed) * this.speedDamping

      // 2. MOUSE → COMBINED PITCH+ROLL as ONE rotation
      let pitchApplied = 0
      let rollApplied = 0
      const rawDX = this.mouseDX
      const rawDY = this.mouseDY

      if (document.pointerLockElement) {
        const dx = this.mouseDX
        const dy = this.mouseDY
        const mag = Math.sqrt(dx * dx + dy * dy)

        if (mag > 0.5) {
          // Combine pitch + roll into ONE rotation — avoids order-dependent asymmetry
          pitchApplied = -dy * this.pitchSpeed   // mouse up → nose up
          rollApplied  = -dx * this.rollSpeed    // mouse right → bank right

          this._tmpAxis.set(pitchApplied, 0, rollApplied).normalize()
          const angle = Math.sqrt(pitchApplied * pitchApplied + rollApplied * rollApplied)
          this._tmpQ.setFromAxisAngle(this._tmpAxis, angle)
          this.planeOrientation.multiply(this._tmpQ)

          if (mag > 3) {
            if (Math.abs(dy) > Math.abs(dx)) {
              this._logDebug(`pitch ${pitchApplied > 0 ? 'UP' : 'DOWN'} ${Math.abs(pitchApplied).toFixed(4)}`)
            } else {
              this._logDebug(`roll ${rollApplied > 0 ? 'LEFT' : 'RIGHT'} ${Math.abs(rollApplied).toFixed(4)}`)
            }
          }
        }
      }

      // Track for reticle
      this._stickX = this.mouseDX * 0.3
      this._stickY = this.mouseDY * 0.3
      this.mouseDX = 0
      this.mouseDY = 0

      this._updateDebugHUD(rawDX, rawDY, pitchApplied, rollApplied)

      // 3. WASD pitch/roll — 0.015 rad/frame, local space
      const wasdRate = 0.015
      if (this.keys['KeyW']) {
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(1, 0, 0),  wasdRate)
        this.planeOrientation.multiply(this._tmpQ)
      }
      if (this.keys['KeyS']) {
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(1, 0, 0), -wasdRate)
        this.planeOrientation.multiply(this._tmpQ)
      }
      if (this.keys['KeyA']) {
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(0, 0, 1),  wasdRate)
        this.planeOrientation.multiply(this._tmpQ)
      }
      if (this.keys['KeyD']) {
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(0, 0, 1), -wasdRate)
        this.planeOrientation.multiply(this._tmpQ)
      }

      // 4. BANK-TO-TURN — local space yaw, NEGATIVE sign, dead zone 0.09
      this._planeRight.set(1, 0, 0).applyQuaternion(this.planeOrientation)
      const bankAmount = Math.asin(THREE.MathUtils.clamp(-this._planeRight.y, -1, 1))
      const effectiveBank = Math.abs(bankAmount) > 0.09 ? bankAmount : 0
      const bankYaw = -effectiveBank * this.bankTurnRate * (this.planeSpeed / this.maxSpeed)

      if (Math.abs(bankYaw) > 0.00001) {
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(0, 1, 0), bankYaw)
        // multiply() = LOCAL space — must NOT be premultiply() (world space couples into pitch)
        this.planeOrientation.multiply(this._tmpQ)
      }

      // 5. AIR ROLL (Q/E — 180° barrel roll)
      if (this.airRolling) {
        const step = Math.sign(this.airRollTarget) * 0.08
        let airRollAngle: number
        if (Math.abs(this.airRollTarget) > Math.abs(step)) {
          this.airRollTarget -= step
          airRollAngle = step
        } else {
          airRollAngle = this.airRollTarget
          this.airRollTarget = 0
          this.airRolling = false
        }
        this._tmpQ.setFromAxisAngle(this._tmpAxis.set(0, 0, 1), airRollAngle)
        this.planeOrientation.multiply(this._tmpQ)
      }

      // 6. NORMALIZE
      this.planeOrientation.normalize()

      // 7. MOVEMENT along nose direction + gravity/lift
      this._noseDir.set(0, 0, -1).applyQuaternion(this.planeOrientation)
      this.planePosition.addScaledVector(this._noseDir, this.planeSpeed)

      this._planeUp.set(0, 1, 0).applyQuaternion(this.planeOrientation)
      const liftUp = Math.max(0, this._planeUp.y)
      const netGravity = this.gravity * (1.0 - liftUp * Math.min(this.planeSpeed / this.minSpeed, 1.0))
      this.planePosition.y -= netGravity

      // Space/Ctrl altitude trim
      if (this.keys['Space'])                                    this.planePosition.y += this.planeSpeed * 0.3
      if (this.keys['ControlLeft'] || this.keys['ControlRight']) this.planePosition.y -= this.planeSpeed * 0.3
    } else {
      this.mouseDX = 0
      this.mouseDY = 0
    }

    // 8. APPLY TO MODEL
    plane.quaternion.copy(this.planeOrientation).multiply(this.modelRotationOffset)
    plane.position.copy(this.planePosition)

    // 9. CAMERA — follows behind/above, blends plane up with world up
    this._fwd.set(0, 0, -1).applyQuaternion(this.planeOrientation)
    this._up.set(0, 1, 0).applyQuaternion(this.planeOrientation)

    this._camTarget.copy(this.planePosition)
      .addScaledVector(this._fwd, -this.cameraOffset.z)
      .addScaledVector(this._up,   this.cameraOffset.y)

    const t = this.cameraTilt
    // camUp = blend of plane up and world up — reuse _up in place
    this._up.multiplyScalar(t).add(this._tmpAxis.set(0, 1 - t, 0)).normalize()
    this._lookTarget.copy(this.planePosition).addScaledVector(this._fwd, 10)
    this._lookM.lookAt(camera.position, this._lookTarget, this._up)
    this._lookQ.setFromRotationMatrix(this._lookM)

    // Frame-rate-independent exponential smoothing — consistent at any fps.
    // Math.exp(-k * delta): k=8 → ~0.12 factor at 60fps; k=12 → ~0.18 at 60fps.
    // Snap threshold eliminates micro-oscillation when nearly at target.
    const posFactor = 1 - Math.exp(-8  * delta)
    const rotFactor = 1 - Math.exp(-12 * delta)

    if (camera.position.distanceToSquared(this._camTarget) < 0.001) {
      camera.position.copy(this._camTarget)
    } else {
      camera.position.lerp(this._camTarget, posFactor)
    }

    if (this._lookQ.dot(camera.quaternion) > 0.9999) {
      camera.quaternion.copy(this._lookQ)
    } else {
      camera.quaternion.slerp(this._lookQ, rotFactor)
    }

    this.euler.setFromQuaternion(camera.quaternion, 'YXZ')

    // Update reticle position
    if (this.reticleEl) {
      this.reticleEl.style.left  = (window.innerWidth  / 2 + this._stickX) + 'px'
      this.reticleEl.style.top   = (window.innerHeight / 2 + this._stickY) + 'px'
      this.reticleEl.style.opacity = '1'
    }

    // Network — client-authoritative position for server body teleport
    const vel = this.getVelocity()
    const q   = this.planeOrientation
    const pos = this.planePosition
    sendMoveAction(vel.x, vel.y, vel.z, 1, q.x, q.y, q.z, q.w, pos.x, pos.y, pos.z)
  }

  // ── toggle plane mode ─────────────────────────────────────────────────────
  private _togglePlaneMode(): void {
    this.planeMode = !this.planeMode
    const plane = this._getActivePlane()
    const cam   = this._camera

    if (this.planeMode) {
      if (cam) this.planePosition.copy(cam.position)
      this.planeEuler.set(0, this.euler.y, 0, 'YXZ')
      this.airRollTarget = 0
      this.airRolling = false
      this.throttle   = 0.5
      this.planeSpeed = this.minSpeed + 0.5 * (this.maxSpeed - this.minSpeed)
      // Init orientation from camera yaw
      this.planeOrientation.setFromEuler(new THREE.Euler(0, this.euler.y, 0, 'YXZ'))
      if (plane) { plane.visible = true; plane.position.copy(this.planePosition) }
      if (this.reticleEl) this.reticleEl.style.display = 'block'
      if (this.debugHUD)  this.debugHUD.style.display  = 'block'
    } else {
      if (cam) {
        cam.position.copy(this.planePosition)
        this.euler.copy(this.planeEuler)
        cam.quaternion.setFromEuler(this.euler)
      }
      if (plane) plane.visible = false
      this.calibrating = false
      if (this.calibrationUI) this.calibrationUI.style.display = 'none'
      if (this.reticleEl) this.reticleEl.style.display = 'none'
      if (this.debugHUD)  this.debugHUD.style.display  = 'none'
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

  private _logDebug(msg: string): void {
    if (!this.debugLog) return
    this._debugLogLines.push(msg)
    if (this._debugLogLines.length > 8) this._debugLogLines.shift()
    this.debugLog.innerHTML = this._debugLogLines.join('<br>')
    this.debugLog.scrollTop = this.debugLog.scrollHeight
  }

  private _updateDebugHUD(mouseX: number, mouseY: number, pitchApplied: number, rollApplied: number): void {
    if (!this.debugHUD || !this.debugLog) return
    this._noseDir.set(0, 0, -1).applyQuaternion(this.planeOrientation)
    this._planeRight.set(1, 0, 0).applyQuaternion(this.planeOrientation)
    const pitch    = Math.asin(THREE.MathUtils.clamp(this._noseDir.y, -1, 1)) * 180 / Math.PI
    const bank     = Math.asin(THREE.MathUtils.clamp(-this._planeRight.y, -1, 1)) * 180 / Math.PI

    if (!this._statsEl) {
      this._statsEl = document.createElement('div')
      this.debugHUD.insertBefore(this._statsEl, this.debugLog)
    }
    this._statsEl.innerHTML = [
      `<span style="color:#a5b4fc">Mouse X:</span> ${mouseX.toFixed(1).padStart(7)}  <span style="color:#a5b4fc">Y:</span> ${mouseY.toFixed(1).padStart(7)}`,
      `<span style="color:#34d399">Pitch:</span>  ${pitchApplied.toFixed(5).padStart(9)}  <span style="color:#fbbf24">Roll:</span> ${rollApplied.toFixed(5).padStart(9)}`,
      `<span style="color:#f87171">Nose°:</span>  ${pitch.toFixed(1).padStart(7)}  <span style="color:#f87171">Bank°:</span> ${bank.toFixed(1).padStart(7)}`,
      `<span style="color:#888">Speed:</span>  ${this.planeSpeed.toFixed(3)}  <span style="color:#888">Thr:</span> ${(this.throttle * 100).toFixed(0)}%`,
    ].join('<br>')
  }

  private _ensureMeshes(scene: THREE.Scene): void {
    if (this.placeholderPlane) return

    // Placeholder geometry
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

    // Load GLB
    const baseUrl = (import.meta as unknown as { env: { BASE_URL: string } }).env.BASE_URL
    new GLTFLoader().load(
      baseUrl + 'models/plane.glb',
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

    // Calibration UI
    const calDiv = document.createElement('div')
    calDiv.style.cssText = `
      position: fixed; top: 14px; right: 14px;
      background: rgba(10,10,20,0.92); border: 2px solid #f59e0b;
      border-radius: 12px; padding: 20px 28px; z-index: 100;
      font-family: monospace; color: #fbbf24; text-align: center;
      display: none; pointer-events: none;
    `
    calDiv.innerHTML = `
      <div style="font-size:1.1rem;font-weight:bold;margin-bottom:8px;">CALIBRATION MODE</div>
      <div style="font-size:0.8rem;color:rgba(255,255,255,0.6);line-height:1.6;">
        Arrows — Y/X axis &bull; U/O — Z axis &bull; C — exit
      </div>
    `
    document.body.appendChild(calDiv)
    this.calibrationUI = calDiv

    // Debug HUD
    const hud = document.createElement('div')
    hud.style.cssText = `
      position: fixed; bottom: 60px; left: 14px;
      background: rgba(10,10,20,0.85); border: 1px solid rgba(255,255,255,0.1);
      border-radius: 8px; padding: 10px 14px; z-index: 25;
      font-family: monospace; font-size: 0.7rem; color: rgba(255,255,255,0.6);
      line-height: 1.6; display: none; min-width: 200px; pointer-events: none;
    `
    document.body.appendChild(hud)
    this.debugHUD = hud

    const logArea = document.createElement('div')
    logArea.style.cssText = `
      max-height: 100px; overflow-y: auto; margin-top: 6px;
      border-top: 1px solid rgba(255,255,255,0.08); padding-top: 4px;
      font-size: 0.65rem; color: rgba(255,255,255,0.4);
    `
    hud.appendChild(logArea)
    this.debugLog = logArea

    // Reticle
    const reticle = document.createElement('div')
    reticle.style.cssText = `
      position: fixed; width: 20px; height: 20px;
      border: 1.5px solid rgba(255,255,255,0.35);
      border-radius: 50%; pointer-events: none; z-index: 16;
      display: none; transform: translate(-50%, -50%);
    `
    const dot = document.createElement('div')
    dot.style.cssText = `
      position: absolute; top: 50%; left: 50%;
      width: 3px; height: 3px;
      background: rgba(255,255,255,0.6);
      border-radius: 50%; transform: translate(-50%, -50%);
    `
    reticle.appendChild(dot)
    document.body.appendChild(reticle)
    this.reticleEl = reticle
  }
}
