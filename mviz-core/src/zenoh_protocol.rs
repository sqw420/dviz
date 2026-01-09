//! MViz Zenoh Protocol - Universal message format for any application
//!
//! This protocol allows ANY application to publish visualization data
//! without MViz needing application-specific code.
//!
//! ## Topic Structure
//! `{prefix}/{entity_path}` -> logs to Rerun at `{entity_path}`
//!
//! Example: `mviz/world/vehicle/body` -> logs to `world/vehicle/body`
//!
//! ## Message Format
//! JSON header describing the data type, followed by optional binary payload
//!
//! ## Supported Types (maps to Rerun archetypes)
//! - `points3d` - Point clouds
//! - `boxes3d` - 3D boxes
//! - `arrows3d` - Arrows/vectors
//! - `linestrips3d` - Line strips
//! - `transform3d` - Coordinate transforms
//! - `text` - Text annotations
//! - `scalar` - Time-series scalar values
//! - `image` - Images (binary payload)

use serde::{Deserialize, Serialize};

/// Universal message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvizMessage {
    /// Data type (maps to Rerun archetype)
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Optional timestamp (seconds since start)
    #[serde(default)]
    pub timestamp: Option<f64>,

    /// Type-specific data (JSON for simple types)
    #[serde(default)]
    pub data: serde_json::Value,

    /// For binary data: format description
    #[serde(default)]
    pub format: Option<String>,

    /// For binary data: element count
    #[serde(default)]
    pub count: Option<u32>,
}

/// Points3D data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Points3DData {
    /// XYZ positions (inline JSON or binary follows)
    #[serde(default)]
    pub positions: Option<Vec<[f32; 3]>>,

    /// Optional colors [r,g,b,a] 0-255
    #[serde(default)]
    pub colors: Option<Vec<[u8; 4]>>,

    /// Optional single color for all points
    #[serde(default)]
    pub color: Option<[u8; 4]>,

    /// Point radius
    #[serde(default)]
    pub radius: Option<f32>,

    /// Optional radii per point
    #[serde(default)]
    pub radii: Option<Vec<f32>>,
}

/// Boxes3D data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boxes3DData {
    /// Centers [x, y, z]
    pub centers: Vec<[f32; 3]>,

    /// Sizes [w, h, d] or half-sizes
    pub sizes: Vec<[f32; 3]>,

    /// Use half-sizes instead of full sizes
    #[serde(default)]
    pub half_sizes: bool,

    /// Optional quaternions [x, y, z, w]
    #[serde(default)]
    pub quaternions: Option<Vec<[f32; 4]>>,

    /// Optional colors
    #[serde(default)]
    pub colors: Option<Vec<[u8; 4]>>,

    /// Single color for all boxes
    #[serde(default)]
    pub color: Option<[u8; 4]>,
}

/// Arrows3D data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arrows3DData {
    /// Arrow start points (origins)
    pub origins: Vec<[f32; 3]>,

    /// Arrow direction vectors
    pub vectors: Vec<[f32; 3]>,

    /// Optional colors
    #[serde(default)]
    pub colors: Option<Vec<[u8; 4]>>,

    /// Single color for all arrows
    #[serde(default)]
    pub color: Option<[u8; 4]>,
}

/// LineStrips3D data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineStrips3DData {
    /// List of line strips, each strip is a list of points
    pub strips: Vec<Vec<[f32; 3]>>,

    /// Optional colors per strip
    #[serde(default)]
    pub colors: Option<Vec<[u8; 4]>>,

    /// Single color for all strips
    #[serde(default)]
    pub color: Option<[u8; 4]>,
}

/// Transform3D data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform3DData {
    /// Translation [x, y, z]
    #[serde(default)]
    pub translation: Option<[f32; 3]>,

    /// Rotation as quaternion [x, y, z, w]
    #[serde(default)]
    pub rotation: Option<[f32; 4]>,

    /// Or rotation as 3x3 matrix (row-major)
    #[serde(default)]
    pub rotation_matrix: Option<[[f32; 3]; 3]>,

    /// Or full 4x4 matrix (row-major)
    #[serde(default)]
    pub matrix4x4: Option<[f32; 16]>,
}

/// Text annotation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextData {
    /// The text content
    pub text: String,

    /// Optional 3D position for TextLog
    #[serde(default)]
    pub position: Option<[f32; 3]>,
}

/// Scalar time-series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarData {
    /// The scalar value
    pub value: f64,
}

/// Binary payload formats
pub mod binary_formats {
    /// Points: x,y,z as f32 (12 bytes per point)
    pub const POINTS_XYZ_F32: &str = "xyz_f32";

    /// Points: x,y,z,r,g,b,a as f32,f32,f32,u8,u8,u8,u8 (16 bytes per point)
    pub const POINTS_XYZRGBA: &str = "xyzrgba";

    /// Image: raw RGB bytes
    pub const IMAGE_RGB8: &str = "rgb8";

    /// Image: raw RGBA bytes
    pub const IMAGE_RGBA8: &str = "rgba8";
}

/// Parse a Zenoh message into header and optional binary payload
pub fn parse_message(bytes: &[u8]) -> Option<(MvizMessage, Option<&[u8]>)> {
    // Find newline separator (if present, binary data follows)
    if let Some(newline_pos) = bytes.iter().position(|&b| b == b'\n') {
        let header_str = std::str::from_utf8(&bytes[..newline_pos]).ok()?;
        let header: MvizMessage = serde_json::from_str(header_str).ok()?;
        let binary = if newline_pos + 1 < bytes.len() {
            Some(&bytes[newline_pos + 1..])
        } else {
            None
        };
        Some((header, binary))
    } else {
        // No binary data, entire message is JSON
        let json_str = std::str::from_utf8(bytes).ok()?;
        let header: MvizMessage = serde_json::from_str(json_str).ok()?;
        Some((header, None))
    }
}

/// Serialize a message with optional binary payload
pub fn serialize_message(header: &MvizMessage, binary: Option<&[u8]>) -> Vec<u8> {
    let mut result = serde_json::to_string(header).unwrap_or_default().into_bytes();
    if let Some(data) = binary {
        result.push(b'\n');
        result.extend_from_slice(data);
    }
    result
}
