//! Dora Receiver - Background thread for receiving sensor data from Dora dataflow
//!
//! This module runs a Dora node in a background thread and forwards sensor data
//! to the Makepad UI thread via crossbeam channels.

use crossbeam_channel::{Receiver, Sender, bounded};
use dora_node_api::{DoraNode, Event, arrow::array::Float32Array};
use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Sensor data received from Dora dataflow
#[derive(Clone, Debug, Default)]
pub struct DoraData {
    /// Vehicle pose [x, y, theta, velocity]
    pub pose: Option<VehiclePose>,
    /// Vehicle state [steering, accel, yaw_rate]
    pub state: Option<VehicleState>,
    /// IMU data [accel_x/y/z, gyro_x/y/z]
    pub imu: Option<ImuData>,
    /// Target point from planner
    pub target: Option<[f32; 2]>,
    /// Waypoints from planner
    pub waypoints: Option<Vec<[f32; 2]>>,
    /// Frame count
    pub frame_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct VehiclePose {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
    pub velocity: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VehicleState {
    pub steering: f32,
    pub acceleration: f32,
    pub yaw_rate: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ImuData {
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
}

/// Message sent from Dora thread to UI thread
#[derive(Clone, Debug)]
pub enum DoraMessage {
    /// New sensor data received
    Data(DoraData),
    /// Dora node connected successfully
    Connected,
    /// Dora node disconnected or error
    Disconnected(String),
    /// Status update
    Status(String),
}

/// Dora receiver that runs in a background thread
pub struct DoraReceiver {
    /// Channel to receive messages from Dora thread
    rx: Receiver<DoraMessage>,
    /// Handle to the background thread
    _handle: JoinHandle<()>,
    /// Flag to signal shutdown
    running: Arc<AtomicBool>,
}

impl DoraReceiver {
    /// Start a new Dora receiver in a background thread
    pub fn start() -> Self {
        let (tx, rx) = bounded::<DoraMessage>(100);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            Self::run_dora_loop(tx, running_clone);
        });

        Self {
            rx,
            _handle: handle,
            running,
        }
    }

    /// Try to receive the next message (non-blocking)
    pub fn try_recv(&self) -> Option<DoraMessage> {
        self.rx.try_recv().ok()
    }

    /// Check if connected to Dora
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Signal the receiver to stop
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// The main Dora event loop (runs in background thread)
    fn run_dora_loop(tx: Sender<DoraMessage>, running: Arc<AtomicBool>) {
        log::info!("DoraReceiver: Starting Dora node...");

        // Try to initialize Dora node from environment
        let node_result = DoraNode::init_from_env();

        let (_node, mut events) = match node_result {
            Ok(n) => {
                let _ = tx.send(DoraMessage::Connected);
                let _ = tx.send(DoraMessage::Status("Dora node connected".to_string()));
                log::info!("DoraReceiver: Dora node initialized successfully");
                n
            }
            Err(e) => {
                let msg = format!("Failed to init Dora: {}", e);
                log::warn!("DoraReceiver: {}", msg);
                let _ = tx.send(DoraMessage::Disconnected(msg.clone()));
                let _ = tx.send(DoraMessage::Status(msg));
                return;
            }
        };

        let mut data = DoraData::default();
        let mut frame_count: u64 = 0;

        // Main event loop
        while running.load(Ordering::Relaxed) {
            // Use recv with timeout to allow checking running flag
            let event = match events.recv() {
                Some(e) => e,
                None => {
                    log::info!("DoraReceiver: Event stream ended");
                    break;
                }
            };

            match event {
                Event::Input { id, metadata: _, data: arrow_data } => {
                    let input_id = id.as_str();
                    frame_count += 1;

                    // Try to extract as Float32Array
                    let float_data: Option<Vec<f32>> = arrow_data
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .map(|arr| arr.values().to_vec());

                    let Some(values) = float_data else {
                        continue;
                    };

                    match input_id {
                        "sim_pose" => {
                            if values.len() >= 4 {
                                data.pose = Some(VehiclePose {
                                    x: values[0],
                                    y: values[1],
                                    theta: values[2],
                                    velocity: values[3],
                                });
                            }
                        }

                        "sim_state" => {
                            if values.len() >= 7 {
                                data.state = Some(VehicleState {
                                    steering: values[4],
                                    acceleration: values[5],
                                    yaw_rate: values[6],
                                });
                            }
                        }

                        "imu_msg" => {
                            if values.len() >= 9 {
                                data.imu = Some(ImuData {
                                    accel: [values[6], values[7], values[8]],
                                    gyro: [values[3], values[4], values[5]],
                                });
                            }
                        }

                        "target_point" => {
                            if values.len() >= 2 {
                                data.target = Some([values[0], values[1]]);
                            }
                        }

                        "waypoints" => {
                            let waypoints: Vec<[f32; 2]> = values
                                .chunks(2)
                                .filter(|chunk| chunk.len() == 2)
                                .map(|chunk| [chunk[0], chunk[1]])
                                .collect();
                            if !waypoints.is_empty() {
                                data.waypoints = Some(waypoints);
                            }
                        }

                        _ => {}
                    }

                    // Send data update every few frames to avoid flooding
                    if frame_count % 3 == 0 {
                        data.frame_count = frame_count;
                        let _ = tx.send(DoraMessage::Data(data.clone()));
                    }
                }

                Event::Stop(_) => {
                    log::info!("DoraReceiver: Received stop signal");
                    let _ = tx.send(DoraMessage::Status("Dora stopped".to_string()));
                    break;
                }

                _ => {}
            }
        }

        running.store(false, Ordering::Relaxed);
        let _ = tx.send(DoraMessage::Disconnected("Dora loop ended".to_string()));
        log::info!("DoraReceiver: Exiting Dora loop");
    }
}

impl Drop for DoraReceiver {
    fn drop(&mut self) {
        self.stop();
    }
}
