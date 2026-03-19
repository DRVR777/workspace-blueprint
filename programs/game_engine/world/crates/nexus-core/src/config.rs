//! Per-world physics configuration.
//!
//! Maps to WORLD_PHYSICS_CONFIG in simulation MANIFEST.md.
//! Each world can override these parameters independently.

use crate::math::{Vec3f32, Vec3f64, Aabb64};

/// Gravity computation mode for a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GravityMode {
    /// Standard directional gravity (default: Y-down).
    Directional = 0,
    /// Gravity pulls toward a center point (walk on all sides of a sphere/cube).
    Spherical = 1,
    /// No gravity. Momentum-only movement.
    Zero = 2,
    /// Multiple zones with different gravity settings.
    ZoneBased = 3,
}

/// A spatial region with its own gravity override.
#[derive(Debug, Clone)]
pub struct GravityZone {
    pub bounds: Aabb64,
    pub gravity: Vec3f32,
    pub mode: GravityMode,
    pub center: Vec3f64,
    pub priority: u8,
    pub blend_distance: f32,
}

/// Complete physics configuration for a world.
/// Read by `run_tick` each tick — never hardcoded.
#[derive(Debug, Clone)]
pub struct WorldPhysicsConfig {
    pub gravity: Vec3f32,
    pub gravity_mode: GravityMode,
    pub gravity_center: Vec3f64,
    pub damping_coefficient: f32,
    pub angular_damping: f32,
    pub time_scale: f32,
    pub max_velocity: f32,
    pub enable_ccd: bool,
    pub gravity_zones: Vec<GravityZone>,
}

impl Default for WorldPhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3f32::new(0.0, -9.8, 0.0),
            gravity_mode: GravityMode::Directional,
            gravity_center: Vec3f64::ZERO,
            damping_coefficient: 0.01,
            angular_damping: 0.05,
            time_scale: 1.0,
            max_velocity: 300.0,
            enable_ccd: true,
            gravity_zones: Vec::new(),
        }
    }
}

impl WorldPhysicsConfig {
    /// Compute the effective gravity force on a body at a given position.
    pub fn gravity_at(&self, position: Vec3f64, mass: f32) -> Vec3f32 {
        match self.gravity_mode {
            GravityMode::Directional => self.gravity * mass,
            GravityMode::Spherical => {
                let dir = (self.gravity_center - position).to_f32().normalized();
                let magnitude = self.gravity.magnitude();
                dir * magnitude * mass
            }
            GravityMode::Zero => Vec3f32::ZERO,
            GravityMode::ZoneBased => {
                // Find highest-priority zone containing the position
                let mut best_zone: Option<&GravityZone> = None;
                for zone in &self.gravity_zones {
                    if zone.bounds.contains_point(position) {
                        match best_zone {
                            Some(current) if zone.priority > current.priority => {
                                best_zone = Some(zone);
                            }
                            None => {
                                best_zone = Some(zone);
                            }
                            _ => {}
                        }
                    }
                }

                match best_zone {
                    Some(zone) => match zone.mode {
                        GravityMode::Spherical => {
                            let dir = (zone.center - position).to_f32().normalized();
                            let magnitude = zone.gravity.magnitude();
                            dir * magnitude * mass
                        }
                        _ => zone.gravity * mass,
                    },
                    // Fall back to world default if no zone matches
                    None => self.gravity * mass,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gravity_is_y_down() {
        let config = WorldPhysicsConfig::default();
        assert_eq!(config.gravity_mode, GravityMode::Directional);
        assert!(config.gravity.y < 0.0);
    }

    #[test]
    fn directional_gravity_force() {
        let config = WorldPhysicsConfig::default();
        let force = config.gravity_at(Vec3f64::ZERO, 10.0);
        assert!((force.y - (-98.0)).abs() < 0.01); // 9.8 * 10
    }

    #[test]
    fn zero_gravity_force() {
        let config = WorldPhysicsConfig {
            gravity_mode: GravityMode::Zero,
            ..Default::default()
        };
        let force = config.gravity_at(Vec3f64::ZERO, 100.0);
        assert_eq!(force, Vec3f32::ZERO);
    }

    #[test]
    fn spherical_gravity_pulls_toward_center() {
        let config = WorldPhysicsConfig {
            gravity_mode: GravityMode::Spherical,
            gravity_center: Vec3f64::ZERO,
            gravity: Vec3f32::new(0.0, -9.8, 0.0), // magnitude used
            ..Default::default()
        };
        // Body above center → force should point downward (toward center)
        let force = config.gravity_at(Vec3f64::new(0.0, 100.0, 0.0), 1.0);
        assert!(force.y < 0.0);
        // Body to the right → force should point left (toward center)
        let force = config.gravity_at(Vec3f64::new(100.0, 0.0, 0.0), 1.0);
        assert!(force.x < 0.0);
    }
}
