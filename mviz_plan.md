# Development Plan: Robotics Visualizer

A parallel development plan for a team using AI-assisted coding tools (Claude Code, Cursor, Copilot, etc.).

---

## Team Structure

### Recommended Team: 5-7 Developers

| Role | Stream | Primary Crates |
|------|--------|----------------|
| **Dev A** | Core Foundation | `rv-core` |
| **Dev B** | Transform System | `rv-transform` |
| **Dev C** | Rerun Integration | `rv-rerun` |
| **Dev D** | URDF & Meshes | `rv-urdf` |
| **Dev E** | Displays (Part 1) | `rv-displays` (Grid, Axes, TF, PointCloud) |
| **Dev F** | Displays (Part 2) | `rv-displays` (Marker, Path, Image, Robot) |
| **Dev G** | UI & Integration | `rv-ui`, `rv-views`, `rv-tools` |

---

## Development Phases

```
Week 1-2        Week 3-4        Week 5-6        Week 7-8        Week 9-10
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Phase 0 │───▶│ Phase 1 │───▶│ Phase 2 │───▶│ Phase 3 │───▶│ Phase 4 │
│  Setup  │    │  Core   │    │ Displays│    │  Robot  │    │ Polish  │
└─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘
```

---

## Phase 0: Project Setup (Week 1)

**All Developers - Sequential Start**

### Task 0.1: Workspace Setup
**Assignee**: Dev A (then shared)
**Duration**: 2 hours

```
AI Prompt:
"Create a Rust workspace with the following structure:
- Workspace root with Cargo.toml
- 8 crates: rv-core, rv-transform, rv-rerun, rv-urdf, rv-displays, rv-tools, rv-views, rv-ui
- apps/robotics-viz binary crate
- Set up workspace dependencies for: glam, serde, serde_yaml, rerun, urdf-rs, parking_lot, thiserror
- Use Rust 2021 edition, minimum version 1.75"
```

**Acceptance Criteria**:
- [ ] `cargo build` succeeds for empty workspace
- [ ] All crates have proper Cargo.toml with workspace dependencies
- [ ] CI workflow file created (.github/workflows/ci.yml)

### Task 0.2: Development Environment Docs
**Assignee**: Dev G
**Duration**: 2 hours

Create `CONTRIBUTING.md` with:
- Makepad setup instructions (nightly Rust)
- Rerun viewer installation
- Code style (rustfmt, clippy)
- Branch naming conventions
- PR template

---

## Phase 1: Core Foundation (Week 2-3)

### Parallel Stream A: Core Types (Dev A)

#### Task 1.1: Transform Types
**Duration**: 4 hours

```
AI Prompt:
"Implement core transform types in rv-core/src/types/transform.rs:
- FrameId: newtype wrapper around String with Hash, Eq
- Timestamp: i64 nanoseconds with now(), from_secs_f64()
- Transform: translation (Vec3) + rotation (Quat) with:
  - IDENTITY constant
  - to_mat4(), to_affine()
  - mul(), inverse()
  - transform_point(), transform_vector()
  - lerp() for interpolation
- StampedTransform: Transform with timestamp and frame IDs
- Pose: position + orientation + frame_id + timestamp
Use glam for math types. Add serde derives."
```

**Acceptance Criteria**:
- [ ] Unit tests for transform composition and inverse
- [ ] Serde round-trip tests
- [ ] Documentation with examples

#### Task 1.2: Point Cloud Types
**Duration**: 3 hours

```
AI Prompt:
"Implement point cloud types in rv-core/src/types/point_cloud.rs:
- Color: RGBA u8 with common colors (WHITE, RED, etc.) and constructors
- PointCloud: positions Vec<Vec3>, optional colors/intensities, frame_id, timestamp
- PointCloudStyle enum: Points, Squares, Circles, Spheres, Boxes
- ColorMode enum: FlatColor, RGB, Intensity{min,max,colormap}, AxisColor{axis,min,max,colormap}
- Colormap enum: Jet, Rainbow, Turbo, Viridis, Grayscale
- Axis enum: X, Y, Z
Include with_capacity constructor and len/is_empty methods."
```

#### Task 1.3: Marker Types
**Duration**: 3 hours

```
AI Prompt:
"Implement marker types in rv-core/src/types/marker.rs matching RViz visualization_msgs/Marker:
- MarkerType enum: Arrow, Cube, Sphere, Cylinder, LineStrip, LineList, CubeList, SphereList, Points, TextViewFacing, MeshResource, TriangleList
- MarkerAction enum: Add, Modify, Delete, DeleteAll
- MarkerScale: x, y, z with uniform() and to_vec3()
- Marker struct with: id, ns, marker_type, action, pose, scale, color, lifetime_ns, frame_locked, points, colors, text, mesh_resource
- MarkerArray: Vec<Marker>
Include builder pattern for Marker."
```

#### Task 1.4: Display Trait
**Duration**: 4 hours

```
AI Prompt:
"Implement display plugin trait in rv-core/src/traits/display.rs:
- StatusLevel enum: Ok, Warn, Error
- Status struct: level, name, message
- DisplayContext trait (Send + Sync): fixed_frame(), transform_buffer(), rerun_bridge(), queue_render(), current_time()
- Display trait (Send + Sync):
  - name(), type_id(), entity_path()
  - initialize(), on_enable(), on_disable(), update(), reset()
  - is_enabled(), set_enabled()
  - status(), set_status(), clear_status()
  - properties(), properties_mut() returning &dyn Any
  - save_config(), load_config()
- DisplayFactory trait: create(), type_id(), display_name(), description()
- DisplayRegistry with register(), create(), available_types()"
```

#### Task 1.5: Configuration Schema
**Duration**: 3 hours

```
AI Prompt:
"Implement configuration types in rv-core/src/config/schema.rs with serde:
- AppConfig: version, global_options, displays Vec, views, panels, window
- GlobalOptions: fixed_frame String, frame_rate u32, background_color [f32;4]
- DisplayConfig: type_id, name, enabled, properties HashMap<String, serde_yaml::Value>
  - with() builder method and get<T>() method
- ViewsConfig: current String, saved Vec<SavedView>
- SavedView: name, controller_type, position, rotation, focal_point, distance
- PanelsConfig with panel states
- WindowConfig: width, height, x, y, maximized
- ConfigManager with load() and save() methods
All with Default implementations."
```

