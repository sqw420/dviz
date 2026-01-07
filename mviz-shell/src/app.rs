//! MViz App - Main application shell
//!
//! Combines Makepad UI for controls with Rerun for 3D visualization.

use makepad_widgets::*;
use mviz_rerun_bridge::{RerunBridge, RerunConfig, SensorSimulator};
use std::io::Write;

fn debug_log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/mviz_debug.log")
    {
        let _ = writeln!(f, "[{}] {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0), msg);
    }
}

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mviz_widgets::theme::*;
    use mviz_widgets::sensor_panel::SensorGroup;
    use mviz_widgets::control_bar::ControlBar;

    // App icon
    MVIZ_ICON = dep("crate://self/resources/icons/viz.svg")

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { title: "MViz Rerun", inner_size: vec2(1000, 700) }
                pass: { clear_color: #1a1a1a }

                body = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    show_bg: true
                    draw_bg: { color: #1a1a1a }

                    // Header
                    header = <View> {
                        width: Fill, height: Fit
                        flow: Right
                        spacing: 12
                        align: {y: 0.5}
                        padding: {left: 20, right: 20, top: 15, bottom: 15}
                        show_bg: true
                        draw_bg: { color: #252525 }

                        <Icon> {
                            draw_icon: {
                                svg_file: (MVIZ_ICON)
                                fn get_color(self) -> vec4 { return #3b82f6; }
                            }
                            icon_walk: {width: 28, height: 28}
                        }

                        <Label> {
                            text: "MViz Rerun"
                            draw_text: {
                                color: #ffffff
                                text_style: { font_size: 20.0 }
                            }
                        }

                        <View> { width: Fill, height: 1 }

                        sim_toggle = <Button> {
                            text: "Start Simulation"
                            draw_text: { color: #fff }
                        }

                        launch_btn = <Button> {
                            text: "Launch Rerun Viewer"
                            draw_text: { color: #fff }
                        }
                    }

                    // Main content
                    content = <View> {
                        width: Fill, height: Fill
                        flow: Right
                        padding: 20
                        spacing: 20

                        // Left panel - Sensor displays
                        left_panel = <View> {
                            width: 340, height: Fill
                            flow: Down
                            spacing: 16

                            // IMU Panel
                            imu_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 16
                                spacing: 8
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "IMU Sensor"
                                    draw_text: { color: #ffffff, text_style: { font_size: 14.0 } }
                                }

                                imu_accel = <Label> {
                                    text: "Accel: 0.00, 0.00, 9.81"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }

                                imu_gyro = <Label> {
                                    text: "Gyro: 0.00, 0.00, 0.00"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }
                            }

                            // Vehicle Panel
                            vehicle_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 16
                                spacing: 8
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "Vehicle State"
                                    draw_text: { color: #ffffff, text_style: { font_size: 14.0 } }
                                }

                                vehicle_pos = <Label> {
                                    text: "Position: 0.00, 0.00"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }

                                vehicle_speed = <Label> {
                                    text: "Speed: 0.00 m/s"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }

                                vehicle_heading = <Label> {
                                    text: "Heading: 0.0 deg"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }
                            }

                            // LiDAR Panel
                            lidar_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 16
                                spacing: 8
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "LiDAR"
                                    draw_text: { color: #ffffff, text_style: { font_size: 14.0 } }
                                }

                                lidar_points = <Label> {
                                    text: "Points: 0"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }
                            }

                            // Simulation Panel
                            sim_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 16
                                spacing: 8
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "Simulation"
                                    draw_text: { color: #ffffff, text_style: { font_size: 14.0 } }
                                }

                                sim_time = <Label> {
                                    text: "Time: 0.00s"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }

                                sim_fps = <Label> {
                                    text: "FPS: 0"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
                                }
                            }

                            <View> { width: Fill, height: Fill }

                            status_label = <Label> {
                                text: "Status: Ready"
                                draw_text: { color: #606060, text_style: { font_size: 10.0 } }
                            }
                        }

                        // Right panel - Info
                        right_panel = <RoundedView> {
                            width: Fill, height: Fill
                            flow: Down
                            padding: 24
                            spacing: 16
                            show_bg: true
                            draw_bg: { color: #252525, border_radius: 8.0 }

                            <Label> {
                                text: "Rerun Visualization"
                                draw_text: { color: #ffffff, text_style: { font_size: 18.0 } }
                            }

                            info_text = <Label> {
                                width: Fill
                                text: "1. Click 'Launch Rerun Viewer' to open 3D visualization\n2. Click 'Start Simulation' to begin sensor data streaming\n\nThe viewer will display:\n- Vehicle body (blue box)\n- Vehicle path (yellow line)\n- LiDAR point cloud (colored by height)\n- IMU acceleration/velocity vectors\n- Ground grid reference"
                                draw_text: { color: #a0a0a0, text_style: { font_size: 13.0 }, wrap: Word }
                            }

                            <View> { width: Fill, height: Fill }

                            <Label> {
                                text: "Keyboard shortcuts:\nSpace - Toggle simulation"
                                draw_text: { color: #505050, text_style: { font_size: 11.0 } }
                            }
                        }
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] rerun_bridge: Option<RerunBridge>,
    #[rust] simulator: Option<SensorSimulator>,
    #[rust] simulation_running: bool,
    #[rust] frame_count: u64,
    #[rust] last_fps_time: f64,
    #[rust] fps: f64,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        mviz_widgets::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        debug_log("=== MViz App Started ===");
        self.rerun_bridge = None;
        self.simulator = Some(SensorSimulator::new());
        self.simulation_running = false;
        self.frame_count = 0;
        self.last_fps_time = 0.0;
        self.fps = 0.0;
        // Request first frame
        cx.start_interval(0.02); // 50 Hz update rate
        debug_log("Timer started at 50Hz");
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Launch Rerun button
        if self.ui.button(id!(launch_btn)).clicked(actions) {
            self.launch_rerun(cx);
        }

        // Simulation toggle button
        if self.ui.button(id!(sim_toggle)).clicked(actions) {
            self.toggle_simulation(cx);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Handle startup/shutdown via MatchEvent
        self.match_event(cx, event);

        // Capture actions emitted during UI event handling
        let actions = cx.capture_actions(|cx| {
            self.ui.handle_event(cx, event, &mut Scope::empty());
        });

        // Handle widget actions (button clicks, etc.)
        if !actions.is_empty() {
            self.handle_actions(cx, &actions);
        }

        // Handle keyboard input
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::Space {
                self.toggle_simulation(cx);
            }
        }

        // Handle timer for simulation updates
        if let Event::Timer(_te) = event {
            if self.simulation_running {
                self.simulation_step(cx);
                self.ui.redraw(cx);
            }
        }
    }
}

impl App {
    fn launch_rerun(&mut self, cx: &mut Cx) {
        debug_log("Launching Rerun viewer...");
        let config = RerunConfig::new("mviz_sensors").with_spawn(true);
        let mut bridge = RerunBridge::new(config);

        match bridge.spawn_viewer() {
            Ok(()) => {
                debug_log("Rerun viewer spawned successfully!");
                // Log ground grid
                match bridge.log_ground_grid() {
                    Ok(()) => debug_log("Ground grid logged"),
                    Err(e) => debug_log(&format!("Failed to log grid: {}", e)),
                }

                self.rerun_bridge = Some(bridge);
                self.ui.label(id!(status_label)).set_text(cx, "Status: Rerun Connected");
                self.ui.button(id!(launch_btn)).set_text(cx, "Rerun Connected");
            }
            Err(e) => {
                debug_log(&format!("Failed to spawn Rerun: {}", e));
                self.ui.label(id!(status_label)).set_text(cx, &format!("Error: {}", e));
            }
        }
        self.ui.redraw(cx);
    }

    fn toggle_simulation(&mut self, cx: &mut Cx) {
        self.simulation_running = !self.simulation_running;
        debug_log(&format!("Simulation toggled: running={}", self.simulation_running));

        if self.simulation_running {
            self.ui.button(id!(sim_toggle)).set_text(cx, "Stop Simulation");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Simulation Running");
        } else {
            self.ui.button(id!(sim_toggle)).set_text(cx, "Start Simulation");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Simulation Paused");
        }
        self.ui.redraw(cx);
    }

    fn simulation_step(&mut self, cx: &mut Cx) {
        let Some(simulator) = &mut self.simulator else { return };

        // Advance simulation
        simulator.step();
        self.frame_count += 1;

        // Generate sensor data
        let imu_data = simulator.generate_imu();
        let pose_data = simulator.generate_pose();

        // Generate LiDAR less frequently (every 5 frames)
        let lidar_data = if self.frame_count % 5 == 0 {
            Some(simulator.generate_lidar())
        } else {
            None
        };

        let path = simulator.path_history().to_vec();
        let sim_time = simulator.time();

        // Update FPS counter
        if sim_time - self.last_fps_time >= 1.0 {
            self.fps = self.frame_count as f64 / sim_time;
            self.last_fps_time = sim_time;
        }

        // Update UI labels
        self.ui.label(id!(imu_accel)).set_text(cx,
            &format!("Accel: {:.2}, {:.2}, {:.2}",
                imu_data.linear_acceleration[0],
                imu_data.linear_acceleration[1],
                imu_data.linear_acceleration[2]));

        self.ui.label(id!(imu_gyro)).set_text(cx,
            &format!("Gyro: {:.2}, {:.2}, {:.2}",
                imu_data.angular_velocity[0],
                imu_data.angular_velocity[1],
                imu_data.angular_velocity[2]));

        self.ui.label(id!(vehicle_pos)).set_text(cx,
            &format!("Position: {:.2}, {:.2}",
                pose_data.position[0],
                pose_data.position[1]));

        let speed = (pose_data.velocity[0].powi(2) + pose_data.velocity[1].powi(2)).sqrt();
        self.ui.label(id!(vehicle_speed)).set_text(cx,
            &format!("Speed: {:.2} m/s", speed));

        let heading_deg = pose_data.orientation[2].atan2(pose_data.orientation[3]) * 2.0 * 180.0 / std::f32::consts::PI;
        self.ui.label(id!(vehicle_heading)).set_text(cx,
            &format!("Heading: {:.1} deg", heading_deg));

        if let Some(ref lidar) = lidar_data {
            self.ui.label(id!(lidar_points)).set_text(cx,
                &format!("Points: {}", lidar.points.len()));
        }

        self.ui.label(id!(sim_time)).set_text(cx,
            &format!("Time: {:.2}s", sim_time));

        self.ui.label(id!(sim_fps)).set_text(cx,
            &format!("FPS: {:.0}", self.fps));

        // Log to Rerun if connected
        if let Some(bridge) = &self.rerun_bridge {
            if self.frame_count % 50 == 1 {
                debug_log(&format!("Logging frame {} to Rerun, time={:.2}s", self.frame_count, sim_time));
            }

            // Log IMU
            if let Err(e) = bridge.log_imu(&imu_data) {
                debug_log(&format!("IMU log error: {}", e));
            }

            // Log vehicle pose
            if let Err(e) = bridge.log_vehicle_pose(&pose_data) {
                debug_log(&format!("Pose log error: {}", e));
            }

            // Log path
            if let Err(e) = bridge.log_path(&path, sim_time) {
                debug_log(&format!("Path log error: {}", e));
            }

            // Log LiDAR (less frequently)
            if let Some(lidar) = lidar_data {
                if let Err(e) = bridge.log_lidar(&lidar) {
                    debug_log(&format!("LiDAR log error: {}", e));
                }
            }
        } else if self.frame_count % 50 == 1 {
            debug_log("Simulation running but Rerun not connected");
        }
    }
}
