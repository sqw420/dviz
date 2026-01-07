# MViz Release Notes

## v0.1.2 (2026-01-06)

Phase 1 Streams B+C: Transform System and Rerun Core Adapters.

### New Crate: mviz-transform

Transform system for coordinate frame management, similar to ROS TF.

#### Frame Tree
- `FrameNode` - Node in frame tree with parent/children relationships
- `FrameTree` - Coordinate frame hierarchy with path finding
  - `set_parent()` - Set frame parent relationship
  - `path_to_root()` - Find path from any frame to root
  - `common_ancestor()` - Find common ancestor of two frames
  - `path_between()` - Find transform path between any two frames
  - `remove()` - Remove frame with orphan handling

#### Transform Buffer
- `TransformKey` - (parent, child) frame pair identifier
- `TransformHistory` - Time-indexed transforms with linear interpolation
- `TransformBuffer` - Thread-safe transform buffer
  - `set_transform()` - Store transform at timestamp
  - `lookup_transform()` - Look up transform between any two frames
  - `prune_old()` - Remove transforms older than duration
- `TransformError` - NoPath, TransformNotFound, FrameNotFound, EmptyBuffer

### Enhanced Crate: mviz-rerun-bridge

Core adapters for logging mviz-core types to Rerun viewer.

#### PointCloudCoreAdapter
- Log `PointCloud` to Rerun with color modes:
  - `FlatColor` - Single color for all points
  - `RGB` - Per-point colors from point cloud data
  - `AxisColor` - Color by axis position (X, Y, Z) with colormap
  - `Intensity` - Color by intensity value with colormap
- Configurable point radius

#### TransformCoreAdapter
- Log `Transform` to Rerun as Transform3D
- Log coordinate frame axes (RGB for XYZ)
- Log all frames in a frame tree

#### MarkerCoreAdapter
- Log all `Marker` types to Rerun:
  - Arrow, Cube, Sphere, Cylinder
  - LineStrip, LineList
  - CubeList, SphereList
  - Points, Text, TriangleList
- Log `MarkerArray` collections
- Per-marker colors and per-vertex colors supported

### Tests
- 21 new tests in mviz-transform
- 3 new tests in mviz-rerun-bridge core adapters
- 71 total tests passing across workspace

---

## v0.1.1 (2026-01-06)

Phase 1: Core Foundation - Added mviz-core crate with foundational types.

### New Crate: mviz-core

Core types and traits for the MViz robotics visualizer.

#### Transform Types
- `FrameId` - Coordinate frame identifier
- `Timestamp` - Nanosecond precision timestamps
- `Transform` - 3D rigid transform (translation + quaternion)
- `StampedTransform` - Transform with timestamp and frame IDs
- `Pose` - Position and orientation in a frame

#### Point Cloud Types
- `Color` - RGBA color with constants and HSV conversion
- `Colormap` - 8 colormaps (Jet, Rainbow, Turbo, Viridis, etc.)
- `ColorMode` - Flat, PerPoint, ByAxis, ByIntensity, ByHeight, ByDistance
- `PointCloud` - Points with positions, colors, intensities
- `PointCloudStyle` - Point size, colormap, alpha settings

#### Marker Types
- `MarkerType` - Arrow, Cube, Sphere, Cylinder, LineStrip, LineList, etc.
- `Marker` - Full marker with all properties
- `MarkerBuilder` - Fluent builder pattern
- `MarkerArray` - Collection of markers

#### Display System
- `Display` trait - Base trait for visualization displays
- `DisplayContext` - Context for accessing visualization services
- `DisplayFactory` trait - Factory for creating display instances
- `DisplayRegistry` - Registry of display factories
- `TransformProvider` trait - Coordinate frame transform lookup
- `Properties` - Dynamic property system

#### Configuration
- `AppConfig` - Complete application configuration
- `WindowConfig`, `ViewConfig`, `GlobalConfig`
- `DisplayConfig`, `PanelConfig`
- YAML serialization/deserialization

### Tests
- 44 unit tests passing

---

## v0.1.0 (2026-01-07)

Initial release of MViz - a visualization tool combining Makepad UI with Rerun 3D viewer.

### Features
- **Makepad UI Framework**: Native desktop application with responsive controls
- **Rerun SDK 0.28 Integration**: 3D visualization via spawned Rerun viewer
- **Simulated Sensor Data**:
  - IMU (accelerometer, gyroscope)
  - LiDAR point cloud (1000 points)
  - Vehicle pose with figure-8 trajectory
- **Real-time Streaming**: 50Hz data update rate
- **Interactive Controls**:
  - "Launch Rerun Viewer" button
  - "Start/Stop Simulation" button
  - Space key shortcut for simulation toggle

### Visualization
- Vehicle body (blue 3D box)
- Path history (yellow trail)
- LiDAR point cloud (height-colored)
- IMU vectors (acceleration, angular velocity)
- Ground grid reference

### Project Structure
```
mviz/
├── mviz-shell/          # Main application
├── mviz-widgets/        # Custom UI widgets
├── mviz-rerun-bridge/   # Rerun SDK integration
└── resources/           # Icons and assets
```

### Build
```bash
cargo build --target aarch64-apple-darwin
cargo run --target aarch64-apple-darwin
```

### Requirements
- Rust 2021 edition
- Rerun CLI v0.28+ (`pip install rerun-sdk`)
- macOS (tested on Darwin 24.4.0)