---

### Parallel Stream B: Transform System (Dev B)

#### Task 1.6: Frame Tree
**Duration**: 4 hours
**Depends on**: Task 1.1

```
AI Prompt:
"Implement frame tree in rv-transform/src/frame_tree.rs:
- FrameNode struct: parent Option<FrameId>, children Vec<FrameId>
- FrameTree struct with HashMap<FrameId, FrameNode>
Methods:
- new() -> empty tree
- set_parent(child, parent) - handles reparenting
- get_path_to_root(frame) -> Vec<FrameId>
- find_common_ancestor(frame_a, frame_b) -> Option<FrameId>
- all_frames() -> Vec<FrameId>
Include comprehensive tests for tree operations."
```

#### Task 1.7: Transform Buffer
**Duration**: 6 hours
**Depends on**: Task 1.1, 1.6

```
AI Prompt:
"Implement time-indexed transform buffer in rv-transform/src/transform_buffer.rs:
- TransformHistory: BTreeMap<Timestamp, Transform> with max_duration pruning
  - insert(), prune_old(), get_at() with interpolation, get_latest()
- TransformKey: parent + child FrameId
- TransformBuffer:
  - new(fixed_frame, buffer_duration)
  - set_fixed_frame()
  - set_transform(StampedTransform) - updates tree and history
  - lookup_transform(target, source, time) -> Result<Transform, TransformError>
  - get_transform_to_fixed(frame, time)
  - get_latest_transform_to_fixed(frame)
  - all_frames()
- TransformError enum: NoPath, TransformNotFound, ExtrapolationError
Include interpolation between timestamps. Thread-safe with parking_lot."
```

---

### Parallel Stream C: Rerun Integration (Dev C)

#### Task 1.8: Rerun Bridge
**Duration**: 4 hours

```
AI Prompt:
"Implement Rerun bridge in rv-rerun/src/bridge.rs:
- RerunConfig: app_id, recording_id Option, connect_addr Option, save_path Option
- RerunBridge:
  - new(config) -> Result connecting to Rerun
  - stream() -> &RecordingStream
  - set_time(name, Timestamp)
  - set_sequence(name, i64)
  - log<T: AsComponents>(entity_path, data) -> Result
  - log_static<T: AsComponents>(entity_path, data) -> Result
- RerunError enum with From implementations
Support connect_tcp, save to file, and spawn viewer modes."
```

#### Task 1.9: Point Cloud Adapter
**Duration**: 4 hours
**Depends on**: Task 1.2, 1.8

```
AI Prompt:
"Implement point cloud adapter in rv-rerun/src/adapters/point_cloud.rs:
- PointCloudAdapter struct with static methods:
  - log(bridge, entity_path, cloud, color_mode, point_radius) -> Result
  - compute_colors(cloud, ColorMode) -> Option<Vec<rerun::Color>>
  - colormap_lookup(Colormap, t: f32) -> rerun::Color
  - hsv_to_rgb(h, s, v) -> (f32, f32, f32)
Implement all colormaps: Jet, Rainbow, Viridis, Grayscale, Turbo.
Map PointCloud to rerun::Points3D with positions, colors, radii."
```

#### Task 1.10: Transform Adapter
**Duration**: 3 hours
**Depends on**: Task 1.1, 1.8

```
AI Prompt:
"Implement transform adapter in rv-rerun/src/adapters/transform.rs:
- TransformAdapter struct with static methods:
  - log_transform(bridge, path, Transform) - logs rerun::Transform3D
  - log_frame_axes(bridge, path, Transform, scale) - logs axes as colored arrows (RGB for XYZ)
  - log_all_frames(bridge, buffer, base_path, axis_scale) - logs entire transform tree
Use rerun::Arrows3D for axes visualization with red=X, green=Y, blue=Z."
```

#### Task 1.11: Marker Adapter
**Duration**: 6 hours
**Depends on**: Task 1.3, 1.8

```
AI Prompt:
"Implement marker adapter in rv-rerun/src/adapters/marker.rs:
- MarkerAdapter with log_marker(bridge, base_path, Marker) -> Result
Implement each MarkerType:
- Arrow -> rerun::Arrows3D
- Cube -> rerun::Boxes3D with quaternion
- Sphere -> rerun::Ellipsoids3D
- Cylinder -> placeholder (Rerun lacks native cylinders)
- LineStrip -> rerun::LineStrips3D
- LineList -> rerun::LineStrips3D with pairs
- Points -> rerun::Points3D
- TextViewFacing -> rerun::TextLog + position point
- CubeList/SphereList -> batch shapes
- TriangleList -> rerun::Mesh3D with indices
- MeshResource -> placeholder
Add log_marker_array(bridge, base_path, MarkerArray)."
```

---

### Parallel Stream D: URDF (Dev D)

#### Task 1.12: URDF Parser [COMPLETED]
**Duration**: 6 hours
**Status**: Implemented in mviz-urdf/src/parser.rs

```
AI Prompt:
"Implement URDF parser in rv-urdf/src/parser.rs using urdf-rs:
Data structures:
- RobotDescription: name, links HashMap, joints HashMap, root_link Option
- Link: name, visual Option, collision Option, inertial Option
- Visual: origin Transform, geometry, material Option
- Collision: origin Transform, geometry
- Inertial: origin, mass f32, inertia [f32;6]
- Geometry enum: Box{size}, Cylinder{radius,length}, Sphere{radius}, Mesh{filename,scale}
- Material: name, color Option, texture Option
- Joint: name, joint_type, parent_link, child_link, origin, axis, limits Option
- JointType enum: Fixed, Revolute, Continuous, Prismatic, Floating, Planar
- JointLimits: lower, upper, effort, velocity

Functions:
- parse_urdf(path) -> Result<RobotDescription>
- parse_urdf_string(urdf) -> Result<RobotDescription>
Convert urdf-rs types to our types with proper Transform conversion (xyz + rpy to Vec3 + Quat)."
```

