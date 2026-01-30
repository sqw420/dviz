//! Marker types for visualization primitives
//!
//! Types for representing 3D markers (shapes, lines, text) similar to ROS visualization_msgs.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use super::{Color, FrameId, Timestamp};

// ============================================================================
// MARKER TYPE
// ============================================================================

/// Type of marker to visualize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MarkerType {
    /// Arrow pointing from start to end
    #[default]
    Arrow,
    /// Cube/box
    Cube,
    /// Sphere
    Sphere,
    /// Cylinder
    Cylinder,
    /// Line strip (connected line segments)
    LineStrip,
    /// Line list (pairs of points form individual lines)
    LineList,
    /// Cube list (multiple cubes at given positions)
    CubeList,
    /// Sphere list (multiple spheres at given positions)
    SphereList,
    /// Point cloud (individual points)
    Points,
    /// 3D text
    Text,
    /// Mesh resource (external file)
    MeshResource,
    /// Triangle list (triplets of points form triangles)
    TriangleList,
}

impl MarkerType {
    /// Check if this marker type uses the points field
    pub fn uses_points(&self) -> bool {
        matches!(
            self,
            MarkerType::LineStrip
                | MarkerType::LineList
                | MarkerType::CubeList
                | MarkerType::SphereList
                | MarkerType::Points
                | MarkerType::TriangleList
        )
    }

    /// Check if this marker type uses a mesh resource
    pub fn uses_mesh(&self) -> bool {
        matches!(self, MarkerType::MeshResource)
    }

    /// Check if this marker type uses the text field
    pub fn uses_text(&self) -> bool {
        matches!(self, MarkerType::Text)
    }
}

// ============================================================================
// MARKER ACTION
// ============================================================================

/// Action to perform on marker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MarkerAction {
    /// Add or modify marker
    #[default]
    Add,
    /// Modify existing marker (same as Add)
    Modify,
    /// Delete specific marker
    Delete,
    /// Delete all markers in namespace
    DeleteAll,
}

// ============================================================================
// MARKER SCALE
// ============================================================================

/// Scale for marker dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MarkerScale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for MarkerScale {
    fn default() -> Self {
        Self::uniform(1.0)
    }
}

impl MarkerScale {
    /// Create scale with same value for all dimensions
    pub fn uniform(scale: f32) -> Self {
        Self {
            x: scale,
            y: scale,
            z: scale,
        }
    }

    /// Create scale with individual dimensions
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Convert to Vec3
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl From<f32> for MarkerScale {
    fn from(scale: f32) -> Self {
        Self::uniform(scale)
    }
}

impl From<Vec3> for MarkerScale {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<(f32, f32, f32)> for MarkerScale {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::new(x, y, z)
    }
}

// ============================================================================
// MARKER
// ============================================================================

/// A visualization marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    /// Namespace for grouping markers
    pub ns: String,
    /// Unique ID within namespace
    pub id: i32,
    /// Type of marker
    pub marker_type: MarkerType,
    /// Action to perform
    pub action: MarkerAction,
    /// Position
    pub position: Vec3,
    /// Orientation
    pub orientation: Quat,
    /// Scale
    pub scale: MarkerScale,
    /// Primary color
    pub color: Color,
    /// Lifetime in seconds (0 = forever)
    pub lifetime_secs: f32,
    /// Reference frame
    pub frame_id: FrameId,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Points for line/point markers
    pub points: Vec<Vec3>,
    /// Per-point colors (optional)
    pub colors: Vec<Color>,
    /// Text content (for Text markers)
    pub text: String,
    /// Mesh resource path (for MeshResource markers)
    pub mesh_resource: String,
    /// Whether to use mesh embedded materials
    pub mesh_use_embedded_materials: bool,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            ns: String::new(),
            id: 0,
            marker_type: MarkerType::default(),
            action: MarkerAction::default(),
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            scale: MarkerScale::default(),
            color: Color::WHITE,
            lifetime_secs: 0.0,
            frame_id: FrameId::default(),
            timestamp: Timestamp::default(),
            points: Vec::new(),
            colors: Vec::new(),
            text: String::new(),
            mesh_resource: String::new(),
            mesh_use_embedded_materials: false,
        }
    }
}

impl Marker {
    /// Create a new marker with given namespace and ID
    pub fn new(ns: impl Into<String>, id: i32) -> Self {
        Self {
            ns: ns.into(),
            id,
            ..Default::default()
        }
    }

