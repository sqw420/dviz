# MViz Phase 1 Implementation Plan

## Option 2: Separate Rerun Window with Makepad Control Panel

This plan implements the robotics visualizer using Makepad for the control UI and Rerun as a separate 3D visualization window, as recommended in `mviz_design.md` Section 13.

---

## Overview

```
┌────────────────────────────────┐      ┌────────────────────────────────┐
│     Makepad Control Panel      │      │     Rerun Viewer Window        │
│  ┌──────────────────────────┐  │      │  ┌──────────────────────────┐  │
│  │    Sensor Status Panel   │  │      │  │                          │  │
│  ├──────────────────────────┤  │      │  │    3D Space View         │  │
│  │   Display Controls       │  │ TCP  │  │    - Point Clouds        │  │
│  ├──────────────────────────┤  │◀────▶│  │    - IMU Arrows          │  │
│  │   Connection Status      │  │      │  │    - Vehicle Trail       │  │
│  ├──────────────────────────┤  │      │  ├──────────────────────────┤  │
│  │   Play/Pause/Record      │  │      │  │  Time Series Plots       │  │
│  └──────────────────────────┘  │      │  └──────────────────────────┘  │
└────────────────────────────────┘      └────────────────────────────────┘
         Makepad App                          Rerun Viewer (spawned)
```

---

## Reference Implementation

**Working Code**: `/Users/nupylot/Public/dora-viz/`
- `mviz-shell/` - Makepad control panel application
- `dora-rerun-bridge/` - Rerun data bridge for sensor messages
- `mviz-widgets/` - Custom Makepad widgets (theme, sensor panel, control bar)

**Test Data Source**: `/Users/nupylot/Public/dora-examples/examples/vehicle-path-following/`
- `bicycle_model.py` - Vehicle simulator (outputs sim_pose, sim_state)
- `imu_synthesizer.py` - IMU data generator
- `sim_visualizer.py` - Reference Rerun visualization

---

## Step-by-Step Implementation Plan

### Step 1: Project Setup
**Duration**: 2 hours
**Parallel**: All developers

#### 1.1 Create Workspace Structure

```bash
mkdir -p mviz
cd mviz
```

**Cargo.toml** (workspace root):
```toml
[workspace]
resolver = "2"
members = [
    "mviz-shell",
    "mviz-widgets",
    "mviz-rerun-bridge",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[workspace.dependencies]
# Makepad UI framework
makepad-widgets = { git = "https://github.com/makepad/makepad", branch = "main" }

# Rerun SDK for 3D visualization
rerun = "0.22"

# Math
glam = { version = "0.25", features = ["serde"] }
nalgebra = "0.33"

# Async runtime
tokio = { version = "1", features = ["full", "sync"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Utilities
parking_lot = "0.12"
log = "0.4"
env_logger = "0.11"
eyre = "0.6"
thiserror = "1.0"
crossbeam-channel = "0.5"
rand = "0.8"
```

#### 1.2 Create Crate Directories

```bash
mkdir -p mviz-shell/src
mkdir -p mviz-widgets/src
mkdir -p mviz-rerun-bridge/src
```

**Acceptance Criteria**:
- [ ] `cargo build` succeeds
- [ ] All three crates compile

---

### Step 2: Makepad Widget Library
**Duration**: 4 hours
**Reference**: `/Users/nupylot/Public/dora-viz/mviz-widgets/`

#### 2.1 Create Theme Module

**File**: `mviz-widgets/src/theme.rs`

```
AI Prompt:
"Create Makepad theme module with:
- Color constants: DARK_BG (#1a1a1a), PANEL_BG (#252525), ACCENT_BLUE (#3b82f6), etc.
- Text colors: TEXT_PRIMARY (#ffffff), TEXT_SECONDARY (#a0a0a0), TEXT_MUTED (#606060)
- Font styles: FONT_REGULAR, FONT_MEDIUM, FONT_SEMIBOLD, FONT_BOLD
- Common widget styles for buttons, labels, panels
Reference: dora-viz/mviz-widgets/src/theme.rs"
```

