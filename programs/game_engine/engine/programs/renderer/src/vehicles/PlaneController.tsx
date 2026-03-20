/**
 * PlaneController — exact port of 3dGraphUniverse/src/controls.js plane mode.
 * Includes: physics, reticle, calibration UI, settings panel.
 * Source: 3dGraphUniverse/src/controls.js (plane-mode branch, commit 2df5f10)
 */
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

// ============================================================================
// Tunable values — exposed to settings UI, mutable at runtime
// ============================================================================

const defaults = {
  maxSpeed: 0.4,
  minSpeed: 0.05,
  speedDamping: 0.02,
  gravity: 0.004,
  pitchSpeed: 0.0012,
  rollSpeed: 0.0015,
  bankTurnRate: 0.015,
  airRollSpeed: 0.08,
  autoLevelRate: 0.02,
  stickInputScale: 0.5,
  stickMaxRadius: 100,
  stickDriftBack: 0.94,
  stickDeadZone: 8,
  rotationAngleScale: 0.25,
  cameraOffsetBack: 16,
  cameraOffsetUp: 5,
  cameraLerpPos: 0.04,
  cameraLerpLook: 0.08,
  cameraTilt: 0.7,
  planeScale: 3.4,
}

const SETTINGS_KEY = 'nexusPlaneSettings'

function loadPersistedSettings(): Record<string, number> {
  const settings = { ...defaults }
  try {
    const saved = localStorage.getItem(SETTINGS_KEY)
    if (saved) {
      const obj = JSON.parse(saved)
      for (const k of Object.keys(defaults)) {
        if (obj[k] !== undefined) (settings as any)[k] = obj[k]
      }
    }
  } catch { /* ignore */ }
  return settings
}