#### Task 1.13: Mesh Loader [COMPLETED]
**Duration**: 4 hours
**Depends on**: Task 1.12
**Status**: Implemented in mviz-urdf/src/mesh_loader.rs

```
AI Prompt:
"Implement mesh loader in rv-urdf/src/mesh_loader.rs:
- MeshData: vertices Vec<Vec3>, indices Vec<u32>, normals Option, uvs Option, colors Option
- MeshLoader trait:
  - load(path) -> Result<MeshData>
  - supported_extensions() -> &[&str]
- StlLoader implementing MeshLoader for .stl files
- DaeLoader placeholder for .dae (Collada)
- ObjLoader placeholder for .obj

Use package:// URL resolution pattern for ROS package paths.
Include scale parameter support."
```

---

## Phase 2: Basic Displays (Week 4-5)

### Stream E: Core Displays (Dev E)

#### Task 2.1: Base Display Helper
**Duration**: 2 hours
**Depends on**: Task 1.4

```
AI Prompt:
"Create base display implementation in rv-displays/src/base.rs:
- BaseDisplay struct: name, type_id, entity_path, enabled, statuses Vec
- Methods: new(name, type_id), set_name()
- impl_display_base! macro to reduce boilerplate for Display trait implementations
The macro should implement: name(), type_id(), is_enabled(), set_enabled(), status(), set_status(), clear_status(), entity_path()"
```

#### Task 2.2: Grid Display
**Duration**: 4 hours
**Depends on**: Task 2.1, 1.8

```
AI Prompt:
"Implement grid display in rv-displays/src/grid.rs:
- GridProperties: cell_count u32, cell_size f32, color, alpha, line_width, frame FrameId, offset Vec3, plane GridPlane
- GridPlane enum: XY, XZ, YZ
- GridDisplay struct with base and properties
Methods:
- generate_grid_lines() -> Vec<Vec<Vec3>> - creates line segments for grid
- log_grid(bridge) - logs as rerun::LineStrips3D
Implement full Display trait. Log as static (time-independent).
Default: 10x10 grid, 1m cells, gray color, XY plane."
```

#### Task 2.3: Axes Display
**Duration**: 3 hours
**Depends on**: Task 2.1, 1.10

```
AI Prompt:
"Implement axes display in rv-displays/src/axes.rs:
- AxesProperties: frame FrameId, scale f32, line_width f32
- AxesDisplay struct
In update(): get transform from buffer and log axes using TransformAdapter::log_frame_axes
Show RGB colored axes (red=X, green=Y, blue=Z) at the specified frame.
Default scale: 1.0m"
```

#### Task 2.4: TF Display
**Duration**: 6 hours
**Depends on**: Task 2.1, 1.7, 1.10

```
AI Prompt:
"Implement TF display in rv-displays/src/tf.rs:
- TfProperties: show_names bool, show_arrows bool, show_axes bool, axis_scale f32, frame_timeout Duration, update_rate Duration
- TfDisplay struct
Features:
- Display all frames from TransformBuffer as axes
- Optional frame name labels (text)
- Optional parent-child connection arrows
- Configurable visibility per frame (HashMap<FrameId, bool>)
- Timeout detection for stale frames (show warning color)
In update(): iterate all_frames(), check visibility, log axes and optionally labels/connections."
```

#### Task 2.5: Point Cloud Display
**Duration**: 8 hours
**Depends on**: Task 2.1, 1.9

```
AI Prompt:
"Implement point cloud display in rv-displays/src/point_cloud.rs:
- PointCloudProperties:
  - style: PointCloudStyle
  - color_mode: ColorMode
  - point_size: f32
  - alpha: f32
  - decay_time: Duration (0 = no decay)
  - selectable: bool
  - queue_size: usize
- PointCloudDisplay with message queue and decay management
Methods:
- add_cloud(PointCloud) - adds to queue with timestamp
- prune_old_clouds(now) - removes clouds older than decay_time
- update() - prunes and logs all active clouds
Support for decayed visualization: accumulate multiple clouds over time.
Handle transform lookup for each cloud's frame_id."
```

---

### Stream F: Marker & Path Displays (Dev F)

#### Task 2.6: Marker Display
**Duration**: 6 hours
**Depends on**: Task 2.1, 1.11

```
AI Prompt:
"Implement marker display in rv-displays/src/marker.rs:
- MarkerProperties: namespace_filter Option<String>
- MarkerDisplay with marker cache HashMap<(String, u32), MarkerState>
- MarkerState: marker, last_update, expired bool
Methods:
- process_marker(Marker) - handles Add/Modify/Delete/DeleteAll actions
- update() - check lifetimes, expire old markers, log active ones
- clear_namespace(ns)
- clear_all()
Handle marker lifetimes (None = forever, Some(ns) = auto-expire).
Use MarkerAdapter for logging."
```

#### Task 2.7: Path Display
**Duration**: 4 hours
**Depends on**: Task 2.1

```
AI Prompt:
"Implement path display in rv-displays/src/path.rs:
- PathProperties: color, line_width f32, show_poses bool, pose_style (Arrow/Axes), pose_scale f32, alpha f32
- PathDisplay
Input: Vec<Pose> (path waypoints)
Visualization:
- Line strip connecting all poses
- Optional arrow/axes at each pose
Log as rerun::LineStrips3D + optional Arrows3D for poses.
Transform all poses to fixed frame before logging."
```

#### Task 2.8: Pose Display
**Duration**: 3 hours
**Depends on**: Task 2.1

```
AI Prompt:
"Implement pose display in rv-displays/src/pose.rs:
- PoseProperties: style (Arrow/Axes), scale f32, color, shaft_radius f32, head_radius f32, alpha f32
- PoseDisplay
Input: single Pose
Visualization: arrow or axes at pose location
For arrow: use rerun::Arrows3D
For axes: use TransformAdapter::log_frame_axes pattern"
```

#### Task 2.9: Image Display
**Duration**: 5 hours
**Depends on**: Task 2.1

