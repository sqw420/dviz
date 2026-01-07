//! Point Cloud Display
//!
//! Displays point clouds with various color modes, decay support, and queuing.

use std::collections::VecDeque;
use std::time::Duration;
use mviz_core::{
    Color, ColorMode, DisplayStatus, PointCloud, PointCloudStyle,
};
use mviz_rerun_bridge::PointCloudCoreAdapter;
use crate::base::{BaseDisplay, DisplayUpdateContext};

/// Point cloud display properties
#[derive(Debug, Clone)]
pub struct PointCloudProperties {
    /// Visual style
    pub style: PointCloudStyle,
    /// Color mode
    pub color_mode: ColorMode,
    /// Point size/radius
    pub point_size: f32,
    /// Alpha/transparency
    pub alpha: f32,
    /// Decay time (0 = no decay, shows only latest)
    pub decay_time: Duration,
    /// Whether points are selectable
    pub selectable: bool,
    /// Maximum queue size
    pub queue_size: usize,
}

impl Default for PointCloudProperties {
    fn default() -> Self {
        Self {
            style: PointCloudStyle::Points,
            color_mode: ColorMode::FlatColor(Color::WHITE),
            point_size: 0.01,
            alpha: 1.0,
            decay_time: Duration::ZERO,
            selectable: false,
            queue_size: 10,
        }
    }
}

/// A queued point cloud with metadata
#[derive(Debug)]
struct QueuedCloud {
    /// The point cloud data
    cloud: PointCloud,
    /// When this cloud was received
    receive_time: std::time::Instant,
    /// Entity path suffix for this cloud
    entity_suffix: usize,
}

/// Point cloud display with decay and queue support
pub struct PointCloudDisplay {
    /// Base display state
    pub base: BaseDisplay,
    /// Display properties
    pub props: PointCloudProperties,
    /// Queue of point clouds for decay visualization
    cloud_queue: VecDeque<QueuedCloud>,
    /// Counter for unique entity paths
    entity_counter: usize,
    /// Statistics
    total_points: usize,
}

// Implement common display methods
crate::impl_display_base!(PointCloudDisplay);

impl PointCloudDisplay {
    /// Create a new point cloud display
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = BaseDisplay::new(name, "point_cloud");

        // Add properties
        base.add_property("point_size", 0.01f64);
        base.add_property("alpha", 1.0f64);
        base.add_property("decay_time", 0.0f64);
        base.add_property("selectable", false);

