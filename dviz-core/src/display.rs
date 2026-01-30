//! Display trait and context for visualization components
//!
//! This module defines the core abstraction for visual representations in MViz.
//! Each display type (PointCloud, Marker, Grid, etc.) implements the Display trait.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{FrameId, Timestamp, Transform};

// ============================================================================
// DISPLAY ERROR
// ============================================================================

/// Errors that can occur during display operations
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Display not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Transform not available for frame '{frame}' at time {timestamp:?}")]
    TransformUnavailable { frame: String, timestamp: Timestamp },

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
}

/// Result type for display operations
pub type DisplayResult<T> = Result<T, DisplayError>;

// ============================================================================
// DISPLAY STATUS
// ============================================================================

/// Status of a display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DisplayStatus {
    /// Display is inactive/disabled
    #[default]
    Disabled,
    /// Display is initializing
    Initializing,
    /// Display is active and working
    Ok,
    /// Display has a warning but still functional
    Warning,
    /// Display has an error and cannot function
    Error,
}

impl DisplayStatus {
    /// Check if display is in a functional state
    pub fn is_functional(&self) -> bool {
        matches!(self, DisplayStatus::Ok | DisplayStatus::Warning)
    }

    /// Check if display is enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, DisplayStatus::Disabled)
    }
}

// ============================================================================
// DISPLAY INFO
// ============================================================================

/// Metadata about a display type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Unique type name (e.g., "PointCloud2", "Marker", "Grid")
    pub type_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Description of what this display does
    pub description: String,
    /// Category for grouping in UI (e.g., "rviz_common", "sensor_msgs")
    pub category: String,
}

impl DisplayInfo {
    /// Create new display info
    pub fn new(
        type_name: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            display_name: display_name.into(),
            description: description.into(),
            category: category.into(),
        }
    }
}

// ============================================================================
// DISPLAY PROPERTIES
// ============================================================================

/// Dynamic property value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color([u8; 4]),
    Vec3([f32; 3]),
}

impl PropertyValue {
    /// Get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(v) => Some(*v),
            PropertyValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(v) => Some(v),
            _ => None,
        }
    }
}

impl From<bool> for PropertyValue {
    fn from(v: bool) -> Self {
        PropertyValue::Bool(v)
    }
}

impl From<i64> for PropertyValue {
    fn from(v: i64) -> Self {
        PropertyValue::Int(v)
    }
}

impl From<f64> for PropertyValue {
    fn from(v: f64) -> Self {
        PropertyValue::Float(v)
    }
}

impl From<String> for PropertyValue {
    fn from(v: String) -> Self {
        PropertyValue::String(v)
    }
}

impl From<&str> for PropertyValue {
    fn from(v: &str) -> Self {
        PropertyValue::String(v.to_string())
    }
}

/// Property metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyMeta {
    /// Property name
    pub name: String,
    /// Human-readable label
    pub label: String,
    /// Description/tooltip
    pub description: String,
    /// Default value
    pub default: PropertyValue,
    /// Whether property is read-only
    pub read_only: bool,
}

impl PropertyMeta {
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
        default: impl Into<PropertyValue>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            description: description.into(),
            default: default.into(),
            read_only: false,
        }
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

/// Properties container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Properties {
    values: HashMap<String, PropertyValue>,
}

impl Properties {
    /// Create new empty properties
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Set a property value
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<PropertyValue>) {
        self.values.insert(name.into(), value.into());
    }

    /// Get a property value
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.values.get(name)
    }

    /// Get a bool property
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| v.as_bool())
    }

    /// Get an int property
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|v| v.as_int())
    }

    /// Get a float property
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|v| v.as_float())
    }

    /// Get a string property
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_string())
    }

    /// Check if property exists
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Remove a property
    pub fn remove(&mut self, name: &str) -> Option<PropertyValue> {
        self.values.remove(name)
    }

    /// Get all property names
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }

    /// Merge with another properties set (other takes precedence)
    pub fn merge(&mut self, other: &Properties) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }
}

// ============================================================================
// TRANSFORM PROVIDER
// ============================================================================

/// Trait for providing coordinate frame transforms
pub trait TransformProvider: Send + Sync {
    /// Get transform from source frame to target frame at a given time
    fn lookup_transform(
        &self,
        target_frame: &FrameId,
        source_frame: &FrameId,
        time: &Timestamp,
    ) -> DisplayResult<Transform>;

    /// Check if a transform is available
    fn can_transform(
        &self,
        target_frame: &FrameId,
        source_frame: &FrameId,
        time: &Timestamp,
    ) -> bool;

    /// Get the fixed frame (usually "world" or "map")
    fn fixed_frame(&self) -> &FrameId;

