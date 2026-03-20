/**
 * PlayerController — first-person WASD movement with pointer lock and stair traversal.
 *
 * Ported from ELEV8-source/components/scene/AvatarController.tsx.
 * Stair system ported from ELEV8-source/lib/staircases.ts.
 *
 * Controls:
 *   WASD    — move (forward/back/strafe)
 *   Shift   — sprint (1.8x)
 *   Mouse   — look (pointer lock)
 *   Click   — lock cursor
 *   ESC     — unlock cursor
 *   Space   — jump (Phase 2)
 *   F       — toggle fly mode (debug)
 *
 * Stair system (from ELEV8):
 *   State machine: FLAT or ON_STAIRS.
 *   When player crosses a staircase's boundary plane while moving in the stair's
 *   direction, state switches to ON_STAIRS. Y position is then a pure linear
 *   interpolation of XZ progress along the stair center line — no physics, no
 *   bouncing on steps. On exit, snaps to landing floor Y.
 *   Collision meshes for the stair geometry are ignored while ON_STAIRS to prevent
 *   clipping against individual step faces.
 *
 * Network integration:
 *   Each frame with movement, sends direction to server via sendMoveAction().
 */
import { useEffect, useRef } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import { PointerLockControls } from '@react-three/drei'
import * as THREE from 'three'
import { sendMoveAction } from '../network/useNetworkState'
import { isAnyVehicleActive } from '../vehicles/VehicleSystem'

// ============================================================================
// Constants
// ============================================================================

const SPEED = 5.0            // units per second
const SPRINT_MULT = 1.8      // shift multiplier
const EYE_HEIGHT = 1.6       // camera Y offset from ground
const WALL_CHECK_DIST = 0.4  // metres — stop before walls
const MAX_DELTA = 0.1        // cap frame delta to prevent teleporting
const STAIR_ENTER_DOT = 0.3  // minimum alignment with stair direction to enter
const STAIR_COOLDOWN = 3     // frames to wait after stair state change
const JUMP_VELOCITY = 6.0    // m/s upward on jump
const GRAVITY = 15.0         // m/s² downward
const ISLAND_RADIUS = 50     // metres — must match Terrain.tsx

// ============================================================================
// Stair System (ported from ELEV8 lib/staircases.ts)
// ============================================================================

export interface StaircaseConfig {
  id: string
  bottomCenter: THREE.Vector3
  topCenter: THREE.Vector3
  bottomFloorY: number
  topFloorY: number
  width: number
  ignoredMeshNames: string[]

  // Precomputed (set by createStaircase)
  bottomPlane: THREE.Plane
  topPlane: THREE.Plane
  stairDirectionXZ: THREE.Vector3
  stairLengthXZ: number
  centerLineXZ: THREE.Line3
}

/**
 * Create a staircase config from placement data.
 * Called when a STAIR building piece is placed — generates the boundary planes
 * and center line needed for the traversal state machine.
 */
export function createStaircase(
  id: string,
  bottomCenter: THREE.Vector3,
  topCenter: THREE.Vector3,
  bottomFloorY: number,
  topFloorY: number,
  width: number,
  ignoredMeshNames: string[] = [],
): StaircaseConfig {
  // Project to XZ plane (Y=0) — stair traversal is purely 2D
  const bottomXZ = new THREE.Vector3(bottomCenter.x, 0, bottomCenter.z)
  const topXZ = new THREE.Vector3(topCenter.x, 0, topCenter.z)

  const stairVectorXZ = new THREE.Vector3().subVectors(topXZ, bottomXZ)
  const stairLengthXZ = stairVectorXZ.length()
  const stairDirectionXZ = stairVectorXZ.clone().normalize()

  // Boundary planes: normal faces AWAY from the staircase
  const bottomPlane = new THREE.Plane().setFromNormalAndCoplanarPoint(
    stairDirectionXZ.clone().negate(), bottomXZ
  )
  const topPlane = new THREE.Plane().setFromNormalAndCoplanarPoint(
    stairDirectionXZ.clone(), topXZ
  )

  const centerLineXZ = new THREE.Line3(bottomXZ, topXZ)

  return {
    id, bottomCenter, topCenter, bottomFloorY, topFloorY, width, ignoredMeshNames,
    bottomPlane, topPlane, stairDirectionXZ, stairLengthXZ, centerLineXZ,
  }
}