        Self {
            base,
            props: PointCloudProperties::default(),
            cloud_queue: VecDeque::new(),
            entity_counter: 0,
            total_points: 0,
        }
    }

    /// Add a point cloud to the queue
    pub fn add_cloud(&mut self, cloud: PointCloud) {
        let queued = QueuedCloud {
            cloud,
            receive_time: std::time::Instant::now(),
            entity_suffix: self.entity_counter,
        };
        self.entity_counter = self.entity_counter.wrapping_add(1);

        // Remove oldest if queue is full
        while self.cloud_queue.len() >= self.props.queue_size {
            self.cloud_queue.pop_front();
        }

        self.cloud_queue.push_back(queued);
    }

    /// Set the point cloud (clears queue and sets single cloud)
    pub fn set_cloud(&mut self, cloud: PointCloud) {
        self.cloud_queue.clear();
        self.add_cloud(cloud);
    }

    /// Prune old clouds based on decay time
    fn prune_old_clouds(&mut self) {
        if self.props.decay_time == Duration::ZERO {
            // Keep only latest cloud
            while self.cloud_queue.len() > 1 {
                self.cloud_queue.pop_front();
            }
            return;
        }

        let now = std::time::Instant::now();
        while let Some(front) = self.cloud_queue.front() {
            if now.duration_since(front.receive_time) > self.props.decay_time {
                self.cloud_queue.pop_front();
            } else {
                break;
            }
        }
    }

    /// Set color mode
    pub fn set_color_mode(&mut self, mode: ColorMode) {
        self.props.color_mode = mode;
    }

    /// Set point size
    pub fn set_point_size(&mut self, size: f32) {
        self.props.point_size = size;
    }

    /// Set decay time
    pub fn set_decay_time(&mut self, duration: Duration) {
        self.props.decay_time = duration;
    }

    /// Get total point count across all queued clouds
    pub fn total_points(&self) -> usize {
        self.cloud_queue.iter().map(|q| q.cloud.len()).sum()
    }

    /// Log a single point cloud
    fn log_cloud(
        &self,
        ctx: &DisplayUpdateContext,
        queued: &QueuedCloud,
    ) -> Result<(), mviz_rerun_bridge::RerunError> {
        // Transform cloud to fixed frame if needed
        let transform = if queued.cloud.frame_id == *ctx.fixed_frame {
            None
        } else {
            ctx.transform_buffer
                .lookup_transform(ctx.fixed_frame, &queued.cloud.frame_id, ctx.current_time)
                .ok()
        };

        // Create entity path
        let entity_path = if self.props.decay_time == Duration::ZERO {
            self.entity_path().to_string()
        } else {
            format!("{}/{}", self.entity_path(), queued.entity_suffix)
        };

        // If we have a transform, we need to transform the points
        let cloud = if let Some(tf) = transform {
            let mut transformed = queued.cloud.clone();
            for pos in &mut transformed.positions {
                *pos = tf.transform_point(*pos);
            }
            transformed
        } else {
            queued.cloud.clone()
        };

        PointCloudCoreAdapter::log(
            ctx.stream,
            &entity_path,
            &cloud,
            &self.props.color_mode,
            self.props.point_size,
        )
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Prune old clouds
        self.prune_old_clouds();

        // Log all clouds in queue
        for queued in &self.cloud_queue {
            if let Err(e) = self.log_cloud(ctx, queued) {
                self.set_status(
                    DisplayStatus::Error,
                    Some(format!("Log error: {}", e)),
                );
                return Err(e);
            }
        }

        // Update statistics
        self.total_points = self.total_points();

        // Update status
        let num_clouds = self.cloud_queue.len();
        self.set_status(
            DisplayStatus::Ok,
            Some(format!("{} points in {} clouds", self.total_points, num_clouds)),
        );

        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self, _ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        self.cloud_queue.clear();
        self.total_points = 0;
        Ok(())
    }

    /// Reset the display
    pub fn reset(&mut self) {
        self.props = PointCloudProperties::default();
        self.cloud_queue.clear();
        self.entity_counter = 0;
        self.total_points = 0;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_point_cloud_properties_default() {
        let props = PointCloudProperties::default();
        assert_eq!(props.point_size, 0.01);
        assert_eq!(props.alpha, 1.0);
        assert_eq!(props.decay_time, Duration::ZERO);
    }

    #[test]
    fn test_point_cloud_display_creation() {
        let display = PointCloudDisplay::new("LiDAR");
        assert_eq!(display.name(), "LiDAR");
        assert_eq!(display.type_name(), "point_cloud");
        assert!(display.is_enabled());
    }

    #[test]
    fn test_point_cloud_add_cloud() {
        let mut display = PointCloudDisplay::new("Test");

        let cloud = PointCloud::from_positions(
            vec![Vec3::ZERO, Vec3::ONE],
            "test_frame",
        );

        display.add_cloud(cloud);
        assert_eq!(display.cloud_queue.len(), 1);
        assert_eq!(display.total_points(), 2);
    }

    #[test]
    fn test_point_cloud_queue_limit() {
        let mut display = PointCloudDisplay::new("Test");
        display.props.queue_size = 3;

        for i in 0..5 {
            let cloud = PointCloud::from_positions(
                vec![Vec3::new(i as f32, 0.0, 0.0)],
                "test",
            );
            display.add_cloud(cloud);
        }

        // Should only keep last 3
        assert_eq!(display.cloud_queue.len(), 3);
    }

    #[test]
    fn test_point_cloud_prune_latest_only() {
        let mut display = PointCloudDisplay::new("Test");
        display.props.decay_time = Duration::ZERO; // Latest only

        for _ in 0..3 {
            let cloud = PointCloud::from_positions(vec![Vec3::ZERO], "test");
            display.add_cloud(cloud);
        }

        display.prune_old_clouds();
        assert_eq!(display.cloud_queue.len(), 1);
    }

    #[test]
    fn test_point_cloud_set_cloud() {
        let mut display = PointCloudDisplay::new("Test");

        // Add multiple clouds
        for _ in 0..3 {
            display.add_cloud(PointCloud::from_positions(vec![Vec3::ZERO], "test"));
        }
        assert_eq!(display.cloud_queue.len(), 3);

        // set_cloud should clear and add one
        display.set_cloud(PointCloud::from_positions(vec![Vec3::ONE], "test"));
        assert_eq!(display.cloud_queue.len(), 1);
    }
}
