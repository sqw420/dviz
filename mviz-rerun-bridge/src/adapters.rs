//! Sensor Data Adapters
//!
//! Provides adapters for logging various sensor data types to Rerun.

use rerun::RecordingStream;
use crate::RerunError;

// ============================================================================
// IMU ADAPTER
// ============================================================================

/// Adapter for logging IMU (Inertial Measurement Unit) data
pub struct ImuAdapter;

impl ImuAdapter {
    /// Log IMU data (accelerometer and gyroscope) to Rerun
    ///
    /// # Arguments
    /// * `stream` - Rerun recording stream
    /// * `accel` - Accelerometer readings [x, y, z] in m/s^2
    /// * `gyro` - Gyroscope readings [x, y, z] in rad/s
    pub fn log(
        stream: &RecordingStream,
        accel: [f32; 3],
        gyro: [f32; 3],
    ) -> Result<(), RerunError> {
        // Log accelerometer as green arrow from origin
        stream.log(
            "world/imu/accelerometer",
            &rerun::Arrows3D::from_vectors([[
                accel[0] * 0.1,  // Scale for visibility
                accel[1] * 0.1,
                accel[2] * 0.1,
            ]])
            .with_origins([[0.0, 0.0, 0.0]])
            .with_colors([rerun::Color::from_rgb(16, 185, 129)]),  // Green
        )?;

        // Log gyroscope as red arrow (offset vertically)
        stream.log(
            "world/imu/gyroscope",
            &rerun::Arrows3D::from_vectors([[
                gyro[0],
                gyro[1],
                gyro[2],
            ]])
            .with_origins([[0.0, 0.0, 0.5]])
            .with_colors([rerun::Color::from_rgb(239, 68, 68)]),  // Red
        )?;

        // Log scalar time series for each axis
        stream.log("sensors/accel_x", &rerun::Scalars::new([accel[0] as f64]))?;
        stream.log("sensors/accel_y", &rerun::Scalars::new([accel[1] as f64]))?;
        stream.log("sensors/accel_z", &rerun::Scalars::new([accel[2] as f64]))?;
        stream.log("sensors/gyro_x", &rerun::Scalars::new([gyro[0] as f64]))?;
        stream.log("sensors/gyro_y", &rerun::Scalars::new([gyro[1] as f64]))?;
        stream.log("sensors/gyro_z", &rerun::Scalars::new([gyro[2] as f64]))?;

        Ok(())
    }

    /// Log IMU orientation as a coordinate frame
    pub fn log_orientation(
        stream: &RecordingStream,
        roll: f32,
        pitch: f32,
        yaw: f32,
    ) -> Result<(), RerunError> {
        // Convert Euler angles to quaternion (simplified)
        let (sr, cr) = (roll * 0.5).sin_cos();
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();

        let qw = cr * cp * cy + sr * sp * sy;
        let qx = sr * cp * cy - cr * sp * sy;
        let qy = cr * sp * cy + sr * cp * sy;
        let qz = cr * cp * sy - sr * sp * cy;

        stream.log(
            "world/imu/orientation",
            &rerun::Transform3D::from_rotation(
                rerun::Quaternion::from_xyzw([qx, qy, qz, qw])
            ),
        )?;

        Ok(())
    }
}

// ============================================================================
// POINT CLOUD ADAPTER
// ============================================================================

/// Adapter for logging point cloud data
pub struct PointCloudAdapter;

impl PointCloudAdapter {
    /// Log a point cloud to Rerun
    ///
    /// # Arguments
    /// * `stream` - Rerun recording stream
    /// * `entity_path` - Entity path for the point cloud
    /// * `points` - Vector of 3D points
    /// * `colors` - Optional colors for each point (RGB)
    /// * `radius` - Point radius for visualization
    pub fn log(
        stream: &RecordingStream,
        entity_path: &str,
        points: &[[f32; 3]],
        colors: Option<&[[u8; 3]]>,
        radius: f32,
    ) -> Result<(), RerunError> {
        let positions: Vec<rerun::Position3D> = points
            .iter()
            .map(|p| rerun::Position3D::new(p[0], p[1], p[2]))
            .collect();

        let mut points3d = rerun::Points3D::new(positions)
            .with_radii(vec![rerun::Radius::new_scene_units(radius); points.len()]);

        if let Some(colors) = colors {
            let rerun_colors: Vec<rerun::Color> = colors
                .iter()
                .map(|c| rerun::Color::from_rgb(c[0], c[1], c[2]))
                .collect();
            points3d = points3d.with_colors(rerun_colors);
        } else {
            // Default gray color
            points3d = points3d.with_colors([rerun::Color::from_rgb(200, 200, 200)]);
        }

        stream.log(entity_path, &points3d)?;
        Ok(())
    }

