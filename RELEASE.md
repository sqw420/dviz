# DViz Release Notes

## v0.4.4 (2026-01-30)

### Documentation: Add Demo Images to Examples Directory

Added demo GIF and screenshot files to the examples directory.

#### Files Added

- `examples/dviz1.gif` - Demo animation showing DViz in action
- `examples/dviz2.png` - Screenshot showing the visualization interface

---

## v0.4.3 (2026-01-30)

### Documentation: Add Demo Images to README

Added demo GIF and screenshot to README.md for better project presentation.

#### Changes

- Added `dviz1.gif` demo animation showing DViz in action
- Added `dviz2.png` screenshot showing the visualization interface

---

## v0.4.2 (2026-01-30)

### Fix: Absolute Paths in Dataflow Files

Fixed dataflow YAML files to use absolute paths for the dviz-dora-bridge binary, resolving "No such file or directory" errors when running from the examples directory.

#### Files Fixed

- `dataflow.yml`
- `examples/dataflow-path-following.yml`
- `examples/dataflow-robot.yml`
- `examples/dataflow-mapping.yml`
- `examples/dataflow-ros2.yml`

#### Change

```yaml
# Before (relative path - fails when run from examples/)
path: target/release/dviz-dora-bridge

# After (absolute path - works from any directory)
path: /Users/nupylot/Public/dviz/target/release/dviz-dora-bridge
```

---

## v0.4.1 (2026-01-30)

### Documentation: Comprehensive README

Added comprehensive README.md with installation instructions, usage examples, and distributed debugging setup guide.

#### New Documentation

**README.md:**
- Project overview and feature list
- Installation prerequisites (Rust, Dora CLI, Python dependencies)
- Build instructions
- Quick start guide for single machine usage
- Path following simulation example
- Distributed setup guide (Robot + Debug PC over LAN)
- Zenoh network configuration:
  - Auto-discovery (default multicast scouting)
  - Direct connection with `ZENOH_CONNECT`
  - Zenoh router setup for different subnets
- IP address change handling scenarios
- Network troubleshooting table
- Environment variables reference
- Dataflow examples overview
- Project structure
- Zenoh message protocol documentation

#### Files Reorganized

- Moved dataflow files to `examples/` directory
- Moved design docs to `docs/` directory
- Added `.claude/` settings directory

---

## v0.4.0 (2026-01-30)

### Major: Repository Rename from mviz to dviz

Complete repository rename from "mviz" to "dviz" across the entire codebase.

#### Changes

**Repository & Git:**
- Renamed GitHub repository from `bobd988/mviz` to `bobd988/dviz`
- Updated git remote URL

**Crate Renames (8 crates):**
- `mviz-core` → `dviz-core`
- `mviz-transform` → `dviz-transform`
- `mviz-displays` → `dviz-displays`
- `mviz-urdf` → `dviz-urdf`
- `mviz-shell` → `dviz-shell`
- `mviz-widgets` → `dviz-widgets`
- `mviz-rerun-bridge` → `dviz-rerun-bridge`
- `mviz-rosbag` → `dviz-rosbag`

**Binary Renames:**
- `mviz` → `dviz`
- `mviz-dora-bridge` → `dviz-dora-bridge`

**Dataflow Configurations (5 files):**
- Updated node IDs: `mviz_bridge` → `dviz_bridge`
- Updated Zenoh topic prefix: `mviz` → `dviz`
- Updated build paths and binary names

**Documentation:**
- Renamed `mviz_plan.md` → `dviz_plan.md`
- Renamed `mviz_design.md` → `dviz_design.md`
- Renamed `mviz_features.md` → `dviz_features.md`
- Updated all internal references

**Source Code:**
- Updated all `use mviz_*` imports to `use dviz_*`
- Updated debug log paths: `/tmp/mviz_debug.log` → `/tmp/dviz_debug.log`
- Updated window titles and app names
- Updated comments and documentation strings

---

## v0.3.13 (2026-01-18)

### Feature: Auto-Connect Zenoh and Theme System Enhancement

Removed manual Zenoh connection button and merged mofa-studio theme system for better UI consistency and future dark mode support.

#### Zenoh Auto-Connect

**dviz-shell/src/app.rs:**
- Removed `zenoh_btn` button from toolbar UI
- Added `start_zenoh_connection()` method for automatic startup connection
- Zenoh now auto-connects on app launch via `handle_startup()`
- Status label shows "Zenoh: Connecting...", "Zenoh: Connected" states
- No manual intervention required for LAN data reception

#### Theme System Enhancement

