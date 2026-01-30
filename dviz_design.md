# Architecture & Design: Robotics Visualizer with Makepad + Rerun

This document provides detailed architecture and design specifications for building an RViz-like robotics visualization tool using Makepad and Rerun.

---

## 1. System Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Application Layer                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Panels    │  │   Tools     │  │   Views     │  │  Configuration      │ │
│  │  (Makepad)  │  │  (Makepad)  │  │ Controllers │  │  (serde + YAML)     │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Core Framework                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │  Display Manager │  │ Transform System │  │  Selection Manager         │  │
│  │  (Plugin Host)   │  │ (Frame Tree)     │  │  (Picking, Highlighting)   │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Visualization Layer                                │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    Rerun Integration Bridge                          │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │    │
│  │  │ Points3D │  │Transform │  │  Mesh3D  │  │ Custom Archetypes    │ │    │
│  │  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ (Markers, Scans)     │ │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────────┘ │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                      │                                       │
│                                      ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     Rerun RecordingStream                            │    │
│  │                     (Data Logging & Storage)                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Rendering Layer                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │              Rerun Viewer (Embedded or Standalone)                   │    │
│  │              - 3D Space View                                         │    │
│  │              - 2D Image Views                                        │    │
│  │              - Timeline                                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Data Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Data Source │────▶│   Display    │────▶│    Rerun     │────▶│    Rerun     │
│  (ROS2/File) │     │   Plugin     │     │   Bridge     │     │    Viewer    │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │  Transform   │
                     │   System     │
                     └──────────────┘
```

---

## 2. Crate/Module Structure

### 2.1 Actual Workspace Organization (Implemented)

```
dviz/
├── Cargo.toml                    # Workspace root
├── dviz-core/                    # Core types and protocols [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types/
│   │   │   ├── mod.rs
│   │   │   ├── transform.rs      # Transform, Pose, Timestamp
│   │   │   ├── point_cloud.rs    # PointCloud, Color, ColorMode
│   │   │   └── marker.rs         # Marker types
│   │   ├── config.rs             # AppConfig, DisplayConfig
│   │   ├── display.rs            # Display trait
│   │   └── zenoh_protocol.rs     # Universal vis protocol [NEW]
│   └── Cargo.toml
│
├── dviz-transform/               # Transform system [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── frame_tree.rs         # Frame hierarchy
│   │   └── transform_buffer.rs   # Time-indexed transforms
│   └── Cargo.toml
│
├── dviz-rerun-bridge/            # Rerun integration [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── bridge.rs             # RerunBridge, RerunConfig
│   │   ├── adapters.rs           # Point cloud/marker adapters
│   │   ├── core_adapters.rs      # Transform adapters
│   │   └── simulation.rs         # SensorSimulator
│   └── Cargo.toml
│
├── dviz-displays/                # Display plugins [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── base.rs               # BaseDisplay helper
│   │   ├── grid.rs               # GridDisplay ✓
│   │   ├── axes.rs               # AxesDisplay ✓
│   │   ├── tf.rs                 # TfDisplay ✓
│   │   ├── point_cloud.rs        # PointCloudDisplay ✓
│   │   ├── marker.rs             # MarkerDisplay ✓
│   │   ├── robot_model.rs        # RobotModelDisplay ✓
│   │   └── laser_scan.rs         # LaserScanDisplay ✓
│   └── Cargo.toml
│
├── dviz-urdf/                    # URDF parsing [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── parser.rs             # parse_urdf(), RobotDescription
│   │   ├── robot.rs              # Robot, Link, Joint structs
│   │   └── mesh_loader.rs        # STL/DAE/OBJ loaders
│   └── Cargo.toml
│
├── dviz-rosbag/                  # ROS bag playback [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── player.rs             # RosBagPlayer ✓
│   │   ├── messages.rs           # MessageType enum ✓
│   │   ├── pointcloud.rs         # PointCloud2 parser ✓
│   │   ├── tf.rs                 # TfBuffer ✓
│   │   ├── imu.rs                # ImuProcessor ✓
│   │   └── gps.rs                # GpsProcessor, NMEA ✓
│   └── Cargo.toml
│
├── dviz-widgets/                 # Makepad UI widgets [IMPLEMENTED]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── theme.rs              # Theme colors and styles
│   │   ├── displays_panel.rs     # DisplaysPanel ✓
│   │   ├── properties_panel.rs   # PropertiesPanel ✓
│   │   ├── toolbar.rs            # Toolbar ✓
│   │   ├── log_panel.rs          # LogPanel ✓
│   │   ├── node_detail_panel.rs  # NodeDetailPanel ✓
│   │   ├── dataflow_graph.rs     # DataflowGraphWidget ✓
│   │   ├── sensor_panel.rs       # SensorGroup
│   │   └── control_bar.rs        # ControlBar
│   └── Cargo.toml
│
├── dviz-shell/                   # Main application [IMPLEMENTED]
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs
│   │   ├── app.rs                # App struct, UI layout
│   │   ├── dora_receiver.rs      # Dora dataflow integration
│   │   └── zenoh_receiver.rs     # Zenoh universal receiver [NEW]
│   ├── resources/
│   │   └── icons/viz.svg         # App icon
│   └── Cargo.toml
│
└── docs/                         # Documentation
    ├── dviz_plan.md
    └── dviz_design.md
```

### 2.1.1 Original Design (Reference)

```
robotics-viz/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── rv-core/                  # Core types and traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types/            # Core data types
│   │   │   │   ├── mod.rs
│   │   │   │   ├── transform.rs
│   │   │   │   ├── point_cloud.rs
│   │   │   │   ├── marker.rs
│   │   │   │   └── image.rs
│   │   │   ├── traits/           # Plugin traits
│   │   │   │   ├── mod.rs
│   │   │   │   ├── display.rs
│   │   │   │   ├── tool.rs
│   │   │   │   └── view_controller.rs
│   │   │   └── config/           # Configuration types
│   │   │       ├── mod.rs
│   │   │       └── schema.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-transform/             # Transform/TF system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── frame_tree.rs
│   │   │   ├── transform_buffer.rs
│   │   │   └── interpolation.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-rerun/                 # Rerun integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── bridge.rs
│   │   │   ├── adapters/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── point_cloud.rs
│   │   │   │   ├── transform.rs
│   │   │   │   ├── marker.rs
│   │   │   │   └── mesh.rs
│   │   │   └── viewer.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-displays/              # Built-in displays
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── grid.rs
│   │   │   ├── axes.rs
│   │   │   ├── point_cloud.rs
│   │   │   ├── tf.rs
│   │   │   ├── marker.rs
│   │   │   ├── robot_model.rs
│   │   │   ├── laser_scan.rs
│   │   │   ├── image.rs
│   │   │   └── path.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-tools/                 # Built-in tools
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── move_camera.rs
│   │   │   ├── select.rs
│   │   │   ├── measure.rs
│   │   │   └── publish_point.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-views/                 # View controllers
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── orbit.rs
│   │   │   ├── fps.rs
│   │   │   ├── top_down.rs
│   │   │   └── follower.rs
│   │   └── Cargo.toml
│   │
│   ├── rv-urdf/                  # URDF parsing and robot model
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs
│   │   │   ├── robot.rs
│   │   │   └── mesh_loader.rs
│   │   └── Cargo.toml
│   │
│   └── rv-ui/                    # Makepad UI application
│       ├── src/
│       │   ├── lib.rs
│       │   ├── app.rs
│       │   ├── widgets/
│       │   │   ├── mod.rs
│       │   │   ├── display_panel.rs
│       │   │   ├── properties_panel.rs
│       │   │   ├── view_panel.rs
│       │   │   ├── timeline.rs
│       │   │   └── toolbar.rs
│       │   └── viewport.rs
│       └── Cargo.toml
│
└── apps/
    └── robotics-viz/             # Main application binary
        ├── src/
        │   └── main.rs
        └── Cargo.toml
```

### 2.2 Dependency Graph

```
                         ┌────────────┐
                         │  rv-core   │
                         │ (types,    │
                         │  traits)   │
                         └────────────┘
                               │
           ┌───────────────────┼───────────────────┐
           │                   │                   │
           ▼                   ▼                   ▼
    ┌────────────┐      ┌────────────┐      ┌────────────┐
    │rv-transform│      │  rv-rerun  │      │  rv-urdf   │
    │            │◀────▶│            │      │            │
    └────────────┘      └────────────┘      └────────────┘
           │                   │                   │
           └─────────┬─────────┴─────────┬─────────┘
                     │                   │
                     ▼                   ▼
              ┌────────────┐      ┌────────────┐
              │rv-displays │      │ rv-tools   │
              │            │      │            │
              └────────────┘      └────────────┘
                     │                   │
                     └─────────┬─────────┘
                               │
                               ▼
                        ┌────────────┐
                        │  rv-views  │
                        └────────────┘
                               │
                               ▼
                        ┌────────────┐
                        │   rv-ui    │
                        │ (Makepad)  │
                        └────────────┘
```

---

## 3. Core Data Types

### 3.1 Transform Types (`rv-core/src/types/transform.rs`)

```rust
use glam::{Vec3, Quat, Mat4, Affine3A};
use std::time::Duration;

/// Unique identifier for a coordinate frame
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameId(pub String);

impl FrameId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Timestamp for time-indexed data
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    /// Nanoseconds since epoch
    pub nanos: i64,
}

impl Timestamp {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Self {
            nanos: duration.as_nanos() as i64,
        }
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self {
            nanos: (secs * 1e9) as i64,
        }
    }
}

/// 3D rigid transform (rotation + translation)
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };

    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        Self { translation, rotation }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
        }
    }

    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.translation)
    }

    pub fn to_affine(&self) -> Affine3A {
        Affine3A::from_rotation_translation(self.rotation, self.translation)
    }

    /// Compose two transforms: self * other
    pub fn mul(&self, other: &Transform) -> Transform {
        Transform {
            translation: self.translation + self.rotation * other.translation,
            rotation: self.rotation * other.rotation,
        }
    }

    /// Inverse transform
    pub fn inverse(&self) -> Transform {
        let inv_rot = self.rotation.inverse();
        Transform {
            translation: inv_rot * (-self.translation),
            rotation: inv_rot,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * point + self.translation
    }

    /// Transform a vector (rotation only)
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.rotation * vector
    }

    /// Linear interpolation between transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
        }
    }
}

/// Stamped transform with frame information
#[derive(Debug, Clone)]
pub struct StampedTransform {
    pub transform: Transform,
    pub timestamp: Timestamp,
    pub parent_frame: FrameId,
    pub child_frame: FrameId,
}

/// Pose (position + orientation) in a specific frame
#[derive(Debug, Clone)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: Quat,
    pub frame_id: FrameId,
    pub timestamp: Timestamp,
}

impl Pose {
    pub fn to_transform(&self) -> Transform {
        Transform::new(self.position, self.orientation)
    }
}
```

### 3.2 Point Cloud Types (`rv-core/src/types/point_cloud.rs`)

```rust
use glam::Vec3;
use crate::types::{FrameId, Timestamp};

/// RGBA color (0-255 per channel)
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }
}

/// Single point in a point cloud
#[derive(Debug, Clone, Copy)]
pub struct PointXYZ {
    pub position: Vec3,
}

/// Point with color
#[derive(Debug, Clone, Copy)]
pub struct PointXYZRGB {
    pub position: Vec3,
    pub color: Color,
}

/// Point with intensity
#[derive(Debug, Clone, Copy)]
pub struct PointXYZI {
    pub position: Vec3,
    pub intensity: f32,
}

/// Point cloud data
#[derive(Debug, Clone)]
pub struct PointCloud {
    pub positions: Vec<Vec3>,
    pub colors: Option<Vec<Color>>,
    pub intensities: Option<Vec<f32>>,
    pub frame_id: FrameId,
    pub timestamp: Timestamp,
}

impl PointCloud {
    pub fn new(frame_id: FrameId, timestamp: Timestamp) -> Self {
        Self {
            positions: Vec::new(),
            colors: None,
            intensities: None,
            frame_id,
            timestamp,
        }
    }

    pub fn with_capacity(capacity: usize, frame_id: FrameId, timestamp: Timestamp) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            colors: None,
            intensities: None,
            frame_id,
            timestamp,
        }
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Point cloud rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointCloudStyle {
    Points,
    Squares,
    Circles,
    Spheres,
    Boxes,
}

/// Color transformation mode for point clouds
#[derive(Debug, Clone)]
pub enum ColorMode {
    FlatColor(Color),
    RGB,
    Intensity { min: f32, max: f32, colormap: Colormap },
    AxisColor { axis: Axis, min: f32, max: f32, colormap: Colormap },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colormap {
    Jet,
    Rainbow,
    Turbo,
    Viridis,
    Grayscale,
}
```

### 3.3 Marker Types (`rv-core/src/types/marker.rs`)

```rust
use glam::{Vec3, Quat};
use crate::types::{Color, FrameId, Timestamp, Pose};

/// Marker types (matching RViz visualization_msgs/Marker)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerType {
    Arrow,
    Cube,
    Sphere,
    Cylinder,
    LineStrip,
    LineList,
    CubeList,
    SphereList,
    Points,
    TextViewFacing,
    MeshResource,
    TriangleList,
}

/// Marker action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerAction {
    Add,
    Modify,
    Delete,
    DeleteAll,
}

/// Scale for different marker types
#[derive(Debug, Clone, Copy)]
pub struct MarkerScale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl MarkerScale {
    pub fn uniform(s: f32) -> Self {
        Self { x: s, y: s, z: s }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

/// Visualization marker
#[derive(Debug, Clone)]
pub struct Marker {
    pub id: u32,
    pub ns: String,
    pub marker_type: MarkerType,
    pub action: MarkerAction,
    pub pose: Pose,
    pub scale: MarkerScale,
    pub color: Color,
    pub lifetime_ns: Option<i64>,
    pub frame_locked: bool,

    // Type-specific data
    pub points: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub text: String,
    pub mesh_resource: String,
    pub mesh_use_embedded_materials: bool,
}

impl Marker {
    pub fn new(id: u32, marker_type: MarkerType, frame_id: FrameId) -> Self {
        Self {
            id,
            ns: String::new(),
            marker_type,
            action: MarkerAction::Add,
            pose: Pose {
                position: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                frame_id,
                timestamp: Timestamp::now(),
            },
            scale: MarkerScale::uniform(1.0),
            color: Color::WHITE,
            lifetime_ns: None,
            frame_locked: false,
            points: Vec::new(),
            colors: Vec::new(),
            text: String::new(),
            mesh_resource: String::new(),
            mesh_use_embedded_materials: false,
        }
    }
}

/// Marker array for batch visualization
#[derive(Debug, Clone, Default)]
pub struct MarkerArray {
    pub markers: Vec<Marker>,
}
```

---

## 4. Transform System Architecture

### 4.1 Frame Tree (`rv-transform/src/frame_tree.rs`)

```rust
use std::collections::HashMap;
use rv_core::types::{FrameId, Transform, StampedTransform, Timestamp};

/// Node in the transform tree
#[derive(Debug)]
struct FrameNode {
    parent: Option<FrameId>,
    children: Vec<FrameId>,
}

/// Transform tree managing frame hierarchy
pub struct FrameTree {
    nodes: HashMap<FrameId, FrameNode>,
    root_frame: Option<FrameId>,
}

impl FrameTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_frame: None,
        }
    }

    /// Add or update a frame relationship
    pub fn set_parent(&mut self, child: FrameId, parent: FrameId) {
        // Remove from old parent's children
        if let Some(node) = self.nodes.get(&child) {
            if let Some(old_parent) = &node.parent {
                if let Some(old_parent_node) = self.nodes.get_mut(old_parent) {
                    old_parent_node.children.retain(|c| c != &child);
                }
            }
        }

        // Ensure parent exists
        self.nodes.entry(parent.clone()).or_insert_with(|| FrameNode {
            parent: None,
            children: Vec::new(),
        });

        // Add child to parent
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            if !parent_node.children.contains(&child) {
                parent_node.children.push(child.clone());
            }
        }

        // Update or create child node
        self.nodes.entry(child).or_insert_with(|| FrameNode {
            parent: Some(parent),
            children: Vec::new(),
        }).parent = Some(parent);
    }

    /// Get path from frame to root
    pub fn get_path_to_root(&self, frame: &FrameId) -> Vec<FrameId> {
        let mut path = vec![frame.clone()];
        let mut current = frame;

        while let Some(node) = self.nodes.get(current) {
            if let Some(parent) = &node.parent {
                path.push(parent.clone());
                current = parent;
            } else {
                break;
            }
        }

        path
    }

    /// Find common ancestor of two frames
    pub fn find_common_ancestor(&self, frame_a: &FrameId, frame_b: &FrameId) -> Option<FrameId> {
        let path_a = self.get_path_to_root(frame_a);
        let path_b: std::collections::HashSet<_> = self.get_path_to_root(frame_b).into_iter().collect();

        path_a.into_iter().find(|f| path_b.contains(f))
    }

    /// Get all frame IDs
    pub fn all_frames(&self) -> Vec<FrameId> {
        self.nodes.keys().cloned().collect()
    }
}
```

### 4.2 Transform Buffer (`rv-transform/src/transform_buffer.rs`)

```rust
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use rv_core::types::{FrameId, Transform, StampedTransform, Timestamp};
use crate::frame_tree::FrameTree;

