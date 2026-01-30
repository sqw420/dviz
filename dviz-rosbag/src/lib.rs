//! MViz ROS Bag Playback
//!
//! Provides ROS bag file playback with point cloud visualization through Rerun.
//!
//! ## Pipeline
//!
//! ```text
//! ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐     ┌─────────┐
//! │   rosbag    │────▶│  ros_pointcloud2 │────▶│   TfBuffer      │────▶│  Rerun  │
//! │   (raw)     │     │  (parse points)  │     │  (transform)    │     │  (viz)  │
//! └─────────────┘     └──────────────────┘     └─────────────────┘     └─────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use dviz_rosbag::RosBagPlayer;
//!
//! let mut player = RosBagPlayer::open("recording.bag")?;
//! println!("Topics: {:?}", player.topics());
//! println!("Duration: {:.2}s", player.duration());
//!
//! // Play through the bag
//! while let Some(msg) = player.read_next() {
//!     player.process_message(&msg)?;
//! }
//! ```

pub mod player;
pub mod pointcloud;
pub mod tf;
pub mod messages;
pub mod imu;
pub mod gps;

pub use player::{RosBagPlayer, PlaybackState, TopicInfo};
pub use pointcloud::PointCloudProcessor;
pub use tf::TfBuffer;
pub use messages::{BagMessage, MessageType};
pub use imu::{ImuProcessor, ImuData};
pub use gps::{GpsProcessor, NmeaSentence, GpsPosition, TimeReference, Temperature};

use thiserror::Error;

/// Errors that can occur during bag playback
#[derive(Error, Debug)]
pub enum RosBagError {
    #[error("Failed to open bag file: {0}")]
    OpenError(String),

    #[error("Failed to read message: {0}")]
    ReadError(String),

    #[error("Failed to parse message: {0}")]
    ParseError(String),

    #[error("Transform not found: {0} -> {1}")]
    TransformError(String, String),

    #[error("Unsupported message type: {0}")]
    UnsupportedType(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RosBagError>;