#### 2.2 Create Sensor Panel Widget

**File**: `mviz-widgets/src/sensor_panel.rs`

```
AI Prompt:
"Create Makepad SensorGroup widget for displaying sensor data:
- Header with group title and status indicator (active/inactive dot)
- Sensor rows with label + value (SensorRow widget)
- Support for IMU data: accel_x/y/z, gyro_x/y/z
- Support for position data: x, y, z
- Live update via set_text() methods
Use live_design! macro for declarative UI.
Reference: dora-viz/mviz-widgets/src/sensor_panel.rs"
```

#### 2.3 Create Control Bar Widget

**File**: `mviz-widgets/src/control_bar.rs`

```
AI Prompt:
"Create Makepad ControlBar widget with:
- Rerun connection status section (dot indicator + label)
- Play/Pause toggle button with icon change
- Record button
- Connection status (Connected/Disconnected)
Handle hover states and click events.
Reference: dora-viz/mviz-widgets/src/control_bar.rs"
```

**Acceptance Criteria**:
- [ ] All widgets render correctly
- [ ] Theme colors applied
- [ ] Widgets exported from lib.rs

---

### Step 3: Rerun Bridge Module
**Duration**: 4 hours
**Reference**: `/Users/nupylot/Public/dora-viz/dora-rerun-bridge/`

#### 3.1 Create Rerun Connection Manager

**File**: `mviz-rerun-bridge/src/lib.rs`

```rust
//! Rerun Bridge - Manages connection to Rerun viewer and data logging

use rerun::RecordingStream;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct RerunBridge {
    stream: Option<RecordingStream>,
    recording_id: String,
    connected: bool,
}

impl RerunBridge {
    pub fn new(recording_id: &str) -> Self {
        Self {
            stream: None,
            recording_id: recording_id.to_string(),
            connected: false,
        }
    }

    /// Spawn Rerun viewer as separate window
    pub fn spawn_viewer(&mut self) -> Result<(), RerunError> {
        let stream = rerun::RecordingStreamBuilder::new(&self.recording_id)
            .spawn()
            .map_err(|e| RerunError::SpawnFailed(e.to_string()))?;

        // Set up world coordinates
        stream.log_static(
            "world",
            &rerun::ViewCoordinates::RIGHT_HAND_Z_UP(),
        )?;

        self.stream = Some(stream);
        self.connected = true;
        Ok(())
    }

    /// Connect to existing Rerun viewer
    pub fn connect(&mut self, addr: &str) -> Result<(), RerunError> {
        let stream = rerun::RecordingStreamBuilder::new(&self.recording_id)
            .connect_tcp_opts(addr.parse()?, Default::default())
            .map_err(|e| RerunError::ConnectionFailed(e.to_string()))?;

        self.stream = Some(stream);
        self.connected = true;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn stream(&self) -> Option<&RecordingStream> {
        self.stream.as_ref()
    }
}
```

#### 3.2 Create Sensor Data Adapters

**File**: `mviz-rerun-bridge/src/adapters.rs`

```
AI Prompt:
"Create Rerun adapters for sensor data:

1. ImuAdapter:
   - log_imu(stream, accel: [f32;3], gyro: [f32;3])
   - Log accelerometer as arrow from origin (green)
   - Log gyroscope as arrow (red)
   - Log scalar time series for each axis

2. PointCloudAdapter:
   - log_pointcloud(stream, points: &[[f32;3]], colors: Option<&[[u8;3]]>)
   - Map to rerun::Points3D with radii and colors
   - Support colormap by Z-height

3. PoseAdapter:
   - log_pose(stream, x, y, z, theta)
   - Log position as point (blue)
   - Log orientation as arrow
   - Maintain trajectory buffer and log as LineStrips3D

4. LaserScanAdapter:
   - log_scan(stream, ranges: &[f32], angle_min, angle_increment)
   - Convert polar to Cartesian
   - Log as Points3D with range-based coloring

Reference: dora-viz/dora-rerun-bridge/src/main.rs (log_imu_data, log_position_data, etc.)"
```