/// Time-indexed transform storage
struct TransformHistory {
    transforms: BTreeMap<Timestamp, Transform>,
    max_duration: Duration,
}

impl TransformHistory {
    fn new(max_duration: Duration) -> Self {
        Self {
            transforms: BTreeMap::new(),
            max_duration,
        }
    }

    fn insert(&mut self, timestamp: Timestamp, transform: Transform) {
        self.transforms.insert(timestamp, transform);
        self.prune_old();
    }

    fn prune_old(&mut self) {
        if let Some((&newest, _)) = self.transforms.last_key_value() {
            let cutoff = Timestamp {
                nanos: newest.nanos - self.max_duration.as_nanos() as i64,
            };
            self.transforms.retain(|t, _| *t >= cutoff);
        }
    }

    /// Get transform at exact time or interpolate
    fn get_at(&self, timestamp: Timestamp) -> Option<Transform> {
        // Exact match
        if let Some(t) = self.transforms.get(&timestamp) {
            return Some(*t);
        }

        // Interpolate between nearest timestamps
        let before = self.transforms.range(..timestamp).next_back();
        let after = self.transforms.range(timestamp..).next();

        match (before, after) {
            (Some((&t1, &tf1)), Some((&t2, &tf2))) => {
                let ratio = (timestamp.nanos - t1.nanos) as f32
                    / (t2.nanos - t1.nanos) as f32;
                Some(tf1.lerp(&tf2, ratio))
            }
            (Some((_, &tf)), None) => Some(tf), // Extrapolate from last
            (None, Some((_, &tf))) => Some(tf), // Use first
            (None, None) => None,
        }
    }

    fn get_latest(&self) -> Option<(Timestamp, Transform)> {
        self.transforms.last_key_value().map(|(&t, &tf)| (t, tf))
    }
}

/// Key for transform lookup (parent -> child)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TransformKey {
    parent: FrameId,
    child: FrameId,
}

/// Transform buffer with time-based lookup
pub struct TransformBuffer {
    frame_tree: FrameTree,
    transforms: HashMap<TransformKey, TransformHistory>,
    buffer_duration: Duration,
    fixed_frame: FrameId,
}

impl TransformBuffer {
    pub fn new(fixed_frame: FrameId, buffer_duration: Duration) -> Self {
        Self {
            frame_tree: FrameTree::new(),
            transforms: HashMap::new(),
            buffer_duration,
            fixed_frame,
        }
    }

    /// Set the fixed (world) frame
    pub fn set_fixed_frame(&mut self, frame: FrameId) {
        self.fixed_frame = frame;
    }

    /// Add a transform to the buffer
    pub fn set_transform(&mut self, stamped: StampedTransform) {
        self.frame_tree.set_parent(
            stamped.child_frame.clone(),
            stamped.parent_frame.clone(),
        );

        let key = TransformKey {
            parent: stamped.parent_frame,
            child: stamped.child_frame,
        };

        self.transforms
            .entry(key)
            .or_insert_with(|| TransformHistory::new(self.buffer_duration))
            .insert(stamped.timestamp, stamped.transform);
    }

    /// Lookup transform from source to target frame
    pub fn lookup_transform(
        &self,
        target_frame: &FrameId,
        source_frame: &FrameId,
        timestamp: Timestamp,
    ) -> Result<Transform, TransformError> {
        if target_frame == source_frame {
            return Ok(Transform::IDENTITY);
        }

        // Find path through common ancestor
        let ancestor = self.frame_tree
            .find_common_ancestor(target_frame, source_frame)
            .ok_or(TransformError::NoPath)?;

        // Compute transform: target <- ancestor <- source
        let target_to_ancestor = self.compute_path_transform(target_frame, &ancestor, timestamp)?;
        let source_to_ancestor = self.compute_path_transform(source_frame, &ancestor, timestamp)?;

        Ok(target_to_ancestor.inverse().mul(&source_to_ancestor))
    }

    /// Get transform to fixed frame
    pub fn get_transform_to_fixed(
        &self,
        frame: &FrameId,
        timestamp: Timestamp,
    ) -> Result<Transform, TransformError> {
        self.lookup_transform(&self.fixed_frame, frame, timestamp)
    }

    /// Get latest transform to fixed frame
    pub fn get_latest_transform_to_fixed(
        &self,
        frame: &FrameId,
    ) -> Result<Transform, TransformError> {
        // Use latest timestamp from this frame's transforms
        let key = TransformKey {
            parent: self.fixed_frame.clone(),
            child: frame.clone(),
        };

        // Try direct lookup first
        if let Some(history) = self.transforms.get(&key) {
            if let Some((_, tf)) = history.get_latest() {
                return Ok(tf);
            }
        }

        // Fall back to path computation with current time
        self.get_transform_to_fixed(frame, Timestamp::now())
    }

    fn compute_path_transform(
        &self,
        from: &FrameId,
        to: &FrameId,
        timestamp: Timestamp,
    ) -> Result<Transform, TransformError> {
        let path = self.frame_tree.get_path_to_root(from);
        let mut transform = Transform::IDENTITY;

        for window in path.windows(2) {
            let child = &window[0];
            let parent = &window[1];

            if parent == to {
                break;
            }

            let key = TransformKey {
                parent: parent.clone(),
                child: child.clone(),
            };

            let tf = self.transforms
                .get(&key)
                .and_then(|h| h.get_at(timestamp))
                .ok_or(TransformError::TransformNotFound)?;

            transform = tf.mul(&transform);
        }

        Ok(transform)
    }

    /// Get all available frames
    pub fn all_frames(&self) -> Vec<FrameId> {
        self.frame_tree.all_frames()
    }
}

#[derive(Debug, Clone)]
pub enum TransformError {
    NoPath,
    TransformNotFound,
    ExtrapolationError,
}
```

---

## 5. Rerun Integration Layer

### 5.1 Rerun Bridge (`rv-rerun/src/bridge.rs`)

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use rerun::{RecordingStream, RecordingStreamBuilder};
use rv_core::types::Timestamp;

/// Configuration for Rerun connection
pub struct RerunConfig {
    pub app_id: String,
    pub recording_id: Option<String>,
    pub connect_addr: Option<String>,
    pub save_path: Option<String>,
}

impl Default for RerunConfig {
    fn default() -> Self {
        Self {
            app_id: "robotics-viz".to_string(),
            recording_id: None,
            connect_addr: None,
            save_path: None,
        }
    }
}

/// Main bridge to Rerun
pub struct RerunBridge {
    stream: RecordingStream,
    config: RerunConfig,
}

impl RerunBridge {
    pub fn new(config: RerunConfig) -> Result<Self, RerunError> {
        let mut builder = RecordingStreamBuilder::new(&config.app_id);

        if let Some(id) = &config.recording_id {
            builder = builder.recording_id(id);
        }

        let stream = if let Some(addr) = &config.connect_addr {
            builder.connect_tcp_opts(addr.parse()?, Default::default())?
        } else if let Some(path) = &config.save_path {
            builder.save(path)?
        } else {
            builder.spawn()?
        };

        Ok(Self { stream, config })
    }

    /// Get reference to the recording stream
    pub fn stream(&self) -> &RecordingStream {
        &self.stream
    }

    /// Set current timeline
    pub fn set_time(&self, name: &str, timestamp: Timestamp) {
        self.stream.set_time_nanos(name, timestamp.nanos);
    }

    /// Set sequence number
    pub fn set_sequence(&self, name: &str, sequence: i64) {
        self.stream.set_time_sequence(name, sequence);
    }

    /// Log data with entity path
    pub fn log<T: rerun::AsComponents>(
        &self,
        entity_path: &str,
        data: &T,
    ) -> Result<(), RerunError> {
        self.stream.log(entity_path, data)?;
        Ok(())
    }

    /// Log static data (time-independent)
    pub fn log_static<T: rerun::AsComponents>(
        &self,
        entity_path: &str,
        data: &T,
    ) -> Result<(), RerunError> {
        self.stream.log_static(entity_path, data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RerunError {
    Connection(String),
    Logging(String),
}

impl From<rerun::RecordingStreamError> for RerunError {
    fn from(e: rerun::RecordingStreamError) -> Self {
        RerunError::Connection(e.to_string())
    }
}

impl From<std::net::AddrParseError> for RerunError {
    fn from(e: std::net::AddrParseError) -> Self {
        RerunError::Connection(e.to_string())
    }
}
```

### 5.2 Point Cloud Adapter (`rv-rerun/src/adapters/point_cloud.rs`)

```rust
use rv_core::types::{PointCloud, Color, ColorMode, Colormap};
use crate::bridge::{RerunBridge, RerunError};

/// Adapter for logging point clouds to Rerun
pub struct PointCloudAdapter;

impl PointCloudAdapter {
    /// Log a point cloud to Rerun
    pub fn log(
        bridge: &RerunBridge,
        entity_path: &str,
        cloud: &PointCloud,
        color_mode: &ColorMode,
        point_radius: f32,
    ) -> Result<(), RerunError> {
        let positions: Vec<rerun::Position3D> = cloud.positions
            .iter()
            .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
            .collect();

        let colors = Self::compute_colors(cloud, color_mode);

        let mut points = rerun::Points3D::new(positions)
            .with_radii(vec![rerun::Radius::new_scene_units(point_radius); cloud.len()]);

        if let Some(colors) = colors {
            points = points.with_colors(colors);
        }

        bridge.log(entity_path, &points)
    }

    fn compute_colors(cloud: &PointCloud, mode: &ColorMode) -> Option<Vec<rerun::Color>> {
        match mode {
            ColorMode::FlatColor(c) => {
                Some(vec![rerun::Color::from_rgba(c.r, c.g, c.b, c.a); cloud.len()])
            }
            ColorMode::RGB => {
                cloud.colors.as_ref().map(|colors| {
                    colors.iter()
                        .map(|c| rerun::Color::from_rgba(c.r, c.g, c.b, c.a))
                        .collect()
                })
            }
            ColorMode::Intensity { min, max, colormap } => {
                cloud.intensities.as_ref().map(|intensities| {
                    intensities.iter()
                        .map(|&i| {
                            let normalized = ((i - min) / (max - min)).clamp(0.0, 1.0);
                            Self::colormap_lookup(*colormap, normalized)
                        })
                        .collect()
                })
            }
            ColorMode::AxisColor { axis, min, max, colormap } => {
                Some(cloud.positions.iter()
                    .map(|p| {
                        let value = match axis {
                            rv_core::types::Axis::X => p.x,
                            rv_core::types::Axis::Y => p.y,
                            rv_core::types::Axis::Z => p.z,
                        };
                        let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);
                        Self::colormap_lookup(*colormap, normalized)
                    })
                    .collect())
            }
        }
    }

    fn colormap_lookup(colormap: Colormap, t: f32) -> rerun::Color {
        // Simplified colormap implementation
        let (r, g, b) = match colormap {
            Colormap::Jet => {
                let r = (1.5 - (4.0 * t - 3.0).abs()).clamp(0.0, 1.0);
                let g = (1.5 - (4.0 * t - 2.0).abs()).clamp(0.0, 1.0);
                let b = (1.5 - (4.0 * t - 1.0).abs()).clamp(0.0, 1.0);
                (r, g, b)
            }
            Colormap::Rainbow => {
                let h = t * 360.0;
                Self::hsv_to_rgb(h, 1.0, 1.0)
            }
            Colormap::Viridis => {
                // Simplified viridis approximation
                let r = 0.267 + 0.004 * t + t * t * 0.329;
                let g = 0.004 + t * 0.873 - t * t * 0.377;
                let b = 0.329 + t * 0.421 - t * t * 0.25;
                (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
            }
            Colormap::Grayscale => (t, t, t),
            Colormap::Turbo => {
                // Simplified turbo approximation
                let r = 0.13572 + t * (4.61539 - t * (42.6603 - t * (77.6631 - t * 51.0942)));
                let g = 0.09140 + t * (2.92488 + t * (1.34167 - t * (4.89815 - t * 3.02390)));
                let b = 0.10667 + t * (12.7520 - t * (60.5820 - t * (109.174 - t * 64.6550)));
                (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
            }
        };

        rerun::Color::from_rgb(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match (h / 60.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        (r + m, g + m, b + m)
    }
}
```

### 5.3 Transform Adapter (`rv-rerun/src/adapters/transform.rs`)

```rust
use glam::{Vec3, Quat, Mat3};
use rv_core::types::{FrameId, Transform, StampedTransform};
use rv_transform::TransformBuffer;
use crate::bridge::{RerunBridge, RerunError};

/// Adapter for logging transforms to Rerun
pub struct TransformAdapter;

impl TransformAdapter {
    /// Log a single transform
    pub fn log_transform(
        bridge: &RerunBridge,
        entity_path: &str,
        transform: &Transform,
    ) -> Result<(), RerunError> {
        let tf = rerun::Transform3D::from_translation_rotation(
            rerun::Vec3D::new(
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            rerun::Quaternion::from_xyzw([
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ]),
        );

        bridge.log(entity_path, &tf)
    }

    /// Log coordinate axes visualization at a frame
    pub fn log_frame_axes(
        bridge: &RerunBridge,
        entity_path: &str,
        transform: &Transform,
        scale: f32,
    ) -> Result<(), RerunError> {
        // Log transform first
        Self::log_transform(bridge, entity_path, transform)?;

        // Log axes as arrows
        let origin = transform.translation;
        let x_axis = transform.transform_vector(Vec3::X * scale);
        let y_axis = transform.transform_vector(Vec3::Y * scale);
        let z_axis = transform.transform_vector(Vec3::Z * scale);

        let arrows = rerun::Arrows3D::from_vectors([
            rerun::Vec3D::new(x_axis.x, x_axis.y, x_axis.z),
            rerun::Vec3D::new(y_axis.x, y_axis.y, y_axis.z),
            rerun::Vec3D::new(z_axis.x, z_axis.y, z_axis.z),
        ])
        .with_origins([
            rerun::Position3D::new(origin.x, origin.y, origin.z),
            rerun::Position3D::new(origin.x, origin.y, origin.z),
            rerun::Position3D::new(origin.x, origin.y, origin.z),
        ])
        .with_colors([
            rerun::Color::from_rgb(255, 0, 0),   // X = Red
            rerun::Color::from_rgb(0, 255, 0),   // Y = Green
            rerun::Color::from_rgb(0, 0, 255),   // Z = Blue
        ]);

        bridge.log(&format!("{}/axes", entity_path), &arrows)
    }

    /// Log all transforms from buffer
    pub fn log_all_frames(
        bridge: &RerunBridge,
        buffer: &TransformBuffer,
        base_path: &str,
        axis_scale: f32,
    ) -> Result<(), RerunError> {
        for frame in buffer.all_frames() {
            if let Ok(transform) = buffer.get_latest_transform_to_fixed(&frame) {
                let path = format!("{}/{}", base_path, frame.0);
                Self::log_frame_axes(bridge, &path, &transform, axis_scale)?;
            }
        }
        Ok(())
    }
}
```

