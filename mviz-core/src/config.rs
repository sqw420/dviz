//! Configuration schema for MViz
//!
//! Types for serializing and deserializing application and display configuration.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::display::{Properties, PropertyValue};
use crate::FrameId;

// ============================================================================
// CONFIG ERROR
// ============================================================================

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    ParseError(String),

    #[error("Failed to serialize config: {0}")]
    SerializeError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for config operations
pub type ConfigResult<T> = Result<T, ConfigError>;

// ============================================================================
// WINDOW CONFIG
// ============================================================================

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub width: u32,
    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub height: u32,
    /// Window X position (None = system default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    /// Window Y position (None = system default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    /// Whether window is maximized
    #[serde(default)]
    pub maximized: bool,
    /// Whether window is fullscreen
    #[serde(default)]
    pub fullscreen: bool,
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    800
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            x: None,
            y: None,
            maximized: false,
            fullscreen: false,
        }
    }
}

// ============================================================================
// VIEW CONFIG
// ============================================================================

/// Camera/view configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfig {
    /// View controller type (e.g., "Orbit", "FPS", "TopDown")
    #[serde(default = "default_view_type")]
    pub view_type: String,
    /// Target frame for the camera
    #[serde(default)]
    pub target_frame: FrameId,
    /// Camera position [x, y, z]
    #[serde(default = "default_camera_position")]
    pub position: [f32; 3],
    /// Camera focal point [x, y, z]
    #[serde(default)]
    pub focal_point: [f32; 3],
    /// Camera up vector [x, y, z]
    #[serde(default = "default_camera_up")]
    pub up: [f32; 3],
    /// Field of view in degrees
    #[serde(default = "default_fov")]
    pub fov: f32,
    /// Near clipping plane
    #[serde(default = "default_near")]
    pub near: f32,
    /// Far clipping plane
    #[serde(default = "default_far")]
    pub far: f32,
}

fn default_view_type() -> String {
    "Orbit".to_string()
}

fn default_camera_position() -> [f32; 3] {
    [10.0, 10.0, 10.0]
}

fn default_camera_up() -> [f32; 3] {
    [0.0, 0.0, 1.0]
}

fn default_fov() -> f32 {
    45.0
}

fn default_near() -> f32 {
    0.01
}

fn default_far() -> f32 {
    1000.0
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            view_type: default_view_type(),
            target_frame: FrameId::default(),
            position: default_camera_position(),
            focal_point: [0.0, 0.0, 0.0],
            up: default_camera_up(),
            fov: default_fov(),
            near: default_near(),
            far: default_far(),
        }
    }
}

// ============================================================================
// DISPLAY CONFIG
// ============================================================================

/// Configuration for a single display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display type name (e.g., "PointCloud2", "Marker", "Grid")
    pub display_type: String,
    /// User-assigned name for this display
    pub name: String,
    /// Whether the display is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Display-specific properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl DisplayConfig {
    /// Create a new display config
    pub fn new(display_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            display_type: display_type.into(),
            name: name.into(),
            enabled: true,
            properties: HashMap::new(),
        }
    }

    /// Set a property value
    pub fn with_property(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.properties.insert(key.into(), json_value);
        }
        self
    }

    /// Get a property as a specific type
    pub fn get_property<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.properties
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Convert properties to Properties object
    pub fn to_properties(&self) -> Properties {
        let mut props = Properties::new();
        for (key, value) in &self.properties {
            if let Some(pv) = json_to_property_value(value) {
                props.set(key.clone(), pv);
            }
        }
        props
    }
}