```
AI Prompt:
"Implement image display in rv-displays/src/image.rs:
- ImageProperties: normalize bool, min_value f32, max_value f32, colormap Option<Colormap>
- ImageData: width, height, encoding ImageEncoding, data Vec<u8>
- ImageEncoding enum: RGB8, RGBA8, BGR8, BGRA8, Mono8, Mono16, Depth32F
- ImageDisplay
Methods:
- set_image(ImageData)
- convert_to_rgb(ImageData) -> Vec<u8> - handle encoding conversions
- update() - log as rerun::Image or rerun::DepthImage
Support depth image visualization with colormap."
```

---

### Stream G: UI Foundation (Dev G)

#### Task 2.10: View Controller Trait
**Duration**: 3 hours

```
AI Prompt:
"Implement view controller trait in rv-core/src/traits/view_controller.rs:
- CameraState: position Vec3, rotation Quat, fov_y f32, near_clip, far_clip
  - Methods: view_matrix(), forward(), right(), up(), look_at()
- MouseState: position, delta, left/middle/right bool, scroll_delta
- KeyModifiers: shift, ctrl, alt
- ViewInputEvent enum: MouseMove, MousePress, MouseRelease, Scroll, KeyPress, KeyRelease
- MouseButton enum: Left, Middle, Right
- Key enum: W, A, S, D, Q, E, F, Z, Up, Down, Left, Right, Shift, Ctrl, Alt
- ViewController trait (Send + Sync):
  - name(), camera_state(), camera_state_mut()
  - handle_input(event, viewport_size)
  - update(dt), reset(), look_at(target)
  - set_focal_point(), focal_point()
  - mimic(other) with default implementation"
```

#### Task 2.11: Orbit View Controller
**Duration**: 4 hours
**Depends on**: Task 2.10

```
AI Prompt:
"Implement orbit controller in rv-views/src/orbit.rs:
- OrbitViewController:
  - camera: CameraState
  - focal_point: Vec3
  - distance: f32
  - yaw, pitch: f32 (spherical coordinates)
  - rotate_speed, pan_speed, zoom_speed: f32
  - mouse_state: MouseState
Methods:
- update_camera() - compute position from spherical coords, look at focal point
- rotate(dx, dy) - modify yaw/pitch
- pan(dx, dy) - move focal point in screen space
- zoom(delta) - adjust distance
Input handling:
- Left drag: rotate
- Middle drag or Shift+Left: pan
- Right drag or scroll: zoom
- F key: focus on selection
- Z key: reset view"
```

#### Task 2.12: Tool Trait
**Duration**: 2 hours

```
AI Prompt:
"Implement tool trait in rv-core/src/traits/tool.rs:
- Tool trait (Send + Sync):
  - name(), shortcut_key() -> Option<char>
  - activate(), deactivate()
  - process_mouse_event(ViewportMouseEvent) -> ToolResult
  - process_key_event(KeyEvent) -> ToolResult
  - update(dt)
  - cursor() -> CursorType
- ToolResult enum: None, Render, Finished
- CursorType enum: Default, Crosshair, Move, Rotate, Zoom, Hand
- ToolManager:
  - register(Box<dyn Tool>)
  - set_active(name)
  - active_tool() -> Option<&dyn Tool>
  - handle_input(...)"
```

---

## Phase 3: Robot & Advanced Features (Week 6-7)

### Task 3.1: Robot Model Display (Dev F) [COMPLETED]
**Duration**: 8 hours
**Depends on**: Task 1.12, 1.13, 2.1
**Status**: Implemented in mviz-displays/src/robot_model.rs

```
AI Prompt:
"Implement robot model display in rv-displays/src/robot_model.rs:
- RobotModelProperties:
  - description_source: enum {Topic, File, Parameter}
  - file_path: Option<String>
  - visual_enabled: bool
  - collision_enabled: bool
  - alpha: f32
  - tf_prefix: String
- RobotModelDisplay:
  - robot: Option<RobotDescription>
  - link_visuals: HashMap<String, Vec<MeshData>>
Methods:
- load_urdf(source) - parse and cache robot
- update() - for each link:
  - get transform from buffer (with tf_prefix)
  - log visual meshes at transformed position
Use rerun::Mesh3D for mesh geometry.
Handle Box/Cylinder/Sphere primitives with Rerun shapes."
```

### Task 3.2: LaserScan Display (Dev E)
**Duration**: 5 hours
**Depends on**: Task 2.5

```
AI Prompt:
"Implement laser scan display in rv-displays/src/laser_scan.rs:
- LaserScanData: angle_min, angle_max, angle_increment, range_min, range_max, ranges Vec<f32>, intensities Option<Vec<f32>>
- LaserScanProperties: color_mode (FlatColor/Intensity/Range), point_size, alpha, decay_time
- LaserScanDisplay
Methods:
- set_scan(LaserScanData, frame_id, timestamp)
- scan_to_pointcloud(LaserScanData) -> PointCloud - convert polar to Cartesian
  - Skip invalid ranges (< range_min, > range_max, NaN, Inf)
  - x = range * cos(angle), y = range * sin(angle), z = 0
- Uses PointCloudDisplay internally for rendering
Support intensity-based coloring from intensities field."
```

### Task 3.3: FPS View Controller (Dev G)
**Duration**: 4 hours
**Depends on**: Task 2.10

```
AI Prompt:
"Implement FPS controller in rv-views/src/fps.rs:
- FPSViewController:
  - camera: CameraState
  - yaw, pitch: f32
  - move_speed, look_speed: f32
  - keys_pressed: HashSet<Key>
Methods:
- update(dt) - apply movement based on pressed keys (WASD + QE for up/down)
Input handling:
- Mouse move: adjust yaw/pitch (look around)
- W/S: forward/backward
- A/D: strafe left/right
- Q/E: down/up
- Shift: speed boost
Clamp pitch to avoid gimbal lock."
```

### Task 3.4: Top-Down Ortho Controller (Dev G)
**Duration**: 3 hours
**Depends on**: Task 2.10

