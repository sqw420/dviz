//! DViz-Rerun - Main entry point
//!
//! A visualization application combining Makepad for the control panel
//! and Rerun for 3D sensor data visualization.

fn main() {
    env_logger::init();
    log::info!("Starting DViz-Rerun");
    dviz_shell::app::app_main();
}