**Acceptance Criteria**:
- [ ] Can spawn Rerun viewer
- [ ] IMU data visualizes as arrows
- [ ] Point clouds display correctly
- [ ] Trajectory accumulates over time

---

### Step 4: Makepad Application Shell
**Duration**: 6 hours
**Reference**: `/Users/nupylot/Public/dora-viz/mviz-shell/src/app.rs`

#### 4.1 Create Main Application

**File**: `mviz-shell/src/app.rs`

```
AI Prompt:
"Create Makepad application shell with:

Window layout:
- Header: App icon + title + 'Launch Rerun Viewer' button
- Main content (horizontal split):
  - Left panel (320px): Control bar + Sensor panels + Status footer
  - Right panel (fill): Info/instructions area

App struct fields:
- ui: WidgetRef
- rerun_bridge: RerunBridge
- is_running: Arc<AtomicBool>
- frame_count: u64
- current_fps: f32

Key methods:
- handle_launch_button() - spawn Rerun viewer
- handle_control_buttons() - play/pause toggle
- update_sensor_display() - update IMU/position labels
- simulate_sensor_data() - generate test data for validation

Reference: dora-viz/mviz-shell/src/app.rs (full implementation)"
```

#### 4.2 Implement Event Handling

```
AI Prompt:
"Implement Makepad event handling:

1. Button hover effects (FingerHoverIn/Out)
2. Button click handling (FingerUp)
3. Periodic updates via timer events
4. FPS counter update (once per second)

Wire up:
- Launch Rerun button -> spawn_viewer()
- Play button -> toggle is_running
- When running, call simulate_sensor_data() each frame

Reference: dora-viz/mviz-shell/src/app.rs (handle_event, handle_launch_button)"
```

**Acceptance Criteria**:
- [ ] Window opens with correct layout
- [ ] Launch Rerun button spawns viewer
- [ ] Play/Pause toggles simulation
- [ ] Sensor values update in UI

---

### Step 5: Sensor Data Integration
**Duration**: 4 hours

#### 5.1 IMU Data Visualization

**Test with simulated data first:**

```rust
fn simulate_imu_data(&self) -> ([f32; 3], [f32; 3]) {
    let mut rng = rand::thread_rng();
    let accel = [
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        9.8 + rng.gen_range(-0.1..0.1),
    ];
    let gyro = [
        rng.gen_range(-0.1..0.1),
        rng.gen_range(-0.1..0.1),
        rng.gen_range(-0.1..0.1),
    ];
    (accel, gyro)
}
```

**Expected Rerun visualization:**
- `world/imu/accelerometer` - Green arrow showing acceleration vector
- `world/imu/gyroscope` - Red arrow showing rotation rate
- `sensors/accel_x`, `sensors/accel_y`, `sensors/accel_z` - Time series plots

#### 5.2 LiDAR/Point Cloud Visualization

**Test with simulated scan:**

```rust
fn simulate_lidar_scan(&self) -> Vec<[f32; 3]> {
    let num_points = 360;
    let mut points = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / num_points as f32;
        let range = 5.0 + rand::random::<f32>() * 2.0; // 5-7m range
        points.push([
            range * angle.cos(),
            range * angle.sin(),
            0.0,
        ]);
    }
    points
}
```

**Expected Rerun visualization:**
- `world/lidar` - Point cloud ring around origin

---

### Step 6: Vehicle Path Following Integration
**Duration**: 4 hours

#### 6.1 Connect to Dora Dataflow

Create a Dora node that bridges sensor data to our application:

**File**: `mviz-dora-bridge/src/main.rs`

