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
    use mviz_widgets::displays_panel::DisplaysPanel;
    use mviz_widgets::properties_panel::PropertiesPanel;
    use mviz_widgets::toolbar::Toolbar;

    // App icon
    MVIZ_ICON = dep("crate://self/resources/icons/viz.svg")

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { title: "MViz - Robotics Visualizer", inner_size: vec2(1400, 850) }
                pass: { clear_color: #1a1a1a }

                body = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    show_bg: true
                    draw_bg: { color: #1a1a1a }

                    // ========================================================
                    // TOOLBAR
                    // ========================================================
                    toolbar = <View> {
                        width: Fill, height: 44
                        flow: Right
                        spacing: 8
                        padding: {left: 12, right: 12, top: 4, bottom: 4}
                        align: {y: 0.5}
                        show_bg: true
                        draw_bg: { color: #252525 }

                        // App icon and title
                        <Icon> {
                            draw_icon: {
                                svg_file: (MVIZ_ICON)
                                fn get_color(self) -> vec4 { return #3b82f6; }
                            }
                            icon_walk: {width: 24, height: 24}
                        }

                        <Label> {
                            text: "MViz"
                            draw_text: {
                                color: #ffffff
                                text_style: { font_size: 16.0 }
                            }
                        }

                        <View> { width: 20, height: 1 }

                        // File menu
                        file_btn = <Button> {
                            text: "File"
                            draw_text: { color: #fff }
                        }

                        view_btn = <Button> {
                            text: "View"
                            draw_text: { color: #fff }
                        }

                        <View> { width: Fill, height: 1 }

                        // Frame selector
                        <Label> {
                            text: "Fixed Frame:"
                            draw_text: { color: #888, text_style: { font_size: 11.0 } }
                        }

                        frame_dropdown = <DropDown> {
                            width: 100, height: 26
                        }

                        <View> { width: 20, height: 1 }

                        // Playback controls
                        play_btn = <Button> {
                            text: "Play"
                            draw_text: { color: #fff }
                        }

                        time_label = <Label> {
                            text: "0.00s"
                            draw_text: { color: #aaa, text_style: { font_size: 11.0 } }
                        }

                        <View> { width: 20, height: 1 }

                        // Rerun launch
                        launch_btn = <Button> {
                            text: "Launch Rerun"
                            draw_text: { color: #fff }
                        }
                    }

                    // ========================================================
                    // MAIN CONTENT AREA
                    // ========================================================
                    content = <View> {
                        width: Fill, height: Fill
                        flow: Right
                        padding: 8
                        spacing: 8

                        // LEFT PANEL - Displays
                        left_panel = <View> {
                            width: 280, height: Fill
                            flow: Down
                            spacing: 8

                            // Displays list
                            displays_panel = <DisplaysPanel> {
                                width: Fill, height: 300
                            }

                            // Sensor data panels (from original app)
                            imu_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 12
                                spacing: 6
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "IMU Sensor"
                                    draw_text: { color: #ffffff, text_style: { font_size: 12.0 } }
                                }

                                imu_accel = <Label> {
                                    text: "Accel: 0.00, 0.00, 9.81"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 10.0 } }
                                }

                                imu_gyro = <Label> {
                                    text: "Gyro: 0.00, 0.00, 0.00"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 10.0 } }
                                }
                            }

                            // Vehicle state
                            vehicle_panel = <RoundedView> {
                                width: Fill, height: Fit
                                flow: Down
                                padding: 12
                                spacing: 6
                                show_bg: true
                                draw_bg: { color: #252525, border_radius: 8.0 }

                                <Label> {
                                    text: "Vehicle State"
                                    draw_text: { color: #ffffff, text_style: { font_size: 12.0 } }
                                }

                                vehicle_pos = <Label> {
                                    text: "Position: 0.00, 0.00"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 10.0 } }
                                }

                                vehicle_speed = <Label> {
                                    text: "Speed: 0.00 m/s"
                                    draw_text: { color: #a0a0a0, text_style: { font_size: 10.0 } }
                                }
                            }

                            <View> { width: Fill, height: Fill }

                            // Status
                            status_label = <Label> {
                                text: "Status: Ready"
                                draw_text: { color: #606060, text_style: { font_size: 10.0 } }
                            }
                        }

                        // CENTER - Rerun Viewer Info
                        center_panel = <RoundedView> {
                            width: Fill, height: Fill
                            flow: Down
                            padding: 24
                            spacing: 16
                            show_bg: true
                            draw_bg: { color: #1e1e1e, border_radius: 8.0 }

                            <Label> {
                                text: "3D Visualization (Rerun)"
                                draw_text: { color: #ffffff, text_style: { font_size: 16.0 } }
                            }

                            <Label> {
                                width: Fill
                                text: "The 3D visualization is displayed in the Rerun Viewer window.\n\nClick 'Launch Rerun' to open the viewer, then 'Play' to start the simulation.\n\nThe viewer will display:\n- Vehicle body and path\n- LiDAR point cloud\n- IMU vectors\n- Coordinate frames (TF)"
                                draw_text: { color: #a0a0a0, text_style: { font_size: 12.0 }, wrap: Word }
                            }

                            <View> { width: Fill, height: Fill }

                            // Stats
                            stats_panel = <View> {
                                width: Fill, height: Fit
                                flow: Down
                                spacing: 4

                                sim_time = <Label> {
                                    text: "Simulation Time: 0.00s"
                                    draw_text: { color: #707070, text_style: { font_size: 11.0 } }
                                }

                                sim_fps = <Label> {
                                    text: "Update Rate: 0 Hz"
                                    draw_text: { color: #707070, text_style: { font_size: 11.0 } }
                                }

                                lidar_points = <Label> {
                                    text: "LiDAR Points: 0"
                                    draw_text: { color: #707070, text_style: { font_size: 11.0 } }
                                }
                            }
                        }

                        // RIGHT PANEL - Properties
                        right_panel = <View> {
                            width: 280, height: Fill
                            flow: Down
                            spacing: 8

                            properties_panel = <PropertiesPanel> {
                                width: Fill, height: Fill
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

        // Play/Pause button
        if self.ui.button(id!(play_btn)).clicked(actions) {
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
            self.ui.button(id!(play_btn)).set_text(cx, "Pause");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Simulation Running");
        } else {
            self.ui.button(id!(play_btn)).set_text(cx, "Play");
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

        if let Some(ref lidar) = lidar_data {
            self.ui.label(id!(lidar_points)).set_text(cx,
                &format!("LiDAR Points: {}", lidar.points.len()));
        }

        self.ui.label(id!(sim_time)).set_text(cx,
            &format!("Simulation Time: {:.2}s", sim_time));

        self.ui.label(id!(sim_fps)).set_text(cx,
            &format!("Update Rate: {:.0} Hz", self.fps));

        self.ui.label(id!(time_label)).set_text(cx,
            &format!("{:.2}s", sim_time));

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
