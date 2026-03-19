//! Stage 3: Physics Step (Rapier Integration)
//!
//! This is where ADR-003 (semi-implicit Euler) and ADR-015 (Rapier) combine.
//! Rapier owns collision detection AND constraint solving.
//! Our responsibility: translating between NEXUS types and Rapier's API.
//!
//! Spec: simulation/MANIFEST.md "Stage 3: Physics Step"

use std::collections::HashMap;

use rapier3d::prelude::*;
use nexus_core::math::{Vec3f32, Vec3f64};
use nexus_core::types::{
    PhysicsBody, BodyCategory, ShapeParams, SimulationEvent, SimulationEventType,
    CollisionData, ObjectId,
};
use nexus_core::config::WorldPhysicsConfig;

/// Mapping between NEXUS ObjectId and Rapier RigidBodyHandle.
struct BodyMapping {
    nexus_to_rapier: HashMap<ObjectId, RigidBodyHandle>,
    rapier_to_nexus: HashMap<RigidBodyHandle, ObjectId>,
}

impl BodyMapping {
    fn new() -> Self {
        Self {
            nexus_to_rapier: HashMap::new(),
            rapier_to_nexus: HashMap::new(),
        }
    }

    fn insert(&mut self, nexus_id: ObjectId, rapier_handle: RigidBodyHandle) {
        self.nexus_to_rapier.insert(nexus_id, rapier_handle);
        self.rapier_to_nexus.insert(rapier_handle, nexus_id);
    }
}