    /// Get all known frame IDs
    fn all_frame_ids(&self) -> Vec<FrameId>;
}

/// Simple transform provider that only handles identity transforms
#[derive(Debug, Default)]
pub struct IdentityTransformProvider {
    fixed_frame: FrameId,
}

impl IdentityTransformProvider {
    pub fn new(fixed_frame: impl Into<FrameId>) -> Self {
        Self {
            fixed_frame: fixed_frame.into(),
        }
    }
}

impl TransformProvider for IdentityTransformProvider {
    fn lookup_transform(
        &self,
        _target_frame: &FrameId,
        _source_frame: &FrameId,
        _time: &Timestamp,
    ) -> DisplayResult<Transform> {
        Ok(Transform::IDENTITY)
    }

    fn can_transform(
        &self,
        _target_frame: &FrameId,
        _source_frame: &FrameId,
        _time: &Timestamp,
    ) -> bool {
        true
    }

    fn fixed_frame(&self) -> &FrameId {
        &self.fixed_frame
    }

    fn all_frame_ids(&self) -> Vec<FrameId> {
        vec![self.fixed_frame.clone()]
    }
}

// ============================================================================
// DISPLAY CONTEXT
// ============================================================================

/// Context provided to displays for accessing visualization services
pub struct DisplayContext {
    /// Transform provider
    transform_provider: Arc<dyn TransformProvider>,
    /// Current visualization time
    current_time: RwLock<Timestamp>,
    /// Fixed frame for rendering
    fixed_frame: RwLock<FrameId>,
}

impl DisplayContext {
    /// Create a new display context
    pub fn new(transform_provider: Arc<dyn TransformProvider>) -> Self {
        let fixed_frame = transform_provider.fixed_frame().clone();
        Self {
            transform_provider,
            current_time: RwLock::new(Timestamp::now()),
            fixed_frame: RwLock::new(fixed_frame),
        }
    }

    /// Create a simple context with identity transforms
    pub fn simple(fixed_frame: impl Into<FrameId>) -> Self {
        let provider = Arc::new(IdentityTransformProvider::new(fixed_frame));
        Self::new(provider)
    }

    /// Get the transform provider
    pub fn transform_provider(&self) -> &Arc<dyn TransformProvider> {
        &self.transform_provider
    }

    /// Get current visualization time
    pub fn current_time(&self) -> Timestamp {
        *self.current_time.read()
    }

    /// Set current visualization time
    pub fn set_current_time(&self, time: Timestamp) {
        *self.current_time.write() = time;
    }

    /// Get the fixed frame
    pub fn fixed_frame(&self) -> FrameId {
        self.fixed_frame.read().clone()
    }

    /// Set the fixed frame
    pub fn set_fixed_frame(&self, frame: impl Into<FrameId>) {
        *self.fixed_frame.write() = frame.into();
    }

    /// Look up transform from source to fixed frame
    pub fn transform_to_fixed(&self, source_frame: &FrameId) -> DisplayResult<Transform> {
        let fixed = self.fixed_frame.read().clone();
        let time = *self.current_time.read();
        self.transform_provider
            .lookup_transform(&fixed, source_frame, &time)
    }

    /// Transform a point from source frame to fixed frame
    pub fn transform_point_to_fixed(
        &self,
        point: glam::Vec3,
        source_frame: &FrameId,
    ) -> DisplayResult<glam::Vec3> {
        let transform = self.transform_to_fixed(source_frame)?;
        Ok(transform.transform_point(point))
    }
}

// ============================================================================
// DISPLAY TRAIT
// ============================================================================

/// Unique identifier for a display instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayId(pub u64);

impl DisplayId {
    /// Generate a new unique ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for DisplayId {
    fn default() -> Self {
        Self::new()
    }
}

/// Base trait for all visualization displays
pub trait Display: Send + Sync {
    /// Get the unique ID for this display instance
    fn id(&self) -> DisplayId;

    /// Get display info/metadata
    fn info(&self) -> &DisplayInfo;

    /// Get the display name (user-editable)
    fn name(&self) -> &str;

    /// Set the display name
    fn set_name(&mut self, name: String);

    /// Get current status
    fn status(&self) -> DisplayStatus;

    /// Get status message (for warnings/errors)
    fn status_message(&self) -> Option<&str>;

    /// Check if display is enabled
    fn is_enabled(&self) -> bool;

    /// Enable or disable the display
    fn set_enabled(&mut self, enabled: bool);

    /// Initialize the display with context
    fn initialize(&mut self, ctx: &DisplayContext) -> DisplayResult<()>;

    /// Update the display (called each frame)
    fn update(&mut self, ctx: &DisplayContext) -> DisplayResult<()>;