/// Convert JSON value to PropertyValue
fn json_to_property_value(value: &serde_json::Value) -> Option<PropertyValue> {
    match value {
        serde_json::Value::Bool(b) => Some(PropertyValue::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(PropertyValue::Int(i))
            } else if let Some(f) = n.as_f64() {
                Some(PropertyValue::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::String(s) => Some(PropertyValue::String(s.clone())),
        serde_json::Value::Array(arr) if arr.len() == 3 => {
            // Try to parse as Vec3
            let floats: Option<Vec<f32>> = arr
                .iter()
                .map(|v| v.as_f64().map(|f| f as f32))
                .collect();
            floats.map(|f| PropertyValue::Vec3([f[0], f[1], f[2]]))
        }
        serde_json::Value::Array(arr) if arr.len() == 4 => {
            // Try to parse as Color
            let bytes: Option<Vec<u8>> = arr.iter().map(|v| v.as_u64().map(|i| i as u8)).collect();
            bytes.map(|b| PropertyValue::Color([b[0], b[1], b[2], b[3]]))
        }
        _ => None,
    }
}

// ============================================================================
// GLOBAL CONFIG
// ============================================================================

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Fixed frame for visualization
    #[serde(default = "default_fixed_frame")]
    pub fixed_frame: FrameId,
    /// Background color [r, g, b, a]
    #[serde(default = "default_background_color")]
    pub background_color: [u8; 4],
    /// Frame rate limit (0 = unlimited)
    #[serde(default = "default_frame_rate")]
    pub frame_rate: u32,
    /// Show grid
    #[serde(default = "default_true")]
    pub show_grid: bool,
    /// Show axes
    #[serde(default = "default_true")]
    pub show_axes: bool,
}

fn default_fixed_frame() -> FrameId {
    FrameId::new("world")
}

fn default_background_color() -> [u8; 4] {
    [48, 48, 48, 255] // Dark gray
}

fn default_frame_rate() -> u32 {
    60
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            fixed_frame: default_fixed_frame(),
            background_color: default_background_color(),
            frame_rate: default_frame_rate(),
            show_grid: true,
            show_axes: true,
        }
    }
}

// ============================================================================
// PANEL CONFIG
// ============================================================================

/// Configuration for UI panels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Panel type name
    pub panel_type: String,
    /// Panel name
    pub name: String,
    /// Panel visibility
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Panel position (e.g., "left", "right", "bottom")
    #[serde(default)]
    pub position: String,
    /// Panel-specific properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl PanelConfig {
    pub fn new(panel_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            panel_type: panel_type.into(),
            name: name.into(),
            visible: true,
            position: String::new(),
            properties: HashMap::new(),
        }
    }
}

