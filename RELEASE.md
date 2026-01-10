# MViz Release Notes

## v0.2.5 (2026-01-09)

### Enhancement: Debug Logging and Unit Tests for I/O Activity

Added comprehensive debug logging and unit tests to trace and verify the I/O activity data flow from bridge to UI.

#### Changes

**mviz-core/src/zenoh_protocol.rs:**
- Added 3 unit tests for LogData serialization:
  - `test_log_data_with_port_info` - verifies I/O activity logs parse correctly
  - `test_log_data_without_port_info` - verifies regular logs work without port fields
  - `test_full_mviz_message_with_log` - tests full serialize/parse round-trip

**mviz-shell/src/zenoh_receiver.rs:**
- Added detailed debug logging for log message parsing:
  - Logs raw `msg.data` JSON for each log message
  - Logs parsed `LogData` fields: node_id, port, port_type, has_port_info

**mviz-shell/src/app.rs:**
- Added debug logging for I/O activity routing:
  - Logs when I/O activity is detected and routed to NodeDetailPanel
  - Logs metadata keys for regular logs (to diagnose missing port info)

#### Debug Output Location

Debug output written to:
- Terminal: `[mviz]` prefixed lines
- File: `/tmp/mviz_zenoh_debug.log`

To check I/O activity flow:
```bash
grep -E "Log msg.data|LogData parsed|I/O activity" /tmp/mviz_zenoh_debug.log
```

---

## v0.2.4 (2026-01-09)

### Feature: Live I/O Activity Display in NodeDetailPanel

Changed the NodeDetailPanel's INPUTS and OUTPUTS sections from showing static schema definitions to displaying live message data flowing through ports.

#### Changes

**mviz-rerun-bridge/src/main.rs:**
- Added `publish_io_activity()` - publishes I/O activity logs with port information
- Added `format_values_summary()` - formats float arrays as summary strings for display
- Bridge now publishes I/O activity for:
  - Its own inputs (showing data received from upstream nodes)
  - Source node outputs (showing data the source node emitted)

**mviz-core/src/zenoh_protocol.rs:**
- Added `port: Option<String>` field to `LogData` - identifies which port the message relates to
- Added `port_type: Option<String>` field to `LogData` - "input" or "output"

**mviz-widgets/src/node_detail_panel.rs:**
- Added `IoActivityEntry` struct - timestamp, port_name, data_summary
- Added `input_activity: VecDeque<IoActivityEntry>` to `NodeDisplayState`
- Added `output_activity: VecDeque<IoActivityEntry>` to `NodeDisplayState`
- Added `add_io_activity()` method - routes I/O activity to correct node and port
- Updated `update_io_display()` - now shows live messages instead of definitions

**mviz-shell/src/zenoh_receiver.rs:**
- Extended log entry parsing to extract `port` and `port_type` from JSON data
- Port info now stored in `LogEntry.metadata` for routing in app.rs

**mviz-shell/src/app.rs:**
- Log handler now checks for port info in metadata
- Routes I/O activity logs to `add_io_activity()` instead of regular log panels
- Regular logs (without port info) still go to LogPanel

#### Display Format

Before (v0.2.3):
```
INPUTS:                         OUTPUTS:
• tick (from: dora/timer/...)   • sim_pose -> [simple_planner, ...]
```

After (v0.2.4):
```
INPUTS:                         OUTPUTS:
[0.12] tick: [0.02, ...]        [0.12] sim_pose: [1.23, 4.56, ...]
[0.14] steering_cmd: [0.05]     [0.14] sim_state: [1.23, 4.56, ...]
```

Live data flows in real-time showing the actual values being transmitted through each port.

---

## v0.2.3 (2026-01-09)

### Fix: Add DATAFLOW_PATH to Bridge Environment

The bridge couldn't find the dataflow YAML file because Dora runs nodes from a different working directory. Added `DATAFLOW_PATH` environment variable to the dataflow configuration.

**dataflow-path-following.yml:**
```yaml
- id: mviz_bridge
  env:
    DATAFLOW_PATH: /Users/nupylot/Public/mviz/dataflow-path-following.yml
```

---

## v0.2.2 (2026-01-09)