// Helpers (from ELEV8)
function crossedPlane(plane: THREE.Plane, oldXZ: THREE.Vector3, newXZ: THREE.Vector3): boolean {
  const distOld = plane.distanceToPoint(oldXZ)
  const distNew = plane.distanceToPoint(newXZ)
  return (distOld > 0 && distNew <= 0) || (distOld <= 0 && distNew > 0)
}

function getIntersectionPointXZ(plane: THREE.Plane, oldXZ: THREE.Vector3, newXZ: THREE.Vector3): THREE.Vector3 | null {
  const line = new THREE.Line3(oldXZ, newXZ)
  const target = new THREE.Vector3()
  return plane.intersectLine(line, target)
}

function isWithinStairWidthXZ(pointXZ: THREE.Vector3, centerLine: THREE.Line3, width: number): boolean {
  const closestPoint = new THREE.Vector3()
  centerLine.closestPointToPoint(pointXZ, false, closestPoint)
  return pointXZ.distanceTo(closestPoint) <= width
}

// ============================================================================
// Global staircase registry
// Populated dynamically when stair building pieces are placed.
// ============================================================================

const _staircases: StaircaseConfig[] = []

/** Register a staircase (called by building system when a STAIR piece is placed). */
export function registerStaircase(config: StaircaseConfig): void {
  _staircases.push(config)
}

/** Unregister a staircase (called when a STAIR piece is destroyed). */
export function unregisterStaircase(id: string): void {
  const idx = _staircases.findIndex(s => s.id === id)
  if (idx !== -1) _staircases.splice(idx, 1)
}

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
const _oldPos = new THREE.Vector3()
const _reqPos = new THREE.Vector3()
const _oldXZ = new THREE.Vector3()
const _reqXZ = new THREE.Vector3()
const _diffXZ = new THREE.Vector3()
const _moveDirXZ = new THREE.Vector3()
const _currentXZ = new THREE.Vector3()
const _boundedXZ = new THREE.Vector3()
const _bottomXZ = new THREE.Vector3()
const _relativeXZ = new THREE.Vector3()
const _closestPoint = new THREE.Vector3()
const _clampDiff = new THREE.Vector3()

// ============================================================================
// Component
// ============================================================================

type StairState = 'FLAT' | 'ON_STAIRS'