**dviz-widgets/src/theme.rs:**
- Merged comprehensive Tailwind CSS color palette from mofa-studio
- Added full color scales: Slate, Gray, Blue, Indigo, Green, Red, Emerald, Yellow/Amber, Orange (50-900 shades)
- Added dark theme semantic colors: `DARK_BG_DARK`, `PANEL_BG_DARK`, `TEXT_PRIMARY_DARK`, etc.
- Added `ThemeableView` and `ThemeableRoundedView` base widgets with `dark_mode` instance variable
- Enhanced `PrimaryButton` and `IconButton` with dark mode support
- Added comprehensive documentation with usage examples

**dviz-widgets/src/properties_panel.rs:**
- Replaced hardcoded hex colors with theme constants:
  - `#eef2f7` -> `(INPUT_BG)`
  - `#f1f5f9` -> `(SLATE_100)`
  - `#ef4444` -> `(RED_500)` (X axis)
  - `#22c55e` -> `(GREEN_500)` (Y axis)
  - `#3b82f6` -> `(BLUE_500)` (Z axis)
  - `#f5f7fa` -> `(DARK_BG)`

#### Dark Mode Support (Foundation)

Widgets can now support dark mode via shader instance variables:
```rust
draw_bg: {
    instance dark_mode: 0.0  // 0.0 = light, 1.0 = dark
    fn pixel(self) -> vec4 {
        return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
    }
}
```

Update at runtime via `apply_over`:
```rust
widget.apply_over(cx, live!{ draw_bg: { dark_mode: 1.0 } });
```

---

## v0.3.12 (2026-01-11)

### Feature: Multi-Sensor ROS Bag Visualization

Extended ROS bag playback to visualize all sensor types with proper Rerun entity hierarchy and time synchronization.

#### New Modules

**dviz-rosbag/src/imu.rs:**
- `ImuData` - orientation quaternion, angular velocity, linear acceleration
- `ImuProcessor` - Parses sensor_msgs/Imu from raw bytes

**dviz-rosbag/src/gps.rs:**
- `NmeaSentence` - NMEA sentence with type extraction
- `GpsPosition` - lat, lon, alt, fix_quality, satellites, hdop
- `TimeReference` - GPS time reference with source and offset
- `Temperature` - Temperature reading with variance
- `GpsProcessor` - Parses nmea_msgs/Sentence, sensor_msgs/TimeReference, sensor_msgs/Temperature
- GPGGA/GNGGA parsing for GPS position extraction

#### Message Type Extensions

- Added `MessageType::NmeaSentence`
- Added `MessageType::TimeReference`
- Added `MessageType::Temperature`

#### Rerun Entity Hierarchy

```
world/
├── lidar/                    Points3D (velodyne_points)
├── imu/
│   ├── accel_arrow/          Arrows3D (cyan)
│   ├── gyro_arrow/           Arrows3D (orange)
│   ├── accel_x, accel_y, accel_z  Scalars
│   └── gyro_x, gyro_y, gyro_z     Scalars
├── gps/
│   ├── position/             Points3D (green)
│   ├── status/               TextLog (NMEA)
│   ├── latitude, longitude, altitude  Scalars
│   └── satellites            Scalar
├── time_ref/
│   ├── offset/               Scalar
│   └── status/               TextLog
└── temperature/
    ├── celsius/              Scalar
    └── status/               TextLog
```

#### Tested Bag

**hdl_400.bag** (126.32s, 5 topics):
- `/velodyne_points` (sensor_msgs/PointCloud2) ✓
- `/gpsimu_driver/imu_data` (sensor_msgs/Imu) ✓
- `/gpsimu_driver/nmea_sentence` (nmea_msgs/Sentence) ✓
- `/gpsimu_driver/gpstime` (sensor_msgs/TimeReference) ✓
- `/gpsimu_driver/temperature` (sensor_msgs/Temperature) ✓

#### Documentation

- Updated dviz_plan.md with Task 10.3
- Updated dviz_design.md with Section 9.7

---

## v0.3.11 (2026-01-11)

### Fix: Connect Existing Bag Player to Rerun on Launch

Fixed issue where ROS bag playback showed nothing in Rerun when the bag was loaded before spawning Rerun viewer.

#### Root Cause

When user workflow was:
1. File > Open Bag (loads bag, but `rerun_bridge` is None)
2. Spawn Rerun (creates bridge, but existing player not connected)
3. Play (player has no Rerun stream, nothing displayed)

