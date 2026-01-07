//! MViz Rerun Bridge - Connection and data logging to Rerun viewer
//!
//! This crate manages the connection to a Rerun viewer and provides
//! adapters for logging various sensor data types.

pub mod bridge;
pub mod adapters;
pub mod simulation;

pub use bridge::{RerunBridge, RerunConfig, RerunError};
pub use adapters::{
    ImuAdapter, PointCloudAdapter, PoseAdapter, LaserScanAdapter,
    VehicleAdapter, GridAdapter,
};
pub use simulation::{SensorSimulator, ImuData, LidarScan, LidarPoint, VehiclePose};
