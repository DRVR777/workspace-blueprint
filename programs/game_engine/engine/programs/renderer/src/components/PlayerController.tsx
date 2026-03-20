/**
 * PlayerController — first-person WASD movement with pointer lock.
 *
 * Ported from ELEV8-source/components/scene/AvatarController.tsx.
 * Stripped: e-commerce logic, dream mode, stair system (Phase 1).
 * Added: network action sending via sendMoveAction().
 *
 * Controls:
 *   WASD    — move (forward/back/strafe)
 *   Shift   — sprint (1.8x)
 *   Mouse   — look (pointer lock)
 *   Click   — lock cursor
 *   ESC     — unlock cursor
 *   Space   — jump (Phase 1)
 *   F       — toggle fly mode (debug)
 *
 * Network integration:
 *   Each frame with movement, sends the movement direction to the server
 *   via sendMoveAction(). Server applies force, runs physics, broadcasts
 *   updated position. Client uses server position for all OTHER entities.
 *   Local player uses client-side prediction (immediate movement).
 */
import { useEffect, useRef } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import { PointerLockControls } from '@react-three/drei'
import * as THREE from 'three'
import { sendMoveAction } from '../network/useNetworkState'

// ============================================================================
// Constants
// ============================================================================

const SPEED = 5.0            // units per second
const SPRINT_MULT = 1.8      // shift multiplier
const EYE_HEIGHT = 1.6       // camera Y offset from ground
const WALL_CHECK_DIST = 0.4  // metres — stop before walls
const MAX_DELTA = 0.1        // cap frame delta to prevent teleporting

// ============================================================================
// Input state (module-level — no React re-renders)
// ============================================================================

const KEYS = {
  w: false, a: false, s: false, d: false,
  shift: false, space: false,
}

let _flyMode = false
let _pointerLocked = false

if (typeof window !== 'undefined') {
  window.addEventListener('keydown', (e) => {
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

    switch (e.key.toLowerCase()) {
      case 'w': KEYS.w = true; break
      case 'a': KEYS.a = true; break
      case 's': KEYS.s = true; break
      case 'd': KEYS.d = true; break
      case ' ': KEYS.space = true; break
      case 'f': _flyMode = !_flyMode; console.log(`[Player] Fly mode: ${_flyMode ? 'ON' : 'OFF'}`); break
    }
    if (e.shiftKey) KEYS.shift = true
  })

  window.addEventListener('keyup', (e) => {
    switch (e.key.toLowerCase()) {
      case 'w': KEYS.w = false; break
      case 'a': KEYS.a = false; break
      case 's': KEYS.s = false; break
      case 'd': KEYS.d = false; break
      case ' ': KEYS.space = false; break
    }
    if (!e.shiftKey) KEYS.shift = false
  })
}

// ============================================================================
// Hoisted vectors (zero allocation in hot path)
// ============================================================================

const _forward = new THREE.Vector3()
const _right = new THREE.Vector3()
const _velocity = new THREE.Vector3()
const _rayDir = new THREE.Vector3()
const _raycaster = new THREE.Raycaster()
const _up = new THREE.Vector3(0, 1, 0)

// ============================================================================
// Component
// ============================================================================

export function PlayerController() {
  const { camera, gl, scene } = useThree()
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const controlsRef = useRef<any>(null)

  // --- Pointer lock: click to lock, ESC to unlock ---
  useEffect(() => {
    const canvas = gl.domElement
    const handleClick = () => {
      if (document.pointerLockElement !== canvas) {
        controlsRef.current?.lock()
      }
    }
    canvas.addEventListener('click', handleClick)
    return () => canvas.removeEventListener('click', handleClick)
  }, [gl.domElement])

  useEffect(() => {
    const handleLockChange = () => {
      _pointerLocked = document.pointerLockElement === gl.domElement
    }
    document.addEventListener('pointerlockchange', handleLockChange)
    return () => document.removeEventListener('pointerlockchange', handleLockChange)
  }, [gl.domElement])

  // --- Movement loop (runs every frame) ---
  useFrame((_, delta) => {
    if (!_pointerLocked) return

    const dt = Math.min(delta, MAX_DELTA)

    // Build velocity from input
    _velocity.set(0, 0, 0)
    if (KEYS.w) _velocity.z += 1
    if (KEYS.s) _velocity.z -= 1
    if (KEYS.a) _velocity.x -= 1
    if (KEYS.d) _velocity.x += 1

    const hasInput = _velocity.lengthSq() > 0

    if (!hasInput && !_flyMode) return

    // Compute forward and right from camera direction
    camera.getWorldDirection(_forward)
    if (!_flyMode) _forward.y = 0 // flatten to XZ plane
    _forward.normalize()
    _right.crossVectors(_forward, _up).normalize()

    let moveZ = 0
    let moveX = 0

    if (hasInput) {
      _velocity.normalize()
      const dist = SPEED * (KEYS.shift ? SPRINT_MULT : 1.0) * dt
      moveZ = _velocity.z * dist
      moveX = _velocity.x * dist

      // --- Send movement to server ---
      // Direction in world space (what the server needs to apply force)
      const worldDirX = _forward.x * _velocity.z + _right.x * _velocity.x
      const worldDirY = _flyMode ? (_velocity.y || 0) : 0
      const worldDirZ = _forward.z * _velocity.z + _right.z * _velocity.x
      sendMoveAction(worldDirX, worldDirY, worldDirZ)
    }

    // --- Client-side prediction: move camera immediately ---
    // Server will reconcile later if needed (local-simulation handles that)

    // Forward collision check
    if (moveZ !== 0 && !_flyMode) {
      _rayDir.copy(_forward).multiplyScalar(moveZ > 0 ? 1 : -1)
      _raycaster.set(camera.position, _rayDir)
      const hits = _raycaster.intersectObjects(scene.children, true)
        .filter(h => !h.object.userData.noCollide)

      if (!hits.length || hits[0].distance > Math.abs(moveZ) + WALL_CHECK_DIST) {
        camera.position.addScaledVector(_forward, moveZ)
      }
    } else if (moveZ !== 0) {
      camera.position.addScaledVector(_forward, moveZ)
    }

    // Strafe collision check
    if (moveX !== 0 && !_flyMode) {
      _rayDir.copy(_right).multiplyScalar(moveX > 0 ? 1 : -1)
      _raycaster.set(camera.position, _rayDir)
      const hits = _raycaster.intersectObjects(scene.children, true)
        .filter(h => !h.object.userData.noCollide)

      if (!hits.length || hits[0].distance > Math.abs(moveX) + WALL_CHECK_DIST) {
        camera.position.addScaledVector(_right, moveX)
      }
    } else if (moveX !== 0) {
      camera.position.addScaledVector(_right, moveX)
    }

    // --- Vertical movement ---
    if (_flyMode) {
      const dist = SPEED * dt * (KEYS.shift ? SPRINT_MULT : 1.0)
      if (KEYS.space) camera.position.y += dist
      // Shift+no WASD = descend
      if (KEYS.shift && !KEYS.w && !KEYS.s && !KEYS.a && !KEYS.d) {
        camera.position.y -= dist
      }
    } else {
      // Ground lock: smoothly approach eye height above terrain
      camera.position.y = THREE.MathUtils.lerp(camera.position.y, EYE_HEIGHT, 0.3)
    }
  })

  return (
    <PointerLockControls
      ref={controlsRef}
      selector="#_no_auto_lock"
      makeDefault
    />
  )
}
