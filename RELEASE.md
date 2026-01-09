# MViz Release Notes

## v0.1.8 (2026-01-09)

### Enhancement: Dynamic Node Filter Dropdown

- DropDown labels now dynamically update when new nodes are discovered
- Uses Makepad's `DropDownRef.set_labels()` API for runtime updates
- Filter selection triggers immediate log content refresh
- Level and node filter changes handled via `cx.capture_actions()`

---

## v0.1.7 (2026-01-09)

Phase 6: System Log Panel for Distributed Robotics Debugging

### New Feature: System Log Panel

Real-time log collection and display from dora dataflow nodes over LAN via Zenoh.

#### LogPanel Widget (mviz-widgets/src/log_panel.rs)

- Collapsible panel with entry count display
- Filter by log level (Debug, Info, Warn, Error)
- Filter by node (dynamically populated via Zenoh discovery)
- Text search across messages
- Copy to clipboard and Clear buttons
- Color-coded log entries
- Scrollable log content with newest entries first

#### Protocol Extensions (mviz-core/zenoh_protocol.rs)

- `LogLevel` enum: Debug, Info, Warn, Error with color() method
- `LogEntry` struct: level, message, node_id, timestamp, metadata
- `LogData` struct: JSON payload for log messages

#### Bridge Updates (mviz-rerun-bridge/src/main.rs)

- `publish_log()` helper function for sending log messages
- Bridge startup/shutdown log messages
- Node message count tracking with periodic status logs
- Vehicle state updates logged every 50 frames
- First message notification per source node

#### Zenoh Receiver Updates (mviz-shell/src/zenoh_receiver.rs)

- `ZenohMessage::Log(LogEntry)` - system log entry
- `ZenohMessage::NodeDiscovered(String)` - new node ID
- `discovered_nodes: Arc<RwLock<HashSet<String>>>` - dynamic tracking

#### App Integration (mviz-shell/src/app.rs)

- Log entries processed in `process_zenoh_messages()`
- Node discovery tracking with HashSet
- LogPanel actions: Copy, Clear, ToggleCollapsed

### Documentation

- Added Section 9.3 to mviz_design.md with full architecture
- Added Task 4.5 to mviz_plan.md with acceptance criteria
- Updated dependency graph

---

## v0.1.5 (2026-01-08)

Phase 5: Zenoh Universal Protocol for LAN Visualization

### Architecture

Distributed robotics visualization via Zenoh pub/sub:
- **Robot side**: Dora dataflow with mviz-bridge publishes via Zenoh
- **PC side**: mviz-shell receives via Zenoh, displays in Rerun

### New Crate: mviz-rerun-bridge (Dora Node)

Universal Dora node that publishes ANY sensor data via Zenoh.

#### Supported Data Types
- `points3d` - Point clouds (binary xyz_f32 format)
- `boxes3d` - 3D boxes with quaternion rotation
- `arrows3d` - Arrow vectors (IMU, velocity)
- `linestrips3d` - Line strips (trajectories, paths)
- `transform3d` - Coordinate transforms (4x4 matrix or translation+quaternion)
- `scalar` - Time-series values

#### Environment Variables
- `ZENOH_CONNECT` - Zenoh router address (default: auto-discovery)
- `ZENOH_TOPIC_PREFIX` - Topic prefix (default: "mviz")

### New Module: mviz-core/zenoh_protocol.rs

Universal message format for Zenoh communication:
- JSON header + optional binary payload
- `MvizMessage` struct with type, timestamp, data, format, count
- Serialization/deserialization utilities

### New Module: mviz-shell/zenoh_receiver.rs

Universal Zenoh subscriber for PC-side visualization:
- Subscribes to `{prefix}/**` wildcard topics
- Parses universal message format
- Sends typed `VisData` to UI thread

### Enhanced: mviz-shell/app.rs

- Zenoh connection button for LAN data reception
- Universal message handler (`log_vis_data_to_rerun`)
- Trajectory accumulation for `sim_pose` and `odom_pose`
- LineStrips3D with 0.03 radius for visible trajectory lines

