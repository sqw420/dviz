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
//! - `log` - System log entries from nodes

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

// ============================================================================
// System Log Types
// ============================================================================

/// Log level for system messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    /// Debug information
    Debug,
    /// General information
    #[default]
    Info,
    /// Warning messages
    Warn,
    /// Error messages
    Error,
}

impl LogLevel {
    /// Parse log level from string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" => Self::Debug,
            "INFO" => Self::Info,
            "WARN" | "WARNING" => Self::Warn,
            "ERROR" | "ERR" => Self::Error,
            _ => Self::Info,
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    /// Get color for UI display [r, g, b, a]
    pub fn color(&self) -> [u8; 4] {
        match self {
            Self::Debug => [128, 128, 128, 255], // Gray
            Self::Info => [100, 180, 255, 255],  // Blue
            Self::Warn => [255, 200, 50, 255],   // Yellow/Orange
            Self::Error => [255, 80, 80, 255],   // Red
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// System log entry from a dora node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Log message content
    pub message: String,
    /// Source node ID (e.g., "bicycle_model", "simple_planner")
    pub node_id: String,
    /// Timestamp in seconds since dataflow start
    #[serde(default)]
    pub timestamp: f64,
    /// Optional metadata key-value pairs
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>, node_id: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            node_id: node_id.into(),
            timestamp: 0.0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create with timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Format as display string for UI
    pub fn format_display(&self) -> String {
        format!(
            "[{:.2}s] [{}] [{}] {}",
            self.timestamp,
            self.level.as_str(),
            self.node_id,
            self.message
        )
    }
}

/// Log data payload in MvizMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogData {
    /// Log level
    pub level: String,
    /// Log message
    pub message: String,
    /// Source node ID
    pub node_id: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

// ============================================================================
// Node Definition Types (for dataflow graph inspection)
// ============================================================================

/// Node input port definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInputDef {
    /// Input port name
    pub name: String,
    /// Source in format "node_id/output_name"
    pub source: String,
}

/// Node output port definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutputDef {
    /// Output port name
    pub name: String,
    /// Destination node IDs that receive this output
    #[serde(default)]
    pub destinations: Vec<String>,
}

/// Complete node definition from dataflow YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Node ID
    pub id: String,
    /// Input ports
    #[serde(default)]
    pub inputs: Vec<NodeInputDef>,
    /// Output ports
    #[serde(default)]
    pub outputs: Vec<NodeOutputDef>,
    /// Node operator path (Python/Rust)
    #[serde(default)]
    pub operator: Option<String>,
}

/// Full dataflow definition with all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataflowDefinition {
    /// Dataflow name
    #[serde(default)]
    pub name: String,
    /// All nodes in the dataflow
    pub nodes: Vec<NodeDefinition>,
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