    /// Reset the display state
    fn reset(&mut self);

    /// Get display properties
    fn properties(&self) -> &Properties;

    /// Get mutable display properties
    fn properties_mut(&mut self) -> &mut Properties;

    /// Get property metadata
    fn property_meta(&self) -> Vec<PropertyMeta>;

    /// Called when a property changes
    fn on_property_changed(&mut self, name: &str, value: &PropertyValue);

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Cast to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// ============================================================================
// DISPLAY FACTORY
// ============================================================================

/// Factory trait for creating display instances
pub trait DisplayFactory: Send + Sync {
    /// Get the type name this factory creates
    fn type_name(&self) -> &str;

    /// Get display info for this type
    fn info(&self) -> DisplayInfo;

    /// Create a new display instance
    fn create(&self, name: String) -> Box<dyn Display>;
}

/// Registry of display factories
#[derive(Default)]
pub struct DisplayRegistry {
    factories: HashMap<String, Box<dyn DisplayFactory>>,
}

impl DisplayRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a display factory
    pub fn register(&mut self, factory: Box<dyn DisplayFactory>) {
        let type_name = factory.type_name().to_string();
        self.factories.insert(type_name, factory);
    }

    /// Get a factory by type name
    pub fn get(&self, type_name: &str) -> Option<&dyn DisplayFactory> {
        self.factories.get(type_name).map(|f| f.as_ref())
    }

    /// Create a display by type name
    pub fn create(&self, type_name: &str, name: String) -> Option<Box<dyn Display>> {
        self.factories.get(type_name).map(|f| f.create(name))
    }

    /// Get all registered type names
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(|s| s.as_str())
    }

    /// Get all display infos
    pub fn all_info(&self) -> Vec<DisplayInfo> {
        self.factories.values().map(|f| f.info()).collect()
    }

    /// Get infos grouped by category
    pub fn info_by_category(&self) -> HashMap<String, Vec<DisplayInfo>> {
        let mut result: HashMap<String, Vec<DisplayInfo>> = HashMap::new();
        for factory in self.factories.values() {
            let info = factory.info();
            result
                .entry(info.category.clone())
                .or_default()
                .push(info);
        }
        result
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_status() {
        assert!(!DisplayStatus::Disabled.is_enabled());
        assert!(DisplayStatus::Ok.is_functional());
        assert!(DisplayStatus::Warning.is_functional());
        assert!(!DisplayStatus::Error.is_functional());
    }

    #[test]
    fn test_display_info() {
        let info = DisplayInfo::new(
            "PointCloud2",
            "Point Cloud",
            "Displays point cloud data",
            "sensor_msgs",
        );
        assert_eq!(info.type_name, "PointCloud2");
        assert_eq!(info.category, "sensor_msgs");
    }

    #[test]
    fn test_property_value() {
        let bool_val = PropertyValue::from(true);
        assert_eq!(bool_val.as_bool(), Some(true));

        let int_val = PropertyValue::from(42i64);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let str_val = PropertyValue::from("test");
        assert_eq!(str_val.as_string(), Some("test"));
    }

    #[test]
    fn test_properties() {
        let mut props = Properties::new();
        props.set("enabled", true);
        props.set("alpha", 0.5f64);
        props.set("name", "test display");

        assert_eq!(props.get_bool("enabled"), Some(true));
        assert_eq!(props.get_float("alpha"), Some(0.5));
        assert_eq!(props.get_string("name"), Some("test display"));
        assert!(props.contains("enabled"));
        assert!(!props.contains("missing"));
    }

    #[test]
    fn test_display_context() {
        let ctx = DisplayContext::simple("world");
        assert_eq!(ctx.fixed_frame().as_str(), "world");

        ctx.set_fixed_frame("map");
        assert_eq!(ctx.fixed_frame().as_str(), "map");

        let time = Timestamp::from_secs_f64(1.0);
        ctx.set_current_time(time);
        assert_eq!(ctx.current_time().as_nanos(), time.as_nanos());
    }

    #[test]
    fn test_identity_transform_provider() {
        let provider = IdentityTransformProvider::new("world");
        assert_eq!(provider.fixed_frame().as_str(), "world");

        let transform = provider
            .lookup_transform(
                &FrameId::new("world"),
                &FrameId::new("base_link"),
                &Timestamp::now(),
            )
            .unwrap();

        assert_eq!(transform.translation, glam::Vec3::ZERO);
    }

    #[test]
    fn test_display_id() {
        let id1 = DisplayId::new();
        let id2 = DisplayId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_display_registry() {
        let registry = DisplayRegistry::new();
        assert!(registry.get("PointCloud2").is_none());
    }
}
