# ADR-003: Physics Integration Algorithm
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
Every physics tick, we must integrate forces acting on all dynamic bodies to produce new positions and velocities. The integration algorithm determines accuracy, stability, and computational cost. The choice must balance:
- Stability (does the simulation blow up under large time steps or high forces?)
- Accuracy (does it drift from the "true" solution over time?)
- Speed (how many operations per body per tick?)
- Implementability (can we get it right without months of work?)

The game runs at 50 ticks/second (dt = 20ms). Bodies include: player avatars, thrown objects, animals, projectiles, falling debris.

## Options Considered

**Explicit (Forward) Euler**
- `velocity += acceleration * dt`
- `position += velocity * dt`
- Energy grows over time → simulation explodes under springy constraints
- Verdict: Unsuitable

**Semi-Implicit Euler (Symplectic Euler)**
- `velocity += acceleration * dt`
- `position += velocity * dt`  ← uses *new* velocity, not old
- Energy is conserved (symplectic) → doesn't explode
- Error is O(dt) — first-order accuracy
- Speed: 2 multiplies + 2 adds per axis per body = very fast
- Verdict: **Correct choice for game physics**

**Verlet Integration**
- `position += velocity * dt + 0.5 * acceleration * dt²`
- `velocity = (position - prev_position) / dt`
- Second-order accuracy, naturally handles constraints
- Requires storing previous position (extra memory)
- Slightly slower — two positions in memory per body
- Better for position-based systems (cloth, rope)
- Verdict: Good alternative for soft-body objects, not for rigid bodies at scale

**Runge-Kutta 4 (RK4)**
- 4 sub-step evaluations per tick
- Fourth-order accuracy — very accurate
- 4× more compute than Euler
- At 50 ticks/second with thousands of bodies, this is wasteful
- Accuracy gains over Euler are imperceptible at this tick rate
- Verdict: Overkill for game physics

## Decision

**Semi-Implicit Euler (Symplectic Euler) for all rigid body integration.**

Specifically:
1. Accumulate all forces acting on a body this tick
2. Compute acceleration = total_force / mass
3. Update velocity: `v_new = v + a * dt`
4. Update position: `p_new = p + v_new * dt`  ← note: uses new velocity
5. Apply velocity damping: `v_new *= (1 - damping_coefficient * dt)`
6. Clear accumulated forces for next tick

**Exception**: Cloth, ropes, and soft bodies (when added in Phase 3+) use Verlet or Position-Based Dynamics, specified in their own ADR.

## Consequences

- The physics integrator is a simple, deterministic function — same inputs → same outputs always
- Simulation is reproducible across server and client (enables replay)
- Damping coefficient is a per-body-type tuning parameter — store in object type definition
- At 50 ticks/second, dt = 0.02 seconds. This is stable for typical game force magnitudes.
- Sub-stepping (running multiple integration steps per tick) is available as a fallback if fast objects tunnel through geometry — ADR-004 addresses this