### 5.4 Marker Adapter (`rv-rerun/src/adapters/marker.rs`)

```rust
use glam::Vec3;
use rv_core::types::{Marker, MarkerType, MarkerArray};
use crate::bridge::{RerunBridge, RerunError};

/// Adapter for logging markers to Rerun
pub struct MarkerAdapter;

impl MarkerAdapter {
    /// Log a marker to Rerun
    pub fn log_marker(
        bridge: &RerunBridge,
        base_path: &str,
        marker: &Marker,
    ) -> Result<(), RerunError> {
        let entity_path = format!("{}/{}/{}", base_path, marker.ns, marker.id);
        let color = rerun::Color::from_rgba(
            marker.color.r,
            marker.color.g,
            marker.color.b,
            marker.color.a,
        );

        match marker.marker_type {
            MarkerType::Arrow => {
                Self::log_arrow(bridge, &entity_path, marker, color)
            }
            MarkerType::Cube => {
                Self::log_box(bridge, &entity_path, marker, color)
            }
            MarkerType::Sphere => {
                Self::log_sphere(bridge, &entity_path, marker, color)
            }
            MarkerType::Cylinder => {
                Self::log_cylinder(bridge, &entity_path, marker, color)
            }
            MarkerType::LineStrip => {
                Self::log_line_strip(bridge, &entity_path, marker, color)
            }
            MarkerType::LineList => {
                Self::log_line_list(bridge, &entity_path, marker, color)
            }
            MarkerType::Points => {
                Self::log_points(bridge, &entity_path, marker, color)
            }
            MarkerType::TextViewFacing => {
                Self::log_text(bridge, &entity_path, marker, color)
            }
            MarkerType::CubeList | MarkerType::SphereList => {
                Self::log_shape_list(bridge, &entity_path, marker, color)
            }
            MarkerType::MeshResource => {
                Self::log_mesh_resource(bridge, &entity_path, marker)
            }
            MarkerType::TriangleList => {
                Self::log_triangle_list(bridge, &entity_path, marker, color)
            }
        }
    }

    fn log_arrow(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let start = marker.pose.position;
        let direction = marker.pose.orientation * Vec3::X * marker.scale.x;

        let arrows = rerun::Arrows3D::from_vectors([
            rerun::Vec3D::new(direction.x, direction.y, direction.z),
        ])
        .with_origins([rerun::Position3D::new(start.x, start.y, start.z)])
        .with_colors([color]);

        bridge.log(path, &arrows)
    }

    fn log_box(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let boxes = rerun::Boxes3D::from_centers_and_half_sizes(
            [rerun::Position3D::new(
                marker.pose.position.x,
                marker.pose.position.y,
                marker.pose.position.z,
            )],
            [rerun::HalfSize3D::new(
                marker.scale.x / 2.0,
                marker.scale.y / 2.0,
                marker.scale.z / 2.0,
            )],
        )
        .with_colors([color])
        .with_quaternions([rerun::Quaternion::from_xyzw([
            marker.pose.orientation.x,
            marker.pose.orientation.y,
            marker.pose.orientation.z,
            marker.pose.orientation.w,
        ])]);

        bridge.log(path, &boxes)
    }

    fn log_sphere(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let ellipsoids = rerun::Ellipsoids3D::from_centers_and_half_sizes(
            [rerun::Position3D::new(
                marker.pose.position.x,
                marker.pose.position.y,
                marker.pose.position.z,
            )],
            [rerun::HalfSize3D::new(
                marker.scale.x / 2.0,
                marker.scale.y / 2.0,
                marker.scale.z / 2.0,
            )],
        )
        .with_colors([color]);

        bridge.log(path, &ellipsoids)
    }

    fn log_cylinder(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        // Rerun doesn't have native cylinders, approximate with capsule or mesh
        // For now, use ellipsoid as placeholder
        Self::log_sphere(bridge, path, marker, color)
    }

    fn log_line_strip(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.is_empty() {
            return Ok(());
        }

        let points: Vec<rerun::Position3D> = marker.points
            .iter()
            .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
            .collect();

        let lines = rerun::LineStrips3D::new([points])
            .with_colors([color]);

        bridge.log(path, &lines)
    }

    fn log_line_list(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.len() < 2 {
            return Ok(());
        }

        let segments: Vec<Vec<rerun::Position3D>> = marker.points
            .chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .map(|chunk| vec![
                rerun::Position3D::new(chunk[0].x, chunk[0].y, chunk[0].z),
                rerun::Position3D::new(chunk[1].x, chunk[1].y, chunk[1].z),
            ])
            .collect();

        let lines = rerun::LineStrips3D::new(segments)
            .with_colors(vec![color; marker.points.len() / 2]);

        bridge.log(path, &lines)
    }

    fn log_points(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let positions: Vec<rerun::Position3D> = marker.points
            .iter()
            .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
            .collect();

        let colors = if marker.colors.is_empty() {
            vec![color; positions.len()]
        } else {
            marker.colors.iter()
                .map(|c| rerun::Color::from_rgba(c.r, c.g, c.b, c.a))
                .collect()
        };

        let points = rerun::Points3D::new(positions)
            .with_colors(colors)
            .with_radii(vec![rerun::Radius::new_scene_units(marker.scale.x); marker.points.len()]);

        bridge.log(path, &points)
    }

    fn log_text(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        // Rerun text logging
        let text = rerun::TextLog::new(&marker.text);
        bridge.log(path, &text)?;

        // Also log position as a point for reference
        let point = rerun::Points3D::new([rerun::Position3D::new(
            marker.pose.position.x,
            marker.pose.position.y,
            marker.pose.position.z,
        )])
        .with_colors([color])
        .with_radii([rerun::Radius::new_scene_units(0.01)]);

        bridge.log(&format!("{}/position", path), &point)
    }

    fn log_shape_list(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let positions: Vec<rerun::Position3D> = marker.points
            .iter()
            .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
            .collect();

        let colors = if marker.colors.is_empty() {
            vec![color; positions.len()]
        } else {
            marker.colors.iter()
                .map(|c| rerun::Color::from_rgba(c.r, c.g, c.b, c.a))
                .collect()
        };

        match marker.marker_type {
            MarkerType::CubeList => {
                let boxes = rerun::Boxes3D::from_centers_and_half_sizes(
                    positions,
                    vec![rerun::HalfSize3D::new(
                        marker.scale.x / 2.0,
                        marker.scale.y / 2.0,
                        marker.scale.z / 2.0,
                    ); marker.points.len()],
                )
                .with_colors(colors);
                bridge.log(path, &boxes)
            }
            MarkerType::SphereList => {
                let ellipsoids = rerun::Ellipsoids3D::from_centers_and_half_sizes(
                    positions,
                    vec![rerun::HalfSize3D::new(
                        marker.scale.x / 2.0,
                        marker.scale.y / 2.0,
                        marker.scale.z / 2.0,
                    ); marker.points.len()],
                )
                .with_colors(colors);
                bridge.log(path, &ellipsoids)
            }
            _ => Ok(()),
        }
    }

    fn log_mesh_resource(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
    ) -> Result<(), RerunError> {
        // Mesh resource loading would require additional implementation
        // For now, log a placeholder
        let text = rerun::TextLog::new(format!("Mesh: {}", marker.mesh_resource));
        bridge.log(path, &text)
    }

    fn log_triangle_list(
        bridge: &RerunBridge,
        path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.len() < 3 {
            return Ok(());
        }

        let vertices: Vec<rerun::Position3D> = marker.points
            .iter()
            .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
            .collect();

        let indices: Vec<rerun::TriangleIndices> = (0..marker.points.len() / 3)
            .map(|i| rerun::TriangleIndices::new(
                (i * 3) as u32,
                (i * 3 + 1) as u32,
                (i * 3 + 2) as u32,
            ))
            .collect();

        let vertex_colors = if marker.colors.is_empty() {
            vec![color; vertices.len()]
        } else {
            marker.colors.iter()
                .map(|c| rerun::Color::from_rgba(c.r, c.g, c.b, c.a))
                .collect()
        };

        let mesh = rerun::Mesh3D::new(vertices)
            .with_triangle_indices(indices)
            .with_vertex_colors(vertex_colors);

        bridge.log(path, &mesh)
    }

    /// Log a marker array
    pub fn log_marker_array(
        bridge: &RerunBridge,
        base_path: &str,
        array: &MarkerArray,
    ) -> Result<(), RerunError> {
        for marker in &array.markers {
            Self::log_marker(bridge, base_path, marker)?;
        }
        Ok(())
    }
}
```

---

## 6. Display Plugin System

### 6.1 Display Trait (`rv-core/src/traits/display.rs`)

```rust
use std::any::Any;
use crate::types::{FrameId, Timestamp};
use crate::config::DisplayConfig;

/// Status level for display health
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Ok,
    Warn,
    Error,
}

/// Status message
#[derive(Debug, Clone)]
pub struct Status {
    pub level: StatusLevel,
    pub name: String,
    pub message: String,
}

/// Display context providing access to shared resources
pub trait DisplayContext: Send + Sync {
    /// Get the fixed frame
    fn fixed_frame(&self) -> &FrameId;

    /// Get the transform buffer
    fn transform_buffer(&self) -> &rv_transform::TransformBuffer;

    /// Get the Rerun bridge
    fn rerun_bridge(&self) -> &rv_rerun::RerunBridge;

    /// Queue a render update
    fn queue_render(&self);

    /// Get current time
    fn current_time(&self) -> Timestamp;
}

/// Core trait for all displays
pub trait Display: Send + Sync {
    /// Get the display name
    fn name(&self) -> &str;

    /// Get the display type identifier
    fn type_id(&self) -> &str;

    /// Initialize the display
    fn initialize(&mut self, context: &dyn DisplayContext);

    /// Called when enabled
    fn on_enable(&mut self, context: &dyn DisplayContext);

    /// Called when disabled
    fn on_disable(&mut self, context: &dyn DisplayContext);

    /// Periodic update (~30Hz)
    fn update(
        &mut self,
        context: &dyn DisplayContext,
        wall_dt: std::time::Duration,
        ros_dt: std::time::Duration,
    );

    /// Reset the display state
    fn reset(&mut self, context: &dyn DisplayContext);

    /// Check if enabled
    fn is_enabled(&self) -> bool;

    /// Set enabled state
    fn set_enabled(&mut self, enabled: bool);

    /// Get current status
    fn status(&self) -> &[Status];

    /// Set a status message
    fn set_status(&mut self, level: StatusLevel, name: &str, message: &str);

    /// Clear status messages
    fn clear_status(&mut self);

    /// Get the entity path for Rerun logging
    fn entity_path(&self) -> &str;

    /// Get configurable properties
    fn properties(&self) -> &dyn Any;

    /// Get mutable configurable properties
    fn properties_mut(&mut self) -> &mut dyn Any;

    /// Save configuration
    fn save_config(&self) -> DisplayConfig;

    /// Load configuration
    fn load_config(&mut self, config: &DisplayConfig);
}

/// Registry for display types
pub struct DisplayRegistry {
    factories: std::collections::HashMap<String, Box<dyn DisplayFactory>>,
}

pub trait DisplayFactory: Send + Sync {
    fn create(&self) -> Box<dyn Display>;
    fn type_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn description(&self) -> &str;
}

impl DisplayRegistry {
    pub fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    pub fn register<F: DisplayFactory + 'static>(&mut self, factory: F) {
        self.factories.insert(factory.type_id().to_string(), Box::new(factory));
    }

    pub fn create(&self, type_id: &str) -> Option<Box<dyn Display>> {
        self.factories.get(type_id).map(|f| f.create())
    }

    pub fn available_types(&self) -> Vec<(&str, &str)> {
        self.factories.values()
            .map(|f| (f.type_id(), f.display_name()))
            .collect()
    }
}
```

### 6.2 Base Display Implementation (`rv-displays/src/base.rs`)

```rust
use rv_core::traits::display::{Display, DisplayContext, Status, StatusLevel};
use rv_core::config::DisplayConfig;
use std::any::Any;

/// Base implementation for displays
pub struct BaseDisplay {
    name: String,
    type_id: String,
    entity_path: String,
    enabled: bool,
    statuses: Vec<Status>,
}

impl BaseDisplay {
    pub fn new(name: impl Into<String>, type_id: impl Into<String>) -> Self {
        let name = name.into();
        let entity_path = format!("/displays/{}", name.to_lowercase().replace(' ', "_"));

        Self {
            name,
            type_id: type_id.into(),
            entity_path,
            enabled: true,
            statuses: Vec::new(),
        }
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.entity_path = format!("/displays/{}", self.name.to_lowercase().replace(' ', "_"));
    }
}

/// Macro to simplify display implementation
#[macro_export]
macro_rules! impl_display_base {
    ($struct_name:ident, $type_id:expr) => {
        impl Display for $struct_name {
            fn name(&self) -> &str {
                &self.base.name
            }

            fn type_id(&self) -> &str {
                $type_id
            }

            fn is_enabled(&self) -> bool {
                self.base.enabled
            }

            fn set_enabled(&mut self, enabled: bool) {
                self.base.enabled = enabled;
            }

            fn status(&self) -> &[Status] {
                &self.base.statuses
            }

            fn set_status(&mut self, level: StatusLevel, name: &str, message: &str) {
                self.base.statuses.retain(|s| s.name != name);
                self.base.statuses.push(Status {
                    level,
                    name: name.to_string(),
                    message: message.to_string(),
                });
            }

            fn clear_status(&mut self) {
                self.base.statuses.clear();
            }

            fn entity_path(&self) -> &str {
                &self.base.entity_path
            }
        }
    };
}
```

### 6.3 Grid Display Example (`rv-displays/src/grid.rs`)

