//! ROS Bag Player
//!
//! Core player for reading and playing back ROS bag files.

use std::path::Path;
use std::collections::HashMap;
use rosbag::{RosBag, ChunkRecord, MessageRecord, IndexRecord};
use crate::{Result, RosBagError, BagMessage, MessageType};
use crate::tf::TfBuffer;
use crate::pointcloud::PointCloudProcessor;
use crate::imu::ImuProcessor;
use crate::gps::GpsProcessor;

/// Information about a topic in the bag
#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// Topic name (e.g., "/velodyne_points")
    pub name: String,
    /// Message type (e.g., "sensor_msgs/PointCloud2")
    pub msg_type: String,
    /// Number of messages on this topic
    pub message_count: u64,
    /// MD5 sum of the message definition (hex string)
    pub md5sum: String,
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    /// Not playing, at beginning
    Stopped,
    /// Currently playing
    Playing,
    /// Paused at current position
    Paused,
    /// Reached end of bag
    Finished,
}

/// Connection info from bag
#[derive(Debug, Clone)]
struct ConnectionInfo {
    topic: String,
    msg_type: String,
    md5sum: String,
}

/// Convert nanoseconds to seconds
fn nanos_to_secs(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000_000.0
}

/// ROS Bag Player
///
/// Reads ROS1 bag files and processes messages for visualization.
pub struct RosBagPlayer {
    /// Path to the bag file
    bag_path: String,

    /// Cached topic information
    topics: Vec<TopicInfo>,

    /// Connection ID to topic mapping
    connections: HashMap<u32, ConnectionInfo>,

    /// Current playback time (bag time in seconds)
    current_time: f64,

    /// Start time of the bag (seconds)
    start_time: f64,

    /// End time of the bag (seconds)
    end_time: f64,

    /// Playback state
    state: PlaybackState,

    /// Playback speed multiplier (1.0 = real-time)
    speed: f64,

    /// TF buffer for transforms
    tf_buffer: TfBuffer,

    /// Point cloud processor
    pointcloud_processor: PointCloudProcessor,

    /// IMU processor
    imu_processor: ImuProcessor,

    /// GPS/NMEA processor
    gps_processor: GpsProcessor,

    /// Topics to play (empty = all)
    selected_topics: Vec<String>,

    /// Rerun recording stream
    rerun_stream: Option<rerun::RecordingStream>,
}

impl RosBagPlayer {
    /// Open a ROS bag file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        log::info!("Opening bag file: {}", path_str);

        // Open the bag and extract metadata
        let bag = RosBag::new(&path_str)
            .map_err(|e| RosBagError::OpenError(e.to_string()))?;

        // Extract connection info and build topic list
        let mut connections = HashMap::new();
        let mut topic_counts: HashMap<String, u64> = HashMap::new();
        let mut topic_types: HashMap<String, (String, String)> = HashMap::new();

        let mut start_time = f64::MAX;
        let mut end_time = f64::MIN;

        // Get timing info from index records
        for record in bag.index_records() {
            if let Ok(IndexRecord::ChunkInfo(chunk_info)) = record {
                let chunk_start = nanos_to_secs(chunk_info.start_time);
                let chunk_end = nanos_to_secs(chunk_info.end_time);

                if chunk_start < start_time {
                    start_time = chunk_start;
                }
                if chunk_end > end_time {
                    end_time = chunk_end;
                }
            }
        }

        // Process chunk records for connections and message counts
        for record in bag.chunk_records() {
            match record {
                Ok(ChunkRecord::Chunk(chunk)) => {
                    // Process messages in chunk
                    for msg in chunk.messages() {
                        match msg {
                            Ok(MessageRecord::Connection(conn)) => {
                                let topic = conn.topic.to_string();
                                let msg_type = conn.tp.to_string();
                                // Convert MD5 bytes to hex string
                                let md5sum = conn.md5sum.iter()
                                    .map(|b| format!("{:02x}", b))
                                    .collect::<String>();

                                connections.insert(conn.id, ConnectionInfo {
                                    topic: topic.clone(),
                                    msg_type: msg_type.clone(),
                                    md5sum: md5sum.clone(),
                                });

                                topic_types.insert(topic.clone(), (msg_type, md5sum));
                            }
                            Ok(MessageRecord::MessageData(msg_data)) => {
                                if let Some(conn) = connections.get(&msg_data.conn_id) {
                                    *topic_counts.entry(conn.topic.clone()).or_insert(0) += 1;
                                }
                            }
                            Err(e) => {
                                log::warn!("Error reading message: {}", e);
                            }
                        }
                    }
                }
                Ok(ChunkRecord::IndexData(_)) => {
                    // Index data - skip for now
                }
                Err(e) => {
                    log::warn!("Error reading chunk: {}", e);
                }
            }
        }