```
AI Prompt:
"Implement top-down orthographic controller in rv-views/src/top_down.rs:
- TopDownViewController:
  - camera: CameraState (ortho projection)
  - center: Vec2 (XY position)
  - scale: f32 (zoom level)
  - pan_speed, zoom_speed: f32
Methods:
- update_camera() - position camera above center, looking down
Input:
- Left drag: pan
- Scroll: zoom (adjust scale)
Use orthographic projection instead of perspective."
```

### Task 3.5: Selection Tool (Dev G)
**Duration**: 4 hours
**Depends on**: Task 2.12

```
AI Prompt:
"Implement selection tool in rv-tools/src/select.rs:
- SelectionTool:
  - selection_start: Option<(f32, f32)>
  - current_selection: HashSet<EntityId>
  - mode: SelectMode (Replace, Add, Remove)
- SelectMode based on modifiers (Shift=Add, Ctrl=Remove)
Methods:
- start_selection(pos)
- update_selection(pos) - draw selection rectangle
- finish_selection(pos) - query entities in box
- clear_selection()
For now, store selection state. Actual picking requires Rerun integration."
```

### Task 3.6: Measure Tool (Dev G)
**Duration**: 3 hours
**Depends on**: Task 2.12

```
AI Prompt:
"Implement measure tool in rv-tools/src/measure.rs:
- MeasureTool:
  - start_point: Option<Vec3>
  - end_point: Option<Vec3>
  - measuring: bool
Methods:
- on_click(pos) - set start or end point via 3D picking
- get_distance() -> Option<f32>
- clear()
Visualize: line between points + distance text label.
Log as rerun::LineStrips3D + rerun::TextLog."
```

---

## Phase 4: UI & Integration (Week 8-9)

### Task 4.1: Makepad App Shell (Dev G)
**Duration**: 6 hours

```
AI Prompt:
"Create Makepad application shell in rv-ui/src/app.rs:
Use live_design! macro for declarative UI:
- Window with dark theme (#2a2a2a background)
- Vertical layout: Toolbar (40px) -> Main Content -> Timeline (100px)
- Main Content horizontal: Left Panel (250px) | Viewport (Fill) | Right Panel (300px)
App struct with #[derive(Live, LiveHook)]:
- ui: WidgetRef
- display_manager: DisplayManager
- transform_buffer: TransformBuffer
- rerun_bridge: Option<RerunBridge>
- config: AppConfig
Implement LiveRegister, MatchEvent, AppMain traits.
Initialize Rerun connection in handle_startup."
```

### Task 4.2: Displays Panel Widget (Dev G) [COMPLETED]
**Duration**: 5 hours
**Depends on**: Task 4.1
**Status**: Implemented in mviz-widgets/src/displays_panel.rs

```
AI Prompt:
"Create displays panel in rv-ui/src/widgets/display_panel.rs:
live_design! for:
- DisplaysPanel: header with 'Displays' label + add button, PortalList of DisplayListItem
- DisplayListItem: checkbox (enabled) + name label + status icon
DisplaysPanel widget:
- displays: Vec<DisplayInfo>
- selected_index: Option<usize>
Methods:
- set_displays(Vec<DisplayInfo>)
- selected_display() -> Option<u64>
Handle checkbox toggle to enable/disable displays.
Handle click to select display (show in properties panel)."
```

### Task 4.3: Properties Panel Widget (Dev G) [COMPLETED]
**Duration**: 6 hours
**Depends on**: Task 4.1
**Status**: Implemented in mviz-widgets/src/properties_panel.rs

```
AI Prompt:
"Create properties panel in rv-ui/src/widgets/properties_panel.rs:
PropertyWidget implementations for each type:
- BoolPropertyWidget: checkbox
- FloatPropertyWidget: label + text input with validation
- ColorPropertyWidget: color preview + RGB inputs
- EnumPropertyWidget: dropdown
- Vec3PropertyWidget: X/Y/Z float inputs
PropertiesPanel:
- current_display: Option<DisplayId>
- properties: Vec<Box<dyn PropertyWidget>>
Methods:
- set_display(display) - populate properties from display
- on_property_change(name, value) - notify display of change
Use Makepad's standard widgets (Button, TextInput, CheckBox, DropDown)."
```

### Task 4.4: Toolbar Widget (Dev G) [COMPLETED]
**Duration**: 3 hours
**Depends on**: Task 4.1
**Status**: Implemented in mviz-widgets/src/toolbar.rs

```
AI Prompt:
"Create toolbar in rv-ui/src/widgets/toolbar.rs:
Horizontal layout with:
- File menu button (Open, Save, Save As, Recent)
- Tool buttons (Move, Select, Measure) - toggle group
- View buttons (Reset View, Focus Selection)
- Frame selector dropdown
- Play/Pause button for time control
ToolbarWidget:
- active_tool: String
- fixed_frame: String
- available_frames: Vec<String>
Emit actions for button clicks."
```

### Task 4.5: System Log Panel Widget (Dev G) [COMPLETED]
**Duration**: 6 hours
**Depends on**: Task 4.1
**Status**: Implemented in mviz-widgets/src/log_panel.rs

```
AI Prompt:
"Create system log panel in mviz-widgets/src/log_panel.rs:
Displays log messages from robot nodes over Zenoh with dynamic filtering.

Core Types (in mviz-core/src/zenoh_protocol.rs):
- LogLevel enum: Debug, Info, Warn, Error with color() method
- LogEntry struct: level, message, node_id, timestamp, metadata
- LogData struct: JSON payload for log messages

Zenoh Receiver Updates (in mviz-shell/src/zenoh_receiver.rs):
- ZenohMessage::Log(LogEntry) - system log entry
- ZenohMessage::NodeDiscovered(String) - new node ID
- discovered_nodes: Arc<RwLock<HashSet<String>>> - dynamic tracking

Widget (live_design! macro):
- LogPanel with collapsible header, entry count, Copy/Clear buttons
- Filter row: Level dropdown, Node dropdown (dynamic), Search input
- ScrollYView with log_list for entries
- LogEntryItem: timestamp, level badge, node name, message

LogPanelAction enum:
- CopyClicked, ClearClicked, ToggleCollapsed
- LevelFilterChanged(usize), NodeFilterChanged(String), SearchChanged(String)

LogDisplayEntry struct:
- timestamp: f64, level: u8, level_str, node_id, message

LogPanel struct (#[derive(Live, LiveHook, Widget)]):
- collapsed: bool, entries: Vec<LogDisplayEntry>
- filtered_entries: Vec<usize>, level_filter: usize
- node_filter: String, search_text: String
- discovered_nodes: Vec<String>, max_entries: usize (1000)

Methods:
- add_entry(cx, entry) - add and auto-prune
- clear(cx) - clear all entries
- set_discovered_nodes(cx, nodes) - update node filter dropdown
- get_filtered_text() -> String - for clipboard copy
- apply_filters() - recompute filtered_entries

Features:
1. Dynamic node discovery from log messages
2. Multi-level filtering (All/Debug/Info/Warn/Error)
3. Node-specific filtering
4. Text search in messages and node IDs
5. Color-coded entries (gray/blue/yellow/red)
6. Collapsible panel
7. Copy to clipboard
8. Auto-prune at 1000 entries"
```