    /// Log a point cloud with Z-height based coloring
    pub fn log_with_height_color(
        stream: &RecordingStream,
        entity_path: &str,
        points: &[[f32; 3]],
        radius: f32,
        z_min: f32,
        z_max: f32,
    ) -> Result<(), RerunError> {
        let colors: Vec<[u8; 3]> = points
            .iter()
            .map(|p| {
                let t = ((p[2] - z_min) / (z_max - z_min)).clamp(0.0, 1.0);
                Self::jet_colormap(t)
            })
            .collect();

        Self::log(stream, entity_path, points, Some(&colors), radius)
    }

    /// Jet colormap: blue -> cyan -> green -> yellow -> red
    fn jet_colormap(t: f32) -> [u8; 3] {
        let r = (1.5 - (4.0 * t - 3.0).abs()).clamp(0.0, 1.0);
        let g = (1.5 - (4.0 * t - 2.0).abs()).clamp(0.0, 1.0);
        let b = (1.5 - (4.0 * t - 1.0).abs()).clamp(0.0, 1.0);
        [
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        ]
    }
}

// ============================================================================
// POSE ADAPTER
// ============================================================================

/// Adapter for logging pose/position data with trajectory
pub struct PoseAdapter {
    trajectory: Vec<[f32; 3]>,
    max_trajectory_points: usize,
}

impl PoseAdapter {
    pub fn new(max_trajectory_points: usize) -> Self {
        Self {
            trajectory: Vec::new(),
            max_trajectory_points,
        }
    }

    /// Log a pose to Rerun
    ///
    /// # Arguments
    /// * `stream` - Rerun recording stream
    /// * `x`, `y`, `z` - Position
    /// * `theta` - Heading angle (yaw) in radians
    pub fn log(
        &mut self,
        stream: &RecordingStream,
        x: f32,
        y: f32,
        z: f32,
        theta: f32,
    ) -> Result<(), RerunError> {
        let point = [x, y, z];

        // Log current position as blue point
        stream.log(
            "world/position",
            &rerun::Points3D::new([point])
                .with_radii([0.1])
                .with_colors([rerun::Color::from_rgb(59, 130, 246)]),  // Blue
        )?;

        // Log direction arrow
        let arrow_length = 0.5;
        stream.log(
            "world/direction",
            &rerun::Arrows3D::from_vectors([[
                arrow_length * theta.cos(),
                arrow_length * theta.sin(),
                0.0,
            ]])
            .with_origins([[x, y, z + 0.1]])
            .with_colors([rerun::Color::from_rgb(255, 0, 0)]),  // Red
        )?;

        // Update trajectory
        self.trajectory.push(point);
        if self.trajectory.len() > self.max_trajectory_points {
            self.trajectory.remove(0);
        }

        // Log trajectory as line strip
        if self.trajectory.len() > 1 {
            stream.log(
                "world/trajectory",
                &rerun::LineStrips3D::new([self.trajectory.clone()])
                    .with_colors([rerun::Color::from_rgb(139, 92, 246)]),  // Purple
            )?;
        }

        // Log scalar time series
        stream.log("position/x", &rerun::Scalars::new([x as f64]))?;
        stream.log("position/y", &rerun::Scalars::new([y as f64]))?;
        stream.log("position/z", &rerun::Scalars::new([z as f64]))?;
        stream.log("position/theta", &rerun::Scalars::new([theta as f64]))?;

        Ok(())
    }

    /// Clear the trajectory buffer
    pub fn clear_trajectory(&mut self) {
        self.trajectory.clear();
    }
}

impl Default for PoseAdapter {
    fn default() -> Self {
        Self::new(1000)
    }
}

// ============================================================================
// LASER SCAN ADAPTER
// ============================================================================

/// Adapter for logging laser scan (LiDAR) data
pub struct LaserScanAdapter;