    /// Create an arrow marker
    pub fn arrow(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::Arrow)
    }

    /// Create a cube marker
    pub fn cube(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::Cube)
    }

    /// Create a sphere marker
    pub fn sphere(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::Sphere)
    }

    /// Create a cylinder marker
    pub fn cylinder(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::Cylinder)
    }

    /// Create a line strip marker
    pub fn line_strip(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::LineStrip)
    }

    /// Create a line list marker
    pub fn line_list(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::LineList)
    }

    /// Create a points marker
    pub fn points(ns: impl Into<String>, id: i32) -> MarkerBuilder {
        MarkerBuilder::new(ns, id).marker_type(MarkerType::Points)
    }

    /// Create a text marker
    pub fn text(ns: impl Into<String>, id: i32, content: impl Into<String>) -> MarkerBuilder {
        MarkerBuilder::new(ns, id)
            .marker_type(MarkerType::Text)
            .text(content)
    }

    /// Create a delete action for this marker
    pub fn delete(ns: impl Into<String>, id: i32) -> Self {
        Self {
            ns: ns.into(),
            id,
            action: MarkerAction::Delete,
            ..Default::default()
        }
    }

    /// Create a delete-all action for a namespace
    pub fn delete_all(ns: impl Into<String>) -> Self {
        Self {
            ns: ns.into(),
            action: MarkerAction::DeleteAll,
            ..Default::default()
        }
    }

    /// Check if this marker should be deleted
    pub fn is_delete(&self) -> bool {
        matches!(self.action, MarkerAction::Delete | MarkerAction::DeleteAll)
    }

    /// Check if this marker has expired given the current time
    pub fn is_expired(&self, now: &Timestamp) -> bool {
        if self.lifetime_secs <= 0.0 {
            return false; // Never expires
        }
        let age = now.diff_secs(&self.timestamp);
        age > self.lifetime_secs as f64
    }

    /// Get unique key for this marker (namespace + id)
    pub fn key(&self) -> (String, i32) {
        (self.ns.clone(), self.id)
    }
}

// ============================================================================
// MARKER BUILDER
// ============================================================================

/// Builder for creating markers
#[derive(Debug, Clone)]
pub struct MarkerBuilder {
    marker: Marker,
}

impl MarkerBuilder {
    /// Create a new marker builder
    pub fn new(ns: impl Into<String>, id: i32) -> Self {
        Self {
            marker: Marker::new(ns, id),
        }
    }

    /// Set marker type
    pub fn marker_type(mut self, marker_type: MarkerType) -> Self {
        self.marker.marker_type = marker_type;
        self
    }

    /// Set position
    pub fn position(mut self, position: Vec3) -> Self {
        self.marker.position = position;
        self
    }

    /// Set position from components
    pub fn xyz(mut self, x: f32, y: f32, z: f32) -> Self {
        self.marker.position = Vec3::new(x, y, z);
        self
    }

    /// Set orientation
    pub fn orientation(mut self, orientation: Quat) -> Self {
        self.marker.orientation = orientation;
        self
    }

    /// Set orientation from Euler angles (roll, pitch, yaw)
    pub fn euler(mut self, roll: f32, pitch: f32, yaw: f32) -> Self {
        self.marker.orientation = Quat::from_euler(glam::EulerRot::XYZ, roll, pitch, yaw);
        self
    }

    /// Set scale
    pub fn scale(mut self, scale: impl Into<MarkerScale>) -> Self {
        self.marker.scale = scale.into();
        self
    }

    /// Set uniform scale
    pub fn uniform_scale(mut self, scale: f32) -> Self {
        self.marker.scale = MarkerScale::uniform(scale);
        self
    }

    /// Set color
    pub fn color(mut self, color: Color) -> Self {
        self.marker.color = color;
        self
    }

    /// Set color from RGBA components (0-255)
    pub fn rgba(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.marker.color = Color::rgba(r, g, b, a);
        self
    }

    /// Set color from RGB components (0-255), full opacity
    pub fn rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.marker.color = Color::rgb(r, g, b);
        self
    }

    /// Set lifetime in seconds
    pub fn lifetime(mut self, secs: f32) -> Self {
        self.marker.lifetime_secs = secs;
        self
    }

    /// Set frame ID
    pub fn frame_id(mut self, frame_id: impl Into<FrameId>) -> Self {
        self.marker.frame_id = frame_id.into();
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.marker.timestamp = timestamp;
        self
    }

    /// Set timestamp to now
    pub fn now(mut self) -> Self {
        self.marker.timestamp = Timestamp::now();
        self
    }

    /// Set points (for line/point markers)
    pub fn points(mut self, points: Vec<Vec3>) -> Self {
        self.marker.points = points;
        self
    }

    /// Add a single point
    pub fn point(mut self, point: Vec3) -> Self {
        self.marker.points.push(point);
        self
    }

    /// Set per-point colors
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.marker.colors = colors;
        self
    }

    /// Set text content
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.marker.text = text.into();
        self
    }

    /// Set mesh resource path
    pub fn mesh_resource(mut self, path: impl Into<String>) -> Self {
        self.marker.mesh_resource = path.into();
        self
    }

    /// Build the marker
    pub fn build(self) -> Marker {
        self.marker
    }
}

impl From<MarkerBuilder> for Marker {
    fn from(builder: MarkerBuilder) -> Self {
        builder.build()
    }
}

// ============================================================================
// MARKER ARRAY
// ============================================================================

/// Collection of markers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerArray {
    pub markers: Vec<Marker>,
}

