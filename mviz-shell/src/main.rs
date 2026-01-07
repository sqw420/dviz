//! MViz-Rerun - Main entry point
//!
//! A visualization application combining Makepad for the control panel
//! and Rerun for 3D sensor data visualization.

fn main() {
    env_logger::init();
    log::info!("Starting MViz-Rerun");
    mviz_shell::app::app_main();
}
