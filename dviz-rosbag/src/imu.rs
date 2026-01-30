//! IMU Message Processing
//!
//! Parses sensor_msgs/Imu messages from ROS bags.

use crate::{Result, RosBagError};

/// Parsed IMU data
#[derive(Debug, Clone, Copy, Default)]
pub struct ImuData {
    /// Frame ID
    pub frame_id_len: usize,
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Orientation quaternion (x, y, z, w)
    pub orientation: [f64; 4],
    /// Angular velocity (x, y, z) in rad/s
    pub angular_velocity: [f64; 3],
    /// Linear acceleration (x, y, z) in m/s^2
    pub linear_acceleration: [f64; 3],
}

/// IMU message processor
pub struct ImuProcessor {
    /// Frame ID of the last processed message
    pub frame_id: String,
    /// Timestamp of the last processed message
    pub timestamp: f64,
}

impl ImuProcessor {
    /// Create a new IMU processor
    pub fn new() -> Self {
        Self {
            frame_id: String::new(),
            timestamp: 0.0,
        }
    }

    /// Parse a sensor_msgs/Imu message from raw bytes
    ///
    /// ROS1 Imu message layout:
    /// - header (seq: 4, stamp: 8, frame_id: 4+len)
    /// - orientation (x,y,z,w: 4*8=32 bytes)
    /// - orientation_covariance (9*8=72 bytes)
    /// - angular_velocity (x,y,z: 3*8=24 bytes)
    /// - angular_velocity_covariance (9*8=72 bytes)
    /// - linear_acceleration (x,y,z: 3*8=24 bytes)
    /// - linear_acceleration_covariance (9*8=72 bytes)
    pub fn parse(&mut self, data: &[u8]) -> Result<ImuData> {
        if data.len() < 100 {
            return Err(RosBagError::ParseError("Data too short for Imu message".into()));
        }

        let mut offset = 0;

        // Parse header
        // seq (4 bytes)
        offset += 4;

        // stamp.sec (4 bytes) + stamp.nsec (4 bytes)
        let sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        self.timestamp = sec as f64 + nsec as f64 * 1e-9;
        offset += 8;

        // frame_id (string: 4 byte length + chars)
        let frame_id_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        self.frame_id = String::from_utf8_lossy(&data[offset..offset + frame_id_len]).to_string();
        offset += frame_id_len;

        // Orientation quaternion (x, y, z, w) - 4 doubles
        let orientation = [
            f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 24..offset + 32].try_into().unwrap()),
        ];
        offset += 32;

        // Skip orientation_covariance (9 doubles = 72 bytes)
        offset += 72;

        // Angular velocity (x, y, z) - 3 doubles
        let angular_velocity = [
            f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap()),
        ];
        offset += 24;

        // Skip angular_velocity_covariance (9 doubles = 72 bytes)
        offset += 72;

        // Linear acceleration (x, y, z) - 3 doubles
        let linear_acceleration = [
            f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap()),
            f64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap()),
        ];

        Ok(ImuData {
            frame_id_len,
            timestamp: self.timestamp,
            orientation,
            angular_velocity,
            linear_acceleration,
        })
    }
}

impl Default for ImuProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imu_processor_new() {
        let proc = ImuProcessor::new();
        assert!(proc.frame_id.is_empty());
    }
}