### New Dataflow Configurations

- `dataflow-path-following.yml` - Vehicle path following with Zenoh bridge
- `dataflow-mapping.yml` - Vehicle mapping with point cloud
- `dataflow-robot.yml` - Generic robot dataflow
- `dataflow.yml` - Base dataflow template

### Usage

```bash
# Robot side (headless)
dora start dataflow-path-following.yml --name pathfollow

# PC side (with display)
cargo run -p mviz-shell
# Click "Spawn Rerun" then "Connect Zenoh"
```

---

## v0.1.4 (2026-01-08)

Phase 4: URDF Robot Model Loading with Rerun Built-in Data Loader

### New Features

#### URDF Robot Model Display
- Integrated Rerun's built-in URDF data loader (introduced in Rerun 0.24)
- Load robot models from URDF files with proper mesh rendering
- SO-ARM100 robot arm as test model with STL mesh files
- Robot displayed under `world/robot/*` entity hierarchy for proper 3D space integration

#### Add Display Button
- Fixed "Add Display" button functionality in DisplaysPanel
- Cycles through display types: Grid, Axes, PointCloud, LaserScan, TF
- Each click adds a different visualization to Rerun viewer

#### Test Buttons
- "Test Laser" - Displays 360-degree laser scan visualization
- "Test Robot" - Loads SO-ARM100 robot arm from URDF with STL meshes

### New Files
- `so100.urdf` - SO-ARM100 robot arm URDF description
- `assets/*.stl` - 13 STL mesh files for robot visualization
- `car.urdf` - Simple car model with wheels, body, sensors
- `mviz-displays/src/laser_scan.rs` - Laser scan simulation and display
- `mviz-displays/src/robot_model.rs` - Robot model display plugin

### Technical Changes
- DisplaysPanelAction enum for widget event handling
- Entity paths use `world/robot/*` prefix for proper Rerun coordinate space
- Uses `RecordingStream::log_file_from_path()` for URDF loading

---

## v0.1.3 (2026-01-07)

Phase 2: Display Plugins + Phase 3: Makepad UI Shell

### New Crate: mviz-displays

Display plugin system with visualization types.

#### Display Types
- **BaseDisplay** - Common display functionality with property system
- **GridDisplay** - Ground plane grid (configurable size, cell count, color)
- **AxesDisplay** - Coordinate axes visualization (RGB for XYZ)
- **PointCloudDisplay** - Point cloud with color modes (Flat, RGB, Intensity, Axis)
- **MarkerDisplay** - All marker types with lifetime management
- **TfDisplay** - Transform frame tree visualization

#### Features
- Property system integration (PropertyValue, PropertyMeta, Properties)
- DisplayUpdateContext for transform lookups and Rerun logging
- Marker lifetime management with automatic expiration

### Enhanced Crate: mviz-widgets

New UI widgets for the control panel.

#### DisplaysPanel
- List of visualization displays with icons
- Enable/disable checkboxes per display
- Status indicators (OK, Warning, Error)
- Add display button

#### PropertiesPanel
Property editors for selected display:
- BoolProperty (checkbox)
- FloatProperty (slider with value label)
- StringProperty (text input)
- ColorProperty (color swatch + RGB display)
- Vec3Property (X, Y, Z inputs with color labels)
- EnumProperty (dropdown)

#### Toolbar
Enhanced application toolbar:
- Frame selector dropdown
- PlayPauseButton with animated play/pause icon
- StepButton for frame stepping (forward/backward)
- SpeedSelector dropdown
- TimeDisplay

### Enhanced App: mviz-shell

Three-column layout application:
- **Left Panel**: DisplaysPanel + IMU/Vehicle sensor panels
- **Center Panel**: Rerun viewer info with simulation stats
- **Right Panel**: PropertiesPanel for display configuration

### Tests
- 29 new tests in mviz-displays
- All key functions validated:
  - App launches successfully
  - Launch Rerun spawns viewer
  - Play/Pause simulation works
  - Sensor data updates at 50Hz
  - Data streams to Rerun viewer

---

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
