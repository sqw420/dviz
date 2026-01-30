# DViz

A modern robotics visualization tool built in Rust, combining Makepad UI, Rerun 3D visualization, and Zenoh distributed networking for debugging robots and autonomous systems.

## Features

- **Distributed-first architecture**: Debug robots remotely over LAN using Zenoh auto-discovery
- **Dora dataflow integration**: Visualize sensor pipelines and node activity
- **Rerun 3D visualization**: Point clouds, transforms, markers, robot models
- **ROS bag playback**: Multi-sensor support (LiDAR, IMU, GPS, TF)
- **URDF robot models**: Load and visualize robot meshes

## Installation

### Prerequisites

```bash
# Rust (nightly required for Makepad)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly
rustup default nightly

# Dora CLI
cargo install dora-cli

# Python dependencies (for example dataflows)
pip install dora-rs pyyaml numpy pyarrow
```

### Build DViz

```bash
git clone https://github.com/bobd988/dviz.git
cd dviz
cargo build --release
```

This builds:
- `dviz` - Main visualization shell
- `dviz-dora-bridge` - Dora node that publishes data via Zenoh

## Quick Start (Single Machine)

```bash
# Terminal 1: Start Dora daemon
dora up

# Terminal 2: Run a dataflow
dora start dataflow.yml --name dviz

# Terminal 3: Run DViz
cargo run -p dviz-shell --release
```

In the DViz UI:
1. Click **Spawn Rerun** to open the 3D viewer
2. Zenoh auto-connects on startup (status shows "Zenoh: Connected")
3. View vehicle simulation data in Rerun

## Example: Path Following Simulation

This example runs a vehicle path-following simulation with:
- `bicycle_model` - Vehicle physics simulator
- `simple_planner` - Pure pursuit path following
- `imu_synthesizer` - Synthetic IMU generation
- `dviz_bridge` - Publishes data via Zenoh

### Run Locally

```bash
# Terminal 1
dora up

# Terminal 2
dora start examples/dataflow-path-following.yml --name pathfollow

# Terminal 3
cargo run -p dviz-shell --release
```

### View in Rerun

The visualization shows:
- **Vehicle pose**: Arrow showing position and heading
- **Waypoints**: Line strip showing planned path
- **Target point**: Current pursuit target
- **IMU data**: Acceleration/gyroscope scalars

## Distributed Setup (Robot + Debug PC)

DViz is designed for remote debugging where:
- **Robot** runs the dataflow headlessly (no display)
- **Debug PC** runs DViz to visualize robot data over LAN

### Architecture

```
┌─────────────────────┐         Zenoh (LAN)         ┌─────────────────────┐
│       ROBOT         │ ─────────────────────────── │      DEBUG PC       │
│  (192.168.1.10)     │         UDP Multicast       │   (192.168.1.20)    │
│                     │         224.0.0.225:7446    │                     │
│  dora up            │                             │  cargo run -p       │
│  dora start         │                             │    dviz-shell       │
│    dataflow-robot   │         ────────────>       │                     │
│      .yml           │         sensor data         │  Rerun 3D viewer    │
└─────────────────────┘                             └─────────────────────┘
```

### Step 1: Robot Side (Headless)

SSH into the robot and run:

```bash
cd /path/to/dviz

# Build the bridge (first time only)
cargo build --release -p dviz-rerun-bridge

# Start Dora daemon
dora up

# Run the robot dataflow
dora start examples/dataflow-robot.yml --name robot
```

The robot will publish data to Zenoh topics:
- `dviz/pose` - Vehicle position
- `dviz/state` - Vehicle state
- `dviz/imu` - IMU readings
- `dviz/waypoints` - Planned path

### Step 2: Debug PC Side

```bash
cd /path/to/dviz

# Run DViz
cargo run -p dviz-shell --release
```

DViz auto-connects via Zenoh multicast scouting. Both machines must be on the **same LAN subnet**.

## Zenoh Network Configuration

### Default: Auto-Discovery (Recommended)

Zenoh uses **multicast scouting** by default:
- UDP multicast on `224.0.0.225:7446`
- Works automatically on the same LAN subnet
- No configuration needed

### When Auto-Discovery Fails

If machines are on different subnets or multicast is blocked:

#### Option 1: Direct Connection

Set environment variable on the subscriber (debug PC):

```bash
ZENOH_CONNECT=tcp/192.168.1.10:7447 cargo run -p dviz-shell
```

