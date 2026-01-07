# MViz Release Notes

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
