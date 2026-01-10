//! MViz App - Main application shell
//!
//! Combines Makepad UI for controls with Rerun for 3D visualization.
//! Supports both standalone simulation and Dora dataflow integration.

use makepad_widgets::*;
use mviz_rerun_bridge::{RerunBridge, RerunConfig, SensorSimulator};
use mviz_displays::laser_scan::simulate_laser_scan;
use mviz_widgets::{DisplaysPanelAction, DisplayType, DisplaysPanelWidgetRefExt, PropertiesPanelWidgetRefExt, LogPanelAction, LogDisplayEntry, LogPanelWidgetRefExt, NodeDetailPanelAction, NodeDetailPanelWidgetRefExt, NodeInput, NodeOutput, DataflowGraphAction};
use mviz_widgets::dataflow_graph::DataflowGraphWidgetWidgetRefExt;
use crate::dora_receiver::{DoraReceiver, DoraMessage, DoraData};
use crate::zenoh_receiver::{ZenohReceiver, ZenohMessage, VisData, parse_points_xyz_f32};
use mviz_core::zenoh_protocol::{Points3DData, Boxes3DData, Arrows3DData, LineStrips3DData, Transform3DData, ScalarData, LogLevel, binary_formats};
use std::io::Write;
use std::collections::HashSet;