### Fix: Node Definition Timing Issue

Fixed issue where NodeDetailPanel showed "(definition not available)" even when definitions were being published. The root cause was that node definitions were only published once at bridge startup, which could miss late-joining Zenoh subscribers.

#### Changes

**mviz-rerun-bridge/src/main.rs:**
- Refactored into separate functions:
  - `parse_dataflow_definitions()` - parses YAML, returns `Vec<NodeDefinition>`
  - `publish_node_definitions()` - publishes definitions to Zenoh
- Node definitions now stored and republished every 3 seconds
- Late-joining subscribers (like mviz-shell) now receive definitions reliably
- Removed duplicate legacy function code

#### Technical Details

The original implementation used `session.put()` once at startup:
```rust
// Old: One-shot publish - late subscribers miss it
publish_dataflow_definitions(&session, &prefix, &path).await;
```

The fix stores definitions and republishes periodically:
```rust
// New: Periodic republish for late-joining subscribers
let node_definitions = parse_dataflow_definitions(&path);
publish_node_definitions(&session, &prefix, &node_definitions).await;

while let Some(event) = events.recv() {
    if last_def_publish.elapsed() >= Duration::from_secs(3) {
        publish_node_definitions(&session, &prefix, &node_definitions).await;
        last_def_publish = Instant::now();
    }
    // ... handle events
}
```

---

## v0.2.1 (2026-01-09)

### Feature: Node Definition Publishing from Dataflow YAML

The bridge now parses the dataflow YAML and publishes node definitions via Zenoh, enabling the NodeDetailPanel to display actual input/output ports for each node.

#### Bridge Changes (mviz-rerun-bridge/src/main.rs)

- Added `publish_dataflow_definitions()` - parses dataflow YAML and publishes node definitions
- Parses both operator-style and direct inputs/outputs from YAML
- Computes output destinations by searching for nodes that consume each output
- Publishes to `{prefix}/definitions/{node_id}` topics
- Auto-discovers dataflow YAML from `DATAFLOW_PATH` env or common paths

#### Protocol Changes (mviz-core/src/zenoh_protocol.rs)

- Added `NodeInputDef` struct: port name and source reference
- Added `NodeOutputDef` struct: port name and destination list
- Added `NodeDefinition` struct: complete node with inputs/outputs
- Added `DataflowDefinition` struct: full dataflow graph

#### Receiver Changes (mviz-shell/src/zenoh_receiver.rs)

- Added `ZenohMessage::NodeDef(NodeDefinition)` variant
- Handles `node_definition` message type from bridge
- Tracks nodes from definitions in discovered_nodes

#### App Integration (mviz-shell/src/app.rs)

- Handles `ZenohMessage::NodeDef` in `process_zenoh_messages()`
- Converts protocol types to widget types (NodeInput, NodeOutput)
- Calls `set_node_definition()` on NodeDetailPanel

#### Result

When the bridge parses a dataflow like:
```yaml
nodes:
  - id: bicycle_model
    operator:
      inputs:
        tick: dora/timer/millis/20
        steering_cmd: simple_planner/steering_cmd
      outputs:
        - sim_pose
        - sim_state
```

The NodeDetailPanel will now show:
```
INPUTS:                          OUTPUTS:
• tick (from: dora/timer/...)    • sim_pose -> [simple_planner, mviz_bridge]
• steering_cmd (from: ...)       • sim_state -> [imu_synthesizer, ...]
```

---

## v0.2.0 (2026-01-09)

### Implementation: Node Detail Panel (Phase 7)

Replaced center panel static text with an interactive Node Detail Panel for dataflow node inspection.

#### New Widget: NodeDetailPanel (mviz-widgets/src/node_detail_panel.rs)

- **Node Selector**: DropDown populated dynamically from discovered nodes
- **I/O Display**: Two-column layout showing inputs (yellow) and outputs (blue)
- **Filtered Logs**: Shows only logs from the selected node
- **Status Indicator**: Visual indicator showing Ready/Active state
- **Clear Button**: Clear logs for current node

#### Widget Architecture

