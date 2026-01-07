//! Transform types for coordinate frame transformations
//!
//! Core types for representing 3D transforms, timestamps, and frame relationships.

use glam::{Affine3A, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// FRAME ID
// ============================================================================

/// Identifier for a coordinate frame
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameId(pub String);

impl FrameId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for FrameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for FrameId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for FrameId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl PartialEq for FrameId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FrameId {}

impl Hash for FrameId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// ============================================================================
// TIMESTAMP
// ============================================================================

/// Timestamp in nanoseconds since epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// Create timestamp from nanoseconds
    pub fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// Create timestamp from seconds (floating point)
    pub fn from_secs_f64(secs: f64) -> Self {
        Self((secs * 1_000_000_000.0) as i64)
    }

    /// Create timestamp from Duration
    pub fn from_duration(duration: Duration) -> Self {
        Self(duration.as_nanos() as i64)
    }

    /// Get current time
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| Self::from_duration(d))
            .unwrap_or(Self(0))
    }

    /// Convert to nanoseconds
    pub fn as_nanos(&self) -> i64 {
        self.0
    }

    /// Convert to seconds (floating point)
    pub fn as_secs_f64(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }

    /// Convert to Duration (returns None if negative)
    pub fn as_duration(&self) -> Option<Duration> {
        if self.0 >= 0 {
            Some(Duration::from_nanos(self.0 as u64))
        } else {
            None
        }
    }

    /// Time difference in seconds
    pub fn diff_secs(&self, other: &Timestamp) -> f64 {
        (self.0 - other.0) as f64 / 1_000_000_000.0
    }
}

impl std::ops::Sub for Timestamp {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        let diff = self.0 - other.0;
        Duration::from_nanos(diff.unsigned_abs())
    }
}

// ============================================================================
// TRANSFORM
// ============================================================================

/// 3D rigid transform (translation + rotation)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// Translation component
    pub translation: Vec3,
    /// Rotation component (quaternion)
    pub rotation: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// Identity transform (no translation or rotation)
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };

    /// Create a new transform
    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        Self { translation, rotation }
    }

    /// Create from translation only
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
        }
    }

    /// Create from rotation only
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
        }
    }

    /// Create from Euler angles (roll, pitch, yaw in radians)
    pub fn from_euler(translation: Vec3, roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            translation,
            rotation: Quat::from_euler(glam::EulerRot::XYZ, roll, pitch, yaw),
        }
    }

    /// Convert to 4x4 matrix
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.translation)
    }

    /// Convert to affine transform
    pub fn to_affine(&self) -> Affine3A {
        Affine3A::from_rotation_translation(self.rotation, self.translation)
    }

    /// Create from 4x4 matrix
    pub fn from_mat4(mat: Mat4) -> Self {
        let (_, rotation, translation) = mat.to_scale_rotation_translation();
        Self { translation, rotation }
    }

    /// Compose two transforms (self * other)
    pub fn mul(&self, other: &Transform) -> Transform {
        Transform {
            translation: self.translation + self.rotation * other.translation,
            rotation: self.rotation * other.rotation,
        }
    }

    /// Inverse transform
    pub fn inverse(&self) -> Transform {
        let inv_rotation = self.rotation.inverse();
        Transform {
            translation: inv_rotation * (-self.translation),
            rotation: inv_rotation,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * point + self.translation
    }

    /// Transform a vector (rotation only, no translation)
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.rotation * vector
    }

    /// Linear interpolation between two transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
        }
    }

    /// Get Euler angles (roll, pitch, yaw) in radians
    pub fn to_euler(&self) -> (f32, f32, f32) {
        self.rotation.to_euler(glam::EulerRot::XYZ)
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, other: Self) -> Transform {
        Transform {
            translation: self.translation + self.rotation * other.translation,
            rotation: self.rotation * other.rotation,
        }
    }
}

impl std::ops::Mul<Vec3> for Transform {
    type Output = Vec3;

    fn mul(self, point: Vec3) -> Vec3 {
        self.transform_point(point)
    }
}

// ============================================================================
// STAMPED TRANSFORM
// ============================================================================

/// Transform with timestamp and frame IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StampedTransform {
    /// The transform
    pub transform: Transform,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Parent frame ID
    pub parent_frame: FrameId,
    /// Child frame ID
    pub child_frame: FrameId,
}

impl StampedTransform {
    /// Create a new stamped transform
    pub fn new(
        transform: Transform,
        timestamp: Timestamp,
        parent_frame: impl Into<FrameId>,
        child_frame: impl Into<FrameId>,
    ) -> Self {
        Self {
            transform,
            timestamp,
            parent_frame: parent_frame.into(),
            child_frame: child_frame.into(),
        }
    }