```
AI Prompt:
"Create Dora node for receiving vehicle-path-following sensor data:

Inputs (from dora dataflow):
- sim_pose: [x, y, theta, velocity] from bicycle_model
- sim_state: [x, y, theta, velocity, steering, accel, yaw_rate]
- imu_msg: [roll, pitch, yaw, gyro_x/y/z, accel_x/y/z] from imu_synthesizer

For each input:
1. Parse the pyarrow array to f32 values
2. Log to Rerun using appropriate adapter
3. Update trajectory buffer for path visualization

Reference:
- dora-viz/dora-rerun-bridge/src/main.rs
- vehicle-path-following/src/sim_visualizer.py"
```

#### 6.2 Dataflow Configuration

**File**: `dataflow.yml`

```yaml
nodes:
  # Vehicle simulation from dora-examples
  - id: bicycle_model
    operator:
      python: ../dora-examples/examples/vehicle-path-following/src/bicycle_model.py
      inputs:
        tick: dora/timer/millis/20
        steering_cmd: simple_planner/steering_cmd
        throttle_cmd: simple_planner/throttle_cmd
      outputs:
        - sim_pose
        - sim_state

  - id: simple_planner
    operator:
      python: ../dora-examples/examples/vehicle-path-following/src/simple_planner.py
      inputs:
        sim_pose: bicycle_model/sim_pose
      outputs:
        - steering_cmd
        - throttle_cmd
        - target_point
        - waypoints

  - id: imu_synthesizer
    operator:
      python: ../dora-examples/examples/vehicle-path-following/src/imu_synthesizer.py
      inputs:
        sim_state: bicycle_model/sim_state
      outputs:
        - imu_msg

  # Our Rerun bridge
  - id: mviz-rerun-bridge
    build: cargo build --package mviz-rerun-bridge --release
    path: target/release/mviz-rerun-bridge
    inputs:
      sim_pose: bicycle_model/sim_pose
      sim_state: bicycle_model/sim_state
      imu_msg: imu_synthesizer/imu_msg
      target_point: simple_planner/target_point
      waypoints: simple_planner/waypoints
    env:
      RERUN_RECORDING_ID: mviz_vehicle
      RERUN_SPAWN: "true"
```

**Acceptance Criteria**:
- [ ] Vehicle position updates in Rerun
- [ ] IMU arrows animate with vehicle motion
- [ ] Trajectory trail accumulates
- [ ] Target point and waypoints visible

---

### Step 7: Validation Testing
**Duration**: 2 hours

#### 7.1 IMU Validation Checklist

| Test | Expected Result |
|------|-----------------|
| Accelerometer at rest | Arrow pointing up (Z = 9.8) |
| Vehicle accelerating | Arrow tilts in direction of motion |
| Gyroscope during turn | Arrow shows yaw rate magnitude |
| Time series plots | Smooth curves matching sensor values |

#### 7.2 LiDAR/Point Cloud Validation

| Test | Expected Result |
|------|-----------------|
| Static scan | Ring of points around vehicle |
| Range coloring | Near=blue, Far=red gradient |
| Multiple scans | Points accumulate with decay |
| Transform | Points in correct world frame |

#### 7.3 Integration Validation

```bash
# Terminal 1: Start Makepad control panel
cargo run --package mviz-shell --release

# Terminal 2: Start Dora dataflow
cd ../dora-examples/examples/vehicle-path-following
dora up
dora start dataflow_sim.yml --name validation

# Expected:
# 1. Makepad window shows sensor values updating
# 2. Rerun viewer shows vehicle moving along path
# 3. IMU arrows animate
# 4. Trajectory trail follows vehicle
```

---

## File Structure Summary

```
mviz/
├── Cargo.toml                    # Workspace root
├── dataflow.yml                  # Dora dataflow configuration
├── README.md
│
├── mviz-shell/                   # Makepad control panel
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs
│       └── app.rs                # Application logic
│
├── mviz-widgets/                 # Reusable Makepad widgets
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── theme.rs              # Colors, fonts, styles
│       ├── sensor_panel.rs       # Sensor data display
│       └── control_bar.rs        # Play/pause, status
│
└── mviz-rerun-bridge/           # Rerun integration
    ├── Cargo.toml
    └── src/
        ├── lib.rs                # RerunBridge struct
        ├── main.rs               # Dora node (optional)
        └── adapters.rs           # IMU, PointCloud, Pose adapters
```