**Acceptance Criteria**:
- [x] Log messages displayed from Zenoh `mviz/**` topics with type="log"
- [x] Dynamic node discovery populates filter dropdown
- [x] Level filtering works correctly
- [x] Search filtering works
- [x] Copy exports filtered text
- [x] Clear removes all entries
- [x] Panel is collapsible
- [x] Color coding by log level

**Execution Results (2026-01-09)**:

Test Environment:
- Robot side: Path-following dataflow with mviz-dora-bridge
- PC side: mviz-shell with Zenoh connection

Nodes Discovered via Zenoh:
| Node ID | Description |
|---------|-------------|
| sim_pose | Vehicle simulation pose publisher |
| bicycle_model | Bicycle model dynamics node |
| sim_state | Simulation state manager |
| target_point | Target waypoint generator |
| imu_msg | IMU sensor data publisher |

Log Entry Performance:
- Entries accumulated: 1 → 51 → 101 → ... → 701+ (50 entries/batch)
- Total Zenoh messages: 57,000+
- Node dropdown updated in real-time as nodes were discovered
- Filter changes applied immediately

Implementation Notes:
- v0.1.7: Initial LogPanel with display rendering
- v0.1.8: Dynamic node filter using `DropDownRef.set_labels()` API

### Task 4.6: Display Manager (Dev A)
**Duration**: 4 hours
**Depends on**: Task 1.4

```
AI Prompt:
"Implement display manager in rv-core or rv-ui:
- DisplayManager:
  - displays: HashMap<u64, Box<dyn Display>>
  - next_id: u64
  - registry: DisplayRegistry
Methods:
- add(display) -> u64 - returns display ID
- remove(id)
- get(id) -> Option<&dyn Display>
- get_mut(id) -> Option<&mut dyn Display>
- update_all(context, wall_dt, ros_dt) - calls update on all enabled displays
- initialize_all(context)
- save_all() -> Vec<DisplayConfig>
- load_all(configs, context)
Thread-safe with parking_lot::RwLock."
```

### Task 4.7: Configuration Save/Load (Dev A)
**Duration**: 3 hours
**Depends on**: Task 1.5, 4.5

```
AI Prompt:
"Implement configuration save/load integration:
In App:
- save_configuration(path) - collect config from all components
  - Global options
  - Display configs from DisplayManager
  - Current view controller state
  - Panel visibility states
  - Window geometry
- load_configuration(path) - restore all state
  - Create displays from configs
  - Set global options
  - Restore view
  - Set panel states
Add file dialog integration for Open/Save menu items."
```

### Task 4.8: Node Detail Panel Widget (Dev G) [COMPLETED]
**Duration**: 8 hours
**Depends on**: Task 4.1, Task 4.5
**Status**: Implemented in mviz-widgets/src/node_detail_panel.rs

```
AI Prompt:
"Create node detail panel in mviz-widgets/src/node_detail_panel.rs:
Displays detailed information about individual dataflow nodes including
input/output connections and filtered logs.

Layout:
┌─────────────────────────────────────────────────────────────────────┐
│ NODE: [dropdown]                                             [●]   │
├─────────────────────────────────────────────────────────────────────┤
│ INPUTS:                          │ OUTPUTS:                        │
│  • port (from: source/output)    │  • port → [dest1, dest2]       │
├─────────────────────────────────────────────────────────────────────┤
│ LOGS:                                                  [Clear]     │
│ [timestamp] message...                                              │
└─────────────────────────────────────────────────────────────────────┘

Protocol Types (mviz-core/src/zenoh_protocol.rs):
- NodeInput: name, source (e.g., 'camera/image')
- NodeOutput: name, destinations Vec<String>
- NodeDefinition: id, node_type, inputs, outputs, env, status
- NodeStatus: Running, Stopped, Error(String), Unknown
- DataflowDefinition: name, nodes Vec<NodeDefinition>, timestamp

Zenoh Message Extensions:
- ZenohMessage::DataflowDefinition(DataflowDefinition)
- ZenohMessage::NodeStatusUpdate(String, NodeStatus)

Bridge Updates:
- publish_dataflow_definition() - parse YAML and publish on startup
- parse_node_inputs(node) - extract input ports from YAML
- parse_node_outputs(node, all_nodes) - extract outputs with destinations

Widget (live_design! macro):
- NodeDetailPanel with header, node dropdown, status indicator
- Two-column I/O section: inputs (yellow) | outputs (blue)
- Scrollable logs section with Clear button

NodeDetailPanelAction enum:
- NodeSelected(String), ClearLogsClicked

NodeDetailPanel struct (#[derive(Live, LiveHook, Widget)]):
- nodes: HashMap<String, NodeDisplayState>
- selected_node: Option<String>
- node_logs: Vec<LogDisplayEntry>
- max_logs: usize (500)

Methods:
- set_dataflow(cx, definition) - populate from DataflowDefinition
- add_discovered_node(cx, node_id) - add node from discovery
- add_log(cx, entry) - add log entry (filtered by selected node)
- clear_logs(cx) - clear logs for current node
- update_node_status(cx, node_id, status)

Features:
1. Node selector dropdown (from discovered nodes or dataflow definition)
2. Input ports with source node/output
3. Output ports with destination nodes list
4. Status indicator (green=running, yellow=unknown, red=error)
5. Filtered logs (only for selected node)
6. Real-time log streaming via Zenoh
7. Clear logs button"
```