export function PlayerController() {
  const { camera, gl, scene } = useThree()
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const controlsRef = useRef<any>(null)

  // Stair state machine (refs — no re-renders)
  const stairStateRef = useRef<{
    state: StairState
    currentStairId: string | null
    cooldownFrames: number
  }>({ state: 'FLAT', currentStairId: null, cooldownFrames: 0 })

  const currentFloorYRef = useRef<number | null>(null)

  // Jump state
  const verticalVelRef = useRef(0)
  const isGroundedRef = useRef(true)

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

    // Yield to any active vehicle (plane, submarine, car, drone, etc.)
    if (isAnyVehicleActive()) return

    const dt = Math.min(delta, MAX_DELTA)

    // Initialize floor Y on first frame
    if (currentFloorYRef.current === null) {
      currentFloorYRef.current = camera.position.y - EYE_HEIGHT
    }

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
    if (!_flyMode) _forward.y = 0
    _forward.normalize()
    _right.crossVectors(_forward, _up).normalize()

    // Save old position and compute requested position
    _oldPos.copy(camera.position)
    _reqPos.copy(camera.position)

    let moveZ = 0
    let moveX = 0

    if (hasInput) {
      _velocity.normalize()
      const dist = SPEED * (KEYS.shift ? SPRINT_MULT : 1.0) * dt
      moveZ = _velocity.z * dist
      moveX = _velocity.x * dist

      _reqPos.addScaledVector(_forward, moveZ)
      _reqPos.addScaledVector(_right, moveX)

      // Send movement to server
      const worldDirX = _forward.x * _velocity.z + _right.x * _velocity.x
      const worldDirY = _flyMode ? (_velocity.y || 0) : 0
      const worldDirZ = _forward.z * _velocity.z + _right.z * _velocity.x
      sendMoveAction(worldDirX, worldDirY, worldDirZ)
    }

    // ================================================================
    // STAIR BOUNDARY PROCESSING (from ELEV8 AvatarController)
    // Runs BEFORE collision checks — determines if we're entering/exiting stairs
    // ================================================================

    if (stairStateRef.current.cooldownFrames > 0) {
      stairStateRef.current.cooldownFrames--
    } else if (!_flyMode && _staircases.length > 0) {
      _oldXZ.set(_oldPos.x, 0, _oldPos.z)
      _reqXZ.set(_reqPos.x, 0, _reqPos.z)
      _diffXZ.subVectors(_reqXZ, _oldXZ)

      if (_diffXZ.lengthSq() > 0.000001) {
        _moveDirXZ.copy(_diffXZ).normalize()

        if (stairStateRef.current.state === 'FLAT') {
          // Check if we're entering any staircase
          for (const stair of _staircases) {
            // Entering from bottom (walking up)
            if (crossedPlane(stair.bottomPlane, _oldXZ, _reqXZ)) {
              const intersect = getIntersectionPointXZ(stair.bottomPlane, _oldXZ, _reqXZ)
              if (intersect && isWithinStairWidthXZ(intersect, stair.centerLineXZ, stair.width)) {
                if (_moveDirXZ.dot(stair.stairDirectionXZ) > STAIR_ENTER_DOT) {
                  stairStateRef.current.state = 'ON_STAIRS'
                  stairStateRef.current.currentStairId = stair.id
                  stairStateRef.current.cooldownFrames = STAIR_COOLDOWN
                  break
                }
              }
            }
            // Entering from top (walking down)
            if (crossedPlane(stair.topPlane, _oldXZ, _reqXZ)) {
              const intersect = getIntersectionPointXZ(stair.topPlane, _oldXZ, _reqXZ)
              if (intersect && isWithinStairWidthXZ(intersect, stair.centerLineXZ, stair.width)) {
                if (_moveDirXZ.dot(stair.stairDirectionXZ) < -STAIR_ENTER_DOT) {
                  stairStateRef.current.state = 'ON_STAIRS'
                  stairStateRef.current.currentStairId = stair.id
                  stairStateRef.current.cooldownFrames = STAIR_COOLDOWN
                  break
                }
              }
            }
          }
        } else {
          // ON_STAIRS — check if exiting
          const stair = _staircases.find(s => s.id === stairStateRef.current.currentStairId)
          if (stair) {
            if (crossedPlane(stair.bottomPlane, _oldXZ, _reqXZ)) {
              stairStateRef.current.state = 'FLAT'
              stairStateRef.current.currentStairId = null
              stairStateRef.current.cooldownFrames = STAIR_COOLDOWN
              currentFloorYRef.current = stair.bottomFloorY
            } else if (crossedPlane(stair.topPlane, _oldXZ, _reqXZ)) {
              stairStateRef.current.state = 'FLAT'
              stairStateRef.current.currentStairId = null
              stairStateRef.current.cooldownFrames = STAIR_COOLDOWN
              currentFloorYRef.current = stair.topFloorY
            }
          } else {
            // Stair was destroyed while we were on it — snap to flat
            stairStateRef.current.state = 'FLAT'
            stairStateRef.current.currentStairId = null
          }
        }
      }
    }

    // ================================================================
    // COLLISION FILTERING (from ELEV8)
    // While on stairs, ignore the stair mesh to prevent step-face clipping
    // ================================================================

    let ignoredMeshNames: string[] = []
    if (stairStateRef.current.state === 'ON_STAIRS') {
      const stair = _staircases.find(s => s.id === stairStateRef.current.currentStairId)
      if (stair) ignoredMeshNames = stair.ignoredMeshNames
    }

    // ================================================================
    // COLLISION-CHECKED MOVEMENT
    // ================================================================

    if (moveZ !== 0 && !_flyMode) {
      _rayDir.copy(_forward).multiplyScalar(moveZ > 0 ? 1 : -1)
      _raycaster.set(camera.position, _rayDir)
      const hits = _raycaster.intersectObjects(scene.children, true)
        .filter(h =>
          !h.object.userData.noCollide &&
          !h.object.userData.isTrigger &&
          !ignoredMeshNames.includes(h.object.name)
        )

      if (!hits.length || hits[0].distance > Math.abs(moveZ) + WALL_CHECK_DIST) {
        camera.position.addScaledVector(_forward, moveZ)
      }
    } else if (moveZ !== 0) {
      camera.position.addScaledVector(_forward, moveZ)
    }

    if (moveX !== 0 && !_flyMode) {
      _rayDir.copy(_right).multiplyScalar(moveX > 0 ? 1 : -1)
      _raycaster.set(camera.position, _rayDir)
      const hits = _raycaster.intersectObjects(scene.children, true)
        .filter(h =>
          !h.object.userData.noCollide &&
          !h.object.userData.isTrigger &&
          !ignoredMeshNames.includes(h.object.name)
        )

      if (!hits.length || hits[0].distance > Math.abs(moveX) + WALL_CHECK_DIST) {
        camera.position.addScaledVector(_right, moveX)
      }
    } else if (moveX !== 0) {
      camera.position.addScaledVector(_right, moveX)
    }

    // ================================================================
    // ISLAND BOUNDARY — prevent walking off the edge
    // ================================================================

    if (!_flyMode) {
      const distFromCenter = Math.sqrt(
        camera.position.x * camera.position.x + camera.position.z * camera.position.z
      )
      if (distFromCenter > ISLAND_RADIUS - 1.0) {
        // Push back toward center
        const pushBack = (distFromCenter - (ISLAND_RADIUS - 1.0)) / distFromCenter
        camera.position.x -= camera.position.x * pushBack
        camera.position.z -= camera.position.z * pushBack
      }
    }

    // ================================================================
    // VERTICAL POSITION (Y) — jump physics + ground collision
    // ================================================================

    if (_flyMode) {
      const dist = SPEED * dt * (KEYS.shift ? SPRINT_MULT : 1.0)
      if (KEYS.space) camera.position.y += dist
      if (KEYS.shift && !KEYS.w && !KEYS.s && !KEYS.a && !KEYS.d) {
        camera.position.y -= dist
      }
    } else if (stairStateRef.current.state === 'ON_STAIRS') {
      // ON_STAIRS: Y is a pure linear function of XZ progress along the stair
      // This is the key insight from ELEV8 — Y is DECOUPLED from look angle
      const stair = _staircases.find(s => s.id === stairStateRef.current.currentStairId)!

      _currentXZ.set(camera.position.x, 0, camera.position.z)

      // Width boundary clamping — prevent sideways clipping off stair edge
      if (!isWithinStairWidthXZ(_currentXZ, stair.centerLineXZ, stair.width)) {
        stair.centerLineXZ.closestPointToPoint(_currentXZ, false, _closestPoint)
        _clampDiff.subVectors(_currentXZ, _closestPoint)
        _clampDiff.normalize().multiplyScalar(stair.width)
        const clampedXZ = _closestPoint.clone().add(_clampDiff)
        camera.position.x = clampedXZ.x
        camera.position.z = clampedXZ.z
      }

      // Mathematical pure progression: Y = lerp(bottomY, topY, progress)
      _boundedXZ.set(camera.position.x, 0, camera.position.z)
      _bottomXZ.set(stair.bottomCenter.x, 0, stair.bottomCenter.z)
      _relativeXZ.subVectors(_boundedXZ, _bottomXZ)

      const distProj = _relativeXZ.dot(stair.stairDirectionXZ)
      const progress = THREE.MathUtils.clamp(distProj / stair.stairLengthXZ, 0, 1)

      // Absolute snap — no physics, no bouncing, just math
      camera.position.y = THREE.MathUtils.lerp(
        stair.bottomFloorY, stair.topFloorY, progress
      ) + EYE_HEIGHT
    } else {
      // FLAT: jump physics + gravity + ground collision
      const floorY = (currentFloorYRef.current || 0)
      const targetY = floorY + EYE_HEIGHT

      // Jump initiation
      if (KEYS.space && isGroundedRef.current) {
        verticalVelRef.current = JUMP_VELOCITY
        isGroundedRef.current = false
      }

      if (!isGroundedRef.current) {
        // In air — apply gravity
        verticalVelRef.current -= GRAVITY * dt
        camera.position.y += verticalVelRef.current * dt

        // Ground collision
        if (camera.position.y <= targetY) {
          camera.position.y = targetY
          verticalVelRef.current = 0
          isGroundedRef.current = true
        }
      } else {
        // On ground — snap to floor height
        camera.position.y = THREE.MathUtils.lerp(camera.position.y, targetY, 0.3)
      }
    }
  })

  // Disable PointerLockControls when a vehicle is active —
  // otherwise it fights with the vehicle's camera control.
  // We disconnect/reconnect the controls manually each frame.
  useFrame(() => {
    const controls = controlsRef.current as any
    if (!controls) return
    if (isAnyVehicleActive()) {
      if (controls.isLocked) controls.unlock()
      controls.enabled = false
    } else {
      controls.enabled = true
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
