//! Zenoh Receiver - Universal receiver for any application data
//!
//! This receiver handles the MViz universal protocol and passes typed data
//! to be logged to Rerun. It doesn't know about specific applications.

use crossbeam_channel::{Receiver, Sender, bounded};
use mviz_core::zenoh_protocol::{MvizMessage, parse_message};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::io::Write;

fn debug_log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/mviz_zenoh_debug.log")
    {
        let _ = writeln!(f, "[{}] {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0), msg);
    }
}

/// Universal visualization data received from Zenoh
#[derive(Clone, Debug)]
pub struct VisData {
    /// Entity path (from Zenoh topic after prefix)
    pub entity_path: String,

    /// Message type (points3d, boxes3d, etc.)
    pub msg_type: String,

    /// Timestamp
    pub timestamp: f64,

    /// The full message (for JSON data types)
    pub message: MvizMessage,

    /// Binary payload (for point clouds, images)
    pub binary: Option<Vec<u8>>,
}

/// Message sent from Zenoh thread to UI thread
#[derive(Clone, Debug)]
pub enum ZenohMessage {
    /// Visualization data received
    Data(VisData),
    /// Connected to Zenoh
    Connected,
    /// Disconnected or error
    Disconnected(String),
    /// Status update
    Status(String),
}

/// Zenoh receiver that runs in a background thread
pub struct ZenohReceiver {
    rx: Receiver<ZenohMessage>,
    _handle: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl ZenohReceiver {
    /// Start a new Zenoh receiver in a background thread
    pub fn start(connect_addr: Option<String>, topic_prefix: Option<String>) -> Self {
        let (tx, rx) = bounded::<ZenohMessage>(500);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let prefix = topic_prefix.unwrap_or_else(|| "mviz".to_string());

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(Self::run_zenoh_loop(tx, running_clone, connect_addr, prefix));
        });

        Self {
            rx,
            _handle: handle,
            running,
        }
    }

    /// Try to receive the next message (non-blocking)
    pub fn try_recv(&self) -> Option<ZenohMessage> {
        self.rx.try_recv().ok()
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Signal the receiver to stop
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// The main Zenoh event loop
    async fn run_zenoh_loop(
        tx: Sender<ZenohMessage>,
        running: Arc<AtomicBool>,
        connect_addr: Option<String>,
        topic_prefix: String,
    ) {
        log::info!("ZenohReceiver: Starting universal receiver...");
        let _ = tx.send(ZenohMessage::Status("Connecting to Zenoh...".to_string()));

        // Configure Zenoh
        let config = if let Some(addr) = connect_addr {
            log::info!("ZenohReceiver: Connecting to router: {}", addr);
            zenoh::Config::from_json5(&format!(
                r#"{{ connect: {{ endpoints: ["{}"] }} }}"#, addr
            )).unwrap_or_default()
        } else {
            log::info!("ZenohReceiver: Using multicast scouting");
            zenoh::Config::default()
        };

        // Open Zenoh session
        let session = match zenoh::open(config).await {
            Ok(s) => {
                log::info!("ZenohReceiver: Zenoh session opened");
                let _ = tx.send(ZenohMessage::Connected);
                let _ = tx.send(ZenohMessage::Status("Zenoh connected".to_string()));
                s
            }
            Err(e) => {
                let msg = format!("Failed to open Zenoh session: {}", e);
                log::error!("ZenohReceiver: {}", msg);
                let _ = tx.send(ZenohMessage::Disconnected(msg));
                return;
            }
        };

        // Subscribe to ALL topics under prefix with wildcard
        let all_topics = format!("{}/**", topic_prefix);
        log::info!("ZenohReceiver: Subscribing to {}", all_topics);

        let subscriber = match session.declare_subscriber(&all_topics).await {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("Failed to subscribe: {}", e);
                log::error!("ZenohReceiver: {}", msg);
                let _ = tx.send(ZenohMessage::Disconnected(msg));
                return;
            }
        };

        let _ = tx.send(ZenohMessage::Status(format!("Subscribed to {}", all_topics)));
        debug_log(&format!("Subscribed to {}", all_topics));

        let mut frame_count: u64 = 0;
        let prefix_with_slash = format!("{}/", topic_prefix);
        debug_log(&format!("Prefix: '{}', waiting for messages...", prefix_with_slash));

        // Main receive loop
        while running.load(Ordering::Relaxed) {
            match tokio::time::timeout(
                std::time::Duration::from_millis(100),
                subscriber.recv_async(),
            ).await {
                Ok(Ok(sample)) => {
                    let key = sample.key_expr().as_str();
                    let bytes = sample.payload().to_bytes();

                    debug_log(&format!("Got message on topic '{}', {} bytes", key, bytes.len()));

                    // Extract entity path from topic (remove prefix)
                    let entity_path = if key.starts_with(&prefix_with_slash) {
                        key[prefix_with_slash.len()..].to_string()
                    } else {
                        key.to_string()
                    };

                    // Parse universal message format
                    if let Some((msg, binary)) = parse_message(&bytes) {
                        frame_count += 1;

                        debug_log(&format!("Parsed {} @ {}, binary={:?}",
                            msg.msg_type, entity_path, binary.map(|b| b.len())));

                        let vis_data = VisData {
                            entity_path,
                            msg_type: msg.msg_type.clone(),
                            timestamp: msg.timestamp.unwrap_or(0.0),
                            message: msg,
                            binary: binary.map(|b| b.to_vec()),
                        };

                        let _ = tx.send(ZenohMessage::Data(vis_data));

                        if frame_count % 100 == 0 {
                            debug_log(&format!("Received {} messages total", frame_count));
                        }
                    } else {
                        debug_log(&format!("Failed to parse message from {}, first 100 bytes: {:?}",
                            key, String::from_utf8_lossy(&bytes[..bytes.len().min(100)])));
                    }
                }
                Ok(Err(_)) => {
                    break;
                }
                Err(_) => {
                    // Timeout, continue
                }
            }
        }

        running.store(false, Ordering::Relaxed);
        let _ = tx.send(ZenohMessage::Disconnected("Zenoh loop ended".to_string()));
        log::info!("ZenohReceiver: Exiting, received {} messages", frame_count);
    }
}

impl Drop for ZenohReceiver {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Parse binary point cloud data (xyz_f32 format)
pub fn parse_points_xyz_f32(binary: &[u8], count: u32) -> Vec<[f32; 3]> {
    let mut points = Vec::with_capacity(count as usize);
    for chunk in binary.chunks_exact(12) {
        let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
        points.push([x, y, z]);
    }
    points
}