**Acceptance Criteria**:
- [x] Node dropdown populated from discovered nodes
- [x] Inputs display shows live I/O activity with timestamps
- [x] Outputs display shows live I/O activity with timestamps
- [x] Status indicator updates based on node selection
- [x] Logs filtered to selected node only
- [x] Clear button removes logs for current node
- [x] Node switching filters logs for new node
- [x] Dropdown updates when new nodes discovered from logs

**Dependencies**:
1. Bridge must publish `mviz/dataflow/definition` topic on startup
2. Bridge must parse dataflow YAML for node I/O structure
3. Logs must include node_id field for filtering

**Zenoh Topics**:
| Topic | Direction | Content |
|-------|-----------|---------|
| `mviz/dataflow/definition` | Bridge → Shell | Full dataflow graph JSON |
| `mviz/node/{node_id}/status` | Bridge → Shell | Node status updates |
| `mviz/logs` | Bridge → Shell | Log entries (existing) |

---

## Phase 5: Polish & Testing (Week 10)

### Task 5.1: Integration Testing (All Devs)
**Duration**: Distributed

```
AI Prompt for each crate:
"Create integration tests for [crate_name]:
- Test public API with realistic scenarios
- Test error handling
- Test edge cases
- Use test fixtures for sample data (URDF files, point clouds, etc.)
Put tests in tests/ directory."
```

### Task 5.2: Documentation (All Devs)
**Duration**: Distributed

```
AI Prompt:
"Add comprehensive documentation:
- Module-level docs with examples
- Public API rustdoc with examples
- README.md for each crate
- Architecture diagrams in docs/"
```

### Task 5.3: Performance Optimization (Dev C, E)
**Duration**: 4 hours

Focus areas:
- Point cloud batching and LOD
- Transform caching
- Lazy UI updates
- Rerun logging optimization

### Task 5.4: Error Handling Polish (Dev A)
**Duration**: 3 hours

```
AI Prompt:
"Improve error handling across all crates:
- Use thiserror for all error types
- Ensure all errors have context
- Add error recovery where appropriate
- Log errors consistently
- Display user-friendly error messages in UI"
```

---

## Synchronization Points

### Daily Standups
- 15 min async standup (Slack/Discord)
- Share: completed, in progress, blockers

### Integration Checkpoints

| Checkpoint | When | Participants | Goal |
|------------|------|--------------|------|
| **Core API Review** | End of Week 2 | All | Review rv-core traits before implementation |
| **Transform Integration** | End of Week 3 | B, C, E, F | Verify transform buffer works with displays |
| **Rerun Demo** | End of Week 4 | All | Demo basic displays working in Rerun |
| **Robot Model Demo** | End of Week 6 | D, F | Demo URDF loading and visualization |
| **UI Integration** | End of Week 8 | All | Full app running with all components |
| **Feature Complete** | End of Week 9 | All | All planned features implemented |
| **Release Candidate** | End of Week 10 | All | Testing and polish complete |

---

## Git Workflow

### Branch Strategy
```
main
 └── develop
      ├── feature/rv-core-types (Dev A)
      ├── feature/rv-transform (Dev B)
      ├── feature/rv-rerun-bridge (Dev C)
      ├── feature/rv-urdf (Dev D)
      ├── feature/displays-basic (Dev E)
      ├── feature/displays-marker (Dev F)
      └── feature/ui-shell (Dev G)
```

### Merge Rules
- PR requires 1 approval
- CI must pass (cargo build, test, clippy, fmt)
- Merge to develop daily when possible
- Merge develop to main at each checkpoint

---

## AI Coding Best Practices

### Prompt Templates

**For New File:**
```
"Create [filename] in [crate]/src/[path]:
- [List structs/enums with fields]
- [List methods with signatures]
- [Specify traits to implement]
- [List dependencies to use]
Include unit tests. Use idiomatic Rust."
```

**For Bug Fix:**
```
"Fix bug in [file:line]:
Problem: [description]
Expected: [behavior]
Actual: [behavior]
Provide minimal fix with explanation."
```

**For Refactoring:**
```
"Refactor [component] to:
- [Goal 1]
- [Goal 2]
Maintain backward compatibility with existing API.
Show before/after for key changes."
```

