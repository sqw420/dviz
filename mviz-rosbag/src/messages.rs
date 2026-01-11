//! Message Types for ROS Bag Playback
//!
//! Defines the message structures used during bag playback.

use serde::{Deserialize, Serialize};

/// A message read from a ROS bag
#[derive(Debug, Clone)]
pub struct BagMessage {
    /// Topic name
    pub topic: String,
    /// Message type
    pub msg_type: MessageType,
    /// Timestamp (seconds since bag start)
    pub timestamp: f64,
    /// Raw message data
    pub data: Vec<u8>,
}

/// Known ROS message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    /// sensor_msgs/PointCloud2
    PointCloud2,
    /// tf2_msgs/TFMessage or tf/tfMessage
    TfMessage,
    /// sensor_msgs/LaserScan
    LaserScan,
    /// sensor_msgs/Image
    Image,
    /// nav_msgs/Odometry
    Odometry,
    /// sensor_msgs/Imu
    Imu,
    /// geometry_msgs/PoseStamped
    PoseStamped,
    /// geometry_msgs/Twist
    Twist,
    /// Unknown message type
    Unknown(String),
}

impl MessageType {
    /// Parse message type from ROS type string
    pub fn from_ros_type(ros_type: &str) -> Self {
        match ros_type {
            "sensor_msgs/PointCloud2" => MessageType::PointCloud2,
            "tf2_msgs/TFMessage" | "tf/tfMessage" => MessageType::TfMessage,
            "sensor_msgs/LaserScan" => MessageType::LaserScan,
            "sensor_msgs/Image" => MessageType::Image,
            "nav_msgs/Odometry" => MessageType::Odometry,
            "sensor_msgs/Imu" => MessageType::Imu,
            "geometry_msgs/PoseStamped" => MessageType::PoseStamped,
            "geometry_msgs/Twist" => MessageType::Twist,
            _ => MessageType::Unknown(ros_type.to_string()),
        }
    }

    /// Get the ROS type string
    pub fn ros_type(&self) -> &str {
        match self {
            MessageType::PointCloud2 => "sensor_msgs/PointCloud2",
            MessageType::TfMessage => "tf2_msgs/TFMessage",
            MessageType::LaserScan => "sensor_msgs/LaserScan",
            MessageType::Image => "sensor_msgs/Image",
            MessageType::Odometry => "nav_msgs/Odometry",
            MessageType::Imu => "sensor_msgs/Imu",
            MessageType::PoseStamped => "geometry_msgs/PoseStamped",
            MessageType::Twist => "geometry_msgs/Twist",
            MessageType::Unknown(s) => s,
        }
    }

    /// Check if this is a visualization-relevant type
    pub fn is_visualizable(&self) -> bool {
        matches!(
            self,
            MessageType::PointCloud2
                | MessageType::LaserScan
                | MessageType::Image
                | MessageType::Odometry
                | MessageType::PoseStamped
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_parsing() {
        assert_eq!(
            MessageType::from_ros_type("sensor_msgs/PointCloud2"),
            MessageType::PointCloud2
        );
        assert_eq!(
            MessageType::from_ros_type("tf2_msgs/TFMessage"),
            MessageType::TfMessage
        );
        assert_eq!(
            MessageType::from_ros_type("custom/Type"),
            MessageType::Unknown("custom/Type".to_string())
        );
    }

    #[test]
    fn test_visualizable() {
        assert!(MessageType::PointCloud2.is_visualizable());
        assert!(!MessageType::TfMessage.is_visualizable());
    }
}
