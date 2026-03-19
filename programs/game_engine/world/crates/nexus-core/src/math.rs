//! Math primitives: Vec3, Quat, AABB, Mat4.
//!
//! These are the engine's canonical math types. All layers use these.
//! When interfacing with Rapier, convert at the boundary (nexus-simulation handles this).

use std::ops::{Add, Sub, Mul, Neg};

// =============================================================================
// Vec3f32 — used for velocities, forces, normals (game-scale precision)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3f32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f32 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN: Self = Self { x: 0.0, y: -1.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: -1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn magnitude_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag < f32::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).magnitude()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    pub fn clamped_magnitude(self, max: f32) -> Self {
        let mag = self.magnitude();
        if mag > max && mag > f32::EPSILON {
            self * (max / mag)
        } else {
            self
        }
    }
}

impl Add for Vec3f32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Vec3f32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f32> for Vec3f32 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Neg for Vec3f32 {
    type Output = Self;
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl std::ops::AddAssign for Vec3f32 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::MulAssign<f32> for Vec3f32 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

// =============================================================================
// Vec3f64 — used for positions (world-scale precision, large worlds)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3f64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3f64 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Convert to f32 for physics calculations (positions near origin are fine as f32).
    pub fn to_f32(self) -> Vec3f32 {
        Vec3f32::new(self.x as f32, self.y as f32, self.z as f32)
    }

    /// Convert from f32 physics result back to f64 world position.
    pub fn from_f32(v: Vec3f32) -> Self {
        Self::new(v.x as f64, v.y as f64, v.z as f64)
    }
}

impl Add for Vec3f64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Vec3f64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

// =============================================================================
// Quat32 — unit quaternion for orientations
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat32 {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn normalized(self) -> Self {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if mag < f32::EPSILON {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
                w: self.w / mag,
            }
        }
    }

    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        let other = if dot < 0.0 {
            dot = -dot;
            Self::new(-other.x, -other.y, -other.z, -other.w)
        } else {
            other
        };

        // Fall back to lerp for nearly identical orientations
        if dot > 0.9995 {
            return Self::new(
                self.x + (other.x - self.x) * t,
                self.y + (other.y - self.y) * t,
                self.z + (other.z - self.z) * t,
                self.w + (other.w - self.w) * t,
            ).normalized();
        }

        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let s0 = ((1.0 - t) * theta).sin() / sin_theta;
        let s1 = (t * theta).sin() / sin_theta;

        Self::new(
            self.x * s0 + other.x * s1,
            self.y * s0 + other.y * s1,
            self.z * s0 + other.z * s1,
            self.w * s0 + other.w * s1,
        )
    }
}

// =============================================================================
// AABB64 — axis-aligned bounding box (world precision)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb64 {
    pub min: Vec3f64,
    pub max: Vec3f64,
}

impl Aabb64 {
    pub fn new(min: Vec3f64, max: Vec3f64) -> Self {
        Self { min, max }
    }

    pub fn contains_point(&self, point: Vec3f64) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn center(&self) -> Vec3f64 {
        Vec3f64::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    pub fn extents(&self) -> Vec3f64 {
        self.max - self.min
    }

    pub fn longest_axis(&self) -> u8 {
        let e = self.extents();
        if e.x >= e.y && e.x >= e.z { 0 }
        else if e.y >= e.z { 1 }
        else { 2 }
    }

    pub fn split_at_midpoint(&self) -> (Self, Self) {
        let axis = self.longest_axis();
        let center = self.center();
        let mut a = *self;
        let mut b = *self;
        match axis {
            0 => { a.max.x = center.x; b.min.x = center.x; }
            1 => { a.max.y = center.y; b.min.y = center.y; }
            _ => { a.max.z = center.z; b.min.z = center.z; }
        }
        (a, b)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3f32_magnitude() {
        let v = Vec3f32::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vec3f32_normalized() {
        let v = Vec3f32::new(0.0, 5.0, 0.0).normalized();
        assert!((v.magnitude() - 1.0).abs() < 1e-6);
        assert!((v.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn vec3f32_dot_product() {
        let a = Vec3f32::new(1.0, 0.0, 0.0);
        let b = Vec3f32::new(0.0, 1.0, 0.0);
        assert!((a.dot(b)).abs() < f32::EPSILON); // perpendicular
    }

    #[test]
    fn vec3f32_cross_product() {
        let x = Vec3f32::new(1.0, 0.0, 0.0);
        let y = Vec3f32::new(0.0, 1.0, 0.0);
        let z = x.cross(y);
        assert!((z.z - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vec3f32_clamped_magnitude() {
        let v = Vec3f32::new(100.0, 0.0, 0.0);
        let clamped = v.clamped_magnitude(5.0);
        assert!((clamped.magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn vec3f64_distance() {
        let a = Vec3f64::new(0.0, 0.0, 0.0);
        let b = Vec3f64::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn vec3f64_to_f32_roundtrip() {
        let v = Vec3f64::new(1.5, 2.5, 3.5);
        let f32_v = v.to_f32();
        let back = Vec3f64::from_f32(f32_v);
        assert!((v.x - back.x).abs() < 1e-6);
    }

    #[test]
    fn quat_identity() {
        let q = Quat32::IDENTITY;
        assert!((q.w - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn quat_normalize() {
        let q = Quat32::new(1.0, 1.0, 1.0, 1.0).normalized();
        let mag = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_contains_point() {
        let aabb = Aabb64::new(Vec3f64::ZERO, Vec3f64::new(10.0, 10.0, 10.0));
        assert!(aabb.contains_point(Vec3f64::new(5.0, 5.0, 5.0)));
        assert!(!aabb.contains_point(Vec3f64::new(11.0, 5.0, 5.0)));
    }

    #[test]
    fn aabb_split() {
        let aabb = Aabb64::new(Vec3f64::ZERO, Vec3f64::new(100.0, 50.0, 50.0));
        assert_eq!(aabb.longest_axis(), 0); // X is longest
        let (a, b) = aabb.split_at_midpoint();
        assert!((a.max.x - 50.0).abs() < 1e-10);
        assert!((b.min.x - 50.0).abs() < 1e-10);
    }
}