The bag player's `set_rerun_stream()` was only called in `open_rosbag()`, which checks for `rerun_bridge`. If Rerun wasn't launched first, the player was never connected.

#### Fix

Modified `launch_rerun()` in `dviz-shell/src/app.rs` to connect existing bag player when Rerun is spawned:

```rust
// Connect existing bag player to Rerun if one is loaded
if let Some(ref mut player) = self.rosbag_player {
    if let Some(stream) = bridge.stream() {
        player.set_rerun_stream(stream.clone());
        debug_log("Connected existing bag player to Rerun stream");
    }
}
```

#### Result

Both workflows now work:
- Spawn Rerun first, then Open Bag, then Play
- Open Bag first, then Spawn Rerun, then Play

---

## v0.3.10 (2026-01-10)

### Feature: ROS Bag Playback (Phase 10)

Added ROS1 bag file playback support with visualization pipeline:
rosbag → PointCloud2 parsing → TF transforms → Rerun visualization

#### New Crate: dviz-rosbag

**Core Player (player.rs):**
- `RosBagPlayer::open()` - Open bag file and parse metadata
- `RosBagPlayer::play()` / `pause()` / `stop()` - Playback control
- `RosBagPlayer::seek()` - Seek to specific time
- `RosBagPlayer::update()` - Process messages up to target time
- `TopicInfo` - Topic metadata (name, type, count, md5sum)
- `PlaybackState` enum - Stopped, Playing, Paused, Finished

**Message Types (messages.rs):**
- `BagMessage` - topic, msg_type, timestamp, data
- `MessageType` enum - PointCloud2, TfMessage, LaserScan, Image, Odometry, Imu, PoseStamped, Twist, Unknown

**Point Cloud Processing (pointcloud.rs):**
- `Point` - x, y, z, intensity, ring
- `PointCloudProcessor::parse()` - Parse PointCloud2 from raw bytes
- Handles ROS1 message layout with field extraction

**Transform Buffer (tf.rs):**
- `Transform` - translation + quaternion with compose(), inverse()
- `StampedTransform` - Transform with frame IDs and timestamp
- `TfBuffer::lookup_transform()` - Chain lookup through common ancestor
- `TfBuffer::process_tf_message()` - Parse TF messages

#### UI Integration (dviz-shell)

**File Dialog:**
- File button opens native file dialog via `rfd` crate
- Filter for `.bag` files

**Playback Controls:**
- Play button toggles bag playback (or simulation if no bag)
- Timer-driven update at 50Hz
- PointCloud2 logged to Rerun with topic-based entity paths

#### Dependencies Added
- `rosbag = "0.6"` - ROS1 bag file reading
- `rfd = "0.15"` - Native file dialog

---

## v0.3.9 (2026-01-10)

### Documentation: Sync Plan and Design with Actual Implementation

Comprehensive update to reflect actual codebase state.

#### Tasks Marked as COMPLETED

**dviz_plan.md:**
- Task 1.12: URDF Parser (dviz-urdf/src/parser.rs)
- Task 1.13: Mesh Loader (dviz-urdf/src/mesh_loader.rs)
- Task 3.1: Robot Model Display (dviz-displays/src/robot_model.rs)
- Task 4.2: Displays Panel Widget (dviz-widgets/src/displays_panel.rs)
- Task 4.3: Properties Panel Widget (dviz-widgets/src/properties_panel.rs)
- Task 4.4: Toolbar Widget (dviz-widgets/src/toolbar.rs)
- Task 4.8: Node Detail Panel Widget (dviz-widgets/src/node_detail_panel.rs)
- Updated dependency graph with checkmarks for completed tasks

**dviz_design.md:**
- Section 9.4: Node Detail Panel - marked as IMPLEMENTED
- Section 10: URDF Integration - marked as IMPLEMENTED

#### Remaining Pending Tasks

- Task 3.2: LaserScan Display
- Task 3.3-3.6: View Controllers and Tools (FPS, TopDown, Select, Measure)
- Task 4.6-4.7: Display Manager and Configuration Save/Load
- Phase 5: Integration Testing, Documentation, Optimization

---

## v0.3.8 (2026-01-10)

### Documentation: Updated Design and Plan

Updated dviz_design.md and dviz_plan.md with implemented features from Phase 8-9.

#### Changes

**dviz_design.md:**
- Added Section 9.5: Dataflow Graph Widget (Phase 8)
- Added Section 9.6: UI Layout Configuration (Phase 9)

**dviz_plan.md:**
- Added Phase 8: Dataflow Graph Visualization (Tasks 8.1-8.4)
- Added Phase 9: UI Layout Improvements (Tasks 9.1-9.3)
- Updated dependency graph with new phases