// ============================================================================
// APP CONFIG
// ============================================================================

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Config file version for compatibility
    #[serde(default = "default_version")]
    pub version: String,
    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,
    /// View/camera configuration
    #[serde(default)]
    pub view: ViewConfig,
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
    /// Display configurations
    #[serde(default)]
    pub displays: Vec<DisplayConfig>,
    /// Panel configurations
    #[serde(default)]
    pub panels: Vec<PanelConfig>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            window: WindowConfig::default(),
            view: ViewConfig::default(),
            global: GlobalConfig::default(),
            displays: Vec::new(),
            panels: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Create a new empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a default config with common displays
    pub fn with_defaults() -> Self {
        Self {
            displays: vec![
                DisplayConfig::new("Grid", "Grid")
                    .with_property("cell_count", 20)
                    .with_property("cell_size", 1.0)
                    .with_property("color", [128, 128, 128, 128]),
                DisplayConfig::new("Axes", "Axes").with_property("length", 1.0),
            ],
            ..Default::default()
        }
    }

    /// Add a display configuration
    pub fn add_display(&mut self, display: DisplayConfig) {
        self.displays.push(display);
    }

    /// Add a panel configuration
    pub fn add_panel(&mut self, panel: PanelConfig) {
        self.panels.push(panel);
    }

    /// Find a display by name
    pub fn find_display(&self, name: &str) -> Option<&DisplayConfig> {
        self.displays.iter().find(|d| d.name == name)
    }

    /// Find a display by name (mutable)
    pub fn find_display_mut(&mut self, name: &str) -> Option<&mut DisplayConfig> {
        self.displays.iter_mut().find(|d| d.name == name)
    }

    /// Load config from YAML file
    pub fn load_from_file(path: impl AsRef<Path>) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Parse config from YAML string
    pub fn from_yaml(yaml: &str) -> ConfigResult<Self> {
        serde_yaml::from_str(yaml).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save config to YAML file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> ConfigResult<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Convert config to YAML string
    pub fn to_yaml(&self) -> ConfigResult<String> {
        serde_yaml::to_string(self).map_err(|e| ConfigError::SerializeError(e.to_string()))
    }

    /// Validate the configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Check version
        if self.version.is_empty() {
            return Err(ConfigError::InvalidConfig("Missing version".to_string()));
        }

        // Check for duplicate display names
        let mut names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for display in &self.displays {
            if !names.insert(&display.name) {
                return Err(ConfigError::InvalidConfig(format!(
                    "Duplicate display name: {}",
                    display.name
                )));
            }
        }

        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 800);
        assert!(!config.maximized);
    }

    #[test]
    fn test_view_config_default() {
        let config = ViewConfig::default();
        assert_eq!(config.view_type, "Orbit");
        assert_eq!(config.fov, 45.0);
    }

    #[test]
    fn test_display_config() {
        let config = DisplayConfig::new("PointCloud2", "LiDAR Points")
            .with_property("point_size", 2.0)
            .with_property("color_mode", "intensity");

        assert_eq!(config.display_type, "PointCloud2");
        assert_eq!(config.name, "LiDAR Points");
        assert!(config.enabled);
        assert_eq!(config.get_property::<f64>("point_size"), Some(2.0));
        assert_eq!(
            config.get_property::<String>("color_mode"),
            Some("intensity".to_string())
        );
    }

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.fixed_frame.as_str(), "world");
        assert!(config.show_grid);
        assert!(config.show_axes);
    }

    #[test]
    fn test_app_config_with_defaults() {
        let config = AppConfig::with_defaults();
        assert_eq!(config.displays.len(), 2);
        assert!(config.find_display("Grid").is_some());
        assert!(config.find_display("Axes").is_some());
    }

    #[test]
    fn test_app_config_yaml_roundtrip() {
        let mut config = AppConfig::with_defaults();
        config.add_display(
            DisplayConfig::new("PointCloud2", "Test Cloud").with_property("point_size", 1.5),
        );

        let yaml = config.to_yaml().unwrap();
        let restored = AppConfig::from_yaml(&yaml).unwrap();

        assert_eq!(restored.displays.len(), config.displays.len());
        assert!(restored.find_display("Test Cloud").is_some());
    }

    #[test]
    fn test_app_config_validation() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());

        // Test duplicate detection
        let mut config2 = AppConfig::default();
        config2.add_display(DisplayConfig::new("Grid", "Duplicate"));
        config2.add_display(DisplayConfig::new("Grid", "Duplicate"));
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_json_to_property_value() {
        assert_eq!(
            json_to_property_value(&serde_json::json!(true)),
            Some(PropertyValue::Bool(true))
        );
        assert_eq!(
            json_to_property_value(&serde_json::json!(42)),
            Some(PropertyValue::Int(42))
        );
        assert_eq!(
            json_to_property_value(&serde_json::json!(3.14)),
            Some(PropertyValue::Float(3.14))
        );
        assert_eq!(
            json_to_property_value(&serde_json::json!("test")),
            Some(PropertyValue::String("test".to_string()))
        );
    }

    #[test]
    fn test_display_config_to_properties() {
        let config = DisplayConfig::new("Test", "Test")
            .with_property("enabled", true)
            .with_property("alpha", 0.5)
            .with_property("name", "test");

        let props = config.to_properties();
        assert_eq!(props.get_bool("enabled"), Some(true));
        assert_eq!(props.get_float("alpha"), Some(0.5));
        assert_eq!(props.get_string("name"), Some("test"));
    }
}