```rust
use rv_core::traits::display::{Display, DisplayContext, Status, StatusLevel};
use rv_core::types::{Color, FrameId};
use rv_core::config::DisplayConfig;
use rv_rerun::RerunBridge;
use std::any::Any;

use crate::base::BaseDisplay;

/// Grid display properties
#[derive(Debug, Clone)]
pub struct GridProperties {
    pub cell_count: u32,
    pub cell_size: f32,
    pub color: Color,
    pub alpha: f32,
    pub line_width: f32,
    pub frame: FrameId,
    pub offset: glam::Vec3,
    pub plane: GridPlane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridPlane {
    XY,
    XZ,
    YZ,
}

impl Default for GridProperties {
    fn default() -> Self {
        Self {
            cell_count: 10,
            cell_size: 1.0,
            color: Color::rgba(128, 128, 128, 128),
            alpha: 0.5,
            line_width: 0.01,
            frame: FrameId::new("world"),
            offset: glam::Vec3::ZERO,
            plane: GridPlane::XY,
        }
    }
}

/// Grid display
pub struct GridDisplay {
    base: BaseDisplay,
    properties: GridProperties,
}

impl GridDisplay {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: BaseDisplay::new(name, "rv_displays/Grid"),
            properties: GridProperties::default(),
        }
    }

    fn generate_grid_lines(&self) -> Vec<Vec<glam::Vec3>> {
        let mut lines = Vec::new();
        let half_size = self.properties.cell_count as f32 * self.properties.cell_size / 2.0;
        let offset = self.properties.offset;

        for i in 0..=self.properties.cell_count {
            let t = i as f32 * self.properties.cell_size - half_size;

            match self.properties.plane {
                GridPlane::XY => {
                    // Horizontal lines
                    lines.push(vec![
                        glam::Vec3::new(-half_size + offset.x, t + offset.y, offset.z),
                        glam::Vec3::new(half_size + offset.x, t + offset.y, offset.z),
                    ]);
                    // Vertical lines
                    lines.push(vec![
                        glam::Vec3::new(t + offset.x, -half_size + offset.y, offset.z),
                        glam::Vec3::new(t + offset.x, half_size + offset.y, offset.z),
                    ]);
                }
                GridPlane::XZ => {
                    lines.push(vec![
                        glam::Vec3::new(-half_size + offset.x, offset.y, t + offset.z),
                        glam::Vec3::new(half_size + offset.x, offset.y, t + offset.z),
                    ]);
                    lines.push(vec![
                        glam::Vec3::new(t + offset.x, offset.y, -half_size + offset.z),
                        glam::Vec3::new(t + offset.x, offset.y, half_size + offset.z),
                    ]);
                }
                GridPlane::YZ => {
                    lines.push(vec![
                        glam::Vec3::new(offset.x, -half_size + offset.y, t + offset.z),
                        glam::Vec3::new(offset.x, half_size + offset.y, t + offset.z),
                    ]);
                    lines.push(vec![
                        glam::Vec3::new(offset.x, t + offset.y, -half_size + offset.z),
                        glam::Vec3::new(offset.x, t + offset.y, half_size + offset.z),
                    ]);
                }
            }
        }

        lines
    }

    fn log_grid(&self, bridge: &RerunBridge) -> Result<(), rv_rerun::RerunError> {
        let lines = self.generate_grid_lines();

        let strips: Vec<Vec<rerun::Position3D>> = lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|p| rerun::Position3D::new(p.x, p.y, p.z))
                    .collect()
            })
            .collect();

        let color = rerun::Color::from_rgba(
            self.properties.color.r,
            self.properties.color.g,
            self.properties.color.b,
            (self.properties.alpha * 255.0) as u8,
        );

        let line_strips = rerun::LineStrips3D::new(strips)
            .with_colors(vec![color; lines.len()])
            .with_radii(vec![rerun::Radius::new_scene_units(self.properties.line_width); lines.len()]);

        bridge.log_static(self.entity_path(), &line_strips)
    }
}

impl Display for GridDisplay {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn type_id(&self) -> &str {
        "rv_displays/Grid"
    }

    fn initialize(&mut self, context: &dyn DisplayContext) {
        if let Err(e) = self.log_grid(context.rerun_bridge()) {
            self.set_status(StatusLevel::Error, "Grid", &format!("Failed to log grid: {:?}", e));
        } else {
            self.set_status(StatusLevel::Ok, "Grid", "Grid displayed");
        }
    }

    fn on_enable(&mut self, context: &dyn DisplayContext) {
        let _ = self.log_grid(context.rerun_bridge());
    }

    fn on_disable(&mut self, _context: &dyn DisplayContext) {
        // Could clear the grid from Rerun if needed
    }

    fn update(
        &mut self,
        _context: &dyn DisplayContext,
        _wall_dt: std::time::Duration,
        _ros_dt: std::time::Duration,
    ) {
        // Grid is static, no updates needed
    }

    fn reset(&mut self, context: &dyn DisplayContext) {
        let _ = self.log_grid(context.rerun_bridge());
    }

    fn is_enabled(&self) -> bool {
        self.base.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.base.enabled = enabled;
    }

    fn status(&self) -> &[Status] {
        &self.base.statuses
    }

    fn set_status(&mut self, level: StatusLevel, name: &str, message: &str) {
        self.base.statuses.retain(|s| s.name != name);
        self.base.statuses.push(Status {
            level,
            name: name.to_string(),
            message: message.to_string(),
        });
    }

    fn clear_status(&mut self) {
        self.base.statuses.clear();
    }

    fn entity_path(&self) -> &str {
        &self.base.entity_path
    }

    fn properties(&self) -> &dyn Any {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut dyn Any {
        &mut self.properties
    }

    fn save_config(&self) -> DisplayConfig {
        DisplayConfig::new(self.type_id())
            .with("cell_count", self.properties.cell_count)
            .with("cell_size", self.properties.cell_size)
            .with("alpha", self.properties.alpha)
    }

    fn load_config(&mut self, config: &DisplayConfig) {
        if let Some(v) = config.get::<u32>("cell_count") {
            self.properties.cell_count = v;
        }
        if let Some(v) = config.get::<f32>("cell_size") {
            self.properties.cell_size = v;
        }
        if let Some(v) = config.get::<f32>("alpha") {
            self.properties.alpha = v;
        }
    }
}
```

---

## 7. View Controller System

### 7.1 View Controller Trait (`rv-core/src/traits/view_controller.rs`)

```rust
use glam::{Vec3, Quat, Mat4};
use crate::types::Timestamp;

/// Camera state
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov_y: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            rotation: Quat::IDENTITY,
            fov_y: 60.0_f32.to_radians(),
            near_clip: 0.01,
            far_clip: 1000.0,
        }
    }
}

impl CameraState {
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalize();
        self.rotation = Quat::from_rotation_arc(-Vec3::Z, direction);
    }
}

/// Mouse button state
#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub position: (f32, f32),
    pub delta: (f32, f32),
    pub left: bool,
    pub middle: bool,
    pub right: bool,
    pub scroll_delta: f32,
}

/// Keyboard modifier state
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Input event for view controllers
pub enum ViewInputEvent {
    MouseMove { state: MouseState, modifiers: KeyModifiers },
    MousePress { button: MouseButton, position: (f32, f32), modifiers: KeyModifiers },
    MouseRelease { button: MouseButton, position: (f32, f32), modifiers: KeyModifiers },
    Scroll { delta: f32, modifiers: KeyModifiers },
    KeyPress { key: Key, modifiers: KeyModifiers },
    KeyRelease { key: Key, modifiers: KeyModifiers },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    W, A, S, D, Q, E,
    F, Z,
    Up, Down, Left, Right,
    Shift, Ctrl, Alt,
}

/// View controller trait
pub trait ViewController: Send + Sync {
    /// Get the controller name
    fn name(&self) -> &str;

    /// Get current camera state
    fn camera_state(&self) -> &CameraState;

    /// Get mutable camera state
    fn camera_state_mut(&mut self) -> &mut CameraState;

    /// Handle input event
    fn handle_input(&mut self, event: &ViewInputEvent, viewport_size: (f32, f32));

    /// Periodic update
    fn update(&mut self, dt: std::time::Duration);

    /// Reset to default view
    fn reset(&mut self);

    /// Look at a specific point
    fn look_at(&mut self, target: Vec3);

    /// Set focal point (for orbit controllers)
    fn set_focal_point(&mut self, point: Vec3);

    /// Get focal point if applicable
    fn focal_point(&self) -> Option<Vec3>;

    /// Mimic another view controller's view
    fn mimic(&mut self, other: &dyn ViewController) {
        *self.camera_state_mut() = *other.camera_state();
    }
}
```

### 7.2 Orbit View Controller (`rv-views/src/orbit.rs`)

```rust
use glam::{Vec3, Quat, Mat4};
use rv_core::traits::view_controller::*;

/// Orbit view controller (rotate around focal point)
pub struct OrbitViewController {
    camera: CameraState,
    focal_point: Vec3,
    distance: f32,
    yaw: f32,   // Rotation around Y axis
    pitch: f32, // Rotation around X axis

    // Interaction settings
    rotate_speed: f32,
    pan_speed: f32,
    zoom_speed: f32,

    // State
    mouse_state: MouseState,
}

impl OrbitViewController {
    pub fn new() -> Self {
        let mut controller = Self {
            camera: CameraState::default(),
            focal_point: Vec3::ZERO,
            distance: 10.0,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: std::f32::consts::FRAC_PI_4,
            rotate_speed: 0.005,
            pan_speed: 0.01,
            zoom_speed: 0.1,
            mouse_state: MouseState::default(),
        };
        controller.update_camera();
        controller
    }

    fn update_camera(&mut self) {
        // Calculate camera position from spherical coordinates
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();

        self.camera.position = self.focal_point + Vec3::new(x, y, z);
        self.camera.look_at(self.focal_point);
    }

    fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.rotate_speed;
        self.pitch = (self.pitch + dy * self.rotate_speed)
            .clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
        self.update_camera();
    }

    fn pan(&mut self, dx: f32, dy: f32) {
        let right = self.camera.right();
        let up = self.camera.up();
        let pan = right * (-dx * self.pan_speed * self.distance)
            + up * (dy * self.pan_speed * self.distance);
        self.focal_point += pan;
        self.update_camera();
    }

    fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 - delta * self.zoom_speed;
        self.distance = self.distance.clamp(0.1, 1000.0);
        self.update_camera();
    }
}

impl ViewController for OrbitViewController {
    fn name(&self) -> &str {
        "Orbit"
    }

    fn camera_state(&self) -> &CameraState {
        &self.camera
    }

    fn camera_state_mut(&mut self) -> &mut CameraState {
        &mut self.camera
    }

    fn handle_input(&mut self, event: &ViewInputEvent, _viewport_size: (f32, f32)) {
        match event {
            ViewInputEvent::MouseMove { state, modifiers } => {
                let dx = state.delta.0;
                let dy = state.delta.1;

                if state.left && !modifiers.shift {
                    self.rotate(dx, dy);
                } else if state.middle || (state.left && modifiers.shift) {
                    self.pan(dx, dy);
                } else if state.right {
                    self.zoom(dy * 0.01);
                }

                self.mouse_state = *state;
            }
            ViewInputEvent::Scroll { delta, .. } => {
                self.zoom(*delta);
            }
            ViewInputEvent::KeyPress { key, .. } => {
                match key {
                    Key::F => {
                        // Focus on selection (would need selection context)
                    }
                    Key::Z => {
                        self.reset();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, _dt: std::time::Duration) {
        // No continuous updates needed for orbit controller
    }

    fn reset(&mut self) {
        self.focal_point = Vec3::ZERO;
        self.distance = 10.0;
        self.yaw = std::f32::consts::FRAC_PI_4;
        self.pitch = std::f32::consts::FRAC_PI_4;
        self.update_camera();
    }

    fn look_at(&mut self, target: Vec3) {
        self.focal_point = target;
        self.update_camera();
    }

    fn set_focal_point(&mut self, point: Vec3) {
        self.focal_point = point;
        self.update_camera();
    }

    fn focal_point(&self) -> Option<Vec3> {
        Some(self.focal_point)
    }
}
```

---

## 8. Configuration System