fn debug_log(msg: &str) {
    // Print to stderr for terminal visibility
    eprintln!("[mviz] {}", msg);

    // Also write to log file
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
    use mviz_widgets::log_panel::LogPanel;
    use mviz_widgets::node_detail_panel::NodeDetailPanel;
    use mviz_widgets::dataflow_graph::DataflowGraphWidget;

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

                        // Rerun launch (standalone mode only)
                        launch_btn = <Button> {
                            text: "Spawn Rerun"
                            draw_text: { color: #fff }
                        }

                        <View> { width: 10, height: 1 }

                        // Zenoh connection for LAN data
                        zenoh_btn = <Button> {
                            text: "Connect Zenoh"
                            draw_text: { color: #fbbf24 }
                        }

                        zenoh_status = <Label> {
                            text: ""
                            draw_text: { color: #22c55e, text_style: { font_size: 10.0 } }
                        }

                        <View> { width: 10, height: 1 }

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
                    // MAIN CONTENT AREA - Resizable Splitter Layout
                    // ========================================================
                    content = <Splitter> {
                        width: Fill, height: Fill
                        axis: Horizontal
                        align: FromA(280.0)

                        // LEFT PANEL - Displays
                        a = <View> {
                            width: Fill, height: Fill
                            flow: Down
                            spacing: 8
                            padding: 8

                            // Displays list
                            displays_panel = <DisplaysPanel> {
                                width: Fill, height: 300
                            }

                            // Dataflow Graph (replaces IMU/Vehicle/Stats panels)
                            dataflow_graph = <DataflowGraphWidget> {
                                width: Fill, height: Fill
                            }

                            // Status
                            status_label = <Label> {
                                text: "Status: Ready"
                                draw_text: { color: #606060, text_style: { font_size: 10.0 } }
                            }
                        }

                        // CENTER + RIGHT - Nested Splitter
                        b = <Splitter> {
                            width: Fill, height: Fill
                            axis: Horizontal
                            align: FromB(280.0)

                            // CENTER - Node Detail Panel
                            a = <View> {
                                width: Fill, height: Fill
                                padding: 8

                                node_detail_panel = <NodeDetailPanel> {
                                    width: Fill, height: Fill
                                }
                            }

                            // RIGHT PANEL - Properties + System Log
                            b = <View> {
                                width: Fill, height: Fill
                                flow: Down
                                spacing: 8
                                padding: 8

                                properties_panel = <PropertiesPanel> {
                                    width: Fill, height: 200
                                }

                                // System Log Panel
                                log_panel = <LogPanel> {
                                    width: Fill, height: Fill
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

app_main!(App);

/// Data source mode for the application
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DataSource {
    /// Use built-in simulator
    #[default]
    Simulator,
    /// Use Dora dataflow for live sensor data (legacy)
    Dora,
    /// Use Zenoh for LAN communication (robot -> PC)
    Zenoh,
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] rerun_bridge: Option<RerunBridge>,
    #[rust] simulator: Option<SensorSimulator>,
    #[rust] simulation_running: bool,
    #[rust] frame_count: u64,
    #[rust] last_fps_time: f64,
    #[rust] fps: f64,
    // Dora integration (legacy)
    #[rust] dora_receiver: Option<DoraReceiver>,
    #[rust] dora_data: DoraData,
    #[rust] data_source: DataSource,
    #[rust] dora_connected: bool,
    // Zenoh integration (LAN communication - universal receiver)
    #[rust] zenoh_receiver: Option<ZenohReceiver>,
    #[rust] zenoh_connected: bool,
    #[rust] zenoh_message_count: u64,
    // System log state
    #[rust] discovered_nodes: HashSet<String>,
    #[rust] log_entry_count: u64,
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
        // Dora integration (legacy)
        self.dora_receiver = None;
        self.dora_data = DoraData::default();
        self.data_source = DataSource::Simulator;
        self.dora_connected = false;
        // Zenoh integration (universal receiver)
        self.zenoh_receiver = None;
        self.zenoh_connected = false;
        self.zenoh_message_count = 0;
        // System log state
        self.discovered_nodes = HashSet::new();
        self.log_entry_count = 0;

        // Initialize Fixed Frame dropdown with common coordinate frames
        let frame_labels = vec![
            "world".to_string(),
            "map".to_string(),
            "odom".to_string(),
            "base_link".to_string(),
            "base_footprint".to_string(),
        ];
        self.ui.drop_down(id!(frame_dropdown)).set_labels(cx, frame_labels);

        // Request first frame
        cx.start_interval(0.02); // 50 Hz update rate
        debug_log("Timer started at 50Hz");
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // File menu button
        if self.ui.button(id!(file_btn)).clicked(actions) {
            self.ui.label(id!(status_label)).set_text(cx, "File menu: Open, Save, Export (coming soon)");
            debug_log("File menu clicked");
        }

        // View menu button
        if self.ui.button(id!(view_btn)).clicked(actions) {
            self.ui.label(id!(status_label)).set_text(cx, "View menu: Panels, Layout, Reset (coming soon)");
            debug_log("View menu clicked");
        }

        // Frame dropdown changed
        if let Some(index) = self.ui.drop_down(id!(frame_dropdown)).changed(actions) {
            let frames = ["world", "map", "odom", "base_link", "base_footprint"];
            let frame_name = frames.get(index).unwrap_or(&"world");
            self.ui.label(id!(status_label)).set_text(cx, &format!("Fixed Frame: {}", frame_name));
            debug_log(&format!("Fixed frame changed to: {}", frame_name));
        }

        // Spawn Rerun button (standalone mode)
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

        // Zenoh connection button
        if self.ui.button(id!(zenoh_btn)).clicked(actions) {
            self.toggle_zenoh_connection(cx);
        }

        // Handle DisplaysPanel actions
        for action in actions {
            match action.as_widget_action().cast::<DisplaysPanelAction>() {
                DisplaysPanelAction::AddDisplayClicked => {
                    self.add_display(cx);
                }
                DisplaysPanelAction::DisplaySelected(id) => {
                    self.on_display_selected(cx, id);
                }
                DisplaysPanelAction::DisplayToggled(id, enabled) => {
                    self.on_display_toggled(cx, id, enabled);
                }
                DisplaysPanelAction::DisplayDeleted(id) => {
                    self.on_display_deleted(cx, id);
                }
                _ => {}
            }
        }

        // Handle LogPanel actions
        for action in actions {
            match action.as_widget_action().cast::<LogPanelAction>() {
                LogPanelAction::CopyClicked => {
                    let log_text = self.ui.log_panel(id!(log_panel)).get_filtered_text();
                    cx.copy_to_clipboard(&log_text);
                    debug_log("Copied logs to clipboard");
                }
                LogPanelAction::ClearClicked => {
                    self.ui.log_panel(id!(log_panel)).clear(cx);
                    self.log_entry_count = 0;
                    debug_log("Cleared system log");
                }
                _ => {}
            }
        }

        // Handle NodeDetailPanel actions
        for action in actions {
            match action.as_widget_action().cast::<NodeDetailPanelAction>() {
                NodeDetailPanelAction::NodeSelected(node_id) => {
                    debug_log(&format!("Node selected: {}", node_id));
                }
                NodeDetailPanelAction::ClearLogsClicked => {
                    debug_log("Node logs cleared");
                }
                _ => {}
            }
        }

        // Handle DataflowGraphWidget actions
        for action in actions {
            match action.as_widget_action().cast::<DataflowGraphAction>() {
                DataflowGraphAction::NodeClicked(node_id) => {
                    debug_log(&format!("Graph node clicked: {}", node_id));
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Selected node: {}", node_id));
                }
                _ => {}
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
            // Process Dora messages if connected
            self.process_dora_messages(cx);
            // Process Zenoh messages if connected
            self.process_zenoh_messages(cx);

            // Run simulation or update based on data source
            match self.data_source {
                DataSource::Simulator => {
                    if self.simulation_running {
                        self.simulation_step(cx);
                        self.ui.redraw(cx);
                    }
                }
                DataSource::Dora => {
                    if self.dora_connected {
                        self.update_from_dora(cx);
                        self.ui.redraw(cx);
                    }
                }
                DataSource::Zenoh => {
                    if self.zenoh_connected {
                        self.update_from_zenoh(cx);
                        self.ui.redraw(cx);
                    }
                }
            }
        }
    }
}

impl App {
    fn launch_rerun(&mut self, cx: &mut Cx) {
        debug_log("Launching Rerun viewer via CLI...");

        // First spawn a native Rerun viewer using the CLI
        let spawn_result = std::process::Command::new("rerun")
            .spawn();

        match spawn_result {
            Ok(_child) => {
                debug_log("Rerun CLI started, waiting for server...");
                // Give the server time to start
                std::thread::sleep(std::time::Duration::from_millis(1500));

                // Now connect to it
                let config = RerunConfig::new("mviz_sensors");
                let mut bridge = RerunBridge::new(config);

                match bridge.connect("rerun+http://127.0.0.1:9876/proxy") {
                    Ok(()) => {
                        debug_log("Connected to Rerun server!");
                        match bridge.log_ground_grid() {
                            Ok(()) => debug_log("Ground grid logged"),
                            Err(e) => debug_log(&format!("Failed to log grid: {}", e)),
                        }
                        self.rerun_bridge = Some(bridge);
                        self.ui.label(id!(status_label)).set_text(cx, "Status: Rerun Connected (Web)");
                        self.ui.button(id!(launch_btn)).set_text(cx, "Rerun Running");
                    }
                    Err(e) => {
                        // Fallback to spawn method
                        debug_log(&format!("Connect failed, trying spawn: {}", e));
                        self.spawn_rerun_fallback(cx);
                    }
                }
            }
            Err(e) => {
                debug_log(&format!("Rerun CLI not found ({}), using SDK spawn", e));
                self.spawn_rerun_fallback(cx);
            }
        }
        self.ui.redraw(cx);
    }

    fn spawn_rerun_fallback(&mut self, cx: &mut Cx) {
        debug_log("Using Rerun SDK spawn fallback...");
        let config = RerunConfig::new("mviz_sensors").with_spawn(true);
        let mut bridge = RerunBridge::new(config);

        match bridge.spawn_viewer() {
            Ok(()) => {
                debug_log("Rerun viewer spawned successfully!");
                match bridge.log_ground_grid() {
                    Ok(()) => debug_log("Ground grid logged"),
                    Err(e) => debug_log(&format!("Failed to log grid: {}", e)),
                }
                self.rerun_bridge = Some(bridge);
                self.ui.label(id!(status_label)).set_text(cx, "Status: Rerun Spawned");
                self.ui.button(id!(launch_btn)).set_text(cx, "Rerun Spawned");
            }
            Err(e) => {
                let error_msg = format!("Failed to spawn Rerun viewer: {}", e);
                debug_log(&error_msg);
                debug_log("Hint: Make sure rerun-sdk is installed: pip install rerun-sdk");
                self.ui.label(id!(status_label)).set_text(cx, "Error: spawn failed (see terminal)");
            }
        }
    }

    fn connect_to_rerun(&mut self, cx: &mut Cx) {
        debug_log("Connecting to existing Rerun viewer (same recording as dora)...");

        // Use SAME recording ID as dora dataflow to share the view
        let config = RerunConfig::new("mviz_vehicle");
        let mut bridge = RerunBridge::new(config);

        // Try to connect to the default Rerun gRPC endpoint
        match bridge.connect("rerun+http://127.0.0.1:9876/proxy") {
            Ok(()) => {
                debug_log("Connected to Rerun viewer (shared recording)!");
                // Don't log grid - dora already did it
                self.rerun_bridge = Some(bridge);
                self.ui.label(id!(status_label)).set_text(cx, "Status: Connected to Dora's Rerun");
                self.ui.button(id!(connect_rerun_btn)).set_text(cx, "Connected");
            }
            Err(e) => {
                debug_log(&format!("Failed to connect: {}", e));
                self.ui.label(id!(status_label)).set_text(cx,
                    "Error: Start dora dataflow first (dora start dataflow.yml)");
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

        // Update time display (other labels removed with IMU/Vehicle/Stats panels)
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
                self.ui.label(id!(status_label)).set_text(cx, &format!("LaserScan: {} points logged{}", cloud.len(), sim_status));
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

    /// Handle display selection - update PropertiesPanel
    fn on_display_selected(&mut self, cx: &mut Cx, id: u64) {
        debug_log(&format!("Display selected: id={}", id));

        // Get display info from DisplaysPanel
        let displays_panel = self.ui.displays_panel(id!(displays_panel));
        if let Some(inner) = displays_panel.borrow() {
            if let Some(display) = inner.get_display(id) {
                // Update properties panel with selected display info
                let display_name = display.name.clone();
                let type_str = format!("Type: {}", display.display_type.name());
                drop(inner); // Release borrow before calling other methods

                self.ui.properties_panel(id!(properties_panel))
                    .set_display(cx, Some(id), &display_name, &type_str);

                // Update status label
                self.ui.label(id!(status_label)).set_text(cx,
                    &format!("Selected: {}", display_name));
            }
        }
        self.ui.redraw(cx);
    }

    /// Handle display enable/disable toggle - update Rerun visibility
    fn on_display_toggled(&mut self, cx: &mut Cx, id: u64, enabled: bool) {
        debug_log(&format!("Display toggled: id={}, enabled={}", id, enabled));

        // Get display info
        let displays_panel = self.ui.displays_panel(id!(displays_panel));
        if let Some(inner) = displays_panel.borrow() {
            if let Some(display) = inner.get_display(id) {
                let action = if enabled { "Enabled" } else { "Disabled" };
                self.ui.label(id!(status_label)).set_text(cx,
                    &format!("{}: {}", action, display.name));

                // TODO: In a full implementation, we would toggle entity visibility in Rerun
                // For now, we could clear/re-log the entity based on enabled state
                if let Some(bridge) = &self.rerun_bridge {
                    match display.display_type {
                        DisplayType::Grid => {
                            if enabled {
                                let _ = bridge.log_ground_grid();
                            }
                            // Note: Rerun doesn't have direct hide/show API,
                            // would need to use entity clear or blueprint
                        }
                        DisplayType::Axes => {
                            if enabled {
                                let _ = bridge.log(
                                    "world/axes",
                                    &rerun::Arrows3D::from_vectors([
                                        [1.0, 0.0, 0.0],
                                        [0.0, 1.0, 0.0],
                                        [0.0, 0.0, 1.0],
                                    ])
                                    .with_origins([[0.0, 0.0, 0.0]; 3])
                                    .with_colors([
                                        rerun::Color::from_rgb(255, 0, 0),
                                        rerun::Color::from_rgb(0, 255, 0),
                                        rerun::Color::from_rgb(0, 0, 255),
                                    ]),
                                );
                            }
                        }
                        _ => {
                            // Other display types will be handled when receiving data
                        }
                    }
                }
            }
        }
        self.ui.redraw(cx);
    }

    /// Handle display deletion
    fn on_display_deleted(&mut self, cx: &mut Cx, id: u64) {
        debug_log(&format!("Display deleted: id={}", id));
        self.ui.label(id!(status_label)).set_text(cx, "Display removed");
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
        // The fusion.urdf file contains a Ford Fusion car with sensors
        let urdf_path = std::path::Path::new("/Users/nupylot/Public/mviz/fusion.urdf");

        if !urdf_path.exists() {
            debug_log(&format!("URDF file not found: {:?}", urdf_path));
            self.ui.label(id!(status_label)).set_text(cx, "Error: fusion.urdf not found!");
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
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Ford Fusion loaded from URDF{}", sim_status));
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

    // ========================================================================
    // DORA INTEGRATION
    // ========================================================================

    /// Toggle Dora connection
    fn toggle_dora_connection(&mut self, cx: &mut Cx) {
        if self.dora_receiver.is_some() {
            // Disconnect
            debug_log("Disconnecting from Dora...");
            if let Some(receiver) = self.dora_receiver.take() {
                receiver.stop();
            }
            self.dora_connected = false;
            self.data_source = DataSource::Simulator;
            self.ui.button(id!(dora_btn)).set_text(cx, "Connect Dora");
            self.ui.label(id!(dora_status)).set_text(cx, "");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Dora Disconnected");
        } else {
            // Connect
            debug_log("Connecting to Dora...");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Connecting to Dora...");
            self.ui.label(id!(dora_status)).set_text(cx, "Connecting...");

            // Start the Dora receiver in background thread
            let receiver = DoraReceiver::start();
            self.dora_receiver = Some(receiver);
            self.data_source = DataSource::Dora;
            self.ui.button(id!(dora_btn)).set_text(cx, "Disconnect Dora");
        }
        self.ui.redraw(cx);
    }

    /// Process incoming Dora messages
    fn process_dora_messages(&mut self, cx: &mut Cx) {
        let Some(receiver) = &self.dora_receiver else { return };

        // Process all pending messages
        while let Some(msg) = receiver.try_recv() {
            match msg {
                DoraMessage::Connected => {
                    debug_log("Dora connected!");
                    self.dora_connected = true;
                    self.ui.button(id!(dora_btn)).set_text(cx, "Disconnect Dora");
                    self.ui.label(id!(dora_status)).set_text(cx, "Connected");
                    self.ui.label(id!(status_label)).set_text(cx, "Status: Dora Connected - Receiving data");
                }
                DoraMessage::Disconnected(reason) => {
                    debug_log(&format!("Dora disconnected: {}", reason));
                    self.dora_connected = false;
                    self.ui.label(id!(dora_status)).set_text(cx, "Disconnected");
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Status: {}", reason));
                }
                DoraMessage::Status(status) => {
                    self.ui.label(id!(dora_status)).set_text(cx, &status);
                }
                DoraMessage::Data(data) => {
                    self.dora_data = data;
                    self.frame_count = self.dora_data.frame_count;
                }
            }
        }
    }

    /// Update UI and Rerun from Dora data
    fn update_from_dora(&mut self, cx: &mut Cx) {
        let data = &self.dora_data;

        // Update FPS counter
        let sim_time = data.frame_count as f64 * 0.02; // ~50Hz
        if sim_time - self.last_fps_time >= 1.0 {
            self.fps = self.frame_count as f64 / sim_time.max(1.0);
            self.last_fps_time = sim_time;
        }

        // Update time display (other labels removed with IMU/Vehicle/Stats panels)
        self.ui.label(id!(time_label)).set_text(cx,
            &format!("{:.2}s", sim_time));

        // Log to Rerun if connected
        if let Some(bridge) = &self.rerun_bridge {
            // Log vehicle pose
            if let Some(ref pose) = data.pose {
                let _ = bridge.log(
                    "world/vehicle/body",
                    &rerun::Boxes3D::from_centers_and_sizes(
                        [[pose.x, pose.y, 0.1_f32]],
                        [[0.5_f32, 0.3, 0.2]],
                    )
                    .with_quaternions([theta_to_quaternion(pose.theta)])
                    .with_colors([rerun::Color::from_rgb(50, 150, 250)]),
                );

                // Direction arrow
                let arrow_len = 0.4_f32;
                let _ = bridge.log(
                    "world/vehicle/direction",
                    &rerun::Arrows3D::from_vectors([[
                        pose.theta.cos() * arrow_len,
                        pose.theta.sin() * arrow_len,
                        0.0,
                    ]])
                    .with_origins([[pose.x, pose.y, 0.2]])
                    .with_colors([rerun::Color::from_rgb(255, 100, 100)]),
                );

                // Velocity arrow
                if pose.velocity.abs() > 0.05 {
                    let _ = bridge.log(
                        "world/vehicle/velocity",
                        &rerun::Arrows3D::from_vectors([[
                            pose.theta.cos() * pose.velocity * 0.3,
                            pose.theta.sin() * pose.velocity * 0.3,
                            0.0,
                        ]])
                        .with_origins([[pose.x, pose.y, 0.3]])
                        .with_colors([rerun::Color::from_rgb(0, 255, 100)]),
                    );
                }
            }

            // Log IMU
            if let Some(ref imu) = data.imu {
                let _ = bridge.log(
                    "world/imu/accelerometer",
                    &rerun::Arrows3D::from_vectors([[
                        imu.accel[0] * 0.1,
                        imu.accel[1] * 0.1,
                        (imu.accel[2] - 9.81) * 0.1,
                    ]])
                    .with_origins([[0.0, 0.0, 0.5]])
                    .with_colors([rerun::Color::from_rgb(16, 185, 129)]),
                );
            }

            // Log waypoints
            if let Some(ref waypoints) = data.waypoints {
                let points: Vec<[f32; 3]> = waypoints.iter()
                    .map(|wp| [wp[0], wp[1], 0.05_f32])
                    .collect();
                if points.len() > 1 {
                    let _ = bridge.log(
                        "world/planner/waypoints",
                        &rerun::LineStrips3D::new([points])
                            .with_colors([rerun::Color::from_rgb(100, 200, 255)]),
                    );
                }
            }

            // Log target point
            if let Some(target) = data.target {
                let _ = bridge.log(
                    "world/planner/target",
                    &rerun::Points3D::new([[target[0], target[1], 0.1_f32]])
                        .with_radii([0.15])
                        .with_colors([rerun::Color::from_rgb(255, 0, 128)]),
                );
            }
        }
    }

    // ========================================================================
    // ZENOH INTEGRATION (LAN Communication)
    // ========================================================================

    /// Toggle Zenoh connection for LAN data reception
    fn toggle_zenoh_connection(&mut self, cx: &mut Cx) {
        if self.zenoh_receiver.is_some() {
            // Disconnect
            debug_log("Disconnecting from Zenoh...");
            if let Some(receiver) = self.zenoh_receiver.take() {
                receiver.stop();
            }
            self.zenoh_connected = false;
            self.data_source = DataSource::Simulator;
            self.ui.button(id!(zenoh_btn)).set_text(cx, "Connect Zenoh");
            self.ui.label(id!(zenoh_status)).set_text(cx, "");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Zenoh Disconnected");
        } else {
            // Connect
            debug_log("Connecting to Zenoh (auto-discovery)...");
            self.ui.label(id!(status_label)).set_text(cx, "Status: Connecting to Zenoh...");
            self.ui.label(id!(zenoh_status)).set_text(cx, "Connecting...");

            // Start Zenoh receiver with auto-discovery (no explicit address)
            let receiver = ZenohReceiver::start(None, Some("mviz".to_string()));
            self.zenoh_receiver = Some(receiver);
            self.data_source = DataSource::Zenoh;
            self.ui.button(id!(zenoh_btn)).set_text(cx, "Disconnect Zenoh");
        }
        self.ui.redraw(cx);
    }

    /// Process incoming Zenoh messages (universal protocol)
    fn process_zenoh_messages(&mut self, cx: &mut Cx) {
        let Some(receiver) = &self.zenoh_receiver else { return };

        // Process all pending messages
        while let Some(msg) = receiver.try_recv() {
            match msg {
                ZenohMessage::Connected => {
                    debug_log("Zenoh connected!");
                    self.zenoh_connected = true;
                    self.ui.button(id!(zenoh_btn)).set_text(cx, "Disconnect Zenoh");
                    self.ui.label(id!(zenoh_status)).set_text(cx, "Connected");
                    self.ui.label(id!(status_label)).set_text(cx, "Status: Zenoh Connected - Universal Receiver");
                }
                ZenohMessage::Disconnected(reason) => {
                    debug_log(&format!("Zenoh disconnected: {}", reason));
                    self.zenoh_connected = false;
                    self.ui.label(id!(zenoh_status)).set_text(cx, "Disconnected");
                    self.ui.label(id!(status_label)).set_text(cx, &format!("Status: {}", reason));
                }
                ZenohMessage::Status(status) => {
                    self.ui.label(id!(zenoh_status)).set_text(cx, &status);
                }
                ZenohMessage::Data(vis_data) => {
                    self.zenoh_message_count += 1;

                    // Log directly to Rerun using universal message format
                    if let Some(bridge) = &self.rerun_bridge {
                        self.log_vis_data_to_rerun(bridge, &vis_data);
                    }

                    // Log periodically
                    if self.zenoh_message_count % 100 == 1 {
                        debug_log(&format!("Zenoh: {} messages, latest: {} @ {}",
                            self.zenoh_message_count, vis_data.msg_type, vis_data.entity_path));
                    }

                    // Update time display
                    self.ui.label(id!(time_label)).set_text(cx,
                        &format!("{:.2}s", vis_data.timestamp));
                }

                ZenohMessage::Log(log_entry) => {
                    self.log_entry_count += 1;

                    // Track discovered node
                    if self.discovered_nodes.insert(log_entry.node_id.clone()) {
                        debug_log(&format!("Discovered node from log: {}", log_entry.node_id));
                        // Update log panel with new node list
                        let nodes: Vec<String> = self.discovered_nodes.iter().cloned().collect();
                        self.ui.log_panel(id!(log_panel)).set_discovered_nodes(cx, nodes.clone());
                        // Also update node detail panel
                        self.ui.node_detail_panel(id!(node_detail_panel)).set_discovered_nodes(cx, nodes);
                    }

                    // Check if this is an I/O activity log (has port info)
                    if let (Some(port), Some(port_type)) = (log_entry.metadata.get("port"), log_entry.metadata.get("port_type")) {
                        // This is I/O activity - add to the node detail panel's I/O display
                        debug_log(&format!("I/O activity: node={}, port={}, type={}, msg={}",
                            log_entry.node_id, port, port_type, log_entry.message));
                        self.ui.node_detail_panel(id!(node_detail_panel)).add_io_activity(
                            cx,
                            &log_entry.node_id,
                            port,
                            port_type,
                            log_entry.timestamp,
                            &log_entry.message,
                        );
                    } else {
                        // Debug: Show what's in metadata for logs without port info
                        if self.log_entry_count % 100 == 1 {
                            debug_log(&format!("Regular log: node={}, metadata keys={:?}",
                                log_entry.node_id, log_entry.metadata.keys().collect::<Vec<_>>()));
                        }
                        // Regular log entry - add to log panels
                        let level_num = match log_entry.level {
                            LogLevel::Debug => 0,
                            LogLevel::Info => 1,
                            LogLevel::Warn => 2,
                            LogLevel::Error => 3,
                        };
                        let display_entry = LogDisplayEntry {
                            timestamp: log_entry.timestamp,
                            level: level_num,
                            level_str: log_entry.level.as_str().to_string(),
                            node_id: log_entry.node_id.clone(),
                            message: log_entry.message.clone(),
                        };

                        // Add to log panel
                        self.ui.log_panel(id!(log_panel)).add_entry(cx, display_entry.clone());
                        // Also add to node detail panel logs
                        self.ui.node_detail_panel(id!(node_detail_panel)).add_log(cx, display_entry);
                    }

                    if self.log_entry_count % 50 == 1 {
                        debug_log(&format!("Log entries: {}", self.log_entry_count));
                    }
                }

                ZenohMessage::NodeDiscovered(node_id) => {
                    if self.discovered_nodes.insert(node_id.clone()) {
                        debug_log(&format!("Node discovered: {}", node_id));
                        let nodes: Vec<String> = self.discovered_nodes.iter().cloned().collect();
                        self.ui.log_panel(id!(log_panel)).set_discovered_nodes(cx, nodes.clone());
                        self.ui.node_detail_panel(id!(node_detail_panel)).set_discovered_nodes(cx, nodes);
                    }
                }

                ZenohMessage::NodeDef(node_def) => {
                    debug_log(&format!("Received node definition: {} ({} inputs, {} outputs)",
                        node_def.id, node_def.inputs.len(), node_def.outputs.len()));

                    // Convert to widget types
                    let inputs: Vec<NodeInput> = node_def.inputs.iter()
                        .map(|i| NodeInput {
                            name: i.name.clone(),
                            source: i.source.clone(),
                        })
                        .collect();

                    let outputs: Vec<NodeOutput> = node_def.outputs.iter()
                        .map(|o| NodeOutput {
                            name: o.name.clone(),
                            destinations: o.destinations.clone(),
                        })
                        .collect();

                    // Set definition on node detail panel
                    self.ui.node_detail_panel(id!(node_detail_panel))
                        .set_node_definition(cx, &node_def.id, inputs, outputs);

                    // Also track as discovered node
                    if self.discovered_nodes.insert(node_def.id.clone()) {
                        let nodes: Vec<String> = self.discovered_nodes.iter().cloned().collect();
                        self.ui.log_panel(id!(log_panel)).set_discovered_nodes(cx, nodes.clone());
                        self.ui.node_detail_panel(id!(node_detail_panel)).set_discovered_nodes(cx, nodes);
                    }
                }

                ZenohMessage::GraphUpdate(graph_update) => {
                    debug_log(&format!("Received graph update: {} nodes, {} edges",
                        graph_update.nodes.len(), graph_update.edges.len()));

                    // Convert to widget format
                    let nodes: Vec<(String, bool)> = graph_update.nodes.iter()
                        .map(|n| (n.id.clone(), n.status == mviz_core::zenoh_protocol::GraphNodeStatus::Active))
                        .collect();

                    let edges: Vec<(String, String, String, String)> = graph_update.edges.iter()
                        .map(|e| (e.from_node.clone(), e.from_port.clone(), e.to_node.clone(), e.to_port.clone()))
                        .collect();

                    // Update the dataflow graph widget
                    self.ui.dataflow_graph_widget(id!(dataflow_graph))
                        .update_from_graph_update(cx, nodes, edges, graph_update.timestamp);
                }
            }
        }
    }

    /// Log universal VisData to Rerun based on message type
    fn log_vis_data_to_rerun(&self, bridge: &RerunBridge, vis_data: &VisData) {
        let entity_path = &vis_data.entity_path;
        let msg = &vis_data.message;

        match vis_data.msg_type.as_str() {
            "points3d" => {
                // Check for binary point cloud data
                if let (Some(binary), Some(format), Some(count)) =
                    (&vis_data.binary, &msg.format, msg.count) {
                    if format == binary_formats::POINTS_XYZ_F32 {
                        let points = parse_points_xyz_f32(binary, count);

                        // Get optional styling from data
                        let radius: f32 = msg.data.get("radius")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(0.03);
                        let color = parse_color(&msg.data, [100, 200, 255, 255]);

                        let _ = bridge.log(
                            entity_path,
                            &rerun::Points3D::new(points)
                                .with_radii([radius])
                                .with_colors([rerun::Color::from_unmultiplied_rgba(
                                    color[0], color[1], color[2], color[3])]),
                        );
                    }
                } else if let Ok(data) = serde_json::from_value::<Points3DData>(msg.data.clone()) {
                    // JSON positions
                    if let Some(positions) = data.positions {
                        let radius = data.radius.unwrap_or(0.05);
                        let color = data.color.unwrap_or([200, 200, 200, 255]);

                        let _ = bridge.log(
                            entity_path,
                            &rerun::Points3D::new(positions)
                                .with_radii([radius])
                                .with_colors([rerun::Color::from_unmultiplied_rgba(
                                    color[0], color[1], color[2], color[3])]),
                        );
                    }
                }
            }

            "boxes3d" => {
                if let Ok(data) = serde_json::from_value::<Boxes3DData>(msg.data.clone()) {
                    let color = data.color.unwrap_or([100, 150, 200, 255]);

                    let mut boxes = if data.half_sizes {
                        rerun::Boxes3D::from_centers_and_half_sizes(data.centers, data.sizes)
                    } else {
                        rerun::Boxes3D::from_centers_and_sizes(data.centers, data.sizes)
                    };

                    if let Some(quats) = data.quaternions {
                        let rerun_quats: Vec<rerun::Quaternion> = quats.iter()
                            .map(|q| rerun::Quaternion::from_xyzw(*q))
                            .collect();
                        boxes = boxes.with_quaternions(rerun_quats);
                    }

                    let _ = bridge.log(
                        entity_path,
                        &boxes.with_colors([rerun::Color::from_unmultiplied_rgba(
                            color[0], color[1], color[2], color[3])]),
                    );
                }
            }

            "arrows3d" => {
                if let Ok(data) = serde_json::from_value::<Arrows3DData>(msg.data.clone()) {
                    let color = data.color.unwrap_or([255, 100, 100, 255]);

                    let _ = bridge.log(
                        entity_path,
                        &rerun::Arrows3D::from_vectors(data.vectors)
                            .with_origins(data.origins)
                            .with_colors([rerun::Color::from_unmultiplied_rgba(
                                color[0], color[1], color[2], color[3])]),
                    );
                }
            }

            "linestrips3d" => {
                match serde_json::from_value::<LineStrips3DData>(msg.data.clone()) {
                    Ok(data) => {
                        let color = data.color.unwrap_or([100, 200, 255, 255]);
                        debug_log(&format!("Logging linestrip: {} strips, {} points, color={:?}",
                            data.strips.len(),
                            data.strips.iter().map(|s| s.len()).sum::<usize>(),
                            color));

                        let result = bridge.log(
                            entity_path,
                            &rerun::LineStrips3D::new(data.strips)
                                .with_radii([0.03])  // Visible line thickness
                                .with_colors([rerun::Color::from_unmultiplied_rgba(
                                    color[0], color[1], color[2], color[3])]),
                        );
                        if let Err(e) = result {
                            debug_log(&format!("Linestrip log error: {}", e));
                        }
                    }
                    Err(e) => {
                        debug_log(&format!("Failed to parse linestrips3d: {}, data: {:?}", e, msg.data));
                    }
                }
            }

            "transform3d" => {
                if let Ok(data) = serde_json::from_value::<Transform3DData>(msg.data.clone()) {
                    if let Some(matrix) = data.matrix4x4 {
                        // 4x4 matrix (row-major, extract 3x3 rotation and translation)
                        let _ = bridge.log(
                            entity_path,
                            &rerun::Transform3D::from_mat3x3([
                                [matrix[0], matrix[1], matrix[2]],
                                [matrix[4], matrix[5], matrix[6]],
                                [matrix[8], matrix[9], matrix[10]],
                            ]).with_translation([matrix[3], matrix[7], matrix[11]]),
                        );
                    } else if let (Some(trans), Some(rot)) = (data.translation, data.rotation) {
                        // Translation + quaternion
                        let _ = bridge.log(
                            entity_path,
                            &rerun::Transform3D::from_translation(trans)
                                .with_quaternion(rerun::Quaternion::from_xyzw(rot)),
                        );
                    } else if let Some(trans) = data.translation {
                        // Translation only
                        let _ = bridge.log(
                            entity_path,
                            &rerun::Transform3D::from_translation(trans),
                        );
                    }
                }
            }

            "scalar" => {
                if let Ok(data) = serde_json::from_value::<ScalarData>(msg.data.clone()) {
                    // Log as a point on a time series (text log as fallback for visibility)
                    let _ = bridge.log(
                        entity_path,
                        &rerun::TextLog::new(format!("{:.4}", data.value)),
                    );
                }
            }

            _ => {
                debug_log(&format!("Unknown message type: {}", vis_data.msg_type));
            }
        }
    }

    /// Update UI from Zenoh data (now handled in process_zenoh_messages)
    fn update_from_zenoh(&mut self, _cx: &mut Cx) {
        // All data is now logged directly in process_zenoh_messages via log_vis_data_to_rerun
        // UI labels for stats were removed with the IMU/Vehicle/Stats panels
    }
}

/// Parse color from JSON data, with default fallback
fn parse_color(data: &serde_json::Value, default: [u8; 4]) -> [u8; 4] {
    if let Some(arr) = data.get("color").and_then(|v| v.as_array()) {
        if arr.len() >= 4 {
            return [
                arr[0].as_u64().unwrap_or(default[0] as u64) as u8,
                arr[1].as_u64().unwrap_or(default[1] as u64) as u8,
                arr[2].as_u64().unwrap_or(default[2] as u64) as u8,
                arr[3].as_u64().unwrap_or(default[3] as u64) as u8,
            ];
        } else if arr.len() >= 3 {
            return [
                arr[0].as_u64().unwrap_or(default[0] as u64) as u8,
                arr[1].as_u64().unwrap_or(default[1] as u64) as u8,
                arr[2].as_u64().unwrap_or(default[2] as u64) as u8,
                255,
            ];
        }
    }
    default
}

/// Convert theta angle to quaternion for Z-axis rotation
fn theta_to_quaternion(theta: f32) -> rerun::Quaternion {
    let half_theta = theta / 2.0;
    rerun::Quaternion::from_xyzw([0.0, 0.0, half_theta.sin(), half_theta.cos()])
}