/// Run one physics step via Rapier. Mutates bodies in place. Returns collision events.
pub fn physics_step(
    bodies: &mut Vec<PhysicsBody>,
    config: &WorldPhysicsConfig,
    dt: f32,
) -> Vec<SimulationEvent> {
    if bodies.is_empty() {
        return Vec::new();
    }

    let effective_dt = dt * config.time_scale;

    // --- Build Rapier world from NEXUS bodies ---

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    let mut mapping = BodyMapping::new();

    // Process bodies in sorted order (determinism: sorted by object_id)
    let mut sorted_indices: Vec<usize> = (0..bodies.len()).collect();
    sorted_indices.sort_by_key(|&i| bodies[i].object_id);

    for &idx in &sorted_indices {
        let body = &bodies[idx];

        // 3a. Apply world gravity as force BEFORE syncing to Rapier
        // (Rapier has its own gravity, but we disable it and apply ours for per-world config)

        let rapier_body = match body.category {
            BodyCategory::Dynamic => {
                let mut rb = RigidBodyBuilder::dynamic()
                    .translation(vector![
                        body.position.x as f32,
                        body.position.y as f32,
                        body.position.z as f32
                    ])
                    .rotation(vector![0.0, 0.0, 0.0]) // TODO: convert Quat32 to Rapier rotation
                    .linvel(vector![body.velocity.x, body.velocity.y, body.velocity.z])
                    .angvel(vector![
                        body.angular_velocity.x,
                        body.angular_velocity.y,
                        body.angular_velocity.z
                    ])
                    .additional_mass(body.mass)
                    .ccd_enabled(config.enable_ccd)
                    .build();
                rb
            }
            BodyCategory::Static => {
                RigidBodyBuilder::fixed()
                    .translation(vector![
                        body.position.x as f32,
                        body.position.y as f32,
                        body.position.z as f32
                    ])
                    .build()
            }
            BodyCategory::Kinematic => {
                RigidBodyBuilder::kinematic_position_based()
                    .translation(vector![
                        body.position.x as f32,
                        body.position.y as f32,
                        body.position.z as f32
                    ])
                    .build()
            }
        };

        let handle = rigid_body_set.insert(rapier_body);
        mapping.insert(body.object_id, handle);

        // Create collider
        let collider = match &body.shape {
            ShapeParams::Sphere { radius } => {
                ColliderBuilder::ball(*radius).build()
            }
            ShapeParams::Box { half_extents } => {
                ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z).build()
            }
            ShapeParams::ConvexHull { vertices } => {
                let points: Vec<Point<f32>> = vertices
                    .iter()
                    .map(|v| point![v.x, v.y, v.z])
                    .collect();
                // Fall back to sphere if convex hull fails
                ColliderBuilder::convex_hull(&points)
                    .unwrap_or_else(|| ColliderBuilder::ball(1.0))
                    .build()
            }
        };

        collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
    }

    // Apply forces to dynamic bodies (gravity + accumulated applied_force)
    for &idx in &sorted_indices {
        let body = &bodies[idx];
        if body.category != BodyCategory::Dynamic {
            continue;
        }

        if let Some(&handle) = mapping.nexus_to_rapier.get(&body.object_id) {
            if let Some(rb) = rigid_body_set.get_mut(handle) {
                // World gravity
                let gravity_force = config.gravity_at(body.position, body.mass);
                rb.add_force(
                    vector![gravity_force.x, gravity_force.y, gravity_force.z],
                    true,
                );

                // Player-applied force (from Stage 2)
                if body.applied_force != Vec3f32::ZERO {
                    rb.add_force(
                        vector![body.applied_force.x, body.applied_force.y, body.applied_force.z],
                        true,
                    );
                }

                // Applied torque
                if body.applied_torque != Vec3f32::ZERO {
                    rb.add_torque(
                        vector![body.applied_torque.x, body.applied_torque.y, body.applied_torque.z],
                        true,
                    );
                }
            }
        }

        // Handle kinematic bodies
        if body.category == BodyCategory::Kinematic && body.scripted_velocity != Vec3f32::ZERO {
            if let Some(&handle) = mapping.nexus_to_rapier.get(&body.object_id) {
                if let Some(rb) = rigid_body_set.get_mut(handle) {
                    let next_pos = Isometry::translation(
                        body.position.x as f32 + body.scripted_velocity.x * effective_dt,
                        body.position.y as f32 + body.scripted_velocity.y * effective_dt,
                        body.position.z as f32 + body.scripted_velocity.z * effective_dt,
                    );
                    rb.set_next_kinematic_position(next_pos);
                }
            }
        }
    }

    // --- 3c. Step Rapier (gravity disabled — we apply our own) ---

    let gravity = vector![0.0, 0.0, 0.0]; // Disabled — we handle gravity per-world
    let integration_parameters = IntegrationParameters {
        dt: effective_dt,
        min_ccd_dt: 0.001,
        erp: 0.8,
        damping_ratio: 0.25,
        joint_erp: 1.0,
        joint_damping_ratio: 1.0,
        allowed_linear_error: 0.001,
        max_penetration_correction: 0.2,
        prediction_distance: 0.002,
        num_solver_iterations: NonZeroUsize::new(4).unwrap(),
        num_additional_friction_iterations: 4,
        num_internal_pgs_iterations: 1,
        max_ccd_substeps: 1,
        ..IntegrationParameters::default()
    };

    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    physics_pipeline.step(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_set,
        &mut collider_set,
        &mut impulse_joint_set,
        &mut multibody_joint_set,
        &mut ccd_solver,
        None,
        &physics_hooks,
        &event_handler,
    );

    // --- 3d. Read back from Rapier → NEXUS bodies ---

    for body in bodies.iter_mut() {
        if body.category != BodyCategory::Dynamic {
            continue;
        }

        if let Some(&handle) = mapping.nexus_to_rapier.get(&body.object_id) {
            if let Some(rb) = rigid_body_set.get(handle) {
                let pos = rb.translation();
                body.position = Vec3f64::new(pos.x as f64, pos.y as f64, pos.z as f64);

                let rot = rb.rotation();
                body.orientation = nexus_core::math::Quat32::new(
                    rot.i, rot.j, rot.k, rot.w,
                );

                let vel = rb.linvel();
                body.velocity = Vec3f32::new(vel.x, vel.y, vel.z);

                let angvel = rb.angvel();
                body.angular_velocity = Vec3f32::new(angvel.x, angvel.y, angvel.z);
            }
        }
    }

    // --- 3e. Post-Rapier velocity damping ---

    let damping_factor = 1.0 - config.damping_coefficient * effective_dt;
    let angular_damping_factor = 1.0 - config.angular_damping * effective_dt;

    for body in bodies.iter_mut() {
        if body.category != BodyCategory::Dynamic {
            continue;
        }

        body.velocity *= damping_factor;
        body.angular_velocity *= angular_damping_factor;

        // Clamp velocity to max
        if body.velocity.magnitude() > config.max_velocity {
            body.velocity = body.velocity.normalized() * config.max_velocity;
        }
    }

    // --- 3f. Clear accumulated forces ---

    for body in bodies.iter_mut() {
        if body.category == BodyCategory::Dynamic {
            body.applied_force = Vec3f32::ZERO;
            body.applied_torque = Vec3f32::ZERO;
        }
    }

    // --- 3g. Collect collision events ---

    let mut collision_events = Vec::new();

    narrow_phase.contact_pairs().for_each(|pair| {
        if pair.has_any_active_contact() {
            let body_a_handle = collider_set.get(pair.collider1)
                .and_then(|c| c.parent())
                .and_then(|h| mapping.rapier_to_nexus.get(&h));
            let body_b_handle = collider_set.get(pair.collider2)
                .and_then(|c| c.parent())
                .and_then(|h| mapping.rapier_to_nexus.get(&h));

            if let (Some(&id_a), Some(&id_b)) = (body_a_handle, body_b_handle) {
                // Extract contact data from the first manifold
                if let Some(manifold) = pair.manifolds.first() {
                    let normal = manifold.local_n1;
                    let contact_point = manifold.points.first().map(|p| {
                        p.local_p1
                    });

                    if let Some(cp) = contact_point {
                        let data = CollisionData {
                            contact_point: Vec3f32::new(cp.x, cp.y, cp.z),
                            contact_normal: Vec3f32::new(normal.x, normal.y, normal.z),
                            penetration_depth: manifold.points.first()
                                .map(|p| -p.dist)
                                .unwrap_or(0.0),
                        };

                        // Encode collision data as payload
                        let payload = encode_collision_data(&data);

                        collision_events.push(SimulationEvent {
                            event_type: SimulationEventType::Collision,
                            object_id: id_a.min(id_b), // deterministic ordering
                            other_id: id_a.max(id_b),
                            payload,
                        });
                    }
                }
            }
        }
    });

    // Sort collision events for determinism
    collision_events.sort_by_key(|e| (e.object_id, e.other_id));

    collision_events
}

fn encode_collision_data(data: &CollisionData) -> Vec<u8> {
    let mut payload = Vec::with_capacity(28);
    payload.extend_from_slice(&data.contact_point.x.to_le_bytes());
    payload.extend_from_slice(&data.contact_point.y.to_le_bytes());
    payload.extend_from_slice(&data.contact_point.z.to_le_bytes());
    payload.extend_from_slice(&data.contact_normal.x.to_le_bytes());
    payload.extend_from_slice(&data.contact_normal.y.to_le_bytes());
    payload.extend_from_slice(&data.contact_normal.z.to_le_bytes());
    payload.extend_from_slice(&data.penetration_depth.to_le_bytes());
    payload
}
