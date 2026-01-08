//! MViz App - Main application shell
//!
//! Combines Makepad UI for controls with Rerun for 3D visualization.

use makepad_widgets::*;
use mviz_rerun_bridge::{RerunBridge, RerunConfig, SensorSimulator};
use mviz_displays::laser_scan::simulate_laser_scan;
use mviz_widgets::DisplaysPanelAction;
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

                        <View> { width: 20, height: 1 }

                        // Test buttons for Phase 4
                        test_laser_btn = <Button> {
                            text: "Test Laser"
                            draw_text: { color: #4ade80 }
                        }

                        test_robot_btn = <Button> {
                            text: "Test Robot"
                            draw_text: { color: #60a5fa }
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

        // Test Laser button
        if self.ui.button(id!(test_laser_btn)).clicked(actions) {
            self.test_laser_scan(cx);
        }

        // Test Robot button
        if self.ui.button(id!(test_robot_btn)).clicked(actions) {
            self.test_robot_model(cx);
        }

        // Handle DisplaysPanel actions
        for action in actions {
            if let DisplaysPanelAction::AddDisplayClicked = action.as_widget_action().cast() {
                self.add_display(cx);
            }
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

    fn test_laser_scan(&mut self, cx: &mut Cx) {
        debug_log("Testing LaserScan display...");

        // Check if Rerun is connected
        let Some(bridge) = &self.rerun_bridge else {
            self.ui.label(id!(status_label)).set_text(cx, "Error: Launch Rerun first!");
            self.ui.redraw(cx);
            return;
        };

        // Create simulated laser scan (360 rays, 10m max range)
        let scan = simulate_laser_scan(360, 10.0);
        debug_log(&format!("Generated laser scan: {} rays", scan.len()));

        // Convert to point cloud and log directly to Rerun
        let cloud = scan.to_point_cloud();
        debug_log(&format!("Converted to point cloud: {} points", cloud.len()));

        // Log the point cloud using the bridge's recording stream
        let positions: Vec<[f32; 3]> = cloud.positions.iter()
            .map(|p| [p.x, p.y, p.z])
            .collect();

        let result = bridge.log(
            "laser_scan/points",
            &rerun::Points3D::new(positions)
                .with_radii([0.02f32])
                .with_colors([rerun::Color::from_rgb(0, 255, 128)]),
        );

        match result {
            Ok(()) => {
                debug_log("LaserScan logged successfully!");
                let sim_status = if self.simulation_running { " (Sim running)" } else { " (Click Play for sim)" };
                self.ui.label(id!(status_label)).set_text(cx, &format!("LaserScan: 360 points logged{}", sim_status));
                self.ui.label(id!(lidar_points)).set_text(cx, &format!("Laser Points: {}", cloud.len()));
            }
            Err(e) => {
                debug_log(&format!("LaserScan log error: {}", e));
                self.ui.label(id!(status_label)).set_text(cx, &format!("Error: {}", e));
            }
        }
        self.ui.redraw(cx);
    }

    fn add_display(&mut self, cx: &mut Cx) {
        debug_log("Add Display clicked!");

        // Cycle through display types for demo
        static DISPLAY_TYPES: &[&str] = &["Grid", "Axes", "PointCloud", "LaserScan", "TF"];
        let display_index = self.frame_count as usize % DISPLAY_TYPES.len();
        let display_type = DISPLAY_TYPES[display_index];

        if let Some(bridge) = &self.rerun_bridge {
            let result = match display_type {
                "Grid" => {
                    bridge.log_ground_grid()
                }
                "Axes" => {
                    // Log coordinate axes
                    bridge.log(
                        "world/axes",
                        &rerun::Arrows3D::from_vectors([
                            [1.0, 0.0, 0.0],  // X - Red
                            [0.0, 1.0, 0.0],  // Y - Green
                            [0.0, 0.0, 1.0],  // Z - Blue
                        ])
                        .with_origins([[0.0, 0.0, 0.0]; 3])
                        .with_colors([
                            rerun::Color::from_rgb(255, 0, 0),
                            rerun::Color::from_rgb(0, 255, 0),
                            rerun::Color::from_rgb(0, 0, 255),
                        ]),
                    )
                }
                "PointCloud" => {
                    // Log sample point cloud
                    let points: Vec<[f32; 3]> = (0..100)
                        .map(|i| {
                            let angle = i as f32 * 0.1;
                            [angle.cos() * 2.0, angle.sin() * 2.0, (i as f32) * 0.05]
                        })
                        .collect();
                    bridge.log(
                        "world/sample_pointcloud",
                        &rerun::Points3D::new(points)
                            .with_radii([0.05])
                            .with_colors([rerun::Color::from_rgb(0, 200, 255)]),
                    )
                }
                "LaserScan" => {
                    // Log 360-degree laser scan
                    let scan = simulate_laser_scan(360, 8.0);
                    let cloud = scan.to_point_cloud();
                    let positions: Vec<[f32; 3]> = cloud.positions.iter()
                        .map(|p| [p.x, p.y, p.z])
                        .collect();
                    bridge.log(
                        "world/laser_scan",
                        &rerun::Points3D::new(positions)
                            .with_radii([0.03])
                            .with_colors([rerun::Color::from_rgb(0, 255, 128)]),
                    )
                }
                "TF" => {
                    // Log TF coordinate frames
                    let _ = bridge.log(
                        "world/tf/base_link",
                        &rerun::Transform3D::from_translation([0.0, 0.0, 0.0]),
                    );
                    bridge.log(
                        "world/tf/sensor_link",
                        &rerun::Transform3D::from_translation([0.5, 0.0, 0.3]),
                    )
                }
                _ => Ok(()),
            };

            match result {
                Ok(()) => {
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Added: {} display", display_type));
                    debug_log(&format!("Added {} display successfully", display_type));
                }
                Err(e) => {
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Error adding {}: {}", display_type, e));
                    debug_log(&format!("Error adding {} display: {}", display_type, e));
                }
            }

            // Increment frame_count to cycle to next type on next click
            self.frame_count += 1;
        } else {
            self.ui.label(id!(status_label)).set_text(cx, "Launch Rerun first, then Add Display");
            debug_log("Add Display: Rerun not connected");
        }
        self.ui.redraw(cx);
    }

    fn test_robot_model(&mut self, cx: &mut Cx) {
        debug_log("Testing RobotModel display with URDF loader...");

        // Check if Rerun is connected
        let Some(bridge) = &self.rerun_bridge else {
            self.ui.label(id!(status_label)).set_text(cx, "Error: Launch Rerun first!");
            self.ui.redraw(cx);
            return;
        };

        // Use Rerun's built-in URDF data loader for proper mesh rendering
        // The so100.urdf file contains the SO-ARM100 robot arm with STL meshes
        let urdf_path = std::path::Path::new("/Users/nupylot/Public/mviz/so100.urdf");

        if !urdf_path.exists() {
            debug_log(&format!("URDF file not found: {:?}", urdf_path));
            self.ui.label(id!(status_label)).set_text(cx, "Error: so100.urdf not found!");
            self.ui.redraw(cx);
            return;
        }

        // Get the recording stream and use log_file_from_path
        if let Some(stream) = bridge.stream() {
            debug_log(&format!("Loading URDF from: {:?}", urdf_path));

            // Load URDF with Rerun's built-in loader
            // entity_path_prefix puts robot under "world/robot" for visibility
            // static=true means the robot structure doesn't change over time
            match stream.log_file_from_path(
                urdf_path,
                Some("world/robot".into()),  // entity_path_prefix
                true,  // static
            ) {
                Ok(()) => {
                    debug_log("URDF loaded successfully via Rerun data loader!");
                    let sim_status = if self.simulation_running { " (Sim running)" } else { " (Click Play for sim)" };
                    self.ui.label(id!(status_label)).set_text(cx, &format!("SO-ARM100 loaded from URDF{}", sim_status));
                }
                Err(e) => {
                    debug_log(&format!("Failed to load URDF: {}", e));
                    self.ui.label(id!(status_label)).set_text(cx, &format!("URDF load error: {}", e));
                }
            }
        } else {
            debug_log("No recording stream available");
            self.ui.label(id!(status_label)).set_text(cx, "Error: No recording stream");
        }

        self.ui.redraw(cx);
    }

    /// Render PR2 robot structure with simplified geometry
    fn render_pr2_robot(&self, bridge: &RerunBridge) {
        // Box indices for cube meshes
        let box_indices: Vec<[u32; 3]> = vec![
            [0,1,2], [0,2,3], // bottom
            [4,6,5], [4,7,6], // top
            [0,4,5], [0,5,1], // front
            [2,6,7], [2,7,3], // back
            [0,3,7], [0,7,4], // left
            [1,5,6], [1,6,2], // right
        ];

        // PR2 Base (large box at ground level)
        let base_positions: Vec<[f32; 3]> = vec![
            [-0.35, -0.35, 0.0], [0.35, -0.35, 0.0], [0.35, 0.35, 0.0], [-0.35, 0.35, 0.0],
            [-0.35, -0.35, 0.3], [0.35, -0.35, 0.3], [0.35, 0.35, 0.3], [-0.35, 0.35, 0.3],
        ];
        let _ = bridge.log(
            "robot/base_link",
            &rerun::Mesh3D::new(base_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(100, 100, 100, 255)),
        );

        // PR2 Torso (vertical column)
        let torso_positions: Vec<[f32; 3]> = vec![
            [-0.15, -0.15, 0.3], [0.15, -0.15, 0.3], [0.15, 0.15, 0.3], [-0.15, 0.15, 0.3],
            [-0.15, -0.15, 1.2], [0.15, -0.15, 1.2], [0.15, 0.15, 1.2], [-0.15, 0.15, 1.2],
        ];
        let _ = bridge.log(
            "robot/torso_lift_link",
            &rerun::Mesh3D::new(torso_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(80, 80, 80, 255)),
        );

        // PR2 Head (box on top of torso)
        let head_positions: Vec<[f32; 3]> = vec![
            [-0.12, -0.12, 1.2], [0.12, -0.12, 1.2], [0.12, 0.12, 1.2], [-0.12, 0.12, 1.2],
            [-0.12, -0.12, 1.45], [0.12, -0.12, 1.45], [0.12, 0.12, 1.45], [-0.12, 0.12, 1.45],
        ];
        let _ = bridge.log(
            "robot/head_pan_link",
            &rerun::Mesh3D::new(head_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(60, 60, 60, 255)),
        );

        // PR2 Left Shoulder
        let l_shoulder_positions: Vec<[f32; 3]> = vec![
            [0.15, 0.15, 0.9], [0.25, 0.15, 0.9], [0.25, 0.45, 0.9], [0.15, 0.45, 0.9],
            [0.15, 0.15, 1.05], [0.25, 0.15, 1.05], [0.25, 0.45, 1.05], [0.15, 0.45, 1.05],
        ];
        let _ = bridge.log(
            "robot/l_shoulder_pan_link",
            &rerun::Mesh3D::new(l_shoulder_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(51, 102, 204, 255)),
        );

        // PR2 Left Upper Arm
        let l_upper_arm_positions: Vec<[f32; 3]> = vec![
            [0.15, 0.45, 0.85], [0.25, 0.45, 0.85], [0.25, 0.85, 0.85], [0.15, 0.85, 0.85],
            [0.15, 0.45, 1.0], [0.25, 0.45, 1.0], [0.25, 0.85, 1.0], [0.15, 0.85, 1.0],
        ];
        let _ = bridge.log(
            "robot/l_upper_arm_link",
            &rerun::Mesh3D::new(l_upper_arm_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(70, 130, 220, 255)),
        );

        // PR2 Left Forearm
        let l_forearm_positions: Vec<[f32; 3]> = vec![
            [0.15, 0.85, 0.70], [0.25, 0.85, 0.70], [0.25, 1.15, 0.70], [0.15, 1.15, 0.70],
            [0.15, 0.85, 0.85], [0.25, 0.85, 0.85], [0.25, 1.15, 0.85], [0.15, 1.15, 0.85],
        ];
        let _ = bridge.log(
            "robot/l_forearm_link",
            &rerun::Mesh3D::new(l_forearm_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(90, 150, 230, 255)),
        );

        // PR2 Left Gripper
        let l_gripper_positions: Vec<[f32; 3]> = vec![
            [0.17, 1.15, 0.72], [0.23, 1.15, 0.72], [0.23, 1.30, 0.72], [0.17, 1.30, 0.72],
            [0.17, 1.15, 0.83], [0.23, 1.15, 0.83], [0.23, 1.30, 0.83], [0.17, 1.30, 0.83],
        ];
        let _ = bridge.log(
            "robot/l_gripper_palm_link",
            &rerun::Mesh3D::new(l_gripper_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(200, 50, 50, 255)),
        );

        // PR2 Right Shoulder
        let r_shoulder_positions: Vec<[f32; 3]> = vec![
            [0.15, -0.45, 0.9], [0.25, -0.45, 0.9], [0.25, -0.15, 0.9], [0.15, -0.15, 0.9],
            [0.15, -0.45, 1.05], [0.25, -0.45, 1.05], [0.25, -0.15, 1.05], [0.15, -0.15, 1.05],
        ];
        let _ = bridge.log(
            "robot/r_shoulder_pan_link",
            &rerun::Mesh3D::new(r_shoulder_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(51, 102, 204, 255)),
        );

        // PR2 Right Upper Arm
        let r_upper_arm_positions: Vec<[f32; 3]> = vec![
            [0.15, -0.85, 0.85], [0.25, -0.85, 0.85], [0.25, -0.45, 0.85], [0.15, -0.45, 0.85],
            [0.15, -0.85, 1.0], [0.25, -0.85, 1.0], [0.25, -0.45, 1.0], [0.15, -0.45, 1.0],
        ];
        let _ = bridge.log(
            "robot/r_upper_arm_link",
            &rerun::Mesh3D::new(r_upper_arm_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(70, 130, 220, 255)),
        );

        // PR2 Right Forearm
        let r_forearm_positions: Vec<[f32; 3]> = vec![
            [0.15, -1.15, 0.70], [0.25, -1.15, 0.70], [0.25, -0.85, 0.70], [0.15, -0.85, 0.70],
            [0.15, -1.15, 0.85], [0.25, -1.15, 0.85], [0.25, -0.85, 0.85], [0.15, -0.85, 0.85],
        ];
        let _ = bridge.log(
            "robot/r_forearm_link",
            &rerun::Mesh3D::new(r_forearm_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(90, 150, 230, 255)),
        );

        // PR2 Right Gripper
        let r_gripper_positions: Vec<[f32; 3]> = vec![
            [0.17, -1.30, 0.72], [0.23, -1.30, 0.72], [0.23, -1.15, 0.72], [0.17, -1.15, 0.72],
            [0.17, -1.30, 0.83], [0.23, -1.30, 0.83], [0.23, -1.15, 0.83], [0.17, -1.15, 0.83],
        ];
        let _ = bridge.log(
            "robot/r_gripper_palm_link",
            &rerun::Mesh3D::new(r_gripper_positions)
                .with_triangle_indices(box_indices.clone())
                .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(200, 50, 50, 255)),
        );

        debug_log("PR2 robot structure rendered to Rerun");
    }

    /// Render Husky + UR5 robot using Boxes3D for reliable visibility
    fn render_husky_ur5_robot(&self, bridge: &RerunBridge) {
        debug_log("Starting Husky+UR5 rendering with Boxes3D...");

        // Use Boxes3D instead of Mesh3D for more reliable rendering
        // Boxes3D takes centers and half-sizes
        // IMPORTANT: Use "world/robot/*" paths to appear in same 3D space as ground grid

        // Husky Base - dark gray platform
        if let Err(e) = bridge.log(
            "world/robot/husky_base",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.0_f32, 0.0, 0.125]],  // center
                [[0.50, 0.30, 0.125]],     // half-sizes (1.0 x 0.6 x 0.25)
            ).with_colors([rerun::Color::from_rgb(80, 80, 80)]),
        ) {
            debug_log(&format!("Failed to log husky_base: {}", e));
        }

        // Husky Wheels - 4 black wheels
        let wheel_centers = [
            [0.35_f32, 0.35, 0.05],   // front-left
            [0.35, -0.35, 0.05],      // front-right
            [-0.35, 0.35, 0.05],      // rear-left
            [-0.35, -0.35, 0.05],     // rear-right
        ];
        let wheel_half_sizes = [[0.08_f32, 0.05, 0.10]; 4];
        if let Err(e) = bridge.log(
            "world/robot/wheels",
            &rerun::Boxes3D::from_centers_and_half_sizes(wheel_centers, wheel_half_sizes)
                .with_colors([rerun::Color::from_rgb(30, 30, 30)]),
        ) {
            debug_log(&format!("Failed to log wheels: {}", e));
        }

        // UR5 Arm Base - blue base on top of Husky
        if let Err(e) = bridge.log(
            "world/robot/ur5_base",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.0_f32, 0.0, 0.35]],
                [[0.08, 0.08, 0.10]],
            ).with_colors([rerun::Color::from_rgb(50, 80, 180)]),
        ) {
            debug_log(&format!("Failed to log ur5_base: {}", e));
        }

        // UR5 Shoulder - vertical link
        if let Err(e) = bridge.log(
            "world/robot/ur5_shoulder",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.0_f32, 0.0, 0.55]],
                [[0.06, 0.06, 0.10]],
            ).with_colors([rerun::Color::from_rgb(70, 100, 200)]),
        ) {
            debug_log(&format!("Failed to log ur5_shoulder: {}", e));
        }

        // UR5 Upper Arm - horizontal link
        if let Err(e) = bridge.log(
            "world/robot/ur5_upper_arm",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.20_f32, 0.0, 0.65]],
                [[0.20, 0.05, 0.05]],
            ).with_colors([rerun::Color::from_rgb(90, 120, 220)]),
        ) {
            debug_log(&format!("Failed to log ur5_upper_arm: {}", e));
        }

        // UR5 Forearm
        if let Err(e) = bridge.log(
            "world/robot/ur5_forearm",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.50_f32, 0.0, 0.60]],
                [[0.15, 0.04, 0.04]],
            ).with_colors([rerun::Color::from_rgb(110, 140, 230)]),
        ) {
            debug_log(&format!("Failed to log ur5_forearm: {}", e));
        }

        // UR5 Wrist
        if let Err(e) = bridge.log(
            "world/robot/ur5_wrist",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.70_f32, 0.0, 0.55]],
                [[0.05, 0.03, 0.05]],
            ).with_colors([rerun::Color::from_rgb(130, 160, 240)]),
        ) {
            debug_log(&format!("Failed to log ur5_wrist: {}", e));
        }

        // UR5 Gripper - red end effector
        if let Err(e) = bridge.log(
            "world/robot/ur5_gripper",
            &rerun::Boxes3D::from_centers_and_half_sizes(
                [[0.82_f32, 0.0, 0.55]],
                [[0.06, 0.04, 0.03]],
            ).with_colors([rerun::Color::from_rgb(220, 60, 60)]),
        ) {
            debug_log(&format!("Failed to log ur5_gripper: {}", e));
        }

        // Add a bright yellow marker point at origin for visibility
        if let Err(e) = bridge.log(
            "world/robot/marker",
            &rerun::Points3D::new([[0.0_f32, 0.0, 0.8]])
                .with_radii([0.15])
                .with_colors([rerun::Color::from_rgb(255, 255, 0)]),
        ) {
            debug_log(&format!("Failed to log marker: {}", e));
        }

        debug_log("Husky+UR5 robot rendered with Boxes3D under world/robot/*");
    }
}
