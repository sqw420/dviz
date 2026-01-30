# RViz Feature Requirements for Makepad + Rerun Implementation

This document analyzes RViz's core features as requirements for building a similar robotics visualization tool using Makepad (Rust UI framework) and Rerun (visualization SDK).

---

## 1. Core Architecture

### 1.1 Display System
The fundamental visualization unit. Each display:
- Has enable/disable toggle
- Maintains status (OK/Warning/Error) with descriptive messages
- Owns a scene node in the 3D scene graph
- Receives periodic `update()` calls (~30Hz)
- Can subscribe to data topics
- Supports save/load configuration

**Rerun Mapping**: Rerun's `EntityPath` and logging API can replace ROS topics. Each display becomes a Rerun "space view" or custom visualizer.

### 1.2 Plugin Architecture
RViz supports runtime-loadable plugins for:
- Displays (data visualizers)
- Panels (UI widgets)
- Tools (mouse/keyboard interaction modes)
- View Controllers (camera control)
- Frame Transformers (coordinate systems)

**Makepad Mapping**: Rust trait-based plugin system with dynamic loading or compile-time registration.

---

## 2. Coordinate Frame System (Critical)

### 2.1 Transform Hierarchy (TF)
- **Fixed Frame**: Reference frame for all visualizations
- **Frame Manager**: Caches transforms, handles time synchronization
- Supports transform lookup at specific timestamps
- Provides `getTransform(frame, time) -> (position, orientation)`

### 2.2 Transform Features
| Feature | Description |
|---------|-------------|
| Frame hierarchy | Parent-child transform relationships |
| Time-based lookup | Get transform at specific timestamp |
| Transform caching | Performance optimization |
| Sync modes | Off / Exact time / Approximate time |
| Missing transform handling | Status warnings, graceful degradation |

**Rerun Mapping**: Rerun has built-in transform support via `rerun::Transform3D` and entity hierarchy.

---

## 3. Display Types (Priority Order)

### 3.1 Tier 1 - Essential Displays

#### Point Cloud (PointCloud2)
- Render modes: Points, Billboards (squares/circles), Boxes, Spheres
- Color transformers: Flat color, Intensity, RGB, Axis (X/Y/Z)
- Point size control (world units or pixels)
- Alpha transparency
- Decay time (accumulate points over time)
- Selectable points

#### TF Display
- Visualize transform hierarchy as axes
- Show/hide individual frames
- Frame name labels
- Parent-child relationship lines
- Configurable axis scale

#### Robot Model (URDF)
- Load URDF/XACRO robot descriptions
- Visual meshes with materials
- Collision geometry (optional)
- Joint state animation via TF
- Alpha transparency
- Mass/inertia visualization (optional)

#### Marker / MarkerArray
Programmable visualization primitives:
| Type | Description |
|------|-------------|
| Arrow | Direction indicator |
| Cube/Sphere/Cylinder | Basic shapes |
| Line Strip/List | Connected/individual lines |
| Points | Point collection |
| Text (View-Facing) | Billboard text labels |
| Mesh Resource | Load external mesh files |
| Triangle List | Custom mesh geometry |

#### Image
- Display camera images in separate window/panel
- Support common formats (RGB8, BGR8, Mono8, etc.)
- Image transport compression support

#### LaserScan
- Convert to point cloud visualization
- Range-based coloring
- Intensity coloring
- Angle filtering

### 3.2 Tier 2 - Common Displays

#### Grid
- Reference grid on ground plane
- Configurable cell size, count, color
- Line or billboard style

#### Axes
- Display coordinate axes at a frame
- Configurable scale

#### Path
- Visualize nav_msgs/Path as line
- Pose arrows along path (optional)

#### Pose / PoseArray
- Single pose as arrow/axes
- Array of poses with configurable shape

#### Odometry
- Accumulated pose trail
- Covariance ellipse visualization
- Arrow/axes representation

#### Map (OccupancyGrid)
- 2D occupancy grid as ground texture
- Color schemes: Raw, Map, Costmap
- Alpha transparency
- Draw behind (ignore depth)

#### Interactive Markers
- 3D widgets for user interaction
- Move/rotate/scale controls
- Menu support
- Feedback to application

### 3.3 Tier 3 - Specialized Displays

#### Camera
- Image with 3D world overlay
- Frustum visualization

#### DepthCloud
- Depth image to 3D point cloud

#### Range
- Ultrasonic/IR sensor cone

#### Wrench / Effort
- Force/torque visualization on robot joints

#### Polygon
- 2D polygon outline

#### GridCells
- Navigation costmap cells

---

## 4. View Controllers (Camera Control)

### 4.1 Required Controllers

| Controller | Description |
|------------|-------------|
| **Orbit** | Rotate around focal point, zoom in/out |
| **XY Orbit** | Orbit constrained to XY plane |
| **FPS** | First-person shooter style (WASD + mouse look) |
| **Top-Down Ortho** | Orthographic top view for 2D navigation |
| **Third Person Follower** | Follow a target frame |

### 4.2 Common Features
- Near/far clip plane control
- Focal point (look-at target)
- Mouse interaction: Left=rotate, Middle=pan, Right/Scroll=zoom
- Keyboard shortcuts: F=focus on selection, Z=reset view
- Stereo rendering support (optional)
- Invert Z-axis option

---

## 5. Interactive Tools

### 5.1 Tool System
- One active tool at a time
- Keyboard shortcuts for tool switching
- Tool-specific cursor
- Mouse event processing

