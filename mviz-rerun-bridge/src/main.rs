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
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Parse dataflow YAML and return node definitions (for periodic republishing)
fn parse_dataflow_definitions(dataflow_path: &str) -> Vec<NodeDefinition> {
    log::info!("Parsing dataflow YAML: {}", dataflow_path);

    let yaml_content = match std::fs::read_to_string(dataflow_path) {
        Ok(content) => content,
        Err(e) => {
            log::warn!("Failed to read dataflow YAML: {}", e);
            return Vec::new();
        }
    };

    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&yaml_content) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse dataflow YAML: {}", e);
            return Vec::new();
        }
    };

    // Build node definitions
    let mut node_definitions: Vec<NodeDefinition> = Vec::new();

    // First pass: collect all nodes and their outputs
    let mut _node_outputs: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(nodes) = yaml_value.get("nodes").and_then(|n| n.as_sequence()) {
        for node in nodes {
            if let Some(id) = node.get("id").and_then(|i| i.as_str()) {
                let mut outputs = Vec::new();

                // Get outputs from operator section
                if let Some(op) = node.get("operator") {
                    if let Some(out_list) = op.get("outputs").and_then(|o| o.as_sequence()) {
                        for out in out_list {
                            if let Some(out_str) = out.as_str() {
                                outputs.push(out_str.to_string());
                            }
                        }
                    }
                }

                _node_outputs.insert(id.to_string(), outputs);
            }
        }
    }

    // Second pass: build complete definitions with inputs and destinations
    if let Some(nodes) = yaml_value.get("nodes").and_then(|n| n.as_sequence()) {
        for node in nodes {
            if let Some(id) = node.get("id").and_then(|i| i.as_str()) {
                let mut inputs: Vec<NodeInputDef> = Vec::new();
                let mut outputs: Vec<NodeOutputDef> = Vec::new();
                let mut operator_path: Option<String> = None;

                // Get operator path
                if let Some(op) = node.get("operator") {
                    if let Some(python_path) = op.get("python").and_then(|p| p.as_str()) {
                        operator_path = Some(python_path.to_string());
                    } else if let Some(rust_path) = op.get("rust").and_then(|p| p.as_str()) {
                        operator_path = Some(rust_path.to_string());
                    }

                    // Parse inputs
                    if let Some(input_map) = op.get("inputs").and_then(|i| i.as_mapping()) {
                        for (key, value) in input_map {
                            if let (Some(name), Some(source)) = (key.as_str(), value.as_str()) {
                                inputs.push(NodeInputDef {
                                    name: name.to_string(),
                                    source: source.to_string(),
                                });
                            }
                        }
                    }

                    // Parse outputs and find destinations
                    if let Some(out_list) = op.get("outputs").and_then(|o| o.as_sequence()) {
                        for out in out_list {
                            if let Some(out_name) = out.as_str() {
                                // Find which nodes use this output
                                let full_output = format!("{}/{}", id, out_name);
                                let mut destinations: Vec<String> = Vec::new();

                                // Search all other nodes' inputs for this output
                                for other_node in yaml_value.get("nodes").and_then(|n| n.as_sequence()).unwrap_or(&vec![]) {
                                    if let Some(other_id) = other_node.get("id").and_then(|i| i.as_str()) {
                                        if let Some(other_op) = other_node.get("operator") {
                                            if let Some(other_inputs) = other_op.get("inputs").and_then(|i| i.as_mapping()) {
                                                for (_inp_name, inp_source) in other_inputs {
                                                    if let Some(src) = inp_source.as_str() {
                                                        if src == full_output {
                                                            destinations.push(other_id.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Also check direct inputs (non-operator format)
                                        if let Some(direct_inputs) = other_node.get("inputs").and_then(|i| i.as_mapping()) {
                                            for (_inp_name, inp_source) in direct_inputs {
                                                if let Some(src) = inp_source.as_str() {
                                                    if src == full_output {
                                                        destinations.push(other_id.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                outputs.push(NodeOutputDef {
                                    name: out_name.to_string(),
                                    destinations,
                                });
                            }
                        }
                    }
                }

                // Also check direct inputs (non-operator format)
                if let Some(input_map) = node.get("inputs").and_then(|i| i.as_mapping()) {
                    for (key, value) in input_map {
                        if let (Some(name), Some(source)) = (key.as_str(), value.as_str()) {
                            inputs.push(NodeInputDef {
                                name: name.to_string(),
                                source: source.to_string(),
                            });
                        }
                    }
                }

                node_definitions.push(NodeDefinition {
                    id: id.to_string(),
                    inputs,
                    outputs,
                    operator: operator_path,
                });
            }
        }
    }

    log::info!("Parsed {} node definitions", node_definitions.len());
    node_definitions
}

/// Publish node definitions to Zenoh
async fn publish_node_definitions(
    session: &zenoh::Session,
    topic_prefix: &str,
    node_definitions: &[NodeDefinition],
) {
    for node_def in node_definitions {
        let msg = MvizMessage {
            msg_type: "node_definition".to_string(),
            timestamp: Some(0.0),
            data: serde_json::to_value(node_def).unwrap_or_default(),
            format: None,
            count: None,
        };
        let topic = format!("{}/definitions/{}", topic_prefix, node_def.id);
        if let Err(e) = session.put(&topic, serialize_message(&msg, None)).await {
            log::warn!("Failed to publish node definition for {}: {}", node_def.id, e);
        } else {
            log::debug!("Published definition for node: {} ({} inputs, {} outputs)",
                node_def.id, node_def.inputs.len(), node_def.outputs.len());
        }
    }
    if !node_definitions.is_empty() {
        log::info!("Published {} node definitions", node_definitions.len());
    }
}

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

/// Publish I/O activity log - shows live message data on ports
async fn publish_io_activity(
    session: &zenoh::Session,
    topic_prefix: &str,
    timestamp: f64,
    node_id: &str,
    port_name: &str,
    port_type: &str,  // "input" or "output"
    data_summary: &str,
) {
    let log_msg = MvizMessage {
        msg_type: "log".to_string(),
        timestamp: Some(timestamp),
        data: serde_json::json!({
            "level": "INFO",
            "message": data_summary,
            "node_id": node_id,
            "port": port_name,
            "port_type": port_type,
        }),
        format: None,
        count: None,
    };
    let topic = format!("{}/logs", topic_prefix);
    let _ = session.put(&topic, serialize_message(&log_msg, None)).await;
}

/// Format float values as a summary string
fn format_values_summary(values: &[f32], max_values: usize) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    if values.len() <= max_values {
        let formatted: Vec<String> = values.iter().map(|v| format!("{:.2}", v)).collect();
        format!("[{}]", formatted.join(", "))
    } else {
        let formatted: Vec<String> = values.iter().take(max_values).map(|v| format!("{:.2}", v)).collect();
        format!("[{}, ...+{}]", formatted.join(", "), values.len() - max_values)
    }
}

// ============================================================================
// Dynamic Graph Discovery
// ============================================================================

/// Tracks dataflow graph from node definitions and runtime activity
struct GraphState {
    /// All nodes from definitions
    nodes: HashSet<String>,
    /// All edges from definitions (from_node, from_port, to_node, to_port)
    edges: HashSet<(String, String, String, String)>,
    /// Last activity time per node
    node_activity: HashMap<String, f64>,
    /// Input ID to source node mapping (from definitions)
    input_source_map: HashMap<String, (String, String)>, // input_name -> (source_node, source_port)
}

impl GraphState {
    fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
            node_activity: HashMap::new(),
            input_source_map: HashMap::new(),
        }
    }

    /// Initialize graph from node definitions (parses complete topology from YAML)
    fn init_from_definitions(&mut self, definitions: &[NodeDefinition]) {
        for node_def in definitions {
            // Add node
            self.nodes.insert(node_def.id.clone());

            // Process inputs to build edges and input mapping
            for input in &node_def.inputs {
                // Source format: "source_node/output_port"
                if input.source.contains('/') {
                    let parts: Vec<&str> = input.source.split('/').collect();
                    if parts.len() >= 2 {
                        let source_node = parts[0].to_string();
                        let source_port = parts[1].to_string();

                        // Add source node
                        self.nodes.insert(source_node.clone());

                        // Add edge
                        self.edges.insert((
                            source_node.clone(),
                            source_port.clone(),
                            node_def.id.clone(),
                            input.name.clone(),
                        ));

                        // Build input mapping for this node
                        if node_def.id == "mviz_bridge" {
                            self.input_source_map.insert(
                                input.name.clone(),
                                (source_node, source_port),
                            );
                        }
                    }
                } else if input.source.starts_with("dora/timer/") {
                    // Timer input - don't create edge, just note the node exists
                } else {
                    // Simple source name - treat as node
                    self.nodes.insert(input.source.clone());
                }
            }
        }

        log::info!("GraphState initialized from definitions: {} nodes, {} edges",
            self.nodes.len(), self.edges.len());
    }

    /// Record activity from an input message
    fn record_input(&mut self, input_id: &str, timestamp: f64) {
        // Always mark mviz_bridge as active
        self.node_activity.insert("mviz_bridge".to_string(), timestamp);

        // Look up source node from mapping (built from definitions)
        if let Some((source_node, _source_port)) = self.input_source_map.get(input_id) {
            self.node_activity.insert(source_node.clone(), timestamp);
        } else if input_id.contains('/') {
            // Fallback: parse from input_id if it has source/port format
            let parts: Vec<&str> = input_id.split('/').collect();
            if parts.len() >= 2 {
                self.node_activity.insert(parts[0].to_string(), timestamp);
            }
        }
    }

    /// Build a GraphUpdate message
    fn to_graph_update(&self, timestamp: f64) -> GraphUpdate {
        let nodes: Vec<GraphNode> = self.nodes.iter().map(|id| {
            let last_seen = *self.node_activity.get(id).unwrap_or(&0.0);
            let status = if timestamp - last_seen < 2.0 {
                GraphNodeStatus::Active
            } else {
                GraphNodeStatus::Idle
            };
            GraphNode {
                id: id.clone(),
                status,
                last_seen,
            }
        }).collect();

        let edges: Vec<GraphEdge> = self.edges.iter().map(|(from_node, from_port, to_node, to_port)| {
            GraphEdge {
                from_node: from_node.clone(),
                from_port: from_port.clone(),
                to_node: to_node.clone(),
                to_port: to_port.clone(),
            }
        }).collect();

        GraphUpdate {
            nodes,
            edges,
            timestamp,
        }
    }
}

/// Publish graph update to Zenoh
async fn publish_graph_update(
    session: &zenoh::Session,
    topic_prefix: &str,
    graph_state: &GraphState,
    timestamp: f64,
) {
    let graph_update = graph_state.to_graph_update(timestamp);

    let msg = MvizMessage {
        msg_type: "graph_update".to_string(),
        timestamp: Some(timestamp),
        data: serde_json::to_value(&graph_update).unwrap_or_default(),
        format: None,
        count: None,
    };

    let topic = format!("{}/graph", topic_prefix);
    if let Err(e) = session.put(&topic, serialize_message(&msg, None)).await {
        log::warn!("Failed to publish graph update: {}", e);
    } else {
        log::debug!("Published graph update: {} nodes, {} edges",
            graph_update.nodes.len(), graph_update.edges.len());
    }
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

    // Parse and store node definitions for periodic republishing
    let node_definitions: Vec<NodeDefinition> = if let Ok(dataflow_path) = env::var("DATAFLOW_PATH") {
        log::info!("Using DATAFLOW_PATH: {}", dataflow_path);
        let defs = parse_dataflow_definitions(&dataflow_path);
        publish_node_definitions(&session, &topic_prefix, &defs).await;
        defs
    } else {
        // Try common paths
        let possible_paths = [
            "dataflow-mapping.yml",
            "dataflow-path-following.yml",
            "dataflow.yml",
            "../dataflow-mapping.yml",
            "../dataflow-path-following.yml",
            "../dataflow.yml",
        ];
        let mut found_defs = Vec::new();
        for path in &possible_paths {
            if Path::new(path).exists() {
                log::info!("Found dataflow at: {}", path);
                found_defs = parse_dataflow_definitions(path);
                publish_node_definitions(&session, &topic_prefix, &found_defs).await;
                break;
            }
        }
        found_defs
    };

    // Track last time we published definitions (for periodic republishing)
    let mut last_def_publish = std::time::Instant::now();
    const DEF_REPUBLISH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

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

    // Dynamic graph discovery state - initialize from node definitions
    let mut graph_state = GraphState::new();
    if !node_definitions.is_empty() {
        graph_state.init_from_definitions(&node_definitions);
    }
    let mut last_graph_publish = std::time::Instant::now();
    const GRAPH_PUBLISH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

    // Main event loop - handle ANY input generically
    while let Some(event) = events.recv() {
        // Periodically republish node definitions for late-joining subscribers
        if !node_definitions.is_empty() && last_def_publish.elapsed() >= DEF_REPUBLISH_INTERVAL {
            publish_node_definitions(&session, &topic_prefix, &node_definitions).await;
            last_def_publish = std::time::Instant::now();
        }

        // Periodically publish graph updates
        if last_graph_publish.elapsed() >= GRAPH_PUBLISH_INTERVAL {
            let timestamp = start_time.elapsed().as_secs_f64();
            publish_graph_update(&session, &topic_prefix, &graph_state, timestamp).await;
            last_graph_publish = std::time::Instant::now();
        }

        log::info!("Received event: {:?}", std::mem::discriminant(&event));
        match event {
            Event::Input { id, metadata: _, data } => {
                let input_id = id.as_str();
                frame_count += 1;
                let timestamp = start_time.elapsed().as_secs_f64();
                log::info!("Input event: id={}, data_type={}", input_id, data.data_type());

                // Record input for graph discovery
                graph_state.record_input(input_id, timestamp);

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

                // Publish I/O activity for mviz_bridge inputs (showing live message data)
                let data_summary = format_values_summary(&values, 4);
                publish_io_activity(
                    &session,
                    &topic_prefix,
                    timestamp,
                    "mviz_bridge",
                    input_id,
                    "input",
                    &data_summary,
                ).await;

                // Also publish I/O activity for the SOURCE node (its output)
                // input_id format is usually "source_node/output_name" or just "output_name"
                let (src_node, output_name): (&str, &str) = if input_id.contains('/') {
                    let parts: Vec<&str> = input_id.split('/').collect();
                    (parts[0], *parts.get(1).unwrap_or(&input_id))
                } else {
                    // Generic fallback: use source_node parsed from input_id
                    (source_node, input_id)
                };
                publish_io_activity(
                    &session,
                    &topic_prefix,
                    timestamp,
                    src_node,
                    output_name,
                    "output",
                    &data_summary,
                ).await;

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

                    // ROS2 Pose [x,y,z,qx,qy,qz,qw] -> boxes3d + trajectory
                    // Used by geometry_msgs/PoseStamped
                    "pose" | "robot_pose" | "amcl_pose" if values.len() == 7 => {
                        let (x, y, z) = (values[0], values[1], values[2]);
                        let quat = [values[3], values[4], values[5], values[6]];

                        // Add to accumulated trajectory
                        trajectory_points.push([x, y, z.max(0.05)]);

                        // Log pose periodically
                        if *count % 50 == 1 {
                            let msg = format!("Pose at ({:.2}, {:.2}, {:.2})", x, y, z);
                            publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, source_node).await;
                        }

                        // Publish robot body as box
                        let body_header = MvizMessage {
                            msg_type: "boxes3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "centers": [[x, y, z + 0.15]],
                                "sizes": [[0.5, 0.3, 0.3]],
                                "quaternions": [quat],
                                "color": [50, 150, 250, 255]
                            }),
                            format: None,
                            count: None,
                        };
                        let body_topic = format!("{}/world/robot/body", topic_prefix);
                        let _ = session.put(&body_topic, serialize_message(&body_header, None)).await;

                        // Publish accumulated trajectory
                        if trajectory_points.len() >= 2 {
                            let traj_header = MvizMessage {
                                msg_type: "linestrips3d".to_string(),
                                timestamp: Some(timestamp),
                                data: serde_json::json!({
                                    "strips": [trajectory_points.clone()],
                                    "color": [0, 255, 100, 255]
                                }),
                                format: None,
                                count: None,
                            };
                            let traj_topic = format!("{}/world/robot/trajectory", topic_prefix);
                            let _ = session.put(&traj_topic, serialize_message(&traj_header, None)).await;
                        }
                        continue;
                    }

                    // Vehicle pose [x, y, theta, velocity] -> boxes3d + trajectory
                    "sim_pose" | "vehicle_pose" if values.len() >= 3 => {
                        let (x, y, theta) = (values[0], values[1], values[2]);
                        let velocity = if values.len() >= 4 { values[3] } else { 0.0 };

                        // Log vehicle state periodically
                        if *count % 50 == 1 {
                            let msg = format!("Vehicle at ({:.2}, {:.2}), heading {:.1}°, v={:.2}m/s",
                                x, y, theta.to_degrees(), velocity);
                            publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, source_node).await;
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
                            publish_log(&session, &topic_prefix, timestamp, "INFO", &msg, source_node).await;
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

                    // ROS2 Markers [x,y,z,sx,sy,sz,qx,qy,qz,qw] -> boxes3d
                    // Used by visualization_msgs/Marker (CUBE/SPHERE type)
                    "markers" if values.len() == 10 => {
                        let (x, y, z) = (values[0], values[1], values[2]);
                        let (sx, sy, sz) = (values[3], values[4], values[5]);
                        let quat = [values[6], values[7], values[8], values[9]];

                        let header = MvizMessage {
                            msg_type: "boxes3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "centers": [[x, y, z]],
                                "sizes": [[sx, sy, sz]],
                                "quaternions": [quat],
                                "color": [255, 165, 0, 200]  // Orange markers
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/markers".to_string(), serialize_message(&header, None))
                    }

                    // ROS2 Arrow Markers [ox,oy,oz,vx,vy,vz] -> arrows3d
                    // Used by visualization_msgs/Marker (ARROW type)
                    "markers" if values.len() == 6 => {
                        let header = MvizMessage {
                            msg_type: "arrows3d".to_string(),
                            timestamp: Some(timestamp),
                            data: serde_json::json!({
                                "origins": [[values[0], values[1], values[2]]],
                                "vectors": [[values[3], values[4], values[5]]],
                                "color": [255, 100, 100, 255]  // Red arrows
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/markers/arrows".to_string(), serialize_message(&header, None))
                    }

                    // ROS2 Line Markers [x1,y1,z1,x2,y2,z2,...] -> linestrips3d
                    // Used by visualization_msgs/Marker (LINE_STRIP type)
                    "markers" if values.len() >= 6 && values.len() % 3 == 0 => {
                        let points: Vec<[f32; 3]> = values
                            .chunks(3)
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
                                "color": [255, 200, 50, 255]  // Yellow lines
                            }),
                            format: None,
                            count: None,
                        };
                        ("world/markers/lines".to_string(), serialize_message(&header, None))
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
