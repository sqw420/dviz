//! Point Cloud Processing
//!
//! Parses PointCloud2 messages from ROS bags.

use crate::{Result, RosBagError};

/// A single point with position and optional attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
    pub ring: u16,
}

/// Point cloud processor
///
/// Handles parsing of PointCloud2 messages from raw bytes.
pub struct PointCloudProcessor {
    /// Frame ID of the last processed cloud
    pub frame_id: String,
    /// Timestamp of the last processed cloud
    pub timestamp: f64,
}

impl PointCloudProcessor {
    /// Create a new point cloud processor
    pub fn new() -> Self {
        Self {
            frame_id: String::new(),
            timestamp: 0.0,
        }
    }

    /// Parse a PointCloud2 message from raw bytes
    ///
    /// The data should be the serialized ROS message (excluding the connection header).
    pub fn parse(&mut self, data: &[u8]) -> Result<Vec<Point>> {
        // ROS1 PointCloud2 message layout:
        // - header (variable, with frame_id string)
        // - height (4 bytes)
        // - width (4 bytes)
        // - fields (array)
        // - is_bigendian (1 byte)
        // - point_step (4 bytes)
        // - row_step (4 bytes)
        // - data (byte array)
        // - is_dense (1 byte)

        if data.len() < 32 {
            return Err(RosBagError::ParseError("Data too short for PointCloud2".into()));
        }

        // Parse header first
        let mut offset = 0;

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

        // height (4 bytes)
        let height = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // width (4 bytes)
        let width = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // fields array length
        let num_fields = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // Parse fields to understand the point format
        let mut has_intensity = false;
        let mut x_offset = 0u32;
        let mut y_offset = 0u32;
        let mut z_offset = 0u32;
        let mut intensity_offset = 0u32;

        for _ in 0..num_fields {
            // Field name (string)
            let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
            offset += name_len;

            // Field offset (4 bytes)
            let field_offset = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            offset += 4;

            // Field datatype (1 byte)
            let _datatype = data[offset];
            offset += 1;

            // Field count (4 bytes)
            let _count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            offset += 4;

            match name.as_str() {
                "x" => x_offset = field_offset,
                "y" => y_offset = field_offset,
                "z" => z_offset = field_offset,
                "intensity" => {
                    intensity_offset = field_offset;
                    has_intensity = true;
                }
                _ => {}
            }
        }

        // is_bigendian (1 byte)
        let _is_bigendian = data[offset] != 0;
        offset += 1;

        // point_step (4 bytes)
        let point_step = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // row_step (4 bytes)
        let _row_step = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // data array length (4 bytes)
        let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // point data
        let point_data = &data[offset..offset + data_len];

        // Parse points
        let num_points = (height * width) as usize;
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let base = i * point_step;
            if base + point_step > point_data.len() {
                break;
            }

            let x = f32::from_le_bytes(
                point_data[base + x_offset as usize..base + x_offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            let y = f32::from_le_bytes(
                point_data[base + y_offset as usize..base + y_offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            let z = f32::from_le_bytes(
                point_data[base + z_offset as usize..base + z_offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );

            let intensity = if has_intensity {
                f32::from_le_bytes(
                    point_data[base + intensity_offset as usize..base + intensity_offset as usize + 4]
                        .try_into()
                        .unwrap(),
                )
            } else {
                0.0
            };

            // Skip NaN points
            if x.is_nan() || y.is_nan() || z.is_nan() {
                continue;
            }

            points.push(Point {
                x,
                y,
                z,
                intensity,
                ring: 0,
            });
        }

        log::debug!(
            "Parsed {} points from {} ({}x{})",
            points.len(),
            self.frame_id,
            width,
            height
        );

        Ok(points)
    }
}

impl Default for PointCloudProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_default() {
        let p = Point::default();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
        assert_eq!(p.z, 0.0);
    }

    #[test]
    fn test_processor_new() {
        let proc = PointCloudProcessor::new();
        assert!(proc.frame_id.is_empty());
    }
}
