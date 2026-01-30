//! Rerun Bridge - Connection management for Rerun viewer
//!
//! Handles spawning/connecting to Rerun viewer and provides the recording stream.

use rerun::RecordingStream;
use crate::simulation::{ImuData, LidarScan, VehiclePose};

/// Configuration for Rerun connection
#[derive(Debug, Clone)]
pub struct RerunConfig {
    /// Application/recording ID
    pub recording_id: String,
    /// Optional TCP address to connect to existing viewer
    pub connect_addr: Option<String>,
    /// Whether to spawn a new viewer window
    pub spawn_viewer: bool,
}

impl Default for RerunConfig {
    fn default() -> Self {
        Self {
            recording_id: "dviz".to_string(),
            connect_addr: None,
            spawn_viewer: true,
        }
    }
}

impl RerunConfig {
    pub fn new(recording_id: impl Into<String>) -> Self {
        Self {
            recording_id: recording_id.into(),
            ..Default::default()
        }
    }

    pub fn with_spawn(mut self, spawn: bool) -> Self {
        self.spawn_viewer = spawn;
        self
    }

    pub fn with_connect_addr(mut self, addr: impl Into<String>) -> Self {
        self.connect_addr = Some(addr.into());
        self
    }
}

/// Error types for Rerun operations
#[derive(Debug, thiserror::Error)]
pub enum RerunError {
    #[error("Failed to spawn Rerun viewer: {0}")]
    SpawnFailed(String),

    #[error("Failed to connect to Rerun viewer: {0}")]
    ConnectionFailed(String),

    #[error("Failed to log data: {0}")]
    LogError(String),

    #[error("Not connected to Rerun viewer")]
    NotConnected,
}

impl From<rerun::RecordingStreamError> for RerunError {
    fn from(e: rerun::RecordingStreamError) -> Self {
        RerunError::LogError(e.to_string())
    }
}

/// Manages connection to Rerun viewer
pub struct RerunBridge {
    stream: Option<RecordingStream>,
    config: RerunConfig,
    connected: bool,
}

impl RerunBridge {
    /// Create a new RerunBridge with the given configuration
    pub fn new(config: RerunConfig) -> Self {
        Self {
            stream: None,
            config,
            connected: false,
        }
    }

    /// Create with default configuration and custom recording ID
    pub fn with_recording_id(recording_id: impl Into<String>) -> Self {
        Self::new(RerunConfig::new(recording_id))
    }

    /// Spawn a new Rerun viewer window
    pub fn spawn_viewer(&mut self) -> Result<(), RerunError> {
        log::info!("Spawning Rerun viewer with recording ID: {}", self.config.recording_id);

        let stream = rerun::RecordingStreamBuilder::new(self.config.recording_id.clone())
            .spawn()
            .map_err(|e| RerunError::SpawnFailed(e.to_string()))?;

        // Set up world coordinates (right-handed, Z-up)
        stream.log_static(
            "world",
            &rerun::ViewCoordinates::RIGHT_HAND_Z_UP(),
        ).map_err(|e| RerunError::LogError(e.to_string()))?;

        // Log origin marker
        stream.log(
            "world/origin",
            &rerun::Points3D::new([[0.0_f32, 0.0, 0.0]])
                .with_radii([0.05])
                .with_colors([rerun::Color::from_rgb(255, 255, 255)]),
        ).map_err(|e| RerunError::LogError(e.to_string()))?;

        self.stream = Some(stream);
        self.connected = true;

        log::info!("Rerun viewer spawned successfully");
        Ok(())
    }

    /// Connect to an existing Rerun viewer via gRPC
    pub fn connect(&mut self, addr: &str) -> Result<(), RerunError> {
        log::info!("Connecting to Rerun viewer at: {}", addr);

        let stream = rerun::RecordingStreamBuilder::new(self.config.recording_id.clone())
            .connect_grpc_opts(addr)
            .map_err(|e| RerunError::ConnectionFailed(e.to_string()))?;

        // Set up world coordinates
        stream.log_static(
            "world",
            &rerun::ViewCoordinates::RIGHT_HAND_Z_UP(),
        ).map_err(|e| RerunError::LogError(e.to_string()))?;

        self.stream = Some(stream);
        self.connected = true;

        log::info!("Connected to Rerun viewer");
        Ok(())
    }