---

## Timeline

| Day | Task | Deliverable |
|-----|------|-------------|
| 1 | Step 1-2 | Workspace + Widgets |
| 2 | Step 3 | Rerun Bridge |
| 3 | Step 4 | Makepad App Shell |
| 4 | Step 5 | Sensor Visualization |
| 5 | Step 6 | Dora Integration |
| 6 | Step 7 | Validation Testing |

**Total**: 1 week for Phase 1 MVP

---

## Success Criteria

### Minimum Viable Product (MVP)

- [ ] Makepad window launches and displays UI
- [ ] "Launch Rerun" button spawns separate Rerun viewer
- [ ] Simulated sensor data flows to Rerun
- [ ] IMU visualizes as 3D arrows
- [ ] Position visualizes as point + trajectory
- [ ] UI shows real-time sensor values

### Extended Goals

- [ ] Connect to vehicle-path-following dataflow
- [ ] LiDAR point cloud visualization
- [ ] Configuration save/load (YAML)
- [ ] Multiple sensor sources

---

## Key Code Patterns from Reference

### 1. Rerun Viewer Spawn (from dora-viz)

```rust
// mviz-shell/src/app.rs
fn launch_rerun_viewer(&mut self, cx: &mut Cx) {
    let rec = rerun::RecordingStreamBuilder::new("mviz_rerun")
        .spawn()
        .expect("Failed to spawn Rerun viewer");

    // Set coordinate system
    rec.log_static("world", &rerun::ViewCoordinates::RIGHT_HAND_Z_UP()).ok();

    self.rerun_handle = Some(rec);

    // Update UI status indicators
    self.ui.view(ids!(connection_dot))
        .apply_over(cx, live!{ draw_bg: { connected: 1.0 } });
}
```

### 2. IMU Logging (from dora-rerun-bridge)

```rust
fn log_imu_data(rec: &RecordingStream, accel: [f32; 3], gyro: [f32; 3]) {
    // Accelerometer arrow
    rec.log(
        "world/imu/accelerometer",
        &rerun::Arrows3D::from_vectors([[
            accel[0] * 0.1,
            accel[1] * 0.1,
            accel[2] * 0.1,
        ]])
        .with_origins([[0.0, 0.0, 0.0]])
        .with_colors([rerun::Color::from_rgb(16, 185, 129)]),
    ).ok();

    // Time series
    rec.log("sensors/accel_x", &rerun::Scalar::new(accel[0] as f64)).ok();
    rec.log("sensors/accel_y", &rerun::Scalar::new(accel[1] as f64)).ok();
    rec.log("sensors/accel_z", &rerun::Scalar::new(accel[2] as f64)).ok();
}
```

### 3. Vehicle Visualization (from sim_visualizer.py)

```python
# Vehicle box with orientation
rr.log("world/vehicle", rr.Boxes3D(
    centers=[[x, y, height/2]],
    half_sizes=[[length/2, width/2, height/2]],
    rotations=[rr.Quaternion(xyzw=[0, 0, sin(theta/2), cos(theta/2)])],
    colors=[[255, 200, 0, 255]],
))

# Direction arrow
rr.log("world/vehicle_direction", rr.Arrows3D(
    origins=[[x, y, height]],
    vectors=[[cos(theta) * 0.8, sin(theta) * 0.8, 0]],
    colors=[[255, 0, 0, 255]],
))
```

---

## Next Steps After Phase 1

1. **Phase 2**: Add display management UI (add/remove/configure displays)
2. **Phase 3**: Implement transform hierarchy (TF-like)
3. **Phase 4**: Add URDF robot model loading
4. **Evaluate**: Assess need for embedded Rerun (Option 1) based on user feedback
