//! # MViz Dora Bridge - Universal Zenoh Publisher
//!
//! A generic Dora node that publishes ANY sensor data via Zenoh using
//! the MViz universal message protocol.
//!
//! ## Architecture
//! - **Robot side**: This bridge receives Dora inputs, publishes to Zenoh
//! - **PC side**: mviz-shell subscribes to Zenoh, displays in Rerun (generic)
//!
//! ## Universal Protocol
//! Topic: `{prefix}/{entity_path}` -> logs to Rerun at `{entity_path}`
//!
//! Message format: JSON header + optional binary payload
//! ```json
//! {"type": "points3d", "count": 1000, "format": "xyz_f32", "timestamp": 1.5}
//! <binary data>
//! ```
//!
//! ## Supported Data Types
//! - `points3d` - Point clouds (binary xyz_f32)
//! - `boxes3d` - 3D boxes (JSON)
//! - `arrows3d` - Arrows/vectors (JSON)
//! - `linestrips3d` - Line strips (JSON)
//! - `transform3d` - Coordinate transforms (JSON)
//! - `scalar` - Time-series values (JSON)
//! - `log` - System log messages (JSON)
//!
//! ## Environment Variables
//! - `ZENOH_CONNECT`: Zenoh router address (default: auto-discovery)
//! - `ZENOH_TOPIC_PREFIX`: Topic prefix (default: "mviz")
//! - `RUST_LOG`: Logging level

use dora_node_api::{DoraNode, Event, arrow::array::Float32Array};
use eyre::Result;
use mviz_core::zenoh_protocol::*;
use std::env;
use std::collections::HashMap;

