//! TF Transform Buffer
//!
//! Manages coordinate frame transforms from tf/tf2 messages.

use std::collections::HashMap;
use crate::{Result, RosBagError};

/// A 3D transform with translation and rotation
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Translation (x, y, z)
    pub translation: [f64; 3],
    /// Rotation quaternion (x, y, z, w)
    pub rotation: [f64; 4],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
        }
    }
}

impl Transform {
    /// Create an identity transform
    pub fn identity() -> Self {
        Self::default()
    }

    /// Create a transform from translation only
    pub fn from_translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            translation: [x, y, z],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Compose two transforms: self * other
    pub fn compose(&self, other: &Transform) -> Transform {
        // Rotate other's translation by self's rotation, then add self's translation
        let rotated = self.rotate_vector(other.translation);
        let translation = [
            self.translation[0] + rotated[0],
            self.translation[1] + rotated[1],
            self.translation[2] + rotated[2],
        ];

        // Multiply quaternions: self.rotation * other.rotation
        let rotation = self.multiply_quaternion(&other.rotation);

        Transform {
            translation,
            rotation,
        }
    }

    /// Rotate a vector by this transform's rotation
    fn rotate_vector(&self, v: [f64; 3]) -> [f64; 3] {
        let q = self.rotation;
        // v' = q * v * q^-1 (using quaternion-vector multiplication)
        let qx = q[0];
        let qy = q[1];
        let qz = q[2];
        let qw = q[3];

        let vx = v[0];
        let vy = v[1];
        let vz = v[2];

        // Optimized rotation formula
        let tx = 2.0 * (qy * vz - qz * vy);
        let ty = 2.0 * (qz * vx - qx * vz);
        let tz = 2.0 * (qx * vy - qy * vx);

        [
            vx + qw * tx + qy * tz - qz * ty,
            vy + qw * ty + qz * tx - qx * tz,
            vz + qw * tz + qx * ty - qy * tx,
        ]
    }

    /// Multiply two quaternions
    fn multiply_quaternion(&self, other: &[f64; 4]) -> [f64; 4] {
        let (ax, ay, az, aw) = (self.rotation[0], self.rotation[1], self.rotation[2], self.rotation[3]);
        let (bx, by, bz, bw) = (other[0], other[1], other[2], other[3]);

        [
            aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz,
        ]
    }

    /// Get the inverse transform
    pub fn inverse(&self) -> Transform {
        // Inverse rotation is conjugate for unit quaternion
        let inv_rotation = [-self.rotation[0], -self.rotation[1], -self.rotation[2], self.rotation[3]];

        // Inverse translation is -R^(-1) * t
        let inv = Transform {
            translation: [0.0, 0.0, 0.0],
            rotation: inv_rotation,
        };
        let neg_t = [-self.translation[0], -self.translation[1], -self.translation[2]];
        let inv_translation = inv.rotate_vector(neg_t);

        Transform {
            translation: inv_translation,
            rotation: inv_rotation,
        }
    }
}

/// Stamped transform with frame information
#[derive(Debug, Clone)]
pub struct StampedTransform {
    /// Transform
    pub transform: Transform,
    /// Parent frame ID
    pub parent_frame: String,
    /// Child frame ID
    pub child_frame: String,
    /// Timestamp (seconds)
    pub timestamp: f64,
}

/// TF Buffer for storing and looking up transforms
pub struct TfBuffer {
    /// Transform storage: child_frame -> (parent_frame, transform, timestamp)
    transforms: HashMap<String, StampedTransform>,
    /// Static transforms (never expire)
    static_transforms: HashMap<String, StampedTransform>,
}