    /// Create with current timestamp
    pub fn now(
        transform: Transform,
        parent_frame: impl Into<FrameId>,
        child_frame: impl Into<FrameId>,
    ) -> Self {
        Self::new(transform, Timestamp::now(), parent_frame, child_frame)
    }
}

// ============================================================================
// POSE
// ============================================================================

/// Position and orientation in a specific frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    /// Position (x, y, z)
    pub position: Vec3,
    /// Orientation (quaternion)
    pub orientation: Quat,
    /// Reference frame
    pub frame_id: FrameId,
    /// Timestamp
    pub timestamp: Timestamp,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            frame_id: FrameId::default(),
            timestamp: Timestamp::default(),
        }
    }
}

impl Pose {
    /// Create a new pose
    pub fn new(
        position: Vec3,
        orientation: Quat,
        frame_id: impl Into<FrameId>,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            position,
            orientation,
            frame_id: frame_id.into(),
            timestamp,
        }
    }

    /// Create pose with current timestamp
    pub fn now(position: Vec3, orientation: Quat, frame_id: impl Into<FrameId>) -> Self {
        Self::new(position, orientation, frame_id, Timestamp::now())
    }

    /// Create from 2D pose (x, y, theta)
    pub fn from_2d(x: f32, y: f32, theta: f32, frame_id: impl Into<FrameId>) -> Self {
        Self::now(
            Vec3::new(x, y, 0.0),
            Quat::from_rotation_z(theta),
            frame_id,
        )
    }

    /// Convert to Transform
    pub fn to_transform(&self) -> Transform {
        Transform::new(self.position, self.orientation)
    }

    /// Create from Transform
    pub fn from_transform(
        transform: Transform,
        frame_id: impl Into<FrameId>,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            position: transform.translation,
            orientation: transform.rotation,
            frame_id: frame_id.into(),
            timestamp,
        }
    }

    /// Get yaw angle (rotation around Z axis)
    pub fn yaw(&self) -> f32 {
        let (_, _, yaw) = self.orientation.to_euler(glam::EulerRot::XYZ);
        yaw
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_frame_id() {
        let frame = FrameId::new("base_link");
        assert_eq!(frame.as_str(), "base_link");
        assert!(!frame.is_empty());

        let frame2: FrameId = "odom".into();
        assert_eq!(frame2.as_str(), "odom");
    }

    #[test]
    fn test_timestamp() {
        let ts = Timestamp::from_secs_f64(1.5);
        assert_eq!(ts.as_nanos(), 1_500_000_000);
        assert!((ts.as_secs_f64() - 1.5).abs() < 1e-9);

        let ts2 = Timestamp::from_nanos(2_000_000_000);
        let diff = ts2.diff_secs(&ts);
        assert!((diff - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::IDENTITY;
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = t.transform_point(point);
        assert!((transformed - point).length() < 1e-6);
    }

    #[test]
    fn test_transform_translation() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::new(0.0, 0.0, 0.0);
        let transformed = t.transform_point(point);
        assert!((transformed - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
    }

    #[test]
    fn test_transform_rotation() {
        let t = Transform::from_rotation(Quat::from_rotation_z(PI / 2.0));
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = t.transform_point(point);
        assert!((transformed - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn test_transform_inverse() {
        let t = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_z(PI / 4.0),
        );
        let inv = t.inverse();
        let composed = t.mul(&inv);

        assert!(composed.translation.length() < 1e-5);
        assert!((composed.rotation.dot(Quat::IDENTITY) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_transform_composition() {
        let t1 = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t2 = Transform::from_translation(Vec3::new(0.0, 1.0, 0.0));
        let composed = t1.mul(&t2);

        let point = Vec3::ZERO;
        let transformed = composed.transform_point(point);
        assert!((transformed - Vec3::new(1.0, 1.0, 0.0)).length() < 1e-6);
    }

    #[test]
    fn test_transform_lerp() {
        let t1 = Transform::from_translation(Vec3::ZERO);
        let t2 = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        let lerped = t1.lerp(&t2, 0.5);

        assert!((lerped.translation - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
    }

    #[test]
    fn test_pose_yaw() {
        let pose = Pose::from_2d(0.0, 0.0, PI / 2.0, "map");
        assert!((pose.yaw() - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_serde_roundtrip() {
        let transform = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_z(0.5),
        );
        let json = serde_json::to_string(&transform).unwrap();
        let restored: Transform = serde_json::from_str(&json).unwrap();

        assert!((transform.translation - restored.translation).length() < 1e-6);
    }
}