### 8.1 Configuration Schema (`rv-core/src/config/schema.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub global_options: GlobalOptions,
    pub displays: Vec<DisplayConfig>,
    pub views: ViewsConfig,
    pub panels: PanelsConfig,
    pub window: WindowConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            global_options: GlobalOptions::default(),
            displays: Vec::new(),
            views: ViewsConfig::default(),
            panels: PanelsConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOptions {
    pub fixed_frame: String,
    pub frame_rate: u32,
    pub background_color: [f32; 4],
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            fixed_frame: "world".to_string(),
            frame_rate: 30,
            background_color: [0.2, 0.2, 0.2, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub type_id: String,
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub properties: HashMap<String, serde_yaml::Value>,
}

impl DisplayConfig {
    pub fn new(type_id: impl Into<String>) -> Self {
        Self {
            type_id: type_id.into(),
            name: String::new(),
            enabled: true,
            properties: HashMap::new(),
        }
    }

    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.properties.insert(
            key.to_string(),
            serde_yaml::to_value(value).unwrap_or(serde_yaml::Value::Null),
        );
        self
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewsConfig {
    pub current: String,
    pub saved: Vec<SavedView>,
}

impl Default for ViewsConfig {
    fn default() -> Self {
        Self {
            current: "Orbit".to_string(),
            saved: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub name: String,
    pub controller_type: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub focal_point: Option<[f32; 3]>,
    pub distance: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PanelsConfig {
    pub displays_panel: PanelState,
    pub views_panel: PanelState,
    pub properties_panel: PanelState,
    pub selection_panel: PanelState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub visible: bool,
    pub collapsed: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            visible: true,
            collapsed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// Configuration file I/O
pub struct ConfigManager;

impl ConfigManager {
    pub fn load(path: &std::path::Path) -> Result<AppConfig, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(path: &std::path::Path, config: &AppConfig) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(serde_yaml::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(e: serde_yaml::Error) -> Self {
        ConfigError::Parse(e)
    }
}
```

---

## 9. Makepad UI Architecture [IMPLEMENTED]

### 9.1 Application Structure

**Actual Implementation**: `dviz-shell/src/app.rs`

The original design specified `rv-ui/src/app.rs`, but the actual implementation uses `dviz-shell/src/app.rs` with the following key features:

- **Light Theme**: Modern tinted light theme (LIGHT_BG #f0f4f8, PANEL_BG #f8fafc, etc.)
- **Three-Column Layout**: Fixed-width left/right panels (340px each) with center fill
- **Data Sources**: Simulator, Dora (legacy), and Zenoh (universal protocol)
- **ROS Bag Playback**: File dialog for loading .bag files with multi-sensor support
- **Window Configuration**: 1400x850 default size, titled "DViz - Robotics Visualizer"

### 9.1.1 Original Design Reference (`rv-ui/src/app.rs`)

```rust
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    // Main application window
    App = {{App}} {
        ui: <Window> {
            show_bg: true,
            width: Fill,
            height: Fill,

            draw_bg: {
                color: #2a2a2a
            }

            body = <View> {
                flow: Down,
                spacing: 0,

                // Toolbar
                toolbar = <Toolbar> {
                    height: 40,
                }

                // Main content area
                main_content = <View> {
                    flow: Right,
                    spacing: 0,

                    // Left panel (displays list)
                    left_panel = <DisplaysPanel> {
                        width: 250,
                    }

                    // Center (3D viewport / Rerun viewer)
                    viewport = <ViewportWidget> {
                        width: Fill,
                        height: Fill,
                    }

                    // Right panel (properties)
                    right_panel = <PropertiesPanel> {
                        width: 300,
                    }
                }

                // Bottom panel (timeline)
                timeline = <TimelineWidget> {
                    height: 100,
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,

    // Application state
    #[rust] display_manager: DisplayManager,
    #[rust] transform_buffer: rv_transform::TransformBuffer,
    #[rust] rerun_bridge: Option<rv_rerun::RerunBridge>,
    #[rust] config: rv_core::config::AppConfig,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::widgets::display_panel::live_design(cx);
        crate::widgets::properties_panel::live_design(cx);
        crate::widgets::viewport::live_design(cx);
        crate::widgets::toolbar::live_design(cx);
        crate::widgets::timeline::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Initialize Rerun bridge
        let config = rv_rerun::RerunConfig::default();
        match rv_rerun::RerunBridge::new(config) {
            Ok(bridge) => self.rerun_bridge = Some(bridge),
            Err(e) => log!("Failed to connect to Rerun: {:?}", e),
        }

        // Initialize transform buffer
        self.transform_buffer = rv_transform::TransformBuffer::new(
            rv_core::types::FrameId::new(&self.config.global_options.fixed_frame),
            std::time::Duration::from_secs(10),
        );

        // Load default displays
        self.load_default_displays();
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle toolbar actions
        if self.ui.button(id!(add_display_btn)).clicked(actions) {
            self.show_add_display_dialog(cx);
        }

        if self.ui.button(id!(save_config_btn)).clicked(actions) {
            self.save_configuration();
        }

        // Handle display selection
        // Handle property changes
        // etc.
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        // Periodic update for displays
        if let Event::Timer(_) = event {
            self.update_displays(cx);
        }
    }
}

impl App {
    fn load_default_displays(&mut self) {
        // Add grid display
        let grid = rv_displays::GridDisplay::new("Grid");
        self.display_manager.add(Box::new(grid));

        // Add TF display
        let tf = rv_displays::TfDisplay::new("TF");
        self.display_manager.add(Box::new(tf));
    }

    fn update_displays(&mut self, _cx: &mut Cx) {
        if let Some(bridge) = &self.rerun_bridge {
            let context = AppDisplayContext {
                fixed_frame: &rv_core::types::FrameId::new(&self.config.global_options.fixed_frame),
                transform_buffer: &self.transform_buffer,
                rerun_bridge: bridge,
            };

            let dt = std::time::Duration::from_millis(33);
            self.display_manager.update_all(&context, dt, dt);
        }
    }

    fn show_add_display_dialog(&mut self, _cx: &mut Cx) {
        // Show dialog to add new display
    }

    fn save_configuration(&self) {
        // Save current configuration
        let path = std::path::Path::new("config.rviz");
        if let Err(e) = rv_core::config::ConfigManager::save(path, &self.config) {
            log!("Failed to save config: {:?}", e);
        }
    }
}

app_main!(App);
```

### 9.2 Displays Panel Widget [IMPLEMENTED]

**Actual Implementation**: `dviz-widgets/src/displays_panel.rs`

The original design specified `rv-ui/src/widgets/display_panel.rs`. The actual implementation in `dviz-widgets/src/displays_panel.rs` provides:

- Display list with checkboxes for enable/disable
- Add Display button with display type cycling (Grid, Axes, PointCloud, LaserScan, TF)
- Selection events via `DisplaysPanelAction` enum
- Status indicators with light theme colors

### 9.2.1 Original Design Reference (`rv-ui/src/widgets/display_panel.rs`)

```rust
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    DisplaysPanel = {{DisplaysPanel}} {
        width: Fill,
        height: Fill,
        flow: Down,

        draw_bg: {
            color: #333333
        }

        // Header
        header = <View> {
            height: 30,
            flow: Right,
            padding: 5,

            <Label> {
                text: "Displays"
                draw_text: {
                    color: #ffffff
                    text_style: { font_size: 12 }
                }
            }

            <View> { width: Fill }

            add_btn = <Button> {
                text: "+"
                width: 24,
                height: 24,
            }
        }

        // Display list
        display_list = <PortalList> {
            width: Fill,
            height: Fill,

            DisplayItem = <DisplayListItem> {}
        }
    }

    DisplayListItem = {{DisplayListItem}} {
        width: Fill,
        height: 30,
        flow: Right,
        padding: { left: 10, right: 5 },
        align: { y: 0.5 },

        draw_bg: {
            color: #3a3a3a
        }

        enabled_checkbox = <CheckBox> {
            width: 20,
            height: 20,
        }

        name_label = <Label> {
            width: Fill,
            margin: { left: 5 },
            draw_text: {
                color: #ffffff
                text_style: { font_size: 11 }
            }
        }

        status_icon = <View> {
            width: 16,
            height: 16,
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DisplaysPanel {
    #[deref] view: View,

    #[rust] displays: Vec<DisplayInfo>,
    #[rust] selected_index: Option<usize>,
}

#[derive(Clone)]
struct DisplayInfo {
    id: u64,
    name: String,
    type_id: String,
    enabled: bool,
    status: rv_core::traits::display::StatusLevel,
}

impl Widget for DisplaysPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle display list interactions
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl DisplaysPanel {
    pub fn set_displays(&mut self, displays: Vec<DisplayInfo>) {
        self.displays = displays;
    }

    pub fn selected_display(&self) -> Option<u64> {
        self.selected_index.map(|i| self.displays[i].id)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DisplayListItem {
    #[deref] view: View,

    #[rust] display_info: Option<DisplayInfo>,
}

impl Widget for DisplayListItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update checkbox and label from display_info
        if let Some(info) = &self.display_info {
            // self.checkbox(id!(enabled_checkbox)).set_checked(info.enabled);
            // self.label(id!(name_label)).set_text(&info.name);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}
```

### 9.3 System Log Panel Widget [IMPLEMENTED]

**Actual Implementation**: `dviz-widgets/src/log_panel.rs`

The System Log Panel displays log messages from robot nodes over Zenoh, with dynamic node discovery and filtering.

#### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     System Log Panel                         │
├─────────────────────────────────────────────────────────────┤
│  [▼] System Log                    [12 entries] [Copy][Clear]│
├─────────────────────────────────────────────────────────────┤
│  Level: [All ▼]    Node: [All Nodes ▼]    Search: [____]    │
├─────────────────────────────────────────────────────────────┤
│  0.12s [INFO] [bicycle_model] Initialized with dt=0.02      │
│  0.15s [INFO] [simple_planner] Path computed: 42 waypoints  │
│  0.23s [WARN] [localization] GPS signal weak                │
│  1.05s [ERROR] [motor_ctrl] Overcurrent detected            │
│  ...                                                         │
└─────────────────────────────────────────────────────────────┘
```

#### Protocol (Zenoh)

Nodes publish log messages to `dviz/logs` topic using the universal message format:

```json
{
  "type": "log",
  "timestamp": 1.234,
  "data": {
    "level": "INFO",
    "message": "Motor initialized successfully",
    "node_id": "motor_controller",
    "metadata": {"motor_id": "left_wheel"}
  }
}
```

#### Core Types

```rust
// In dviz-core/src/zenoh_protocol.rs

/// Log level for system messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self;
    pub fn as_str(&self) -> &'static str;
    pub fn color(&self) -> [u8; 4];  // RGBA for UI display
}

/// System log entry from a dora node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub node_id: String,
    #[serde(default)]
    pub timestamp: f64,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Log data payload in MvizMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogData {
    pub level: String,
    pub message: String,
    pub node_id: String,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}
```

#### Zenoh Receiver Updates

```rust
// In dviz-shell/src/zenoh_receiver.rs

pub enum ZenohMessage {
    Data(VisData),
    Log(LogEntry),           // System log entry
    NodeDiscovered(String),  // New node ID discovered
    Connected,
    Disconnected(String),
    Status(String),
}

pub struct ZenohReceiver {
    // ... existing fields ...
    discovered_nodes: Arc<RwLock<HashSet<String>>>,  // Dynamic node tracking
}
```

#### Widget Implementation

```rust
use makepad_widgets::*;

live_design! {
    pub LogPanel = {{LogPanel}} <RoundedView> {
        width: Fill, height: 300
        flow: Down

        // Header with collapse, copy, clear buttons
        header = <View> {
            collapse_icon = <View> { /* triangle icon */ }
            <Label> { text: "System Log" }
            entry_count = <Label> { text: "0 entries" }
            copy_btn = <Button> { text: "Copy" }
            clear_btn = <Button> { text: "Clear" }
        }

        // Filter row
        filter_row = <View> {
            <Label> { text: "Level:" }
            level_filter = <DropDown> {
                labels: ["All", "Debug", "Info", "Warn", "Error"]
            }
            <Label> { text: "Node:" }
            node_filter = <DropDown> {
                labels: ["All Nodes"]  // Dynamically populated
            }
            <Label> { text: "Search:" }
            search_input = <TextInput> {}
        }

        // Scrollable log list
        log_scroll = <ScrollYView> {
            log_list = <View> { flow: Down }
        }
    }

    pub LogEntryItem = <View> {
        timestamp = <Label> {}
        level_badge = <View> { level_text = <Label> {} }
        node_name = <Label> {}
        message = <Label> {}
    }
}

/// Actions emitted by LogPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum LogPanelAction {
    None,
    CopyClicked,
    ClearClicked,
    ToggleCollapsed,
    LevelFilterChanged(usize),
    NodeFilterChanged(String),
    SearchChanged(String),
}

/// State for a log entry in the display
#[derive(Clone, Debug)]
pub struct LogDisplayEntry {
    pub timestamp: f64,
    pub level: u8,       // 0=debug, 1=info, 2=warn, 3=error
    pub level_str: String,
    pub node_id: String,
    pub message: String,
}

#[derive(Live, LiveHook, Widget)]
pub struct LogPanel {
    #[deref] view: View,
    #[rust] collapsed: bool,
    #[rust] entries: Vec<LogDisplayEntry>,
    #[rust] filtered_entries: Vec<usize>,
    #[rust] level_filter: usize,      // 0=all, 1-4=specific level
    #[rust] node_filter: String,      // "" = all nodes
    #[rust] search_text: String,
    #[rust] discovered_nodes: Vec<String>,
    #[rust] max_entries: usize,       // Default: 1000
}

impl LogPanel {
    pub fn add_entry(&mut self, cx: &mut Cx, entry: LogDisplayEntry);
    pub fn clear(&mut self, cx: &mut Cx);
    pub fn set_discovered_nodes(&mut self, cx: &mut Cx, nodes: Vec<String>);
    pub fn get_filtered_text(&self) -> String;
    fn apply_filters(&mut self);
}
```

#### Features

1. **Dynamic Node Discovery**: Nodes appear in the filter dropdown as they send logs
2. **Multi-level Filtering**: Filter by Debug/Info/Warn/Error levels
3. **Node Filtering**: Filter logs by specific node ID
4. **Text Search**: Search within log messages and node IDs
5. **Color Coding**: Visual distinction by log level (gray/blue/yellow/red)
6. **Collapsible**: Toggle panel visibility
7. **Copy to Clipboard**: Export filtered logs as text
8. **Auto-prune**: Maintains max 1000 entries to prevent memory issues

#### Dynamic Node Filter Implementation (v0.1.8)

The node filter dropdown dynamically updates when new nodes are discovered from the dataflow:

```rust
// In dviz-widgets/src/log_panel.rs

impl LogPanel {
    /// Updates the node filter dropdown with newly discovered nodes
    fn update_node_filter_dropdown(&mut self, cx: &mut Cx) {
        let mut labels = vec!["All Nodes".to_string()];
        labels.extend(self.discovered_nodes.clone());
        self.drop_down(id!(node_filter)).set_labels(cx, labels);
        self.redraw(cx);
    }
}

impl Widget for LogPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Capture actions from child widgets
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Handle level filter changes
        if let Some(index) = self.drop_down(id!(level_filter)).changed(&actions) {
            self.level_filter = index;
            self.apply_filters();
            self.update_log_content(cx);
            self.update_entry_count(cx);
            cx.widget_action(self.widget_uid(), &scope.path,
                LogPanelAction::LevelFilterChanged(index));
            self.redraw(cx);
        }

        // Handle node filter changes
        if let Some(index) = self.drop_down(id!(node_filter)).changed(&actions) {
            self.node_filter = if index == 0 {
                String::new()  // "All Nodes"
            } else {
                self.discovered_nodes.get(index - 1).cloned().unwrap_or_default()
            };
            self.apply_filters();
            self.update_log_content(cx);
            self.update_entry_count(cx);
            cx.widget_action(self.widget_uid(), &scope.path,
                LogPanelAction::NodeFilterChanged(self.node_filter.clone()));
            self.redraw(cx);
        }
    }
}
```

**Key Makepad APIs Used:**
- `DropDownRef.set_labels(cx, labels)` - Dynamically updates dropdown options at runtime
- `DropDownRef.changed(&actions) -> Option<usize>` - Returns selected index on change
- `cx.capture_actions()` - Captures widget actions for processing

#### Test Results (2026-01-09)

Verified with path-following dataflow running on robot side:

**Nodes Discovered:**
- `sim_pose` - Vehicle simulation pose publisher
- `bicycle_model` - Bicycle model dynamics node
- `sim_state` - Simulation state manager
- `target_point` - Target waypoint generator
- `imu_msg` - IMU sensor data publisher

**Log Entry Accumulation:**
```
Log entries: 1 → 51 → 101 → 151 → 201 → 251 → 301 → 351 → 401 → 451 → 501 → 551 → 601 → 651 → 701+
```

**Performance:**
- 57,000+ Zenoh messages processed
- Real-time node discovery and dropdown updates
- Immediate filter response on selection change

#### Integration in App

```rust
// In dviz-shell/src/app.rs

use dviz_widgets::{LogPanelAction, LogDisplayEntry, LogPanelWidgetRefExt};
use dviz_core::zenoh_protocol::LogLevel;

#[derive(Live, LiveHook)]
pub struct App {
    // ... existing fields ...
    #[rust] discovered_nodes: HashSet<String>,
    #[rust] log_entry_count: u64,
}

impl App {
    fn process_zenoh_messages(&mut self, cx: &mut Cx) {
        while let Some(msg) = receiver.try_recv() {
            match msg {
                ZenohMessage::Log(log_entry) => {
                    let display_entry = LogDisplayEntry {
                        timestamp: log_entry.timestamp,
                        level: match log_entry.level {
                            LogLevel::Debug => 0,
                            LogLevel::Info => 1,
                            LogLevel::Warn => 2,
                            LogLevel::Error => 3,
                        },
                        level_str: log_entry.level.as_str().to_string(),
                        node_id: log_entry.node_id.clone(),
                        message: log_entry.message,
                    };

                    // Update discovered nodes
                    if self.discovered_nodes.insert(log_entry.node_id.clone()) {
                        let nodes: Vec<String> = self.discovered_nodes.iter().cloned().collect();
                        self.ui.log_panel(id!(log_panel)).set_discovered_nodes(cx, nodes);
                    }

                    self.ui.log_panel(id!(log_panel)).add_entry(cx, display_entry);
                }
                ZenohMessage::NodeDiscovered(node_id) => {
                    if self.discovered_nodes.insert(node_id) {
                        let nodes: Vec<String> = self.discovered_nodes.iter().cloned().collect();
                        self.ui.log_panel(id!(log_panel)).set_discovered_nodes(cx, nodes);
                    }
                }
                // ... other message types
            }
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            match action.as_widget_action().cast::<LogPanelAction>() {
                LogPanelAction::CopyClicked => {
                    let text = self.ui.log_panel(id!(log_panel)).get_filtered_text();
                    cx.copy_to_clipboard(&text);
                }
                LogPanelAction::ClearClicked => {
                    self.ui.log_panel(id!(log_panel)).clear(cx);
                }
                _ => {}
            }
        }
    }
}
```

---

## 9.4 Node Detail Panel (Phase 7) [IMPLEMENTED]

**Status**: Implemented in `dviz-widgets/src/node_detail_panel.rs`

A dedicated panel for viewing detailed information about individual dataflow nodes, including their input/output connections and filtered logs.

### Overview

The Node Detail Panel shows detailed node information in the center panel:

1. **Node Header**: Selected node name with status indicator
2. **Input/Output Connections**: Two-column layout showing data flow
3. **Node Logs**: Filtered log entries specific to the selected node

```
┌─────────────────────────────────────────────────────────────────────┐
│ NODE: yolo_detector                                          [●]   │
├─────────────────────────────────────────────────────────────────────┤
│ INPUTS:                          │ OUTPUTS:                        │
│  • image (from: camera)          │  • boxes → [rerun, tracker]     │
│  • tick (from: dora/timer/100ms) │  • masks → [segmentation]       │
├─────────────────────────────────────────────────────────────────────┤
│ LOGS:                                                               │
│ [10:23:45.123] Processing frame 1842                                │
│ [10:23:45.156] Detected 3 objects                                   │
│ [10:23:45.189] ⚠ Inference took 45ms (threshold: 33ms)              │
└─────────────────────────────────────────────────────────────────────┘
```

### Data Model

#### Node Definition Protocol (dviz-core/src/zenoh_protocol.rs)

```rust
/// Input port definition for a node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInput {
    /// Input port name (e.g., "image", "tick")
    pub name: String,
    /// Source node and output (e.g., "camera/image", "dora/timer/100ms")
    pub source: String,
}

/// Output port definition for a node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeOutput {
    /// Output port name (e.g., "boxes", "masks")
    pub name: String,
    /// Destination nodes that subscribe to this output
    pub destinations: Vec<String>,
}

/// Complete node definition from dataflow YAML
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Node ID (e.g., "yolo_detector", "bicycle_model")
    pub id: String,
    /// Node type (python, rust, binary)
    pub node_type: String,
    /// Path to operator (Python script, Rust crate, or binary)
    pub operator_path: Option<String>,
    /// Input ports with sources
    pub inputs: Vec<NodeInput>,
    /// Output ports with destinations
    pub outputs: Vec<NodeOutput>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Status: running, stopped, error
    pub status: NodeStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Running,
    Stopped,
    Error(String),
    Unknown,
}

/// Message containing dataflow graph definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataflowDefinition {
    /// Dataflow name
    pub name: String,
    /// All nodes in the dataflow
    pub nodes: Vec<NodeDefinition>,
    /// Timestamp when published
    pub timestamp: f64,
}
```

#### Zenoh Message Extension

```rust
// In dviz-shell/src/zenoh_receiver.rs

pub enum ZenohMessage {
    Data(VisData),
    Log(LogEntry),
    NodeDiscovered(String),
    DataflowDefinition(DataflowDefinition),  // NEW: Full dataflow graph
    NodeStatusUpdate(String, NodeStatus),     // NEW: Node status changes
    Connected,
    Disconnected(String),
    Status(String),
}
```

### Bridge Updates (dviz-rerun-bridge/src/main.rs)

The bridge should publish the dataflow definition on startup:

```rust
impl MvizBridge {
    /// Parse dataflow YAML and publish node definitions
    fn publish_dataflow_definition(&self, yaml_path: &str) -> Result<()> {
        let contents = std::fs::read_to_string(yaml_path)?;
        let dataflow: serde_yaml::Value = serde_yaml::from_str(&contents)?;

        let mut nodes = Vec::new();

        if let Some(yaml_nodes) = dataflow.get("nodes").and_then(|n| n.as_sequence()) {
            for node in yaml_nodes {
                let id = node.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Parse inputs
                let inputs = self.parse_node_inputs(node);

                // Parse outputs
                let outputs = self.parse_node_outputs(node, yaml_nodes);

                // Determine node type
                let node_type = if node.get("operator").is_some() {
                    if node["operator"].get("python").is_some() {
                        "python".to_string()
                    } else {
                        "rust".to_string()
                    }
                } else if node.get("path").is_some() {
                    "binary".to_string()
                } else {
                    "unknown".to_string()
                };

                nodes.push(NodeDefinition {
                    id,
                    node_type,
                    operator_path: self.get_operator_path(node),
                    inputs,
                    outputs,
                    env: self.parse_env(node),
                    status: NodeStatus::Running,
                });
            }
        }

        let definition = DataflowDefinition {
            name: yaml_path.to_string(),
            nodes,
            timestamp: get_timestamp(),
        };

        // Publish to dviz/dataflow/definition topic
        let json = serde_json::to_string(&definition)?;
        self.session.put("dviz/dataflow/definition", json).await?;

        Ok(())
    }

    fn parse_node_inputs(&self, node: &serde_yaml::Value) -> Vec<NodeInput> {
        let mut inputs = Vec::new();

        // Check operator inputs
        if let Some(op) = node.get("operator") {
            if let Some(input_map) = op.get("inputs").and_then(|i| i.as_mapping()) {
                for (name, source) in input_map {
                    inputs.push(NodeInput {
                        name: name.as_str().unwrap_or("").to_string(),
                        source: source.as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }

        // Check top-level inputs (for binary nodes)
        if let Some(input_map) = node.get("inputs").and_then(|i| i.as_mapping()) {
            for (name, source) in input_map {
                inputs.push(NodeInput {
                    name: name.as_str().unwrap_or("").to_string(),
                    source: source.as_str().unwrap_or("").to_string(),
                });
            }
        }

        inputs
    }

    fn parse_node_outputs(&self, node: &serde_yaml::Value, all_nodes: &[serde_yaml::Value]) -> Vec<NodeOutput> {
        let mut outputs = Vec::new();
        let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");

        // Get output names
        let output_names: Vec<String> = if let Some(op) = node.get("operator") {
            op.get("outputs")
                .and_then(|o| o.as_sequence())
                .map(|seq| seq.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Find destinations for each output
        for output_name in output_names {
            let mut destinations = Vec::new();
            let source_pattern = format!("{}/{}", node_id, output_name);

            // Search all nodes for inputs matching this output
            for other_node in all_nodes {
                let other_id = other_node.get("id").and_then(|v| v.as_str()).unwrap_or("");

                // Check operator inputs
                if let Some(op) = other_node.get("operator") {
                    if let Some(input_map) = op.get("inputs").and_then(|i| i.as_mapping()) {
                        for (_name, source) in input_map {
                            if source.as_str() == Some(&source_pattern) {
                                destinations.push(other_id.to_string());
                            }
                        }
                    }
                }

                // Check top-level inputs
                if let Some(input_map) = other_node.get("inputs").and_then(|i| i.as_mapping()) {
                    for (_name, source) in input_map {
                        if source.as_str() == Some(&source_pattern) {
                            destinations.push(other_id.to_string());
                        }
                    }
                }
            }

            outputs.push(NodeOutput {
                name: output_name,
                destinations,
            });
        }

        outputs
    }
}
```

### Widget Implementation (dviz-widgets/src/node_detail_panel.rs)

```rust
use makepad_widgets::*;
use std::collections::HashMap;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Node detail panel - replaces center panel content
    pub NodeDetailPanel = {{NodeDetailPanel}} <RoundedView> {
        width: Fill, height: Fill
        flow: Down
        padding: 16
        spacing: 12
        show_bg: true
        draw_bg: { color: #1e1e1e, border_radius: 8.0 }

        // Header with node selector
        header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 12
            align: {y: 0.5}

            <Label> {
                text: "NODE:"
                draw_text: { color: #888888, text_style: { font_size: 12.0 } }
            }

            node_selector = <DropDown> {
                width: 200, height: 28
                labels: ["Select Node..."]
            }

            <View> { width: Fill, height: 1 }

            status_indicator = <View> {
                width: 12, height: 12
                show_bg: true
                draw_bg: { color: #22c55e, border_radius: 6.0 }
            }

            status_label = <Label> {
                text: "Running"
                draw_text: { color: #22c55e, text_style: { font_size: 11.0 } }
            }
        }

        // Separator
        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: #333333 }
        }

        // Input/Output columns
        io_section = <View> {
            width: Fill, height: 150
            flow: Right
            spacing: 16

            // Inputs column
            inputs_column = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 6

                <Label> {
                    text: "INPUTS:"
                    draw_text: { color: #fbbf24, text_style: { font_size: 11.0 } }
                }

                inputs_list = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    spacing: 4
                }
            }

            // Vertical separator
            <View> {
                width: 1, height: Fill
                show_bg: true
                draw_bg: { color: #333333 }
            }

            // Outputs column
            outputs_column = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 6

                <Label> {
                    text: "OUTPUTS:"
                    draw_text: { color: #60a5fa, text_style: { font_size: 11.0 } }
                }

                outputs_list = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    spacing: 4
                }
            }
        }

        // Separator
        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: #333333 }
        }

        // Logs section header
        logs_header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            align: {y: 0.5}

            <Label> {
                text: "LOGS:"
                draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
            }

            log_count = <Label> {
                text: "0 entries"
                draw_text: { color: #606060, text_style: { font_size: 10.0 } }
            }

            <View> { width: Fill, height: 1 }

            clear_logs_btn = <Button> {
                width: Fit, height: 24
                padding: {left: 8, right: 8}
                text: "Clear"
                draw_text: { color: #888 }
            }
        }

        // Scrollable logs area
        logs_scroll = <ScrollYView> {
            width: Fill, height: Fill

            logs_content = <Label> {
                width: Fill, height: Fit
                text: "No logs for this node"
                draw_text: {
                    color: #707070
                    text_style: { font_size: 10.0, font: {path: dep("crate://makepad-widgets/resources/IBMPlexMono-Regular.ttf")} }
                    wrap: Word
                }
            }
        }
    }

    // Individual I/O port item
    pub IOPortItem = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 4

        bullet = <Label> {
            text: "•"
            draw_text: { color: #606060, text_style: { font_size: 10.0 } }
        }

        port_name = <Label> {
            text: "port"
            draw_text: { color: #ffffff, text_style: { font_size: 10.0 } }
        }

        connection = <Label> {
            text: "(from: source)"
            draw_text: { color: #888888, text_style: { font_size: 10.0 } }
        }
    }
}

/// Actions emitted by NodeDetailPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum NodeDetailPanelAction {
    None,
    NodeSelected(String),
    ClearLogsClicked,
}

/// Display state for a node
#[derive(Clone, Debug)]
pub struct NodeDisplayState {
    pub id: String,
    pub node_type: String,
    pub status: NodeStatus,
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutput>,
}

#[derive(Live, LiveHook, Widget)]
pub struct NodeDetailPanel {
    #[deref] view: View,

    /// All known nodes from dataflow
    #[rust] nodes: HashMap<String, NodeDisplayState>,

    /// Currently selected node ID
    #[rust] selected_node: Option<String>,

    /// Filtered logs for selected node
    #[rust] node_logs: Vec<LogDisplayEntry>,

    /// Maximum logs to keep per node
    #[rust] max_logs: usize,
}

impl NodeDetailPanel {
    /// Set dataflow definition (called when received from Zenoh)
    pub fn set_dataflow(&mut self, cx: &mut Cx, definition: &DataflowDefinition) {
        self.nodes.clear();

        let mut node_labels = vec!["Select Node...".to_string()];

        for node_def in &definition.nodes {
            self.nodes.insert(node_def.id.clone(), NodeDisplayState {
                id: node_def.id.clone(),
                node_type: node_def.node_type.clone(),
                status: node_def.status.clone(),
                inputs: node_def.inputs.clone(),
                outputs: node_def.outputs.clone(),
            });
            node_labels.push(node_def.id.clone());
        }

        // Update dropdown
        self.drop_down(id!(node_selector)).set_labels(cx, node_labels);
        self.redraw(cx);
    }

    /// Add a node from discovery (without full definition)
    pub fn add_discovered_node(&mut self, cx: &mut Cx, node_id: String) {
        if !self.nodes.contains_key(&node_id) {
            self.nodes.insert(node_id.clone(), NodeDisplayState {
                id: node_id.clone(),
                node_type: "unknown".to_string(),
                status: NodeStatus::Unknown,
                inputs: Vec::new(),
                outputs: Vec::new(),
            });

            // Update dropdown labels
            let mut labels: Vec<String> = vec!["Select Node...".to_string()];
            labels.extend(self.nodes.keys().cloned());
            self.drop_down(id!(node_selector)).set_labels(cx, labels);
            self.redraw(cx);
        }
    }

    /// Add a log entry (will be filtered to selected node)
    pub fn add_log(&mut self, cx: &mut Cx, entry: LogDisplayEntry) {
        // Only keep logs for currently selected node to save memory
        if let Some(ref selected) = self.selected_node {
            if entry.node_id == *selected {
                self.node_logs.push(entry);

                // Prune if too many
                if self.node_logs.len() > self.max_logs {
                    self.node_logs.remove(0);
                }

                self.update_logs_display(cx);
            }
        }
    }

    /// Clear logs for current node
    pub fn clear_logs(&mut self, cx: &mut Cx) {
        self.node_logs.clear();
        self.update_logs_display(cx);
    }

    /// Update node status
    pub fn update_node_status(&mut self, cx: &mut Cx, node_id: &str, status: NodeStatus) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.status = status;

            if self.selected_node.as_deref() == Some(node_id) {
                self.update_status_display(cx);
            }
        }
    }

    fn update_io_display(&mut self, cx: &mut Cx) {
        let Some(ref node_id) = self.selected_node else {
            self.label(id!(inputs_list)).set_text(cx, "No node selected");
            self.label(id!(outputs_list)).set_text(cx, "No node selected");
            return;
        };

        let Some(node) = self.nodes.get(node_id) else {
            return;
        };

        // Build inputs text
        let inputs_text = if node.inputs.is_empty() {
            "  (no inputs)".to_string()
        } else {
            node.inputs.iter()
                .map(|input| format!("  • {} (from: {})", input.name, input.source))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Build outputs text
        let outputs_text = if node.outputs.is_empty() {
            "  (no outputs)".to_string()
        } else {
            node.outputs.iter()
                .map(|output| {
                    let dests = if output.destinations.is_empty() {
                        "[]".to_string()
                    } else {
                        format!("[{}]", output.destinations.join(", "))
                    };
                    format!("  • {} → {}", output.name, dests)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Update labels (using inputs_list/outputs_list as Labels for simplicity)
        // In full implementation, these would be dynamic views
        self.label(id!(inputs_list.content)).set_text(cx, &inputs_text);
        self.label(id!(outputs_list.content)).set_text(cx, &outputs_text);
    }

    fn update_status_display(&mut self, cx: &mut Cx) {
        let Some(ref node_id) = self.selected_node else { return };
        let Some(node) = self.nodes.get(node_id) else { return };

        let (status_text, status_color) = match &node.status {
            NodeStatus::Running => ("Running", "#22c55e"),
            NodeStatus::Stopped => ("Stopped", "#888888"),
            NodeStatus::Error(msg) => ("Error", "#ef4444"),
            NodeStatus::Unknown => ("Unknown", "#fbbf24"),
        };

        self.label(id!(status_label)).set_text(cx, status_text);
        // Note: Color would need to be set via draw_bg in live_design or custom shader
    }

    fn update_logs_display(&mut self, cx: &mut Cx) {
        if self.node_logs.is_empty() {
            self.label(id!(logs_content)).set_text(cx, "No logs for this node");
            self.label(id!(log_count)).set_text(cx, "0 entries");
        } else {
            let logs_text = self.node_logs.iter()
                .rev()  // Newest first
                .take(100)  // Limit display
                .map(|entry| {
                    let level_prefix = match entry.level {
                        0 => "   ",  // Debug
                        1 => "   ",  // Info
                        2 => " ⚠ ",  // Warn
                        3 => " ✖ ",  // Error
                        _ => "   ",
                    };
                    format!("[{:.3}]{}{}", entry.timestamp, level_prefix, entry.message)
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.label(id!(logs_content)).set_text(cx, &logs_text);
            self.label(id!(log_count)).set_text(cx, &format!("{} entries", self.node_logs.len()));
        }
    }
}

impl Widget for NodeDetailPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Handle node selection
        if let Some(index) = self.drop_down(id!(node_selector)).changed(&actions) {
            if index == 0 {
                // "Select Node..." - deselect
                self.selected_node = None;
                self.node_logs.clear();
            } else {
                // Get node ID from index
                let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
                if let Some(node_id) = node_ids.get(index - 1) {
                    self.selected_node = Some(node_id.clone());
                    self.node_logs.clear();  // Clear logs when switching nodes
                    cx.widget_action(self.widget_uid(), &scope.path,
                        NodeDetailPanelAction::NodeSelected(node_id.clone()));
                }
            }

            self.update_io_display(cx);
            self.update_status_display(cx);
            self.update_logs_display(cx);
            self.redraw(cx);
        }

        // Handle clear logs button
        if self.button(id!(clear_logs_btn)).clicked(&actions) {
            self.clear_logs(cx);
            cx.widget_action(self.widget_uid(), &scope.path,
                NodeDetailPanelAction::ClearLogsClicked);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
```

### App Integration (dviz-shell/src/app.rs)

```rust
// Update center panel in live_design!
center_panel = <NodeDetailPanel> {
    width: Fill, height: Fill
}

// In App struct
#[derive(Live, LiveHook)]
pub struct App {
    // ... existing fields ...
    #[rust] dataflow_definition: Option<DataflowDefinition>,
}

// In process_zenoh_messages()
ZenohMessage::DataflowDefinition(definition) => {
    debug_log(&format!("Received dataflow definition: {} nodes", definition.nodes.len()));
    self.dataflow_definition = Some(definition.clone());
    self.ui.node_detail_panel(id!(center_panel)).set_dataflow(cx, &definition);
}

ZenohMessage::NodeStatusUpdate(node_id, status) => {
    self.ui.node_detail_panel(id!(center_panel))
        .update_node_status(cx, &node_id, status);
}

ZenohMessage::Log(log_entry) => {
    // ... existing log panel code ...

    // Also send to node detail panel
    let display_entry = LogDisplayEntry { /* ... */ };
    self.ui.node_detail_panel(id!(center_panel)).add_log(cx, display_entry.clone());
}
```

### Zenoh Topics

| Topic | Direction | Content |
|-------|-----------|---------|
| `dviz/dataflow/definition` | Bridge → Shell | Full dataflow graph JSON |
| `dviz/node/{node_id}/status` | Bridge → Shell | Node status updates |
| `dviz/logs` | Bridge → Shell | Log entries (existing) |

### Features

1. **Node Selection**: Dropdown populated from discovered nodes or dataflow definition
2. **Input Display**: Shows each input port with its source (node/output)
3. **Output Display**: Shows each output port with its destination nodes
4. **Status Indicator**: Color-coded status (green=running, yellow=unknown, red=error)
5. **Filtered Logs**: Only logs from selected node are displayed
6. **Clear Logs**: Button to clear logs for current node
7. **Real-time Updates**: Logs stream in as they arrive from Zenoh

### Example Dataflow Visualization

For `dataflow-path-following.yml`:

```
Node: bicycle_model
┌─────────────────────────────────────────────────────────────────────┐
│ INPUTS:                          │ OUTPUTS:                        │
│  • tick (from: dora/timer/20ms)  │  • sim_pose → [simple_planner,  │
│  • steering_cmd (from:           │                dviz_bridge]     │
│      simple_planner/steering)    │  • sim_state → [imu_synthesizer,│
│  • throttle_cmd (from:           │                 dviz_bridge]    │
│      simple_planner/throttle)    │                                 │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 10. URDF Integration [IMPLEMENTED]

**Status**: Implemented in `dviz-urdf/` crate and `dviz-displays/src/robot_model.rs`

### 10.1 URDF Loader (`dviz-urdf/src/parser.rs`) [IMPLEMENTED]

```rust
use std::path::Path;
use std::collections::HashMap;
use glam::{Vec3, Quat};
use rv_core::types::{FrameId, Transform};

/// Robot description parsed from URDF
pub struct RobotDescription {
    pub name: String,
    pub links: HashMap<String, Link>,
    pub joints: HashMap<String, Joint>,
    pub root_link: Option<String>,
}

pub struct Link {
    pub name: String,
    pub visual: Option<Visual>,
    pub collision: Option<Collision>,
    pub inertial: Option<Inertial>,
}

pub struct Visual {
    pub origin: Transform,
    pub geometry: Geometry,
    pub material: Option<Material>,
}

pub struct Collision {
    pub origin: Transform,
    pub geometry: Geometry,
}

pub struct Inertial {
    pub origin: Transform,
    pub mass: f32,
    pub inertia: [f32; 6], // ixx, ixy, ixz, iyy, iyz, izz
}

pub enum Geometry {
    Box { size: Vec3 },
    Cylinder { radius: f32, length: f32 },
    Sphere { radius: f32 },
    Mesh { filename: String, scale: Vec3 },
}

pub struct Material {
    pub name: String,
    pub color: Option<[f32; 4]>,
    pub texture: Option<String>,
}

pub struct Joint {
    pub name: String,
    pub joint_type: JointType,
    pub parent_link: String,
    pub child_link: String,
    pub origin: Transform,
    pub axis: Vec3,
    pub limits: Option<JointLimits>,
}

pub enum JointType {
    Fixed,
    Revolute,
    Continuous,
    Prismatic,
    Floating,
    Planar,
}

pub struct JointLimits {
    pub lower: f32,
    pub upper: f32,
    pub effort: f32,
    pub velocity: f32,
}

/// Parse URDF from file
pub fn parse_urdf(path: &Path) -> Result<RobotDescription, UrdfError> {
    let urdf_robot = urdf_rs::read_file(path)?;
    convert_urdf(urdf_robot)
}

/// Parse URDF from string
pub fn parse_urdf_string(urdf: &str) -> Result<RobotDescription, UrdfError> {
    let urdf_robot = urdf_rs::read_from_string(urdf)?;
    convert_urdf(urdf_robot)
}

fn convert_urdf(urdf: urdf_rs::Robot) -> Result<RobotDescription, UrdfError> {
    let mut links = HashMap::new();
    let mut joints = HashMap::new();

    // Convert links
    for link in urdf.links {
        let visual = link.visual.first().map(|v| {
            Visual {
                origin: convert_origin(&v.origin),
                geometry: convert_geometry(&v.geometry),
                material: v.material.as_ref().map(convert_material),
            }
        });

        let collision = link.collision.first().map(|c| {
            Collision {
                origin: convert_origin(&c.origin),
                geometry: convert_geometry(&c.geometry),
            }
        });

        let inertial = link.inertial.as_ref().map(|i| {
            Inertial {
                origin: convert_origin(&i.origin),
                mass: i.mass.value as f32,
                inertia: [
                    i.inertia.ixx as f32,
                    i.inertia.ixy as f32,
                    i.inertia.ixz as f32,
                    i.inertia.iyy as f32,
                    i.inertia.iyz as f32,
                    i.inertia.izz as f32,
                ],
            }
        });

        links.insert(link.name.clone(), Link {
            name: link.name,
            visual,
            collision,
            inertial,
        });
    }

    // Convert joints
    for joint in urdf.joints {
        joints.insert(joint.name.clone(), Joint {
            name: joint.name,
            joint_type: convert_joint_type(&joint.joint_type),
            parent_link: joint.parent.link,
            child_link: joint.child.link,
            origin: convert_origin(&joint.origin),
            axis: Vec3::new(
                joint.axis.xyz[0] as f32,
                joint.axis.xyz[1] as f32,
                joint.axis.xyz[2] as f32,
            ),
            limits: joint.limit.as_ref().map(|l| JointLimits {
                lower: l.lower as f32,
                upper: l.upper as f32,
                effort: l.effort as f32,
                velocity: l.velocity as f32,
            }),
        });
    }

    // Find root link (link with no parent joint)
    let child_links: std::collections::HashSet<_> = joints.values()
        .map(|j| &j.child_link)
        .collect();

    let root_link = links.keys()
        .find(|name| !child_links.contains(*name))
        .cloned();

    Ok(RobotDescription {
        name: urdf.name,
        links,
        joints,
        root_link,
    })
}

fn convert_origin(origin: &urdf_rs::Pose) -> Transform {
    let translation = Vec3::new(
        origin.xyz[0] as f32,
        origin.xyz[1] as f32,
        origin.xyz[2] as f32,
    );

    let rotation = Quat::from_euler(
        glam::EulerRot::XYZ,
        origin.rpy[0] as f32,
        origin.rpy[1] as f32,
        origin.rpy[2] as f32,
    );

    Transform::new(translation, rotation)
}

fn convert_geometry(geom: &urdf_rs::Geometry) -> Geometry {
    match geom {
        urdf_rs::Geometry::Box { size } => Geometry::Box {
            size: Vec3::new(size[0] as f32, size[1] as f32, size[2] as f32),
        },
        urdf_rs::Geometry::Cylinder { radius, length } => Geometry::Cylinder {
            radius: *radius as f32,
            length: *length as f32,
        },
        urdf_rs::Geometry::Sphere { radius } => Geometry::Sphere {
            radius: *radius as f32,
        },
        urdf_rs::Geometry::Mesh { filename, scale } => Geometry::Mesh {
            filename: filename.clone(),
            scale: scale.map(|s| Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32))
                .unwrap_or(Vec3::ONE),
        },
    }
}

fn convert_material(mat: &urdf_rs::Material) -> Material {
    Material {
        name: mat.name.clone(),
        color: mat.color.as_ref().map(|c| {
            [c.rgba[0] as f32, c.rgba[1] as f32, c.rgba[2] as f32, c.rgba[3] as f32]
        }),
        texture: mat.texture.as_ref().map(|t| t.filename.clone()),
    }
}

fn convert_joint_type(jt: &urdf_rs::JointType) -> JointType {
    match jt {
        urdf_rs::JointType::Fixed => JointType::Fixed,
        urdf_rs::JointType::Revolute => JointType::Revolute,
        urdf_rs::JointType::Continuous => JointType::Continuous,
        urdf_rs::JointType::Prismatic => JointType::Prismatic,
        urdf_rs::JointType::Floating => JointType::Floating,
        urdf_rs::JointType::Planar => JointType::Planar,
    }
}

#[derive(Debug)]
pub enum UrdfError {
    ParseError(String),
    IoError(std::io::Error),
}

impl From<urdf_rs::UrdfError> for UrdfError {
    fn from(e: urdf_rs::UrdfError) -> Self {
        UrdfError::ParseError(e.to_string())
    }
}
```

---

## 11. Cargo Dependencies

### 11.1 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/rv-core",
    "crates/rv-transform",
    "crates/rv-rerun",
    "crates/rv-displays",
    "crates/rv-tools",
    "crates/rv-views",
    "crates/rv-urdf",
    "crates/rv-ui",
    "apps/robotics-viz",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/robotics-viz"

[workspace.dependencies]
# Math
glam = { version = "0.25", features = ["serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# Async
tokio = { version = "1", features = ["full"] }

# Logging
log = "0.4"
env_logger = "0.11"

# Synchronization
parking_lot = "0.12"

# Rerun
rerun = "0.19"

# URDF
urdf-rs = "0.8"

# Makepad (use git for latest)
makepad-widgets = { git = "https://github.com/makepad/makepad", branch = "main" }

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

### 11.2 Example Crate Cargo.toml (`rv-core/Cargo.toml`)

```toml
[package]
name = "rv-core"
version.workspace = true
edition.workspace = true

[dependencies]
glam = { workspace = true }
serde = { workspace = true }
serde_yaml = { workspace = true }
parking_lot = { workspace = true }
thiserror = { workspace = true }
```

### 11.3 Rerun Crate Cargo.toml (`rv-rerun/Cargo.toml`)

```toml
[package]
name = "rv-rerun"
version.workspace = true
edition.workspace = true

[dependencies]
rv-core = { path = "../rv-core" }
rv-transform = { path = "../rv-transform" }
rerun = { workspace = true }
glam = { workspace = true }
parking_lot = { workspace = true }
thiserror = { workspace = true }
```

---

## 12. Summary

This design provides:

1. **Modular Architecture**: Clean separation between core types, transforms, visualization, and UI
2. **Rerun Integration**: Complete bridge layer mapping RViz concepts to Rerun archetypes
3. **Makepad UI**: Native Rust UI with declarative DSL
4. **Extensible Plugin System**: Trait-based displays, tools, and view controllers
5. **RViz-Compatible Features**: Transform hierarchy, displays, markers, URDF support
6. **Configuration Persistence**: YAML-based save/load compatible with RViz concepts

The architecture prioritizes:
- **Performance**: Using `glam` for SIMD math, Rerun for efficient visualization
- **Type Safety**: Rust's type system for compile-time correctness
- **Extensibility**: Trait-based plugins for custom displays and tools
- **Cross-Platform**: Makepad's multi-platform rendering + Rerun's web support

---

## 13. Makepad + Rerun Integration Options

Two primary approaches for integrating Rerun visualization with Makepad UI:

### Option 1: Embedded Rerun Viewer in Makepad

Embed Rerun's 3D viewport as a widget within the Makepad application window.

```
┌─────────────────────────────────────────────────────────────────┐
│                     Single Makepad Window                        │
│  ┌──────────┬────────────────────────────┬──────────────────┐   │
│  │          │                            │                  │   │
│  │ Displays │    Embedded Rerun          │   Properties     │   │
│  │  Panel   │    3D Viewport             │     Panel        │   │
│  │          │    (egui/wgpu texture)     │                  │   │
│  │          │                            │                  │   │
│  └──────────┴────────────────────────────┴──────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                      Timeline                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

#### Implementation Approach
```rust
// Embed Rerun viewer as texture in Makepad
struct EmbeddedRerunViewport {
    rerun_memory_sink: MemorySinkStorage,
    egui_ctx: egui::Context,
    render_texture: Texture,
}

impl EmbeddedRerunViewport {
    fn render_to_texture(&mut self) {
        // Render Rerun viewer to offscreen buffer
        // Copy to Makepad texture
        // Display in Makepad Image widget
    }
}
```

#### Pros

| Advantage | Description |
|-----------|-------------|
| **Unified UX** | Single window, consistent look and feel |
| **Seamless Interaction** | Mouse/keyboard events handled in one place |
| **Tight Integration** | Direct access to Rerun's scene graph for picking |
| **Professional Appearance** | Looks like a polished, integrated application |
| **Window Management** | No need to manage multiple windows |
| **Deployment** | Single executable, simpler distribution |
| **State Synchronization** | Easier to keep UI and visualization in sync |

#### Cons

| Disadvantage | Description |
|--------------|-------------|
| **High Complexity** | Must integrate egui (Rerun's UI) with Makepad's rendering |
| **Rendering Conflicts** | Two GPU frameworks (Makepad + Rerun/wgpu) may conflict |
| **Limited Rerun Features** | Cannot use Rerun's native panels, timeline, blueprints |
| **Maintenance Burden** | Must track Rerun internal API changes |
| **Performance Overhead** | Texture copy between renderers adds latency |
| **Memory Usage** | Double-buffering for texture sharing |
| **Debugging Difficulty** | Harder to debug rendering issues |
| **Rerun Updates** | New Rerun features may not work in embedded mode |

#### Technical Challenges
1. **egui-Makepad Bridge**: Rerun uses egui; need to render egui to texture or reimplement viewer
2. **Input Forwarding**: Translate Makepad events to egui/Rerun events
3. **GPU Context Sharing**: Share wgpu context between Makepad and Rerun
4. **Rerun Viewer API**: `rerun_viewer` crate's public API is limited for embedding

---

### Option 2: Separate Rerun Window with Makepad Control Panel

Run Rerun Viewer as a separate window; Makepad provides UI controls and sends data to Rerun.

```
┌────────────────────────────┐      ┌────────────────────────────┐
│     Makepad Window         │      │     Rerun Viewer Window    │
│  ┌──────────────────────┐  │      │  ┌──────────────────────┐  │
│  │    Displays Panel    │  │      │  │                      │  │
│  ├──────────────────────┤  │      │  │    3D Space View     │  │
│  │   Properties Panel   │  │◀────▶│  │                      │  │
│  ├──────────────────────┤  │ IPC  │  │                      │  │
│  │    Toolbar/Tools     │  │      │  ├──────────────────────┤  │
│  ├──────────────────────┤  │      │  │  Rerun Timeline      │  │
│  │      Timeline        │  │      │  └──────────────────────┘  │
│  └──────────────────────┘  │      └────────────────────────────┘
└────────────────────────────┘
```

#### Implementation Approach
```rust
// Makepad app sends data to Rerun via RecordingStream
struct RerunConnection {
    stream: RecordingStream,
}

impl RerunConnection {
    fn new() -> Self {
        // Spawn Rerun viewer as separate process
        let stream = RecordingStreamBuilder::new("robotics-viz")
            .spawn()  // Launches Rerun Viewer window
            .unwrap();
        Self { stream }
    }

    fn log_point_cloud(&self, path: &str, cloud: &PointCloud) {
        // Send data to Rerun viewer
        self.stream.log(path, &to_rerun_points(cloud)).unwrap();
    }
}
```

#### Pros

| Advantage | Description |
|-----------|-------------|
| **Simple Integration** | Just use Rerun SDK's logging API |
| **Full Rerun Features** | All Rerun features work: blueprints, timeline, selection |
| **Stable API** | Uses official, stable RecordingStream API |
| **Independent Updates** | Rerun viewer updates don't break your app |
| **Multi-Monitor** | Users can put 3D view on second monitor |
| **Development Speed** | Faster to implement, less code to maintain |
| **Rerun Web Support** | Can use Rerun's web viewer for remote viewing |
| **Debugging** | Can debug each component independently |
| **Process Isolation** | Crash in viewer doesn't crash control panel |

#### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Split UX** | Users must manage two windows |
| **Coordination** | Need IPC for selection sync, camera control |
| **Inconsistent Look** | Different UI frameworks = different appearance |
| **Window Focus** | Input focus switching can be confusing |
| **Selection Sync** | Complex to synchronize selections between windows |
| **Launch Complexity** | Must ensure Rerun viewer is installed/running |
| **Latency** | IPC adds some latency for interactions |
| **Deployment** | Users need Rerun viewer installed separately |

#### Technical Challenges
1. **Selection Synchronization**: Clicking in Rerun should update Makepad properties panel
2. **Camera Control**: Tools in Makepad need to control Rerun camera (limited API)
3. **Process Management**: Handle Rerun viewer launch, restart, connection loss
4. **Bidirectional Communication**: Rerun → Makepad events (selection, hover) not well supported

---

### Comparison Matrix

| Criteria | Option 1: Embedded | Option 2: Separate | Winner |
|----------|-------------------|-------------------|--------|
| **Implementation Time** | 3-4 months | 2-4 weeks | Option 2 |
| **Maintenance Effort** | High | Low | Option 2 |
| **User Experience** | Excellent | Good | Option 1 |
| **Feature Completeness** | Limited | Full Rerun | Option 2 |
| **Performance** | Good (with optimization) | Excellent | Option 2 |
| **Debugging** | Difficult | Easy | Option 2 |
| **Cross-Platform** | Complex | Native support | Option 2 |
| **Future Compatibility** | Risky | Stable | Option 2 |
| **Professional Polish** | Higher | Lower | Option 1 |
| **Multi-Monitor Support** | Limited | Excellent | Option 2 |

---

### Recommended Approach: Hybrid Strategy

**Phase 1: Start with Option 2 (Separate Windows)**
- Get working product quickly
- Full Rerun feature access
- Learn integration patterns

**Phase 2: Evaluate Embedded Option**
- After core features stable
- If Rerun releases embedding API
- If UX feedback demands single window

```
Timeline:
┌─────────────────────────────────────────────────────────────────┐
│ Month 1-3: Option 2 (Separate Windows)                          │
│ - Full functionality                                            │
│ - User feedback                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Month 4-6: Evaluate & Decide                                    │
│ - Assess user feedback                                          │
│ - Check Rerun embedding API progress                            │
│ - Prototype embedded if viable                                  │
├─────────────────────────────────────────────────────────────────┤
│ Month 7+: Option 1 if justified                                 │
│ - Only if strong UX need                                        │
│ - Rerun provides stable embedding API                           │
└─────────────────────────────────────────────────────────────────┘
```

---

### Option 2 Implementation Details

#### Architecture for Separate Window Approach

```rust
// Main application structure
pub struct RoboticsVizApp {
    // Makepad UI
    ui: MakepadUI,

    // Rerun connection (separate viewer)
    rerun: RerunConnection,

    // Shared state
    displays: DisplayManager,
    transforms: TransformBuffer,
    config: AppConfig,
}

// Rerun connection management
pub struct RerunConnection {
    stream: RecordingStream,
    viewer_process: Option<Child>,
}

impl RerunConnection {
    pub fn connect_or_spawn() -> Result<Self> {
        // Try to connect to existing viewer
        if let Ok(stream) = RecordingStreamBuilder::new("robotics-viz")
            .connect_tcp()
        {
            return Ok(Self { stream, viewer_process: None });
        }

        // Spawn new viewer
        let stream = RecordingStreamBuilder::new("robotics-viz")
            .spawn()?;

        Ok(Self { stream, viewer_process: None })
    }

    pub fn ensure_connected(&mut self) -> Result<()> {
        // Reconnect if connection lost
        // ...
    }
}
```

#### Selection Synchronization (Future Rerun Feature)

```rust
// Hypothetical future API for selection sync
impl RerunConnection {
    // Subscribe to selection changes in Rerun viewer
    pub fn on_selection_changed<F>(&self, callback: F)
    where F: Fn(Vec<EntityPath>) + Send + 'static
    {
        // When Rerun adds selection events...
        // self.stream.subscribe_selection(callback);
    }

    // Workaround: Poll selection via RPC (if available)
    pub fn poll_selection(&self) -> Option<Vec<EntityPath>> {
        // Not currently supported by Rerun
        None
    }
}
```

#### Camera Control (Limited)

```rust
// Current limitation: Cannot directly control Rerun camera
// Workaround: Use Rerun blueprints to set initial view

impl RerunConnection {
    pub fn set_initial_view(&self, eye: Vec3, target: Vec3, up: Vec3) {
        // Use blueprint API (when available) or
        // Log a "camera hint" entity that Rerun could interpret
        self.stream.log_static(
            "/_camera_hint",
            &rerun::ViewCoordinates::RUB, // Right-Up-Back
        ).ok();
    }
}
```

---

### Decision Checklist

Choose **Option 1 (Embedded)** if:
- [ ] Single-window UX is critical requirement
- [ ] You have 3+ months for integration work
- [ ] Team has egui/wgpu experience
- [ ] Willing to maintain custom Rerun integration
- [ ] Limited Rerun features are acceptable

Choose **Option 2 (Separate)** if:
- [ ] Need working product quickly
- [ ] Want full Rerun feature set
- [ ] Team is small or has limited Rust graphics experience
- [ ] Multi-monitor support is valuable
- [ ] Long-term maintenance cost is a concern

**Recommendation**: Start with Option 2 for MVP, evaluate Option 1 for v2.0 if user feedback demands unified window experience.

---

## 9.5 Dataflow Graph Widget (Phase 8) [IMPLEMENTED v0.3.1-v0.3.2]

### Overview

Embedded visualization of the Dora dataflow graph directly in the Makepad UI. The graph is inferred dynamically from message flow patterns via Zenoh without requiring YAML files on the PC side.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Bridge (Robot Side)                             │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ GraphState                                                       ││
│  │  - nodes: HashMap<String, GraphNodeState>                        ││
│  │  - input_source_map: HashMap<(node, input), (source, output)>    ││
│  │  - infer edges from input patterns (source_node/output_port)     ││
│  │  - publish graph_update every 2 seconds via Zenoh                ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼ Zenoh (dviz/graph_update)
┌─────────────────────────────────────────────────────────────────────┐
│                      Shell (PC Side)                                 │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ DataflowGraphWidget                                              ││
│  │  - nodes: Vec<GraphDisplayNode>                                  ││
│  │  - edges: Vec<GraphDisplayEdge>                                  ││
│  │  - ASCII-style text rendering with box characters               ││
│  │  - Click detection for node selection                            ││
│  │  - Scrollable canvas with ScrollBars                             ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### Protocol Types (dviz-core/src/zenoh_protocol.rs)

```rust
pub enum GraphNodeStatus { Active, Idle, Error }

pub struct GraphNode {
    pub id: String,
    pub status: GraphNodeStatus,
    pub last_seen: f64,
}

pub struct GraphEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

pub struct GraphUpdate {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub timestamp: f64,
}
```

### Display Format

```
     bicycle_model / sim_pose
           ↓
  ┌───────────────────────────┐
  │ 🟢 simple_planner [RUN] ◀ │
  └───────────────────────────┘
     ↓ 2 output(s)

━━━ 3 nodes, 2 edges ━━━
Updated: 2.1s ago
```

### Status Indicators

| Icon | Text | Meaning |
|------|------|---------|
| 🟢 | [RUN] | Node is actively processing |
| ⚪ | [---] | Node is idle |
| ◀ | | Currently selected node |

### Widget Features

- **Scrollable canvas**: ScrollBars for large graphs
- **Grid background**: Subtle grid pattern via shader
- **Click detection**: Select nodes by clicking
- **Dynamic updates**: Real-time graph updates via Zenoh
- **Hierarchical layout**: compute_layout() positions nodes by dependency level

---

## 9.6 UI Layout Configuration (Phase 9) [IMPLEMENTED v0.3.4-v0.3.7]

### Three-Column Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ Toolbar (44px)                                                       │
├──────────────┬─────────────────────────────┬────────────────────────┤
│ Left Panel   │ Center Panel                │ Right Panel            │
│ (340px)      │ (flexible)                  │ (340px)                │
│              │                             │                        │
│ DisplaysPanel│ NodeDetailPanel             │ PropertiesPanel        │
│ (height:300) │ (fills remaining)           │ (height:200)           │
│              │                             │                        │
│ DataflowGraph│                             │ LogPanel               │
│ (scrollable) │                             │ (fills remaining)      │
│              │                             │                        │
│ StatusLabel  │                             │                        │
├──────────────┴─────────────────────────────┴────────────────────────┤
│ Dividers: 1px #333 lines between panels                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Panel Configuration

| Panel | Width | Background | Contents |
|-------|-------|------------|----------|
| Left | 340px | #1e1e1e | DisplaysPanel, DataflowGraphWidget, StatusLabel |
| Center | Fill | #1a1a1a | NodeDetailPanel |
| Right | 340px | #1e1e1e | PropertiesPanel, LogPanel |

### Implementation Notes

1. **Splitter Widget Issue**: Makepad's Splitter widget has rendering bugs (only divider shows, children don't render). Using fixed-width Views instead.

2. **Scrollable Graph**: DataflowGraphWidget uses `scroll_bars: <ScrollBars> {}` for scrolling when graph content exceeds visible area.

3. **Divider Lines**: 1px Views with `show_bg: true` and `draw_bg: { color: #333 }` between panels.

### live_design! Structure

```rust
content = <View> {
    width: Fill, height: Fill
    flow: Right
    spacing: 0

    left_panel = <View> {
        width: 340, height: Fill
        flow: Down
        // ... DisplaysPanel, DataflowGraphWidget, StatusLabel
    }

    <View> { width: 1, height: Fill, show_bg: true, draw_bg: { color: #333 } }

    center_panel = <View> {
        width: Fill, height: Fill
        // ... NodeDetailPanel
    }

    <View> { width: 1, height: Fill, show_bg: true, draw_bg: { color: #333 } }

    right_panel = <View> {
        width: 340, height: Fill
        flow: Down
        // ... PropertiesPanel, LogPanel
    }
}
```

---

## 9.7 ROS Bag Multi-Sensor Visualization (Phase 10.3) [IMPLEMENTED v0.3.12]

### Overview

Extended ROS bag playback to visualize all sensor types with proper Rerun entity hierarchy and time synchronization.

### Entity Hierarchy

```
world/
├── lidar/                    ← PointCloud2 (velodyne_points)
│   └── Points3D
├── imu/                      ← sensor_msgs/Imu
│   ├── accel_arrow/          Arrows3D (cyan, linear acceleration)
│   ├── gyro_arrow/           Arrows3D (orange, angular velocity)
│   ├── accel_x, accel_y, accel_z   Scalars (time series)
│   └── gyro_x, gyro_y, gyro_z      Scalars (time series)
├── gps/                      ← nmea_msgs/Sentence
│   ├── position/             Points3D (green, GPS position)
│   ├── status/               TextLog (NMEA sentences)
│   ├── latitude, longitude, altitude  Scalars
│   └── satellites            Scalar
├── time_ref/                 ← sensor_msgs/TimeReference
│   ├── offset/               Scalar (time offset from GPS)
│   └── status/               TextLog
└── temperature/              ← sensor_msgs/Temperature
    ├── celsius/              Scalar (time series)
    └── status/               TextLog
```

### Message Parsers

#### IMU Parser (imu.rs)

```rust
pub struct ImuData {
    pub timestamp: f64,
    pub orientation: [f64; 4],          // quaternion (x, y, z, w)
    pub angular_velocity: [f64; 3],     // rad/s
    pub linear_acceleration: [f64; 3],  // m/s^2
}

pub struct ImuProcessor {
    pub frame_id: String,
    pub timestamp: f64,
}

impl ImuProcessor {
    pub fn parse(&mut self, data: &[u8]) -> Result<ImuData>;
}
```

ROS1 sensor_msgs/Imu layout:
- header (seq: 4, stamp: 8, frame_id: 4+len)
- orientation (4 × 8 bytes = 32 bytes)
- orientation_covariance (9 × 8 = 72 bytes, skipped)
- angular_velocity (3 × 8 = 24 bytes)
- angular_velocity_covariance (72 bytes, skipped)
- linear_acceleration (3 × 8 = 24 bytes)
- linear_acceleration_covariance (72 bytes, skipped)

#### GPS/NMEA Parser (gps.rs)

```rust
pub struct NmeaSentence {
    pub timestamp: f64,
    pub sentence: String,
    pub sentence_type: String,  // e.g., "GPGGA", "GPRMC"
}

pub struct GpsPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub fix_quality: u8,
    pub num_satellites: u8,
    pub hdop: f32,
}

pub struct TimeReference {
    pub timestamp: f64,
    pub source: String,
    pub time_ref: f64,
}

pub struct Temperature {
    pub timestamp: f64,
    pub temperature: f64,
    pub variance: f64,
}

pub struct GpsProcessor {
    pub frame_id: String,
    pub timestamp: f64,
    pub last_position: Option<GpsPosition>,
}

impl GpsProcessor {
    pub fn parse_nmea(&mut self, data: &[u8]) -> Result<NmeaSentence>;
    pub fn parse_time_reference(&mut self, data: &[u8]) -> Result<TimeReference>;
    pub fn parse_temperature(&mut self, data: &[u8]) -> Result<Temperature>;
}
```

NMEA GPGGA parsing extracts:
- Latitude/Longitude from fields 2-5 (ddmm.mmmmm format)
- Fix quality from field 6
- Number of satellites from field 7
- HDOP from field 8
- Altitude from field 9

### Rerun Visualization

#### IMU Visualization

```rust
fn process_imu(&mut self, msg: &BagMessage) -> Result<()> {
    let imu_data = self.imu_processor.parse(&msg.data)?;
    
    // Angular velocity as orange arrow
    stream.log("world/imu/gyro_arrow",
        &rerun::Arrows3D::from_vectors([[av[0], av[1], av[2]]])
            .with_colors([Color::from_rgb(255, 165, 0)])
            .with_origins([[0.0, 0.0, 0.0]]));
    
    // Linear acceleration as cyan arrow
    stream.log("world/imu/accel_arrow",
        &rerun::Arrows3D::from_vectors([[la[0], la[1], la[2]]])
            .with_colors([Color::from_rgb(0, 255, 255)])
            .with_origins([[0.0, 0.0, 0.0]]));
    
    // Scalar time series for each axis
    stream.log("world/imu/accel_x", &rerun::Scalars::new([la[0]]));
    stream.log("world/imu/gyro_x", &rerun::Scalars::new([av[0]]));
    // ... etc
}
```

#### GPS Visualization

```rust
fn process_nmea(&mut self, msg: &BagMessage) -> Result<()> {
    let nmea = self.gps_processor.parse_nmea(&msg.data)?;
    
    // Log NMEA sentence as text
    stream.log("world/gps/status",
        &rerun::TextLog::new(format!("[{}] {}", nmea.sentence_type, nmea.sentence)));
    
    // If position available, log as 3D point
    if let Some(pos) = self.gps_processor.last_position() {
        // Convert lat/lon to meters (rough approximation)
        let x = pos.longitude * 111320.0 * (pos.latitude.to_radians().cos());
        let y = pos.latitude * 110540.0;
        let z = pos.altitude;
        
        stream.log("world/gps/position",
            &rerun::Points3D::new([[x, y, z]])
                .with_colors([Color::from_rgb(0, 255, 0)])
                .with_radii([0.5]));
        
        // Log lat/lon/alt as scalars
        stream.log("world/gps/latitude", &rerun::Scalars::new([pos.latitude]));
        stream.log("world/gps/longitude", &rerun::Scalars::new([pos.longitude]));
    }
}
```

### Message Type Mapping

| ROS Type | MessageType | Rerun Types |
|----------|-------------|-------------|
| sensor_msgs/PointCloud2 | PointCloud2 | Points3D |
| sensor_msgs/Imu | Imu | Arrows3D, Scalars |
| nmea_msgs/Sentence | NmeaSentence | Points3D, TextLog, Scalars |
| sensor_msgs/TimeReference | TimeReference | Scalars, TextLog |
| sensor_msgs/Temperature | Temperature | Scalars, TextLog |
| tf2_msgs/TFMessage | TfMessage | (internal TfBuffer) |

### Time Synchronization

All messages use `bag_time` timeline:
```rust
let bag_time_ms = ((msg.timestamp - self.start_time) * 1000.0) as i64;
stream.set_time_sequence("bag_time", bag_time_ms);
```

This ensures Rerun displays all sensor data synchronized to bag playback time.