impl MarkerArray {
    /// Create empty marker array
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
        }
    }

    /// Create from vector of markers
    pub fn from_markers(markers: Vec<Marker>) -> Self {
        Self { markers }
    }

    /// Add a marker
    pub fn push(&mut self, marker: impl Into<Marker>) {
        self.markers.push(marker.into());
    }

    /// Add multiple markers
    pub fn extend(&mut self, markers: impl IntoIterator<Item = Marker>) {
        self.markers.extend(markers);
    }

    /// Number of markers
    pub fn len(&self) -> usize {
        self.markers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.markers.is_empty()
    }

    /// Clear all markers
    pub fn clear(&mut self) {
        self.markers.clear();
    }

    /// Iterate over markers
    pub fn iter(&self) -> impl Iterator<Item = &Marker> {
        self.markers.iter()
    }

    /// Iterate mutably over markers
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Marker> {
        self.markers.iter_mut()
    }

    /// Filter out expired markers
    pub fn remove_expired(&mut self, now: &Timestamp) {
        self.markers.retain(|m| !m.is_expired(now));
    }
}

impl IntoIterator for MarkerArray {
    type Item = Marker;
    type IntoIter = std::vec::IntoIter<Marker>;

    fn into_iter(self) -> Self::IntoIter {
        self.markers.into_iter()
    }
}

impl<'a> IntoIterator for &'a MarkerArray {
    type Item = &'a Marker;
    type IntoIter = std::slice::Iter<'a, Marker>;

    fn into_iter(self) -> Self::IntoIter {
        self.markers.iter()
    }
}

impl FromIterator<Marker> for MarkerArray {
    fn from_iter<I: IntoIterator<Item = Marker>>(iter: I) -> Self {
        Self {
            markers: iter.into_iter().collect(),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_type_uses_points() {
        assert!(!MarkerType::Arrow.uses_points());
        assert!(!MarkerType::Cube.uses_points());
        assert!(MarkerType::LineStrip.uses_points());
        assert!(MarkerType::LineList.uses_points());
        assert!(MarkerType::Points.uses_points());
        assert!(MarkerType::TriangleList.uses_points());
    }

    #[test]
    fn test_marker_scale() {
        let scale = MarkerScale::uniform(2.0);
        assert_eq!(scale.x, 2.0);
        assert_eq!(scale.y, 2.0);
        assert_eq!(scale.z, 2.0);

        let scale2 = MarkerScale::new(1.0, 2.0, 3.0);
        let vec = scale2.to_vec3();
        assert_eq!(vec, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_marker_builder() {
        let marker = Marker::cube("test", 1)
            .position(Vec3::new(1.0, 2.0, 3.0))
            .uniform_scale(0.5)
            .rgb(255, 0, 0)
            .frame_id("world")
            .build();

        assert_eq!(marker.ns, "test");
        assert_eq!(marker.id, 1);
        assert_eq!(marker.marker_type, MarkerType::Cube);
        assert_eq!(marker.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(marker.scale.x, 0.5);
        assert_eq!(marker.color.r, 255);
        assert_eq!(marker.frame_id.as_str(), "world");
    }

    #[test]
    fn test_marker_line_strip() {
        let marker = Marker::line_strip("path", 0)
            .point(Vec3::new(0.0, 0.0, 0.0))
            .point(Vec3::new(1.0, 0.0, 0.0))
            .point(Vec3::new(1.0, 1.0, 0.0))
            .scale(0.1)
            .build();

        assert_eq!(marker.marker_type, MarkerType::LineStrip);
        assert_eq!(marker.points.len(), 3);
    }

    #[test]
    fn test_marker_text() {
        let marker = Marker::text("labels", 0, "Hello World")
            .xyz(0.0, 0.0, 1.0)
            .scale(0.2)
            .build();

        assert_eq!(marker.marker_type, MarkerType::Text);
        assert_eq!(marker.text, "Hello World");
    }

    #[test]
    fn test_marker_delete() {
        let delete = Marker::delete("test", 5);
        assert!(delete.is_delete());
        assert_eq!(delete.action, MarkerAction::Delete);

        let delete_all = Marker::delete_all("test");
        assert!(delete_all.is_delete());
        assert_eq!(delete_all.action, MarkerAction::DeleteAll);
    }

    #[test]
    fn test_marker_array() {
        let mut array = MarkerArray::new();
        array.push(Marker::cube("test", 0).build());
        array.push(Marker::sphere("test", 1).build());

        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());

        let collected: Vec<_> = array.iter().map(|m| m.id).collect();
        assert_eq!(collected, vec![0, 1]);
    }

    #[test]
    fn test_marker_key() {
        let marker = Marker::new("namespace", 42);
        let key = marker.key();
        assert_eq!(key, ("namespace".to_string(), 42));
    }

    #[test]
    fn test_serde_roundtrip() {
        let marker = Marker::arrow("test", 1)
            .position(Vec3::new(1.0, 2.0, 3.0))
            .scale(0.5)
            .build();

        let json = serde_json::to_string(&marker).unwrap();
        let restored: Marker = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.ns, marker.ns);
        assert_eq!(restored.id, marker.id);
        assert_eq!(restored.marker_type, marker.marker_type);
    }
}