/// Publish a log message to Zenoh
async fn publish_log(
    session: &zenoh::Session,
    topic_prefix: &str,
    timestamp: f64,
    level: &str,
    message: &str,
    node_id: &str,
) {
    let log_msg = MvizMessage {
        msg_type: "log".to_string(),
        timestamp: Some(timestamp),
        data: serde_json::json!({
            "level": level,
            "message": message,
            "node_id": node_id,
        }),
        format: None,
        count: None,
    };
    let topic = format!("{}/logs", topic_prefix);
    let _ = session.put(&topic, serialize_message(&log_msg, None)).await;
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting MViz Universal Dora Bridge");

    // Get configuration from environment
    let topic_prefix = env::var("ZENOH_TOPIC_PREFIX").unwrap_or_else(|_| "mviz".to_string());
    let zenoh_connect = env::var("ZENOH_CONNECT").ok();

    // Initialize Zenoh session
    log::info!("Initializing Zenoh session...");
    let zenoh_config = if let Some(addr) = zenoh_connect {
        log::info!("Connecting to Zenoh router: {}", addr);
        zenoh::Config::from_json5(&format!(
            r#"{{ connect: {{ endpoints: ["{}"] }} }}"#, addr
        )).unwrap_or_default()
    } else {
        log::info!("Using Zenoh auto-discovery (multicast scouting)");
        zenoh::Config::default()
    };

    let session = zenoh::open(zenoh_config)
        .await
        .expect("Failed to open Zenoh session");

    log::info!("Zenoh session opened successfully");
    log::info!("Topic prefix: {}", topic_prefix);

    // Publish bridge startup log
    publish_log(&session, &topic_prefix, 0.0, "INFO", "MViz bridge started, waiting for data...", "mviz_bridge").await;

    // Initialize Dora node
    let (_node, mut events) = DoraNode::init_from_env()?;
    log::info!("Dora node initialized, waiting for events...");

    let mut frame_count: u64 = 0;
    let start_time = std::time::Instant::now();

    // Accumulate trajectory from odometry poses
    let mut trajectory_points: Vec<[f32; 3]> = Vec::new();

    // Track message counts per source node for logging
    let mut node_msg_counts: HashMap<String, u64> = HashMap::new();

    // Main event loop - handle ANY input generically
    while let Some(event) = events.recv() {
        log::info!("Received event: {:?}", std::mem::discriminant(&event));
        match event {
            Event::Input { id, metadata: _, data } => {
                let input_id = id.as_str();
                frame_count += 1;
                let timestamp = start_time.elapsed().as_secs_f64();
                log::info!("Input event: id={}, data_type={}", input_id, data.data_type());

                // Extract source node from input_id (format: "node_name/output_name" or just "output_name")
                let source_node = input_id.split('/').next().unwrap_or(input_id);

                // Track message count per source node
                let count = node_msg_counts.entry(source_node.to_string()).or_insert(0);
                *count += 1;

                // Publish log for first message from each node
                if *count == 1 {
                    let msg = format!("First message received from {}", source_node);
                    publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, source_node).await;
                }

                // Publish periodic status logs (every 100 messages per node)
                if *count % 100 == 0 {
                    let msg = format!("Processed {} messages", count);
                    publish_log(&session, &topic_prefix, timestamp, "DEBUG", &msg, source_node).await;
                }

                // Try to extract as Float32Array (most common for robotics)
                let float_data: Option<Vec<f32>> = data
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .map(|arr| arr.values().to_vec());

                let Some(values) = float_data else {
                    log::warn!("Skipping non-float input: {}, type: {}", input_id, data.data_type());
                    let msg = format!("Skipping non-float data: {}", data.data_type());
                    publish_log(&session, &topic_prefix, timestamp, "WARN", &msg, source_node).await;
                    continue;
                };
                log::info!("Processing {} with {} float values", input_id, values.len());

                // Determine data type and entity path based on input ID
                let (entity_path, payload) = match input_id {
                    // Point cloud inputs -> points3d with binary data
                    "pointcloud" | "lidar" | "scan" | "points" => {
                        if values.len() < 3 {
                            continue;
                        }
                        let num_points = values.len() / 3;
                        let header = MvizMessage {
                            msg_type: "points3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "radius": 0.03,
                                "color": [100, 200, 255, 255]
                            }),
                            format: Some(binary_formats::POINTS_XYZ_F32.to_string()),
                            count: Some(num_points as u32),
                        };
                        // Binary payload: raw float32 xyz data
                        let binary: Vec<u8> = values.iter()
                            .flat_map(|v| v.to_le_bytes())
                            .collect();
                        ("world/pointcloud".to_string(), serialize_message(&header, Some(&binary)))
                    }

                    // Vehicle pose [x, y, theta, velocity] -> boxes3d + trajectory
                    "sim_pose" | "vehicle_pose" | "pose" if values.len() >= 3 => {
                        let (x, y, theta) = (values[0], values[1], values[2]);
                        let velocity = if values.len() >= 4 { values[3] } else { 0.0 };

                        // Log vehicle state periodically
                        if *count % 50 == 1 {
                            let msg = format!("Vehicle at ({:.2}, {:.2}), heading {:.1}°, v={:.2}m/s",
                                x, y, theta.to_degrees(), velocity);
                            publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, "bicycle_model").await;
                        }

                        // Add to accumulated trajectory
                        trajectory_points.push([x, y, 0.05]);

                        // Convert theta to quaternion [x, y, z, w]
                        let half_theta = theta / 2.0;
                        let quat = [0.0, 0.0, half_theta.sin(), half_theta.cos()];

                        // Publish vehicle body
                        let body_header = MvizMessage {
                            msg_type: "boxes3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "centers": [[x, y, 0.1]],
                                "sizes": [[0.5, 0.3, 0.2]],
                                "quaternions": [quat],
                                "color": [50, 150, 250, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        let body_topic = format!("{}/world/vehicle/body", topic_prefix);
                        let _ = session.put(&body_topic, serialize_message(&body_header, None)).await;

                        // Publish accumulated trajectory as green line
                        if trajectory_points.len() >= 2 {
                            let traj_header = MvizMessage {
                                msg_type: "linestrips3d".to_string(),
                                timestamp: Some(timestamp),
                                data: serde_json::json!({
                                    "strips": [trajectory_points.clone()],
                                    "color": [0, 255, 100, 255]  // Green trajectory
                                }),
                                format: None,
                                count: None,
                            };
                            let traj_topic = format!("{}/world/vehicle/trajectory", topic_prefix);
                            let _ = session.put(&traj_topic, serialize_message(&traj_header, None)).await;
                        }

                        // Return a dummy to satisfy match (already published above)
                        continue;
                    }

                    // Odometry pose [4x4 matrix or x,y,z,qx,qy,qz,qw] -> transform3d + trajectory
                    "odom_pose" | "odom" if values.len() >= 7 => {
                        // Extract position for trajectory
                        let (x, y, z) = if values.len() >= 16 {
                            // 4x4 matrix: translation is at indices 3, 7, 11 (column 3)
                            (values[3], values[7], values[11])
                        } else {
                            // x,y,z,qx,qy,qz,qw
                            (values[0], values[1], values[2])
                        };

                        // Add to accumulated trajectory
                        trajectory_points.push([x, y, z]);

                        // Publish transform
                        let transform_header = if values.len() >= 16 {
                            let mut matrix = [0.0_f32; 16];
                            matrix.copy_from_slice(&values[0..16]);
                            MvizMessage {
                                msg_type: "transform3d".to_string(),
                                timestamp: Some(timestamp),
                                data: serde_json::json!({
                                    "matrix4x4": matrix
                                }),
                                format: None,
                                count: None,
                            }
                        } else {
                            MvizMessage {
                                msg_type: "transform3d".to_string(),
                                timestamp: Some(timestamp),
                                data: serde_json::json!({
                                    "translation": [values[0], values[1], values[2]],
                                    "rotation": [values[3], values[4], values[5], values[6]]
                                }),
                                format: None,
                                count: None,
                            }
                        };

                        // Publish sensor pose
                        let pose_topic = format!("{}/world/sensor/pose", topic_prefix);
                        let _ = session.put(&pose_topic, serialize_message(&transform_header, None)).await;

                        // Publish accumulated trajectory as green line
                        if trajectory_points.len() >= 2 {
                            let traj_header = MvizMessage {
                                msg_type: "linestrips3d".to_string(),
                                timestamp: Some(timestamp),
                                data: serde_json::json!({
                                    "strips": [trajectory_points.clone()],
                                    "color": [0, 255, 100, 255]  // Green trajectory
                                }),
                                format: None,
                                count: None,
                            };
                            let traj_topic = format!("{}/world/trajectory", topic_prefix);
                            let _ = session.put(&traj_topic, serialize_message(&traj_header, None)).await;
                        }

                        // Also publish current position as a marker
                        let marker_header = MvizMessage {
                            msg_type: "points3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "positions": [[x, y, z]],
                                "radius": 0.15,
                                "color": [255, 200, 50, 255]  // Yellow marker
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/sensor/position".to_string(), serialize_message(&marker_header, None))
                    }

                    // Waypoints [x1,y1, x2,y2, ...] -> linestrips3d
                    "waypoints" | "path" if values.len() >= 4 => {
                        let points: Vec<[f32; 3]> = values
                            .chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| [c[0], c[1], 0.05])
                            .collect();

                        if points.len() < 2 {
                            continue;
                        }

                        // Log waypoint updates periodically
                        if *count % 100 == 1 {
                            let msg = format!("Path updated with {} waypoints", points.len());
                            publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, "simple_planner").await;
                        }

                        let header = MvizMessage {
                            msg_type: "linestrips3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "strips": [points],
                                "color": [100, 200, 255, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/planner/waypoints".to_string(), serialize_message(&header, None))
                    }

                    // Trajectory [x1,y1,z1,...] -> linestrips3d
                    "trajectory" if values.len() >= 6 => {
                        // Could be [x,y,z,...] or [x,y,z,qx,qy,qz,qw,...]
                        let stride = if values.len() % 7 == 0 { 7 } else { 3 };
                        let points: Vec<[f32; 3]> = values
                            .chunks(stride)
                            .filter(|c| c.len() >= 3)
                            .map(|c| [c[0], c[1], c[2]])
                            .collect();

                        if points.len() < 2 {
                            continue;
                        }

                        let header = MvizMessage {
                            msg_type: "linestrips3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "strips": [points],
                                "color": [255, 150, 50, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/trajectory".to_string(), serialize_message(&header, None))
                    }

                    // Target point [x, y] -> points3d
                    "target_point" | "target" | "goal" if values.len() >= 2 => {
                        let header = MvizMessage {
                            msg_type: "points3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "positions": [[values[0], values[1], 0.1]],
                                "radius": 0.15,
                                "color": [255, 0, 128, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/planner/target".to_string(), serialize_message(&header, None))
                    }

                    // IMU [qx,qy,qz,qw, gx,gy,gz, ax,ay,az] -> arrows3d
                    "imu_msg" | "imu" if values.len() >= 9 => {
                        let accel = [values[6] * 0.1, values[7] * 0.1, (values[8] - 9.81) * 0.1];
                        let header = MvizMessage {
                            msg_type: "arrows3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "origins": [[0.0, 0.0, 0.5]],
                                "vectors": [accel],
                                "color": [16, 185, 129, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/imu/accelerometer".to_string(), serialize_message(&header, None))
                    }

                    // Vehicle state [x,y,theta,v, steering,accel,yaw_rate] -> scalar (could add more)
                    "sim_state" | "vehicle_state" if values.len() >= 7 => {
                        // Publish steering as scalar for time-series
                        let header = MvizMessage {
                            msg_type: "scalar".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "value": values[4]  // steering
                            }),
                            format: None,
                            count: None,
                        };
                        ("metrics/steering".to_string(), serialize_message(&header, None))
                    }

                    // Generic float array - publish as points if 3D-like
                    _ if values.len() >= 3 && values.len() % 3 == 0 => {
                        let num_points = values.len() / 3;
                        let header = MvizMessage {
                            msg_type: "points3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "radius": 0.05,
                                "color": [200, 200, 200, 255]
                            }),
                            format: Some(binary_formats::POINTS_XYZ_F32.to_string()),
                            count: Some(num_points as u32),
                        };
                        let binary: Vec<u8> = values.iter()
                            .flat_map(|v| v.to_le_bytes())
                            .collect();
                        (format!("world/{}", input_id), serialize_message(&header, Some(&binary)))
                    }

                    // Skip unrecognized inputs
                    _ => {
                        log::debug!("Skipping unrecognized input: {} ({} values)", input_id, values.len());
                        continue;
                    }
                };

                // Publish to Zenoh
                let topic = format!("{}/{}", topic_prefix, entity_path);
                if let Err(e) = session.put(&topic, payload).await {
                    log::warn!("Failed to publish to {}: {}", topic, e);
                }

                // Log progress periodically
                if frame_count % 100 == 0 {
                    log::info!("Published {} frames via Zenoh", frame_count);
                }
            }

            Event::Stop(_) => {
                log::info!("Received stop signal");
                let timestamp = start_time.elapsed().as_secs_f64();
                let msg = format!("Bridge stopping after {} total frames", frame_count);
                publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, "mviz_bridge").await;
                break;
            }

            _ => {}
        }
    }

    log::info!("MViz Dora Bridge stopped, published {} total frames", frame_count);
    Ok(())
}