```rust
#[derive(Live, LiveHook, Widget)]
pub struct NodeDetailPanel {
    view: View,
    nodes: Vec<NodeDisplayState>,
    selected_node: Option<String>,
    node_logs: Vec<LogDisplayEntry>,
    all_logs: Vec<LogDisplayEntry>,
    discovered_nodes: Vec<String>,
}
```

#### Key Methods

- `add_discovered_node()` - Add node to dropdown
- `set_discovered_nodes()` - Bulk update node list
- `add_log()` - Add log entry (filtered by node)
- `set_node_definition()` - Set input/output ports
- `filter_logs_for_node()` - Filter stored logs on node selection

#### App Integration (mviz-shell/src/app.rs)

- Center panel now uses `<NodeDetailPanel>` instead of static text
- Log entries routed to both LogPanel and NodeDetailPanel
- Node discovery updates both panels
- NodeDetailPanelAction handling for selection and clear events
- Stats panel moved to left sidebar

#### UI Layout

```
CENTER PANEL:
+--------------------------------------------------+
| NODE: [dropdown]                           [*]   |
|--------------------------------------------------|
| INPUTS:              | OUTPUTS:                  |
|  * tick (from: ...)  |  * pose -> [rerun, ...]   |
|--------------------------------------------------|
| NODE LOGS:                        [Clear]        |
| [0.123] Processing frame 42                      |
| [0.145] Output sent                              |
+--------------------------------------------------+
```

---

## v0.1.9 (2026-01-09)

### Design: Node Detail Panel (Phase 7)

Complete design specification for the Node Detail Panel widget, which will replace the center panel with an interactive dataflow node inspector.

#### Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ NODE: [dropdown]                                             [●]   │
├─────────────────────────────────────────────────────────────────────┤
│ INPUTS:                          │ OUTPUTS:                        │
│  • port (from: source/output)    │  • port → [dest1, dest2]       │
├─────────────────────────────────────────────────────────────────────┤
│ LOGS:                                                  [Clear]     │
│ [timestamp] message...                                              │
└─────────────────────────────────────────────────────────────────────┘
```

#### New Protocol Types (mviz-core/src/zenoh_protocol.rs)

- `NodeInput`: port name and source reference
- `NodeOutput`: port name and destination nodes list
- `NodeDefinition`: complete node definition from dataflow YAML
- `NodeStatus`: Running, Stopped, Error, Unknown
- `DataflowDefinition`: full dataflow graph with all nodes

#### New Zenoh Messages

- `ZenohMessage::DataflowDefinition` - full dataflow graph
- `ZenohMessage::NodeStatusUpdate` - node status changes

#### Bridge Updates

- `publish_dataflow_definition()` - parse YAML and publish on startup
- `parse_node_inputs()` - extract input ports from YAML
- `parse_node_outputs()` - extract outputs with destinations

#### Widget: NodeDetailPanel (mviz-widgets/src/node_detail_panel.rs)

- Node selector dropdown (populated from dataflow definition or discovery)
- Two-column I/O display (inputs in yellow, outputs in blue)
- Status indicator (color-coded)
- Filtered logs section (only selected node)
- Clear logs button

#### Documentation

- Added Section 9.4 to mviz_design.md with full architecture
- Added Task 4.8 to mviz_plan.md with acceptance criteria
- Updated dependency graph

---

## v0.1.8 (2026-01-09)

### Enhancement: Dynamic Node Filter Dropdown

- DropDown labels now dynamically update when new nodes are discovered
- Uses Makepad's `DropDownRef.set_labels()` API for runtime updates
- Filter selection triggers immediate log content refresh
- Level and node filter changes handled via `cx.capture_actions()`

### Tested Nodes Discovered

From path-following dataflow:
- `sim_pose` - Vehicle simulation pose publisher
- `bicycle_model` - Bicycle model dynamics node
- `sim_state` - Simulation state manager
- `target_point` - Target waypoint generator
- `imu_msg` - IMU sensor data publisher

### Performance Metrics

- Log entries accumulated: 700+ in test session
- Zenoh messages processed: 57,000+
- Real-time dropdown updates as nodes discovered

### Documentation

- Updated mviz_design.md Section 9.3 with implementation details
- Updated mviz_plan.md Task 4.5 with execution results

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