---

## v0.3.7 (2026-01-09)

### Enhancement: Wider Side Panels

Increased left and right panel widths for better visibility.

#### Changes

**dviz-shell/src/app.rs:**
- Left panel: 280px → 340px
- Right panel: 280px → 340px
- More room to display dataflow graph, properties, and logs

---

## v0.3.6 (2026-01-09)

### Fix: Revert to Fixed Layout + Green Status Emoji

Reverted to stable fixed-width layout (Splitter widget has rendering bugs). Kept green emoji status indicator.

#### Changes

**dviz-shell/src/app.rs:**
- Reverted to fixed-width three-column View layout
- Left panel: 280px, Center: flexible, Right: 280px
- Note: Makepad Splitter widget causes blank screen - not usable

**dviz-widgets/src/dataflow_graph.rs:**
- Green emoji (🟢) for active nodes, white (⚪) for idle
- Status text: `[RUN]` / `[---]`

#### Display Format

```
  ┌────────────────────┐
  │ 🟢 bicycle_model [RUN] │
  └────────────────────┘
```

---

## v0.3.5 (2026-01-09) - BROKEN

**Do not use** - Splitter still causes blank screen.

---

## v0.3.4 (2026-01-09)

### Fix: Restore Three-Column Layout

Fixed blank screen issue caused by Makepad Splitter widget not rendering children correctly.

#### Changes