### 5.2 Required Tools

| Tool | Shortcut | Function |
|------|----------|----------|
| Move Camera | M | Drag to orbit/pan/zoom |
| Select | S | Box selection of objects |
| Focus Camera | F | Click to focus on point |
| Measure | (none) | Click two points for distance |
| 2D Nav Goal | G | Publish goal pose |
| 2D Pose Estimate | P | Publish initial pose |
| Publish Point | (none) | Click to publish 3D point |
| Interact | I | Interact with interactive markers |

---

## 6. Properties System (UI)

### 6.1 Property Types
| Type | Widget |
|------|--------|
| Bool | Checkbox |
| Int | Spinbox |
| Float | Spinbox with precision |
| String | Text field |
| Enum | Dropdown |
| Color | Color picker |
| Vector3 | X/Y/Z spinboxes |
| Quaternion | X/Y/Z/W spinboxes |
| TF Frame | Dropdown with available frames |
| ROS Topic | Dropdown with available topics |
| File Path | File picker dialog |
| QoS Profile | Reliability dropdown |

### 6.2 Property Features
- Hierarchical tree structure
- Expand/collapse groups
- Read-only properties
- Property change notifications
- Hidden properties (conditional visibility)

**Makepad Mapping**: Makepad's widget system can implement these as custom components.

---

## 7. Selection System

### 7.1 Features
- Box selection in 3D viewport
- Add/Remove/Replace selection modes
- Selection highlighting
- Focus camera on selection
- Selection properties panel
- Per-object selection handlers

### 7.2 Picking
- 3D point picking (click to get 3D coordinate)
- Object picking (click to select object)
- GPU-based color picking for performance

---

## 8. Panels (UI Windows)

### 8.1 Core Panels
| Panel | Function |
|-------|----------|
| Displays | Add/remove/configure displays |
| Views | Switch view controllers, save views |
| Tool Properties | Configure active tool |
| Selection | Show selected object properties |
| Time | Playback control, time display |
| Help | Keyboard shortcuts |

---

## 9. Configuration Persistence

### 9.1 Save/Load
- YAML-based configuration files (.rviz)
- Save: displays, properties, views, window layout
- Recent file list
- Default configuration

### 9.2 Configuration Contents
```yaml
Panels: [...]
Visualization Manager:
  Displays: [...]
  Global Options:
    Fixed Frame: map
    Frame Rate: 30
  Tools: [...]
  Views:
    Current: Orbit
    Saved: [...]
Window Geometry: [...]
```

---

## 10. Data Input (Rerun Integration)

### 10.1 ROS Concept -> Rerun Mapping
| ROS Concept | Rerun Equivalent |
|-------------|------------------|
| Topic subscription | `rerun::RecordingStream` logging |
| Message types | Rerun archetypes (Points3D, Transform3D, etc.) |
| TF transforms | Entity hierarchy + Transform3D |
| Time synchronization | Rerun timeline |
| QoS profiles | N/A (Rerun handles internally) |

### 10.2 Rerun Archetypes to Support
- `Points3D` - Point clouds
- `Transform3D` - Coordinate frames
- `Mesh3D` - Robot meshes
- `Image` - Camera images
- `LineStrips3D` - Paths, laser scans
- `Arrows3D` - Poses, markers
- `Boxes3D`, `Ellipsoids3D` - Markers
- `TextLog` - Status messages

---

## 11. Performance Requirements

### 11.1 Targets
- 30+ FPS with 1M point cloud
- Smooth camera interaction
- Sub-100ms latency for live data
- Memory-efficient point cloud decay

### 11.2 Optimization Techniques (from RViz)
- Point cloud LOD (level of detail)
- Frustum culling
- Transform caching
- Lazy updates (only when data changes)
- Visibility bits for selective rendering

---

## 12. Implementation Priority

### Phase 1: Core Framework
1. 3D viewport with Orbit camera
2. Transform hierarchy / frame management
3. Grid display
4. Point cloud display (basic)
5. Configuration save/load

### Phase 2: Essential Displays
6. TF display
7. Marker display (all types)
8. Image display
9. LaserScan display
10. Selection system

### Phase 3: Robot Visualization
11. URDF parser
12. Robot model display
13. Interactive markers
14. Path/Pose displays

### Phase 4: Polish
15. All view controllers
16. All tools
17. Full property system
18. Performance optimization

---

## 13. Technology Stack Recommendation

| Component | Technology |
|-----------|------------|
| UI Framework | Makepad |
| 3D Rendering | Makepad's GPU abstraction or wgpu |
| Data Logging | Rerun SDK |
| Data Storage | Rerun's Arrow-based storage |
| URDF Parsing | `urdf-rs` crate |
| Math | `nalgebra` or `glam` |
| Serialization | `serde` + YAML |

---

## Appendix: RViz Display Type Reference

### Full Display List
```
Sensor Data:        Geometry:           Robot:
- PointCloud        - Marker            - RobotModel
- PointCloud2       - MarkerArray       - Effort
- LaserScan         - Path              - TF
- Image             - Polygon
- Camera            - Pose              Navigation:
- DepthCloud        - PoseArray         - Map
- Range             - PoseWithCovariance- GridCells
                    - Odometry          - Path
Environment:        - PointStamped
- Grid              - Wrench            Interactive:
- Axes              - Accel             - InteractiveMarkers
                    - Twist
```