impl LaserScanAdapter {
    /// Log a laser scan to Rerun
    ///
    /// # Arguments
    /// * `stream` - Rerun recording stream
    /// * `ranges` - Range measurements in meters
    /// * `angle_min` - Start angle in radians
    /// * `angle_increment` - Angle between measurements in radians
    /// * `range_min`, `range_max` - Valid range bounds
    pub fn log(
        stream: &RecordingStream,
        ranges: &[f32],
        angle_min: f32,
        angle_increment: f32,
        range_min: f32,
        range_max: f32,
    ) -> Result<(), RerunError> {
        let mut points = Vec::with_capacity(ranges.len());
        let mut colors = Vec::with_capacity(ranges.len());

        for (i, &range) in ranges.iter().enumerate() {
            // Skip invalid ranges
            if range < range_min || range > range_max || range.is_nan() || range.is_infinite() {
                continue;
            }

            let angle = angle_min + (i as f32) * angle_increment;
            let x = range * angle.cos();
            let y = range * angle.sin();
            let z = 0.0_f32;

            points.push([x, y, z]);

            // Color by range (blue=near, red=far)
            let t = ((range - range_min) / (range_max - range_min)).clamp(0.0, 1.0);
            let r = (t * 255.0) as u8;
            let b = ((1.0 - t) * 255.0) as u8;
            colors.push([r, 50, b]);
        }

        if !points.is_empty() {
            PointCloudAdapter::log(stream, "world/lidar", &points, Some(&colors), 0.02)?;
        }

        Ok(())
    }

    /// Generate a simulated 360-degree laser scan
    pub fn simulate_scan(num_points: usize, base_range: f32, noise: f32) -> (Vec<f32>, f32, f32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let angle_min = 0.0_f32;
        let angle_increment = std::f32::consts::TAU / num_points as f32;

        let ranges: Vec<f32> = (0..num_points)
            .map(|_| base_range + rng.gen_range(-noise..noise))
            .collect();

        (ranges, angle_min, angle_increment)
    }
}

// ============================================================================
// VEHICLE ADAPTER
// ============================================================================

/// Adapter for logging vehicle visualization (box + direction arrow)
pub struct VehicleAdapter;

impl VehicleAdapter {
    /// Log a vehicle as a 3D box with direction arrow
    pub fn log(
        stream: &RecordingStream,
        x: f32,
        y: f32,
        theta: f32,
        length: f32,
        width: f32,
        height: f32,
        velocity: f32,
    ) -> Result<(), RerunError> {
        // Vehicle box
        let center = [x, y, height / 2.0];
        let half_sizes = [length / 2.0, width / 2.0, height / 2.0];

        // Convert yaw to quaternion (rotation around Z)
        let qw = (theta / 2.0).cos();
        let qz = (theta / 2.0).sin();

        stream.log(
            "world/vehicle",
            &rerun::Boxes3D::from_centers_and_half_sizes([center], [half_sizes])
                .with_quaternions([rerun::Quaternion::from_xyzw([0.0, 0.0, qz, qw])])
                .with_colors([rerun::Color::from_unmultiplied_rgba(255, 200, 0, 255)])
                .with_labels([format!("v={:.2}m/s", velocity)]),
        )?;

        // Direction arrow
        let arrow_length = 0.8;
        stream.log(
            "world/vehicle_direction",
            &rerun::Arrows3D::from_vectors([[
                arrow_length * theta.cos(),
                arrow_length * theta.sin(),
                0.0,
            ]])
            .with_origins([[x, y, height]])
            .with_colors([rerun::Color::from_rgb(255, 0, 0)])
            .with_radii([0.05]),
        )?;

        Ok(())
    }
}

// ============================================================================
// GRID ADAPTER
// ============================================================================

/// Adapter for logging ground grid
pub struct GridAdapter;

impl GridAdapter {
    /// Log a ground plane grid
    pub fn log_ground_grid(
        stream: &RecordingStream,
        size: f32,
        step: f32,
    ) -> Result<(), RerunError> {
        let mut lines: Vec<Vec<[f32; 3]>> = Vec::new();

        let num_lines = (size / step) as i32;
        for i in -num_lines..=num_lines {
            let offset = i as f32 * step;

            // Lines parallel to X
            lines.push(vec![
                [-size, offset, 0.0],
                [size, offset, 0.0],
            ]);

            // Lines parallel to Y
            lines.push(vec![
                [offset, -size, 0.0],
                [offset, size, 0.0],
            ]);
        }

        stream.log_static(
            "world/ground_grid",
            &rerun::LineStrips3D::new(lines)
                .with_colors([rerun::Color::from_unmultiplied_rgba(100, 100, 100, 80)]),
        )?;

        Ok(())
    }
}