Or in the dataflow YAML on the robot:

```yaml
nodes:
  - id: dviz_bridge
    path: target/release/dviz-dora-bridge
    env:
      ZENOH_CONNECT: tcp/192.168.1.20:7447  # Debug PC address
```

#### Option 2: Zenoh Router

Run a Zenoh router as a central hub:

```bash
# On any machine accessible by both robot and PC
zenohd
```

Then configure both sides to connect:

```bash
# Robot
ZENOH_CONNECT=tcp/router-ip:7447 dora start dataflow-robot.yml

# PC
ZENOH_CONNECT=tcp/router-ip:7447 cargo run -p dviz-shell
```

### When IP Addresses Change

#### Scenario 1: Robot IP Changed

1. Find new robot IP: `ip addr` or `hostname -I`
2. If using direct connection, update debug PC:
   ```bash
   ZENOH_CONNECT=tcp/<new-robot-ip>:7447 cargo run -p dviz-shell
   ```
3. If using auto-discovery, no changes needed (same subnet)

#### Scenario 2: Debug PC IP Changed

1. If robot is configured to connect to PC:
   ```yaml
   # Update dataflow-robot.yml
   env:
     ZENOH_CONNECT: tcp/<new-pc-ip>:7447
   ```
2. Restart the dataflow:
   ```bash
   dora stop robot
   dora start examples/dataflow-robot.yml --name robot
   ```

#### Scenario 3: Different Subnet/Network

Use a Zenoh router with a static IP that both can reach:

```bash
# Router machine (e.g., 192.168.1.1)
zenohd --listen tcp/0.0.0.0:7447

# Robot (any subnet)
ZENOH_CONNECT=tcp/192.168.1.1:7447 dora start ...

# PC (any subnet)
ZENOH_CONNECT=tcp/192.168.1.1:7447 cargo run -p dviz-shell
```

### Network Troubleshooting

| Symptom | Cause | Solution |
|---------|-------|----------|
| "Zenoh: Connecting..." hangs | Multicast blocked | Use `ZENOH_CONNECT` |
| No data in Rerun | Firewall blocking | Open UDP 7446, TCP 7447 |
| Intermittent data | Network congestion | Use wired connection |
| Works locally, not remote | Different subnets | Use Zenoh router |

#### Firewall Configuration

```bash
# Linux (robot and PC)
sudo ufw allow 7446/udp  # Zenoh scouting
sudo ufw allow 7447/tcp  # Zenoh data

# Or disable firewall for testing
sudo ufw disable
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ZENOH_CONNECT` | (auto-discovery) | Router address, e.g., `tcp/192.168.1.100:7447` |
| `ZENOH_TOPIC_PREFIX` | `dviz` | Topic namespace for all messages |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

## Dataflow Examples

| File | Description |
|------|-------------|
| `dataflow.yml` | Local simulation with DViz bridge |
| `examples/dataflow-path-following.yml` | Vehicle path following (same as above) |
| `examples/dataflow-robot.yml` | Headless robot dataflow for remote debug |
| `examples/dataflow-ros2.yml` | ROS2 bridge integration |
| `examples/dataflow-mapping.yml` | Point cloud mapping pipeline |

## Project Structure

```
dviz/
├── dviz-core/           # Core types (Transform, PointCloud, Marker)
├── dviz-transform/      # Coordinate frame system
├── dviz-displays/       # Display plugins (Grid, TF, PointCloud, etc.)
├── dviz-urdf/           # URDF parsing and robot models
├── dviz-shell/          # Main application (Makepad UI)
├── dviz-widgets/        # Reusable UI components
├── dviz-rerun-bridge/   # Dora-to-Zenoh bridge node
├── dviz-rosbag/         # ROS bag playback
├── examples/            # Dataflow configurations
└── docs/                # Design documentation
```

## Zenoh Message Protocol

DViz uses a universal JSON protocol that any application can publish:

```json
{
  "type": "points3d",
  "timestamp": 1.5,
  "data": {
    "positions": [[0, 0, 0], [1, 1, 1]],
    "color": [255, 0, 0, 255]
  }
}
```

Supported types: `points3d`, `arrows3d`, `linestrips3d`, `boxes3d`, `transform3d`, `scalar`, `image`, `log`

Topic format: `{prefix}/{entity_path}` (e.g., `dviz/world/vehicle/lidar`)

## License

Apache-2.0