impl TfBuffer {
    /// Create a new TF buffer
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            static_transforms: HashMap::new(),
        }
    }

    /// Add a transform to the buffer
    pub fn set_transform(&mut self, stamped: StampedTransform, is_static: bool) {
        let key = stamped.child_frame.clone();
        if is_static {
            self.static_transforms.insert(key, stamped);
        } else {
            self.transforms.insert(key, stamped);
        }
    }

    /// Look up transform from source_frame to target_frame
    pub fn lookup_transform(
        &self,
        target_frame: &str,
        source_frame: &str,
        _time: f64,
    ) -> Result<Transform> {
        // Normalize frame IDs (remove leading slash if present)
        let target = target_frame.trim_start_matches('/');
        let source = source_frame.trim_start_matches('/');

        if target == source {
            return Ok(Transform::identity());
        }

        // Simple case: direct transform exists
        if let Some(tf) = self.get_transform(source) {
            if tf.parent_frame.trim_start_matches('/') == target {
                return Ok(tf.transform.clone());
            }
        }

        // Try inverse: target -> source
        if let Some(tf) = self.get_transform(target) {
            if tf.parent_frame.trim_start_matches('/') == source {
                return Ok(tf.transform.inverse());
            }
        }

        // Chain lookup through common parent
        // Find path from source to root
        let source_chain = self.get_chain_to_root(source);
        let target_chain = self.get_chain_to_root(target);

        // Find common ancestor
        for (i, src_frame) in source_chain.iter().enumerate() {
            for (j, tgt_frame) in target_chain.iter().enumerate() {
                if src_frame == tgt_frame {
                    // Found common ancestor, compose transforms
                    let mut result = Transform::identity();

                    // Go up from source to common ancestor
                    for k in 0..i {
                        if let Some(tf) = self.get_transform(&source_chain[k]) {
                            result = tf.transform.compose(&result);
                        }
                    }

                    // Go down from common ancestor to target (inverse direction)
                    for k in (0..j).rev() {
                        if let Some(tf) = self.get_transform(&target_chain[k]) {
                            result = result.compose(&tf.transform.inverse());
                        }
                    }

                    return Ok(result);
                }
            }
        }

        Err(RosBagError::TransformError(
            source_frame.to_string(),
            target_frame.to_string(),
        ))
    }

    /// Get transform for a frame (checks both dynamic and static)
    fn get_transform(&self, child_frame: &str) -> Option<&StampedTransform> {
        let frame = child_frame.trim_start_matches('/');
        self.transforms
            .get(frame)
            .or_else(|| self.static_transforms.get(frame))
    }

    /// Get chain of frames from given frame to root
    fn get_chain_to_root(&self, frame: &str) -> Vec<String> {
        let mut chain = vec![frame.trim_start_matches('/').to_string()];
        let mut current = frame.trim_start_matches('/').to_string();

        while let Some(tf) = self.get_transform(&current) {
            let parent = tf.parent_frame.trim_start_matches('/').to_string();
            if chain.contains(&parent) {
                break; // Cycle detection
            }
            chain.push(parent.clone());
            current = parent;
        }

        chain
    }

    /// Process a TF message from raw bytes
    pub fn process_tf_message(&mut self, data: &[u8]) -> Result<()> {
        // TFMessage format:
        // - transforms (array of TransformStamped)

        if data.len() < 4 {
            return Err(RosBagError::ParseError("TF message too short".into()));
        }

        let mut offset = 0;

        // Number of transforms
        let num_transforms = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        for _ in 0..num_transforms {
            if offset + 8 >= data.len() {
                break;
            }

            // Header
            // seq (4 bytes)
            offset += 4;

            // stamp.sec (4 bytes) + stamp.nsec (4 bytes)
            let sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            let nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
            let timestamp = sec as f64 + nsec as f64 * 1e-9;
            offset += 8;

            // frame_id (string)
            let frame_id_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            let parent_frame = String::from_utf8_lossy(&data[offset..offset + frame_id_len]).to_string();
            offset += frame_id_len;

            // child_frame_id (string)
            let child_frame_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            let child_frame = String::from_utf8_lossy(&data[offset..offset + child_frame_len]).to_string();
            offset += child_frame_len;

            // Transform
            // translation (3 * f64 = 24 bytes)
            let tx = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            let ty = f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
            let tz = f64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap());
            offset += 24;

            // rotation (4 * f64 = 32 bytes)
            let rx = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            let ry = f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
            let rz = f64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap());
            let rw = f64::from_le_bytes(data[offset + 24..offset + 32].try_into().unwrap());
            offset += 32;

            let stamped = StampedTransform {
                transform: Transform {
                    translation: [tx, ty, tz],
                    rotation: [rx, ry, rz, rw],
                },
                parent_frame,
                child_frame,
                timestamp,
            };

            self.set_transform(stamped, false);
        }

        Ok(())
    }

    /// Get all known frame IDs
    pub fn all_frames(&self) -> Vec<String> {
        let mut frames: Vec<String> = self
            .transforms
            .keys()
            .chain(self.static_transforms.keys())
            .cloned()
            .collect();
        frames.sort();
        frames.dedup();
        frames
    }

    /// Clear all transforms
    pub fn clear(&mut self) {
        self.transforms.clear();
        // Keep static transforms
    }
}

impl Default for TfBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let tf = Transform::identity();
        assert_eq!(tf.translation, [0.0, 0.0, 0.0]);
        assert_eq!(tf.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_transform_inverse() {
        let tf = Transform::from_translation(1.0, 2.0, 3.0);
        let inv = tf.inverse();
        let composed = tf.compose(&inv);

        // Should be close to identity
        assert!((composed.translation[0]).abs() < 1e-10);
        assert!((composed.translation[1]).abs() < 1e-10);
        assert!((composed.translation[2]).abs() < 1e-10);
    }

    #[test]
    fn test_tf_buffer_same_frame() {
        let buffer = TfBuffer::new();
        let result = buffer.lookup_transform("base_link", "base_link", 0.0);
        assert!(result.is_ok());
    }
}