### Code Review with AI
Before PR, ask AI to review:
```
"Review this code for:
- Correctness
- Error handling
- Performance
- Rust idioms
- Documentation
Suggest improvements."
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Makepad learning curve | Dev G starts early; pair programming |
| Rerun API changes | Pin version; abstract behind bridge |
| URDF complexity | Start with simple robots; defer XACRO |
| Performance issues | Profile early; have optimization sprint |
| Integration delays | Daily integration; feature flags |

---

## Success Metrics

### Phase Completion Criteria

| Phase | Criteria |
|-------|----------|
| Phase 0 | Workspace builds; CI green |
| Phase 1 | Unit tests pass; API review approved |
| Phase 2 | Grid + PointCloud visible in Rerun |
| Phase 3 | Robot model animates with TF |
| Phase 4 | Full UI functional; config save/load works |
| Phase 5 | 30+ FPS with 100k points; all tests pass |

---

---

## Phase 8: Dataflow Graph Visualization [COMPLETED v0.3.1-v0.3.2]

### Task 8.1: Graph State Tracking in Bridge [COMPLETED]
**Duration**: 4 hours
**Status**: Implemented in mviz-rerun-bridge/src/main.rs

- Added `GraphState` struct for tracking discovered graph structure
- `init_from_definitions()` - initialize from YAML node definitions
- `record_input()` - infer edges from input patterns (e.g., `source_node/output_port`)
- `to_graph_update()` - generate GraphUpdate message for publishing
- Publishes graph updates every 2 seconds via Zenoh

### Task 8.2: Graph Protocol Types [COMPLETED]
**Duration**: 2 hours
**Status**: Implemented in mviz-core/src/zenoh_protocol.rs

- `GraphNodeStatus` enum: Active, Idle, Error
- `GraphNode` struct: id, status, last_seen
- `GraphEdge` struct: from_node, from_port, to_node, to_port
- `GraphUpdate` struct: nodes, edges, timestamp

### Task 8.3: DataflowGraphWidget [COMPLETED]
**Duration**: 6 hours
**Status**: Implemented in mviz-widgets/src/dataflow_graph.rs

- ASCII-style text rendering with box characters (┌─┐└─┘)
- Hierarchical layout via `compute_layout()` using BFS
- Click detection for node selection
- Green emoji (🟢) for active nodes, white (⚪) for idle
- Status text: [RUN] / [---]
- DrawNodeBox and DrawEdgeLine shaders (prepared for future graphical rendering)

### Task 8.4: Graph Update Handler [COMPLETED]
**Duration**: 2 hours
**Status**: Implemented in mviz-shell/src/app.rs

- `ZenohMessage::GraphUpdate` handling in `process_zenoh_messages()`
- Converts protocol types to widget types
- Updates DataflowGraphWidget via `update_from_graph_update()`

**Acceptance Criteria**:
- [x] Graph displayed in left panel
- [x] Nodes show active/idle status with colored emoji
- [x] Edges displayed as arrows between nodes
- [x] Click to select nodes
- [x] Real-time updates via Zenoh

---

## Phase 9: UI Layout Improvements [COMPLETED v0.3.4-v0.3.7]

### Task 9.1: Scrollable Graph Canvas [COMPLETED]
**Duration**: 1 hour
**Status**: Implemented in mviz-widgets/src/dataflow_graph.rs

- Added `scroll_bars: <ScrollBars> {}` to graph_canvas View
- Graph content scrollable when larger than visible area
- Changed Label dimensions to Fit for proper sizing

### Task 9.2: Panel Width Configuration [COMPLETED]
**Duration**: 1 hour
**Status**: Implemented in mviz-shell/src/app.rs

- Left panel: 280px → 340px
- Right panel: 280px → 340px
- More space for dataflow graph, properties, and logs

### Task 9.3: Fixed Three-Column Layout [COMPLETED]
**Duration**: 2 hours
**Status**: Implemented in mviz-shell/src/app.rs

- Stable fixed-width View layout (Splitter has rendering bugs)
- Left: 340px, Center: Fill, Right: 340px
- 1px divider lines (#333) between panels
- Explicit `show_bg: true` with background colors

**Note**: Makepad Splitter widget causes blank screen - both nested and single-level configurations fail. Using fixed Views until Splitter is fixed.

**Acceptance Criteria**:
- [x] All three panels visible
- [x] Dataflow graph scrollable
- [x] Wider panels for more content
- [x] Visible divider lines

---

## Appendix: Task Dependencies Graph

```
Task 0.1 (Setup)
    │
    ▼
┌───────────────────────────────────────────────────┐
│                    Phase 1                         │
│                                                    │
│  Task 1.1 ──┬──▶ Task 1.6 ──▶ Task 1.7            │
│  (Transform) │                                     │
│              │                                     │
│              ├──▶ Task 1.8 ──┬──▶ Task 1.9        │
│              │   (Bridge)    ├──▶ Task 1.10       │
│  Task 1.2 ───┤              └──▶ Task 1.11       │
│  (PointCloud)│                                     │
│              │                                     │
│  Task 1.3 ───┘                                     │
│  (Marker)                                          │
│                                                    │
│  Task 1.4 ──▶ Task 1.5                            │
│  (Display)    (Config)                             │
│                                                    │
│  Task 1.12 ──▶ Task 1.13  ✓                       │
│  (URDF) ✓     (Mesh) ✓                             │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│                    Phase 2                         │
│                                                    │
│  Task 2.1 ──┬──▶ Task 2.2 (Grid)                  │
│  (Base)     ├──▶ Task 2.3 (Axes)                  │
│             ├──▶ Task 2.4 (TF)                    │
│             ├──▶ Task 2.5 (PointCloud)            │
│             ├──▶ Task 2.6 (Marker)                │
│             ├──▶ Task 2.7 (Path)                  │
│             ├──▶ Task 2.8 (Pose)                  │
│             └──▶ Task 2.9 (Image)                 │
│                                                    │
│  Task 2.10 ──▶ Task 2.11 (Orbit)                  │
│  (ViewController)                                  │
│                                                    │
│  Task 2.12 (Tool)                                  │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│                    Phase 3                         │
│                                                    │
│  Task 3.1 (RobotModel) ✓                          │
│  Task 3.2 (LaserScan)                             │
│  Task 3.3 (FPS)                                    │
│  Task 3.4 (TopDown)                               │
│  Task 3.5 (Select)                                 │
│  Task 3.6 (Measure)                               │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│                    Phase 4                         │
│                                                    │
│  Task 4.1 ──┬──▶ Task 4.2 (DisplaysPanel) ✓       │
│  (AppShell) ├──▶ Task 4.3 (PropertiesPanel) ✓     │
│             ├──▶ Task 4.4 (Toolbar) ✓             │
│             ├──▶ Task 4.5 (SystemLogPanel) ✓      │
│             └──▶ Task 4.8 (NodeDetailPanel) ✓     │
│                                                    │
│  Task 4.6 ──▶ Task 4.7                            │
│  (Manager)    (Config)                             │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│                    Phase 5                         │
│                                                    │
│  Task 5.1-5.4 (Polish)                            │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│              Phase 8 [COMPLETED]                   │
│                                                    │
│  Task 8.1 (GraphState) ──▶ Task 8.2 (Protocol)    │
│                              │                     │
│                              ▼                     │
│  Task 8.3 (DataflowGraphWidget) ◀─────────────    │
│       │                                            │
│       └──▶ Task 8.4 (Handler)                     │
└───────────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────┐
│              Phase 9 [COMPLETED]                   │
│                                                    │
│  Task 9.1 (ScrollBars)                            │
│  Task 9.2 (Panel Widths: 340px)                   │
│  Task 9.3 (Fixed Layout)                          │
└───────────────────────────────────────────────────┘
```