        // Build topic info list
        let topics: Vec<TopicInfo> = topic_types
            .into_iter()
            .map(|(name, (msg_type, md5sum))| TopicInfo {
                name: name.clone(),
                msg_type,
                message_count: topic_counts.get(&name).copied().unwrap_or(0),
                md5sum,
            })
            .collect();

        log::info!("Found {} topics, duration: {:.2}s", topics.len(), end_time - start_time);

        Ok(Self {
            bag_path: path_str,
            topics,
            connections,
            current_time: start_time,
            start_time,
            end_time,
            state: PlaybackState::Stopped,
            speed: 1.0,
            tf_buffer: TfBuffer::new(),
            pointcloud_processor: PointCloudProcessor::new(),
            imu_processor: ImuProcessor::new(),
            gps_processor: GpsProcessor::new(),
            selected_topics: Vec::new(),
            rerun_stream: None,
        })
    }

    /// Get list of topics in the bag
    pub fn topics(&self) -> &[TopicInfo] {
        &self.topics
    }

    /// Get topic info by name
    pub fn get_topic(&self, name: &str) -> Option<&TopicInfo> {
        self.topics.iter().find(|t| t.name == name)
    }

    /// Get duration of the bag in seconds
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Get start time of the bag
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    /// Get end time of the bag
    pub fn end_time(&self) -> f64 {
        self.end_time
    }

    /// Get current playback time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Start playback
    pub fn play(&mut self) {
        if self.state == PlaybackState::Finished {
            self.current_time = self.start_time;
        }
        self.state = PlaybackState::Playing;
        log::info!("Playback started at {:.2}s", self.current_time);
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
        log::info!("Playback paused at {:.2}s", self.current_time);
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_time = self.start_time;
        log::info!("Playback stopped");
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: f64) {
        self.current_time = time.clamp(self.start_time, self.end_time);
        log::info!("Seeked to {:.2}s", self.current_time);
    }

    /// Step forward by one message
    pub fn step_forward(&mut self) {
        // TODO: Implement stepping
    }

    /// Step backward by one message
    pub fn step_backward(&mut self) {
        // TODO: Implement stepping
    }

    /// Set playback speed (1.0 = real-time)
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.1).min(10.0);
    }

    /// Get playback speed
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Set selected topics to play
    pub fn set_selected_topics(&mut self, topics: Vec<String>) {
        self.selected_topics = topics;
    }

    /// Set rerun recording stream for visualization
    pub fn set_rerun_stream(&mut self, stream: rerun::RecordingStream) {
        self.rerun_stream = Some(stream);
    }

    /// Get TF buffer
    pub fn tf_buffer(&self) -> &TfBuffer {
        &self.tf_buffer
    }

    /// Get mutable TF buffer
    pub fn tf_buffer_mut(&mut self) -> &mut TfBuffer {
        &mut self.tf_buffer
    }

    /// Process messages up to the given wall time delta
    ///
    /// Returns the number of messages processed
    pub fn update(&mut self, wall_dt: f64) -> Result<usize> {
        if self.state != PlaybackState::Playing {
            return Ok(0);
        }

        let target_time = self.current_time + wall_dt * self.speed;

        if target_time >= self.end_time {
            self.state = PlaybackState::Finished;
            self.current_time = self.end_time;
            log::info!("Playback finished");
            return Ok(0);
        }

        // Read and process messages up to target_time
        let messages = self.read_messages_until(target_time)?;
        let count = messages.len();

        for msg in messages {
            self.process_message(&msg)?;
        }

        self.current_time = target_time;
        Ok(count)
    }

    /// Read messages from current_time to target_time
    fn read_messages_until(&self, target_time: f64) -> Result<Vec<BagMessage>> {
        let mut messages = Vec::new();

        // Re-open bag for reading (rosbag crate doesn't support seeking)
        let bag = RosBag::new(&self.bag_path)
            .map_err(|e| RosBagError::OpenError(e.to_string()))?;

        // Convert times to nanoseconds for comparison
        let current_nanos = (self.current_time * 1_000_000_000.0) as u64;
        let target_nanos = (target_time * 1_000_000_000.0) as u64;

        for record in bag.chunk_records() {
            if let Ok(ChunkRecord::Chunk(chunk)) = record {
                for msg in chunk.messages() {
                    if let Ok(MessageRecord::MessageData(msg_data)) = msg {
                        let msg_time = msg_data.time;

                        if msg_time >= current_nanos && msg_time <= target_nanos {
                            if let Some(conn) = self.connections.get(&msg_data.conn_id) {
                                // Filter by selected topics
                                if !self.selected_topics.is_empty()
                                    && !self.selected_topics.contains(&conn.topic)
                                {
                                    continue;
                                }

                                let msg_type = MessageType::from_ros_type(&conn.msg_type);

                                messages.push(BagMessage {
                                    topic: conn.topic.clone(),
                                    msg_type,
                                    timestamp: nanos_to_secs(msg_time),
                                    data: msg_data.data.to_vec(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by timestamp
        messages.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

        Ok(messages)
    }

    /// Process a single message
    fn process_message(&mut self, msg: &BagMessage) -> Result<()> {
        match msg.msg_type {
            MessageType::PointCloud2 => {
                self.process_pointcloud(msg)?;
            }
            MessageType::TfMessage => {
                self.process_tf(msg)?;
            }
            MessageType::Imu => {
                self.process_imu(msg)?;
            }
            MessageType::NmeaSentence => {
                self.process_nmea(msg)?;
            }
            MessageType::TimeReference => {
                self.process_time_reference(msg)?;
            }
            MessageType::Temperature => {
                self.process_temperature(msg)?;
            }
            MessageType::LaserScan => {
                // TODO: Implement laser scan processing
            }
            MessageType::Image => {
                // TODO: Implement image processing
            }
            MessageType::Odometry => {
                // TODO: Implement odometry processing
            }
            MessageType::PoseStamped => {
                // TODO: Implement pose processing
            }
            MessageType::Twist => {
                // TODO: Implement twist processing
            }
            MessageType::Unknown(_) => {
                // Skip unknown message types
            }
        }

        Ok(())
    }

    /// Process a PointCloud2 message
    fn process_pointcloud(&mut self, msg: &BagMessage) -> Result<()> {
        // Parse point cloud
        let points = self.pointcloud_processor.parse(&msg.data)?;

        // Log to Rerun if available
        if let Some(ref stream) = self.rerun_stream {
            // Set timeline
            stream.set_time_sequence("bag_time", ((msg.timestamp - self.start_time) * 1000.0) as i64);

            // Convert to Rerun format
            let positions: Vec<[f32; 3]> = points.iter()
                .map(|p| [p.x, p.y, p.z])
                .collect();

            // Get intensities if available
            let colors: Option<Vec<rerun::Color>> = if points.iter().any(|p| p.intensity > 0.0) {
                Some(points.iter().map(|p| {
                    // Map intensity to grayscale
                    let v = (p.intensity.clamp(0.0, 255.0)) as u8;
                    rerun::Color::from_rgb(v, v, v)
                }).collect())
            } else {
                None
            };

            // Entity path from topic name
            let entity_path = msg.topic.replace("/", "_").trim_start_matches('_').to_string();

            let mut points3d = rerun::Points3D::new(positions)
                .with_radii([0.02]);

            if let Some(colors) = colors {
                points3d = points3d.with_colors(colors);
            }

            stream.log(entity_path, &points3d)
                .map_err(|e| RosBagError::ParseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Process a TF message
    fn process_tf(&mut self, msg: &BagMessage) -> Result<()> {
        self.tf_buffer.process_tf_message(&msg.data)?;
        Ok(())
    }

    /// Process an IMU message
    fn process_imu(&mut self, msg: &BagMessage) -> Result<()> {
        let imu_data = self.imu_processor.parse(&msg.data)?;

        if let Some(ref stream) = self.rerun_stream {
            // Set timeline
            let bag_time_ms = ((msg.timestamp - self.start_time) * 1000.0) as i64;
            stream.set_time_sequence("bag_time", bag_time_ms);

            // Entity path from topic name (e.g., /gpsimu_driver/imu_data -> world/imu)
            let entity_base = "world/imu";

            // Log angular velocity as Arrows3D
            let av = imu_data.angular_velocity;
            stream.log(
                format!("{}/gyro_arrow", entity_base),
                &rerun::Arrows3D::from_vectors([[av[0] as f32, av[1] as f32, av[2] as f32]])
                    .with_colors([rerun::Color::from_rgb(255, 165, 0)]) // Orange
                    .with_origins([[0.0, 0.0, 0.0]]),
            ).ok();

            // Log linear acceleration as Arrows3D
            let la = imu_data.linear_acceleration;
            stream.log(
                format!("{}/accel_arrow", entity_base),
                &rerun::Arrows3D::from_vectors([[la[0] as f32, la[1] as f32, la[2] as f32]])
                    .with_colors([rerun::Color::from_rgb(0, 255, 255)]) // Cyan
                    .with_origins([[0.0, 0.0, 0.0]]),
            ).ok();

            // Log scalar plots for accelerometer axes
            stream.log(
                format!("{}/accel_x", entity_base),
                &rerun::Scalars::new([la[0]]),
            ).ok();
            stream.log(
                format!("{}/accel_y", entity_base),
                &rerun::Scalars::new([la[1]]),
            ).ok();
            stream.log(
                format!("{}/accel_z", entity_base),
                &rerun::Scalars::new([la[2]]),
            ).ok();

            // Log scalar plots for gyroscope axes
            stream.log(
                format!("{}/gyro_x", entity_base),
                &rerun::Scalars::new([av[0]]),
            ).ok();
            stream.log(
                format!("{}/gyro_y", entity_base),
                &rerun::Scalars::new([av[1]]),
            ).ok();
            stream.log(
                format!("{}/gyro_z", entity_base),
                &rerun::Scalars::new([av[2]]),
            ).ok();
        }

        Ok(())
    }

    /// Process an NMEA sentence message
    fn process_nmea(&mut self, msg: &BagMessage) -> Result<()> {
        let nmea = self.gps_processor.parse_nmea(&msg.data)?;

        if let Some(ref stream) = self.rerun_stream {
            // Set timeline
            let bag_time_ms = ((msg.timestamp - self.start_time) * 1000.0) as i64;
            stream.set_time_sequence("bag_time", bag_time_ms);

            let entity_base = "world/gps";

            // Log NMEA sentence as text
            stream.log(
                format!("{}/status", entity_base),
                &rerun::TextLog::new(format!("[{}] {}", nmea.sentence_type, nmea.sentence)),
            ).ok();

            // If we have a GPS position, log it
            if let Some(pos) = self.gps_processor.last_position() {
                // Log position as 3D point (using lat/lon scaled, altitude as Z)
                // For visualization, we scale lat/lon to meters (rough approximation)
                let x = pos.longitude * 111320.0 * (pos.latitude.to_radians().cos());
                let y = pos.latitude * 110540.0;
                let z = pos.altitude;

                stream.log(
                    format!("{}/position", entity_base),
                    &rerun::Points3D::new([[x as f32, y as f32, z as f32]])
                        .with_colors([rerun::Color::from_rgb(0, 255, 0)]) // Green
                        .with_radii([0.5]),
                ).ok();

                // Log lat/lon/alt as scalars for time series
                stream.log(
                    format!("{}/latitude", entity_base),
                    &rerun::Scalars::new([pos.latitude]),
                ).ok();
                stream.log(
                    format!("{}/longitude", entity_base),
                    &rerun::Scalars::new([pos.longitude]),
                ).ok();
                stream.log(
                    format!("{}/altitude", entity_base),
                    &rerun::Scalars::new([pos.altitude]),
                ).ok();
                stream.log(
                    format!("{}/satellites", entity_base),
                    &rerun::Scalars::new([pos.num_satellites as f64]),
                ).ok();
            }
        }

        Ok(())
    }

    /// Process a TimeReference message
    fn process_time_reference(&mut self, msg: &BagMessage) -> Result<()> {
        let time_ref = self.gps_processor.parse_time_reference(&msg.data)?;

        if let Some(ref stream) = self.rerun_stream {
            // Set timeline
            let bag_time_ms = ((msg.timestamp - self.start_time) * 1000.0) as i64;
            stream.set_time_sequence("bag_time", bag_time_ms);

            let entity_base = "world/time_ref";

            // Log time offset as scalar
            let time_offset = time_ref.time_ref - time_ref.timestamp;
            stream.log(
                format!("{}/offset", entity_base),
                &rerun::Scalars::new([time_offset]),
            ).ok();

            // Log as text
            stream.log(
                format!("{}/status", entity_base),
                &rerun::TextLog::new(format!("Source: {}, Offset: {:.6}s", time_ref.source, time_offset)),
            ).ok();
        }

        Ok(())
    }

    /// Process a Temperature message
    fn process_temperature(&mut self, msg: &BagMessage) -> Result<()> {
        let temp = self.gps_processor.parse_temperature(&msg.data)?;

        if let Some(ref stream) = self.rerun_stream {
            // Set timeline
            let bag_time_ms = ((msg.timestamp - self.start_time) * 1000.0) as i64;
            stream.set_time_sequence("bag_time", bag_time_ms);

            let entity_base = "world/temperature";

            // Log temperature as scalar
            stream.log(
                format!("{}/celsius", entity_base),
                &rerun::Scalars::new([temp.temperature]),
            ).ok();

            // Log as text
            stream.log(
                format!("{}/status", entity_base),
                &rerun::TextLog::new(format!("Temperature: {:.1}°C", temp.temperature)),
            ).ok();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state() {
        let state = PlaybackState::Stopped;
        assert_eq!(state, PlaybackState::Stopped);
    }

    #[test]
    fn test_nanos_to_secs() {
        assert_eq!(nanos_to_secs(1_000_000_000), 1.0);
        assert_eq!(nanos_to_secs(500_000_000), 0.5);
    }
}