function persistSettings(settings: Record<string, number>): void {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

// ============================================================================
// Mouse input (module-level)
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
// DOM UI helpers
// ============================================================================

function createReticleEl(): HTMLDivElement {
  let el = document.getElementById('plane-reticle') as HTMLDivElement
  if (el) return el

  el = document.createElement('div')
  el.id = 'plane-reticle'
  el.style.cssText = `
    position: fixed; width: 20px; height: 20px;
    border: 1.5px solid rgba(255,255,255,0.35);
    border-radius: 50%;
    pointer-events: none; z-index: 16;
    display: none;
    transform: translate(-50%, -50%);
  `
  const dot = document.createElement('div')
  dot.style.cssText = `
    position: absolute; top: 50%; left: 50%;
    width: 3px; height: 3px;
    background: rgba(255,255,255,0.6);
    border-radius: 50%;
    transform: translate(-50%, -50%);
  `
  el.appendChild(dot)
  document.body.appendChild(el)
  return el
}

function createCalibrationEl(): HTMLDivElement {
  let el = document.getElementById('plane-calibration-ui') as HTMLDivElement
  if (el) return el

  el = document.createElement('div')
  el.id = 'plane-calibration-ui'
  el.style.cssText = `
    position: fixed; top: 14px; right: 14px;
    background: rgba(10,10,20,0.92); border: 2px solid #f59e0b;
    border-radius: 12px; padding: 20px 28px; z-index: 100;
    font-family: monospace; color: #fbbf24; text-align: center;
    display: none; pointer-events: none;
  `
  el.innerHTML = `
    <div style="font-size:1.1rem;font-weight:bold;margin-bottom:8px;">CALIBRATION MODE</div>
    <div style="font-size:0.8rem;color:rgba(255,255,255,0.6);line-height:1.6;">
      Arrow Left/Right — Y axis<br>
      Arrow Up/Down — X axis<br>
      U / O — Z axis (roll)<br>
      C — exit calibration<br><br>
      Check console (F12) for MODEL_OFFSET values
    </div>
  `
  document.body.appendChild(el)
  return el
}

function createSettingsEl(settings: Record<string, number>): HTMLDivElement {
  let el = document.getElementById('plane-settings-panel') as HTMLDivElement
  if (el) return el

  el = document.createElement('div')
  el.id = 'plane-settings-panel'
  el.style.cssText = `
    position: fixed; bottom: 14px; right: 14px; z-index: 25;
  `

  const toggle = document.createElement('button')
  toggle.textContent = 'Plane Settings'
  toggle.style.cssText = `
    background: rgba(40,40,55,0.9); border: 1px solid rgba(255,255,255,0.15);
    border-radius: 8px; padding: 8px 16px; color: rgba(255,255,255,0.7);
    font-family: inherit; font-weight: 500; font-size: 0.82rem; cursor: pointer;
  `

  const form = document.createElement('div')
  form.style.cssText = `
    margin-bottom: 8px; width: 280px; max-height: 60vh; overflow-y: auto;
    background: rgba(20,20,30,0.92); backdrop-filter: blur(12px);
    border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 16px;
    transform: scaleY(0); opacity: 0; pointer-events: none; height: 0;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); transform-origin: bottom right;
  `

  // Prevent key events from moving the plane while adjusting sliders
  form.addEventListener('keydown', (e) => e.stopPropagation())

  const sliderDefs: Array<{ key: string; label: string; min: number; max: number; step: number }> = [
    { key: 'maxSpeed', label: 'Max Speed', min: 0.1, max: 2.0, step: 0.01 },
    { key: 'minSpeed', label: 'Min Speed', min: 0, max: 0.2, step: 0.005 },
    { key: 'pitchSpeed', label: 'Pitch Rate', min: 0.0002, max: 0.005, step: 0.0001 },
    { key: 'rollSpeed', label: 'Roll Rate', min: 0.0002, max: 0.005, step: 0.0001 },
    { key: 'bankTurnRate', label: 'Bank Turn', min: 0.002, max: 0.06, step: 0.001 },
    { key: 'gravity', label: 'Gravity', min: 0, max: 0.02, step: 0.001 },
    { key: 'speedDamping', label: 'Speed Damp', min: 0.005, max: 0.1, step: 0.005 },
    { key: 'cameraLerpPos', label: 'Cam Follow', min: 0.01, max: 0.2, step: 0.005 },
    { key: 'cameraTilt', label: 'Cam Tilt', min: 0, max: 1.0, step: 0.05 },
    { key: 'planeScale', label: 'Plane Size', min: 0.5, max: 10.0, step: 0.1 },
  ]

  const title = document.createElement('div')
  title.style.cssText = 'font-size:0.7rem; text-transform:uppercase; letter-spacing:1px; color:rgba(99,102,241,0.7); margin-bottom:8px;'
  title.textContent = 'Plane Physics'
  form.appendChild(title)

  sliderDefs.forEach(s => {
    const row = document.createElement('div')
    row.style.cssText = 'display:flex; align-items:center; gap:8px; margin-bottom:5px;'

    const label = document.createElement('label')
    label.style.cssText = 'font-size:0.72rem; color:rgba(255,255,255,0.5); width:75px; flex-shrink:0;'
    label.textContent = s.label

    const slider = document.createElement('input')
    slider.type = 'range'
    slider.min = String(s.min)
    slider.max = String(s.max)
    slider.step = String(s.step)
    slider.value = String(settings[s.key])
    slider.style.cssText = 'flex:1; -webkit-appearance:none; height:3px; background:rgba(255,255,255,0.12); border-radius:2px; outline:none;'

    const val = document.createElement('span')
    val.style.cssText = 'font-size:0.68rem; color:rgba(255,255,255,0.35); width:38px; text-align:right; font-family:monospace;'
    val.textContent = Number(settings[s.key]).toFixed(4)

    slider.addEventListener('input', () => {
      const v = parseFloat(slider.value)
      settings[s.key] = v
      val.textContent = v.toFixed(4)
      persistSettings(settings)
    })

    row.appendChild(label)
    row.appendChild(slider)
    row.appendChild(val)
    form.appendChild(row)
  })

  let open = false
  toggle.addEventListener('click', () => {
    open = !open
    if (open) {
      form.style.transform = 'scaleY(1)'
      form.style.opacity = '1'
      form.style.pointerEvents = 'auto'
      form.style.height = 'auto'
      form.style.padding = '16px'
    } else {
      form.style.transform = 'scaleY(0)'
      form.style.opacity = '0'
      form.style.pointerEvents = 'none'
      form.style.height = '0'
      form.style.padding = '0 16px'
    }
  })

  el.appendChild(form)
  el.appendChild(toggle)
  document.body.appendChild(el)
  return el
}

// ============================================================================
// PlaneVehicle
// ============================================================================

const DEG2 = Math.PI / 90

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active = false
  private throttle = 0.5
  private speed = 0
  private airRollTarget = 0
  private airRolling = false
  private orientation = new THREE.Quaternion()
  private position = new THREE.Vector3()
  private planeMesh: THREE.Group | null = null
  private planeGLB: THREE.Group | null = null
  private glbLoaded = false
  private modelRotationOffset = new THREE.Quaternion(0, 1, 0, 0)
  private keys: Record<string, boolean> = {}
  private calibrating = false

  // Virtual stick
  private stickX = 0
  private stickY = 0

  // Tunable settings (mutable — settings UI writes directly to this, persisted to localStorage)
  public settings: Record<string, number> = loadPersistedSettings()

  // DOM elements
  private reticleEl: HTMLDivElement | null = null
  private calibrationEl: HTMLDivElement | null = null
  private settingsEl: HTMLDivElement | null = null

  constructor() {
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', (e) => {
        this.keys[e.code] = true

        // C toggles calibration
        if (e.code === 'KeyC' && this.active) {
          e.preventDefault()
          this.calibrating = !this.calibrating
          if (this.calibrationEl) {
            this.calibrationEl.style.display = this.calibrating ? 'block' : 'none'
          }
          if (this.calibrating) {
            document.exitPointerLock()
          }
          return
        }

        // Calibration key handling
        if (this.calibrating) {
          let rotated = false
          const q = new THREE.Quaternion()
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
        if (this.active && !this.calibrating) {
          if (e.code === 'KeyQ' && !this.airRolling) { this.airRollTarget = Math.PI; this.airRolling = true }
          if (e.code === 'KeyE' && !this.airRolling) { this.airRollTarget = -Math.PI; this.airRolling = true }
        }
      })
      window.addEventListener('keyup', (e) => { this.keys[e.code] = false })
      ensureMouseListener()
    }
  }

  isActive(): boolean { return this.active }

  enter(position: THREE.Vector3, orientation?: THREE.Quaternion): void {
    this.position.copy(position)
    this.position.y = Math.max(this.position.y, 50)
    // Extract yaw only from orientation (spawn level, facing camera direction)
    if (orientation) {
      const euler = new THREE.Euler().setFromQuaternion(orientation, 'YXZ')
      this.orientation.setFromEuler(new THREE.Euler(0, euler.y, 0, 'YXZ'))
    } else {
      this.orientation.identity()
    }
    this.throttle = 0.5
    this.speed = this.settings.minSpeed
    this.stickX = 0
    this.stickY = 0
    this.calibrating = false
    this.active = true

    // Create DOM elements
    this.reticleEl = createReticleEl()
    this.reticleEl.style.display = 'block'
    this.calibrationEl = createCalibrationEl()
    this.settingsEl = createSettingsEl(this.settings)
    this.settingsEl.style.display = 'block'
  }

  exit(): THREE.Vector3 {
    this.active = false
    this.calibrating = false
    if (this.planeMesh) this.planeMesh.visible = false
    if (this.reticleEl) this.reticleEl.style.display = 'none'
    if (this.calibrationEl) this.calibrationEl.style.display = 'none'
    if (this.settingsEl) this.settingsEl.style.display = 'none'
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

    const s = this.settings
    const keys = this.keys

    if (!this.calibrating) {
      // === Throttle ===
      const boost = keys['ShiftLeft'] || keys['ShiftRight'] ? 2.0 : 1.0
      if (keys['KeyW']) this.throttle = Math.min(1.0, this.throttle + 0.008)
      if (keys['KeyS']) this.throttle = Math.max(0.0, this.throttle - 0.006)
      const targetSpeed = (s.minSpeed + this.throttle * (s.maxSpeed - s.minSpeed)) * boost
      this.speed += (targetSpeed - this.speed) * s.speedDamping

      // === Mouse → virtual stick ===
      if (document.pointerLockElement) {
        this.stickX += _mouseDX * s.stickInputScale
        this.stickY += _mouseDY * s.stickInputScale

        const stickDist = Math.sqrt(this.stickX ** 2 + this.stickY ** 2)
        if (stickDist > s.stickMaxRadius) {
          this.stickX *= s.stickMaxRadius / stickDist
          this.stickY *= s.stickMaxRadius / stickDist
        }
      }
      _mouseDX = 0
      _mouseDY = 0

      // Drift stick back to center
      this.stickX *= s.stickDriftBack
      this.stickY *= s.stickDriftBack

      // Dead zone + combined single-axis rotation
      const stickMag = Math.sqrt(this.stickX * this.stickX + this.stickY * this.stickY)

      if (stickMag > s.stickDeadZone) {
        const effectiveMag = stickMag - s.stickDeadZone
        const nx = this.stickX / stickMag
        const ny = this.stickY / stickMag
        const ex = nx * effectiveMag
        const ey = ny * effectiveMag

        // Combined axis — EXACT signs from working 3dGraphUniverse
        const axis = new THREE.Vector3(
          -ey * s.pitchSpeed,
          0,
          -ex * s.rollSpeed
        ).normalize()

        const angle = effectiveMag * Math.max(s.pitchSpeed, s.rollSpeed) * s.rotationAngleScale
        const inputQ = new THREE.Quaternion().setFromAxisAngle(axis, angle)
        this.orientation.multiply(inputQ)
      } else {
        // Inside dead zone — auto-level both bank AND pitch
        const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.orientation)
        const bankAngle = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
        if (Math.abs(bankAngle) > 0.01) {
          const levelQ = new THREE.Quaternion().setFromAxisAngle(
            new THREE.Vector3(0, 0, 1), -bankAngle * s.autoLevelRate
          )
          this.orientation.multiply(levelQ)
        }

        // Auto-level pitch — nose returns to world horizon
        const noseDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
        const pitchAngle = Math.asin(THREE.MathUtils.clamp(noseDir.y, -1, 1))
        if (Math.abs(pitchAngle) > 0.02) {
          const correction = THREE.MathUtils.clamp(pitchAngle * 0.02, -0.003, 0.003)
          const pitchLevelQ = new THREE.Quaternion().setFromAxisAngle(
            new THREE.Vector3(1, 0, 0), correction
          )
          this.orientation.multiply(pitchLevelQ)
        }
      }

      // === A/D direct yaw (rudder — turn without banking) ===
      if (keys['KeyA'] || keys['KeyD']) {
        const yawDir = keys['KeyA'] ? 1 : -1
        const rudderQ = new THREE.Quaternion().setFromAxisAngle(
          new THREE.Vector3(0, 1, 0), yawDir * 0.012
        )
        this.orientation.multiply(rudderQ)
      }

      // === Bank-to-turn ===
      const planeRight = new THREE.Vector3(1, 0, 0).applyQuaternion(this.orientation)
      const bankAmount = Math.asin(THREE.MathUtils.clamp(-planeRight.y, -1, 1))
      const bankYaw = bankAmount * s.bankTurnRate * (this.speed / s.maxSpeed)
      if (Math.abs(bankYaw) > 0.00001) {
        const yawQ = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), bankYaw)
        this.orientation.premultiply(yawQ)
      }

      // === Air roll (Q/E) ===
      if (keys['KeyQ'] && !this.airRolling) { this.airRollTarget = Math.PI; this.airRolling = true }
      if (keys['KeyE'] && !this.airRolling) { this.airRollTarget = -Math.PI; this.airRolling = true }
      if (this.airRolling) {
        const step = Math.sign(this.airRollTarget) * s.airRollSpeed
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

      // === Movement ===
      const noseDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
      this.position.addScaledVector(noseDir, this.speed)

      // Gravity + lift
      const planeUp = new THREE.Vector3(0, 1, 0).applyQuaternion(this.orientation)
      const liftUp = Math.max(0, planeUp.y)
      const netGravity = s.gravity * (1.0 - liftUp * Math.min(this.speed / s.minSpeed, 1.0))
      this.position.y -= netGravity

      // Space/Ctrl trim
      if (keys['Space']) this.position.y += this.speed * 0.3
      if (keys['ControlLeft'] || keys['ControlRight']) this.position.y -= this.speed * 0.3

      // Floor clamp
      if (this.position.y < 2) this.position.y = 2

    } else {
      _mouseDX = 0
      _mouseDY = 0
    }

    // === Apply to model ===
    if (this.glbLoaded && this.planeGLB) {
      this.planeGLB.scale.setScalar(s.planeScale)
    }
    this.planeMesh!.position.copy(this.position)
    this.planeMesh!.quaternion.copy(this.orientation)
    if (this.glbLoaded) {
      this.planeMesh!.quaternion.multiply(this.modelRotationOffset)
    }

    // === Camera ===
    const fwd = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
    const up = new THREE.Vector3(0, 1, 0).applyQuaternion(this.orientation)

    const cameraTarget = this.position.clone()
      .addScaledVector(fwd, -s.cameraOffsetBack)
      .addScaledVector(up, s.cameraOffsetUp)
    camera.position.lerp(cameraTarget, s.cameraLerpPos)

    const t = s.cameraTilt
    const camUp = up.clone().multiplyScalar(t).add(new THREE.Vector3(0, 1 - t, 0)).normalize()
    const lookTarget = this.position.clone().addScaledVector(fwd, 10)
    const lookM = new THREE.Matrix4().lookAt(camera.position, lookTarget, camUp)
    const lookQ = new THREE.Quaternion().setFromRotationMatrix(lookM)
    camera.quaternion.slerp(lookQ, s.cameraLerpLook)

    // === Reticle ===
    if (this.reticleEl) {
      const maxOff = 80
      let rx = this.stickX
      let ry = this.stickY
      const offDist = Math.sqrt(rx * rx + ry * ry)
      if (offDist > maxOff) {
        rx *= maxOff / offDist
        ry *= maxOff / offDist
      }
      this.reticleEl.style.left = (window.innerWidth / 2 + rx) + 'px'
      this.reticleEl.style.top = (window.innerHeight / 2 + ry) + 'px'
      this.reticleEl.style.opacity = '1'
    }

    // === Network ===
    const nose = new THREE.Vector3(0, 0, -1).applyQuaternion(this.orientation)
    sendMoveAction(nose.x * this.speed, nose.y * this.speed, nose.z * this.speed)
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

    new GLTFLoader().load('/models/plane.glb', (gltf) => {
      this.planeGLB = gltf.scene
      this.planeGLB.scale.setScalar(3.4)
      this.glbLoaded = true
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
