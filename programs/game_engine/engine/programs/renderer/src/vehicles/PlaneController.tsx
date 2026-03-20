/**
 * PlaneController — exact port of 3dGraphUniverse/src/controls.js FlyControls.
 *
 * WASD to move, mouse look, Space/Ctrl for up/down, Shift to boost.
 */
import * as THREE from 'three'
import type { VehicleController } from './VehicleSystem'
import { sendMoveAction } from '../network/useNetworkState'

export class PlaneVehicle implements VehicleController {
  readonly type = 'plane'

  private active = false
  private euler = new THREE.Euler(0, 0, 0, 'YXZ')
  private position = new THREE.Vector3()
  private keys: Record<string, boolean> = {}
  private mouseDX = 0
  private mouseDY = 0

  moveSpeed = 0.3
  lookSpeed = 0.002
  boostMultiplier = 3

  constructor() {
    if (typeof window === 'undefined') return
    window.addEventListener('keydown', e => { this.keys[e.code] = true })
    window.addEventListener('keyup', e => { this.keys[e.code] = false })
    window.addEventListener('mousemove', e => {
      if (!document.pointerLockElement) return
      this.mouseDX += e.movementX
      this.mouseDY += e.movementY
    })
  }

  isActive(): boolean { return this.active }

  enter(position: THREE.Vector3, orientation?: THREE.Quaternion): void {
    this.position.copy(position)
    this.euler.setFromQuaternion(orientation ?? new THREE.Quaternion())
    this.active = true
  }

  exit(): THREE.Vector3 {
    this.active = false
    return this.position.clone()
  }

  getPosition(): THREE.Vector3 { return this.position.clone() }

  getVelocity(): THREE.Vector3 {
    return new THREE.Vector3(0, 0, -1)
      .applyEuler(this.euler)
      .multiplyScalar(this.moveSpeed)
  }

  dispose(_scene: THREE.Scene): void {}

  update(camera: THREE.PerspectiveCamera, _scene: THREE.Scene, _delta: number): void {
    if (!this.active) return

    // Mouse look
    if (document.pointerLockElement) {
      this.euler.y -= this.mouseDX * this.lookSpeed
      this.euler.x -= this.mouseDY * this.lookSpeed
      this.euler.x = Math.max(-Math.PI * 0.47, Math.min(Math.PI * 0.47, this.euler.x))
      camera.quaternion.setFromEuler(this.euler)
    }
    this.mouseDX = 0
    this.mouseDY = 0

    // Movement
    const speed = this.moveSpeed *
      ((this.keys['ShiftLeft'] || this.keys['ShiftRight']) ? this.boostMultiplier : 1)

    const forward = new THREE.Vector3(0, 0, -1).applyQuaternion(camera.quaternion)
    const right = new THREE.Vector3(1, 0, 0).applyQuaternion(camera.quaternion)

    if (this.keys['KeyW']) camera.position.addScaledVector(forward, speed)
    if (this.keys['KeyS']) camera.position.addScaledVector(forward, -speed)
    if (this.keys['KeyA']) camera.position.addScaledVector(right, -speed)
    if (this.keys['KeyD']) camera.position.addScaledVector(right, speed)
    if (this.keys['Space']) camera.position.y += speed
    if (this.keys['ControlLeft'] || this.keys['ControlRight']) camera.position.y -= speed

    this.position.copy(camera.position)

    const vel = this.getVelocity()
    sendMoveAction(vel.x, vel.y, vel.z)
  }
}