**dviz-shell/src/app.rs:**
- Reverted from nested Splitter layout to stable three-column View layout
- Left panel: Fixed 280px width with DisplaysPanel and DataflowGraph
- Center panel: Flexible width with NodeDetailPanel
- Right panel: Fixed 280px width with PropertiesPanel and LogPanel
- Added visible divider lines (1px, #333 color) between panels
- Added explicit `show_bg: true` with background colors for each panel

#### Note

The Splitter-based resizable panels (v0.3.3) caused the UI to render only the splitter bar with no content. This release restores the working fixed-width layout while keeping the scrollable dataflow graph feature.

---

## v0.3.3 (2026-01-09) - BROKEN

**Do not use** - Nested Splitter layout caused blank screen.

---

## v0.3.2 (2026-01-09)

### Enhancement: Improved Dataflow Graph Visualization

Enhanced the DataflowGraphWidget with shader definitions for future graphical rendering and improved graph topology parsing.

#### Changes

**dviz-widgets/src/dataflow_graph.rs:**
- Added `DrawNodeBox` shader with rounded rectangle, active/selected states
- Added `DrawEdgeLine` shader for edge rendering
- Enhanced ASCII-style visual display with box borders (`┌─┐└─┘`)
- Improved edge display with arrows and connection counts
- Added `compute_layout()` for hierarchical node positioning
- Added HashMap for node position tracking

**dviz-rerun-bridge/src/main.rs:**
- Enhanced `GraphState` to initialize from node definitions (`init_from_definitions()`)
- Added `input_source_map` for tracking input-to-source mappings
- Improved edge parsing from YAML input format (`source_node/output_port`)
- Better handling of timer inputs and simple source names

#### Graph Display Format

```
     bicycle_model / sim_pose
           ↓
  ┌───────────────────────────┐
  │ ● simple_planner [ACTIVE] ◀ │
  └───────────────────────────┘
     ↓ 2 output(s)

━━━ 3 nodes, 2 edges ━━━
```

---

## v0.3.1 (2026-01-09)

### Feature: Embedded Dora Dataflow Graph Visualization

Added dynamic dataflow graph visualization directly in the Makepad UI. The graph is inferred from message flow patterns without requiring YAML files on the PC side.

#### Changes

**dviz-core/src/zenoh_protocol.rs:**
- Added `GraphNodeStatus` enum (Active, Idle, Error)
- Added `GraphNode` struct (id, status, last_seen)
- Added `GraphEdge` struct (from_node, from_port, to_node, to_port)
- Added `GraphUpdate` struct (nodes, edges, timestamp)

**dviz-rerun-bridge/src/main.rs:**
- Added `GraphState` struct for tracking discovered graph structure
- Added `record_input()` method to infer edges from input patterns (e.g., `source_node/output_port`)
- Added `to_graph_update()` method to generate graph update messages
- Added `publish_graph_update()` function for Zenoh publishing
- Graph updates published every 2 seconds

**dviz-shell/src/zenoh_receiver.rs:**
- Added `ZenohMessage::GraphUpdate(GraphUpdate)` variant
- Added handling for `graph_update` message type
- Tracks all nodes from graph updates

**dviz-widgets/src/dataflow_graph.rs (new file):**
- Created `DataflowGraphWidget` for displaying the graph
- `GraphDisplayNode` and `GraphDisplayEdge` structures
- Text-based rendering approach (like LogPanel)
- Click detection for node selection
- `update_from_graph_update()` method for dynamic updates

**dviz-widgets/src/lib.rs:**
- Added `pub mod dataflow_graph`
- Added exports for DataflowGraphWidget, DataflowGraphAction, DataflowGraphWidgetRef

**dviz-shell/src/app.rs:**
- Replaced IMU/Vehicle/Stats panels with DataflowGraphWidget in left panel
- Added `ZenohMessage::GraphUpdate` handling in `process_zenoh_messages()`
- Added `DataflowGraphAction::NodeClicked` handling in `handle_actions()`

#### How It Works

1. Bridge tracks input messages with format `source_node/output_port`
2. Infers graph edges: source_node -> current_node (via output_port)
3. Publishes `graph_update` messages via Zenoh every 2 seconds
4. dviz-shell receives updates and displays in DataflowGraphWidget

#### Graph Display Format

```
=== NODES ===

  [ACTIVE] bicycle_model
  [idle] simple_planner <<
  [idle] imu_synthesizer

=== CONNECTIONS ===

  bicycle_model / sim_pose --> simple_planner / pose
  simple_planner / steering_cmd --> bicycle_model / steering_cmd

--- 3 nodes, 2 edges ---
Last update: 2.1s
```

---

## v0.2.9 (2026-01-09)

### Feature: Click Detection on Display List Items

Added click-to-select functionality for the displays panel list items.

#### Changes

**dviz-widgets/src/displays_panel.rs:**
- Added click detection in `handle_event()` for `display_list_content` label
- Calculates clicked item index based on Y position (line height ~18px for font_size 11.0)
- Updates `selected_index` on click
- Emits `DisplaysPanelAction::DisplaySelected(display_id)` when item clicked
- PropertiesPanel updates automatically via existing `on_display_selected()` handler

#### How It Works

When user clicks on a display list item:
1. Hit detection captures click on `display_list_content` area
2. Y position converted to line index: `clicked_index = (click_y - area_y) / line_height`
3. If valid index, updates selection and emits DisplaySelected action
4. App's `on_display_selected()` handler updates PropertiesPanel with display info

---

## v0.2.8 (2026-01-09)

### Fix: Toolbar UI and PropertiesPanel Integration

Fixed three UI issues reported after v0.2.7:

#### 1. Fixed Frame Dropdown
- Initialized dropdown with common coordinate frames: world, map, odom, base_link, base_footprint
- Added frame change handler showing status message

#### 2. File/View Menu Buttons
- Added click handlers with status messages (placeholder for future menus)
- File: "Open, Save, Export (coming soon)"
- View: "Panels, Layout, Reset (coming soon)"

#### 3. PropertiesPanel Selection
- Added `set_display()` method to PropertiesPanel with Cx parameter
- Added `clear_selection()` method
- Added PropertiesPanelRef extension methods
- Fixed `on_display_selected()` to properly update PropertiesPanel header
- Now shows display name and type when selected

#### Files Changed

**dviz-widgets/src/properties_panel.rs:**
- `set_display(cx, display_id, name, type)` - updates header labels
- `clear_selection(cx)` - resets to "No Selection"
- `PropertiesPanelRef` extension impl

**dviz-widgets/src/lib.rs:**
- Export PropertiesPanelRef, PropertiesPanelWidgetRefExt

**dviz-shell/src/app.rs:**
- Import PropertiesPanelWidgetRefExt
- Initialize frame_dropdown in handle_startup
- Add file_btn/view_btn click handlers
- Add frame_dropdown change handler
- Fix on_display_selected to use properties_panel.set_display()

---

## v0.2.7 (2026-01-09)

### Feature: DisplaysPanel Implementation (RViz-Style)

Implemented a full DisplaysPanel widget for managing visualization displays, inspired by RViz's Displays panel.

#### Changes

**dviz-widgets/src/displays_panel.rs:**
- Added `DisplayType` enum with 8 visualization types:
  - Grid, Axes, PointCloud, Markers, TF, LaserScan, Path, Pose
  - Each type has `icon_type()` for shader mapping, `name()` for display
- Added `DisplayInfo` struct with full display metadata:
  - id, name, display_type, enabled, status, status_message
  - topic, color, alpha for rendering configuration
- Added `DisplayListItem` widget with custom shader icons
- Added `DisplaysPanel` widget with:
  - Header showing display count
  - Scrollable list with text-based rendering
  - "Add Display" button with hover effect
- Added `DisplaysPanelAction` enum for event handling:
  - AddDisplayClicked, DisplaySelected, DisplayToggled, DisplayDeleted
- Added `DisplaysPanelDisplayOps` extension trait for ref operations

**dviz-widgets/src/lib.rs:**
- Exports: DisplayInfo, DisplayType, DisplayListItem, DisplaysPanel
- Exports: DisplaysPanelAction, DisplaysPanelDisplayOps, DisplaysPanelWidgetRefExt

**dviz-shell/src/app.rs:**
- Integrated DisplaysPanelAction handling
- Added `on_display_selected()` - updates PropertiesPanel with display info
- Added `on_display_toggled()` - toggles Rerun entity visibility
- Added `on_display_deleted()` - removes display from list

#### Display Panel Features

- Text-based display list with checkbox indicators: `[x]` enabled, `[ ]` disabled
- Selection indicator: ` >` for selected item
- Status display: `[OK]`, `[WARN]`, `[ERR]`
- Format: `  [x] Grid - Grid [OK]`
- Default displays: Grid, Axes (created on initialization)
- Add Display button cycles through display types

#### Technical Notes

- Uses simplified text-based rendering (similar to LogPanel) instead of PortalList
- Widget derive macro auto-generates `DisplaysPanelWidgetRefExt` trait
- Custom extension trait `DisplaysPanelDisplayOps` for additional operations

---

## v0.2.6 (2026-01-09)

### Enhancement: Fully Dynamic I/O Activity Matching

Removed hardcoded path-following-specific node name mappings from the bridge. The I/O activity source matching is now fully dynamic and works with any dataflow.

#### Changes

**dviz-rerun-bridge/src/main.rs:**
- Removed hardcoded input-to-node mappings:
  - ~~`sim_pose` / `sim_state` → `bicycle_model`~~
  - ~~`steering_cmd` / `throttle_cmd` / `target_point` / `waypoints` → `simple_planner`~~
  - ~~`imu_msg` → `imu_synthesizer`~~
- Now uses generic fallback: `(source_node, input_id)` for all inputs
- Changed `publish_log` calls to use dynamic `source_node` instead of hardcoded names

#### How It Works

For any dataflow, the bridge now:
1. If `input_id` contains `/` (e.g., `bicycle_model/sim_pose`): parses source node from it
2. Otherwise: uses the first part of `input_id` as source node

#### Recommendation for New Dataflows

For best results, use Dora's standard input format:
```yaml
inputs:
  pose: source_node/output_name  # Contains '/' - parses correctly
```

---

## v0.2.5 (2026-01-09)

### Enhancement: Debug Logging and Unit Tests for I/O Activity

Added comprehensive debug logging and unit tests to trace and verify the I/O activity data flow from bridge to UI.

#### Changes

**dviz-core/src/zenoh_protocol.rs:**
- Added 3 unit tests for LogData serialization:
  - `test_log_data_with_port_info` - verifies I/O activity logs parse correctly
  - `test_log_data_without_port_info` - verifies regular logs work without port fields
  - `test_full_dviz_message_with_log` - tests full serialize/parse round-trip

**dviz-shell/src/zenoh_receiver.rs:**
- Added detailed debug logging for log message parsing:
  - Logs raw `msg.data` JSON for each log message
  - Logs parsed `LogData` fields: node_id, port, port_type, has_port_info

**dviz-shell/src/app.rs:**
- Added debug logging for I/O activity routing:
  - Logs when I/O activity is detected and routed to NodeDetailPanel
  - Logs metadata keys for regular logs (to diagnose missing port info)

#### Debug Output Location

Debug output written to:
- Terminal: `[dviz]` prefixed lines
- File: `/tmp/dviz_zenoh_debug.log`

To check I/O activity flow:
```bash
grep -E "Log msg.data|LogData parsed|I/O activity" /tmp/dviz_zenoh_debug.log
```

---

## v0.2.4 (2026-01-09)

### Feature: Live I/O Activity Display in NodeDetailPanel

Changed the NodeDetailPanel's INPUTS and OUTPUTS sections from showing static schema definitions to displaying live message data flowing through ports.

#### Changes

**dviz-rerun-bridge/src/main.rs:**
- Added `publish_io_activity()` - publishes I/O activity logs with port information
- Added `format_values_summary()` - formats float arrays as summary strings for display
- Bridge now publishes I/O activity for:
  - Its own inputs (showing data received from upstream nodes)
  - Source node outputs (showing data the source node emitted)

**dviz-core/src/zenoh_protocol.rs:**
- Added `port: Option<String>` field to `LogData` - identifies which port the message relates to
- Added `port_type: Option<String>` field to `LogData` - "input" or "output"

**dviz-widgets/src/node_detail_panel.rs:**
- Added `IoActivityEntry` struct - timestamp, port_name, data_summary
- Added `input_activity: VecDeque<IoActivityEntry>` to `NodeDisplayState`
- Added `output_activity: VecDeque<IoActivityEntry>` to `NodeDisplayState`
- Added `add_io_activity()` method - routes I/O activity to correct node and port
- Updated `update_io_display()` - now shows live messages instead of definitions

**dviz-shell/src/zenoh_receiver.rs:**
- Extended log entry parsing to extract `port` and `port_type` from JSON data
- Port info now stored in `LogEntry.metadata` for routing in app.rs

**dviz-shell/src/app.rs:**
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
- id: dviz_bridge
  env:
    DATAFLOW_PATH: /Users/nupylot/Public/dviz/dataflow-path-following.yml
```

---

## v0.2.2 (2026-01-09)

### Fix: Node Definition Timing Issue

Fixed issue where NodeDetailPanel showed "(definition not available)" even when definitions were being published. The root cause was that node definitions were only published once at bridge startup, which could miss late-joining Zenoh subscribers.

#### Changes

**dviz-rerun-bridge/src/main.rs:**
- Refactored into separate functions:
  - `parse_dataflow_definitions()` - parses YAML, returns `Vec<NodeDefinition>`
  - `publish_node_definitions()` - publishes definitions to Zenoh
- Node definitions now stored and republished every 3 seconds
- Late-joining subscribers (like dviz-shell) now receive definitions reliably
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

#### Bridge Changes (dviz-rerun-bridge/src/main.rs)

- Added `publish_dataflow_definitions()` - parses dataflow YAML and publishes node definitions
- Parses both operator-style and direct inputs/outputs from YAML
- Computes output destinations by searching for nodes that consume each output
- Publishes to `{prefix}/definitions/{node_id}` topics
- Auto-discovers dataflow YAML from `DATAFLOW_PATH` env or common paths

#### Protocol Changes (dviz-core/src/zenoh_protocol.rs)

- Added `NodeInputDef` struct: port name and source reference
- Added `NodeOutputDef` struct: port name and destination list
- Added `NodeDefinition` struct: complete node with inputs/outputs
- Added `DataflowDefinition` struct: full dataflow graph

#### Receiver Changes (dviz-shell/src/zenoh_receiver.rs)

- Added `ZenohMessage::NodeDef(NodeDefinition)` variant
- Handles `node_definition` message type from bridge
- Tracks nodes from definitions in discovered_nodes

#### App Integration (dviz-shell/src/app.rs)

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
• tick (from: dora/timer/...)    • sim_pose -> [simple_planner, dviz_bridge]
• steering_cmd (from: ...)       • sim_state -> [imu_synthesizer, ...]
```

---

## v0.2.0 (2026-01-09)

### Implementation: Node Detail Panel (Phase 7)

Replaced center panel static text with an interactive Node Detail Panel for dataflow node inspection.

#### New Widget: NodeDetailPanel (dviz-widgets/src/node_detail_panel.rs)

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

#### App Integration (dviz-shell/src/app.rs)

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

#### New Protocol Types (dviz-core/src/zenoh_protocol.rs)

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

#### Widget: NodeDetailPanel (dviz-widgets/src/node_detail_panel.rs)

- Node selector dropdown (populated from dataflow definition or discovery)
- Two-column I/O display (inputs in yellow, outputs in blue)
- Status indicator (color-coded)
- Filtered logs section (only selected node)
- Clear logs button

#### Documentation

- Added Section 9.4 to dviz_design.md with full architecture
- Added Task 4.8 to dviz_plan.md with acceptance criteria
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

- Updated dviz_design.md Section 9.3 with implementation details
- Updated dviz_plan.md Task 4.5 with execution results

---

## v0.1.7 (2026-01-09)

Phase 6: System Log Panel for Distributed Robotics Debugging

### New Feature: System Log Panel

Real-time log collection and display from dora dataflow nodes over LAN via Zenoh.

#### LogPanel Widget (dviz-widgets/src/log_panel.rs)

- Collapsible panel with entry count display
- Filter by log level (Debug, Info, Warn, Error)
- Filter by node (dynamically populated via Zenoh discovery)
- Text search across messages
- Copy to clipboard and Clear buttons
- Color-coded log entries
- Scrollable log content with newest entries first

#### Protocol Extensions (dviz-core/zenoh_protocol.rs)

- `LogLevel` enum: Debug, Info, Warn, Error with color() method
- `LogEntry` struct: level, message, node_id, timestamp, metadata
- `LogData` struct: JSON payload for log messages

#### Bridge Updates (dviz-rerun-bridge/src/main.rs)

- `publish_log()` helper function for sending log messages
- Bridge startup/shutdown log messages
- Node message count tracking with periodic status logs
- Vehicle state updates logged every 50 frames
- First message notification per source node

#### Zenoh Receiver Updates (dviz-shell/src/zenoh_receiver.rs)

- `ZenohMessage::Log(LogEntry)` - system log entry
- `ZenohMessage::NodeDiscovered(String)` - new node ID
- `discovered_nodes: Arc<RwLock<HashSet<String>>>` - dynamic tracking

#### App Integration (dviz-shell/src/app.rs)

- Log entries processed in `process_zenoh_messages()`
- Node discovery tracking with HashSet
- LogPanel actions: Copy, Clear, ToggleCollapsed

### Documentation

- Added Section 9.3 to dviz_design.md with full architecture
- Added Task 4.5 to dviz_plan.md with acceptance criteria
- Updated dependency graph

---

## v0.1.5 (2026-01-08)

Phase 5: Zenoh Universal Protocol for LAN Visualization

### Architecture

Distributed robotics visualization via Zenoh pub/sub:
- **Robot side**: Dora dataflow with dviz-bridge publishes via Zenoh
- **PC side**: dviz-shell receives via Zenoh, displays in Rerun

### New Crate: dviz-rerun-bridge (Dora Node)

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
- `ZENOH_TOPIC_PREFIX` - Topic prefix (default: "dviz")

### New Module: dviz-core/zenoh_protocol.rs

Universal message format for Zenoh communication:
- JSON header + optional binary payload
- `MvizMessage` struct with type, timestamp, data, format, count
- Serialization/deserialization utilities

### New Module: dviz-shell/zenoh_receiver.rs

Universal Zenoh subscriber for PC-side visualization:
- Subscribes to `{prefix}/**` wildcard topics
- Parses universal message format
- Sends typed `VisData` to UI thread

### Enhanced: dviz-shell/app.rs

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
cargo run -p dviz-shell
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
- `dviz-displays/src/laser_scan.rs` - Laser scan simulation and display
- `dviz-displays/src/robot_model.rs` - Robot model display plugin

### Technical Changes
- DisplaysPanelAction enum for widget event handling
- Entity paths use `world/robot/*` prefix for proper Rerun coordinate space
- Uses `RecordingStream::log_file_from_path()` for URDF loading

---

## v0.1.3 (2026-01-07)

Phase 2: Display Plugins + Phase 3: Makepad UI Shell

### New Crate: dviz-displays

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

### Enhanced Crate: dviz-widgets

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

### Enhanced App: dviz-shell

Three-column layout application:
- **Left Panel**: DisplaysPanel + IMU/Vehicle sensor panels
- **Center Panel**: Rerun viewer info with simulation stats
- **Right Panel**: PropertiesPanel for display configuration

### Tests
- 29 new tests in dviz-displays
- All key functions validated:
  - App launches successfully
  - Launch Rerun spawns viewer
  - Play/Pause simulation works
  - Sensor data updates at 50Hz
  - Data streams to Rerun viewer

---

## v0.1.2 (2026-01-06)

Phase 1 Streams B+C: Transform System and Rerun Core Adapters.

### New Crate: dviz-transform

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

### Enhanced Crate: dviz-rerun-bridge

Core adapters for logging dviz-core types to Rerun viewer.

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
- 21 new tests in dviz-transform
- 3 new tests in dviz-rerun-bridge core adapters
- 71 total tests passing across workspace

---

## v0.1.1 (2026-01-06)

Phase 1: Core Foundation - Added dviz-core crate with foundational types.

### New Crate: dviz-core

Core types and traits for the DViz robotics visualizer.

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

Initial release of DViz - a visualization tool combining Makepad UI with Rerun 3D viewer.

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
dviz/
├── dviz-shell/          # Main application
├── dviz-widgets/        # Custom UI widgets
├── dviz-rerun-bridge/   # Rerun SDK integration
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