    /// Connect or spawn based on configuration
    pub fn connect_or_spawn(&mut self) -> Result<(), RerunError> {
        if let Some(addr) = self.config.connect_addr.clone() {
            self.connect(&addr)
        } else if self.config.spawn_viewer {
            self.spawn_viewer()
        } else {
            Err(RerunError::ConnectionFailed("No connection method specified".to_string()))
        }
    }

    /// Check if connected to Rerun viewer
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get reference to the recording stream
    pub fn stream(&self) -> Option<&RecordingStream> {
        self.stream.as_ref()
    }

    /// Get mutable reference to the recording stream
    pub fn stream_mut(&mut self) -> Option<&mut RecordingStream> {
        self.stream.as_mut()
    }

    /// Log data to a specific entity path
    pub fn log<T: rerun::AsComponents>(
        &self,
        entity_path: &str,
        data: &T,
    ) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;
        stream.log(entity_path, data)?;
        Ok(())
    }

    /// Log static data (time-independent)
    pub fn log_static<T: rerun::AsComponents>(
        &self,
        entity_path: &str,
        data: &T,
    ) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;
        stream.log_static(entity_path, data)?;
        Ok(())
    }

    /// Set the current time for subsequent logs
    pub fn set_time_seconds(&self, timeline: &str, seconds: f64) {
        if let Some(stream) = &self.stream {
            stream.set_time(timeline, std::time::Duration::from_secs_f64(seconds));
        }
    }

    /// Set the current sequence number
    pub fn set_time_sequence(&self, timeline: &str, sequence: i64) {
        if let Some(stream) = &self.stream {
            stream.set_time_sequence(timeline, sequence);
        }
    }

    /// Disconnect from Rerun viewer
    pub fn disconnect(&mut self) {
        self.stream = None;
        self.connected = false;
        log::info!("Disconnected from Rerun viewer");
    }

    // ========================================================================
    // Sensor Data Logging Methods
    // ========================================================================

    /// Log IMU data
    pub fn log_imu(&self, imu: &ImuData) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;

        // Set time for this data
        stream.set_time("sim_time", std::time::Duration::from_secs_f64(imu.timestamp));

        // Log linear acceleration as arrows
        stream.log(
            "world/vehicle/imu/acceleration",
            &rerun::Arrows3D::from_vectors([[
                imu.linear_acceleration[0] * 0.1,
                imu.linear_acceleration[1] * 0.1,
                (imu.linear_acceleration[2] - 9.81) * 0.1, // Remove gravity for visualization
            ]])
            .with_colors([rerun::Color::from_rgb(255, 100, 100)]),
        )?;

        // Log angular velocity as arrows
        stream.log(
            "world/vehicle/imu/angular_velocity",
            &rerun::Arrows3D::from_vectors([[
                imu.angular_velocity[0],
                imu.angular_velocity[1],
                imu.angular_velocity[2],
            ]])
            .with_colors([rerun::Color::from_rgb(100, 100, 255)]),
        )?;

        // Log raw values as scalars for time series
        stream.log(
            "sensors/imu/accel_x",
            &rerun::Scalars::new([imu.linear_acceleration[0] as f64]),
        )?;
        stream.log(
            "sensors/imu/accel_y",
            &rerun::Scalars::new([imu.linear_acceleration[1] as f64]),
        )?;
        stream.log(
            "sensors/imu/accel_z",
            &rerun::Scalars::new([imu.linear_acceleration[2] as f64]),
        )?;
        stream.log(
            "sensors/imu/gyro_z",
            &rerun::Scalars::new([imu.angular_velocity[2] as f64]),
        )?;

        Ok(())
    }

    /// Log LiDAR point cloud
    pub fn log_lidar(&self, scan: &LidarScan) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;

        stream.set_time("sim_time", std::time::Duration::from_secs_f64(scan.timestamp));

        // Convert points to positions and colors
        let positions: Vec<[f32; 3]> = scan.points
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect();

        let colors: Vec<rerun::Color> = scan.points
            .iter()
            .map(|p| {
                // Color by height (z coordinate)
                let z_norm = ((p.z + 2.0) / 6.0).clamp(0.0, 1.0);
                let r = (z_norm * 255.0) as u8;
                let g = ((1.0 - (z_norm - 0.5).abs() * 2.0) * 255.0) as u8;
                let b = ((1.0 - z_norm) * 255.0) as u8;
                rerun::Color::from_rgb(r, g, b)
            })
            .collect();

        stream.log(
            "world/lidar/points",
            &rerun::Points3D::new(positions)
                .with_radii([0.03_f32])
                .with_colors(colors),
        )?;

        // Log point count as scalar
        stream.log(
            "sensors/lidar/point_count",
            &rerun::Scalars::new([scan.points.len() as f64]),
        )?;

        Ok(())
    }

    /// Log vehicle pose
    pub fn log_vehicle_pose(&self, pose: &VehiclePose) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;

        stream.set_time("sim_time", std::time::Duration::from_secs_f64(pose.timestamp));

        // Log vehicle as a 3D box
        let vehicle_size = [4.5_f32, 2.0, 1.5]; // Length, width, height

        stream.log(
            "world/vehicle/body",
            &rerun::Boxes3D::from_centers_and_sizes(
                [[pose.position[0], pose.position[1], pose.position[2] + 0.75]],
                [vehicle_size],
            )
            .with_quaternions([rerun::Quaternion::from_xyzw(pose.orientation)])
            .with_colors([rerun::Color::from_rgb(50, 150, 250)]),
        )?;

        // Log velocity arrow
        let speed = (pose.velocity[0].powi(2) + pose.velocity[1].powi(2)).sqrt();
        if speed > 0.1 {
            stream.log(
                "world/vehicle/velocity",
                &rerun::Arrows3D::from_vectors([[
                    pose.velocity[0] * 0.3,
                    pose.velocity[1] * 0.3,
                    0.0,
                ]])
                .with_origins([[pose.position[0], pose.position[1], pose.position[2] + 1.5]])
                .with_colors([rerun::Color::from_rgb(0, 255, 100)]),
            )?;
        }

        // Log position as scalar for time series
        stream.log("sensors/vehicle/pos_x", &rerun::Scalars::new([pose.position[0] as f64]))?;
        stream.log("sensors/vehicle/pos_y", &rerun::Scalars::new([pose.position[1] as f64]))?;
        stream.log("sensors/vehicle/speed", &rerun::Scalars::new([speed as f64]))?;

        Ok(())
    }

    /// Log vehicle path history
    pub fn log_path(&self, path: &[[f32; 3]], timestamp: f64) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;

        stream.set_time("sim_time", std::time::Duration::from_secs_f64(timestamp));

        if path.len() >= 2 {
            stream.log(
                "world/vehicle/path",
                &rerun::LineStrips3D::new([path.to_vec()])
                    .with_colors([rerun::Color::from_rgb(255, 200, 50)]),
            )?;
        }

        Ok(())
    }

    /// Log a ground grid for reference
    pub fn log_ground_grid(&self) -> Result<(), RerunError> {
        let stream = self.stream.as_ref().ok_or(RerunError::NotConnected)?;

        let mut lines = Vec::new();
        let grid_size = 20.0_f32;
        let spacing = 2.0_f32;

        // Create grid lines
        let mut x = -grid_size;
        while x <= grid_size {
            lines.push(vec![[x, -grid_size, 0.0], [x, grid_size, 0.0]]);
            lines.push(vec![[-grid_size, x, 0.0], [grid_size, x, 0.0]]);
            x += spacing;
        }

        stream.log_static(
            "world/ground_grid",
            &rerun::LineStrips3D::new(lines)
                .with_colors([rerun::Color::from_rgb(60, 60, 60)]),
        )?;

        Ok(())
    }
}

impl Default for RerunBridge {
    fn default() -> Self {
        Self::new(RerunConfig::default())
    }
}
