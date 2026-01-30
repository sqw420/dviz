//! Base Display Helper
//!
//! Provides common functionality for display implementations.

use std::time::Duration;
use dviz_core::{
    DisplayId, DisplayInfo, DisplayStatus, Properties, PropertyValue,
};

/// Common state for all displays
#[derive(Debug)]
pub struct BaseDisplay {
    /// Display name
    pub name: String,
    /// Display type name
    pub type_name: String,
    /// Entity path in Rerun
    pub entity_path: String,
    /// Whether display is enabled
    pub enabled: bool,
    /// Current status
    pub status: DisplayStatus,
    /// Status message
    pub status_message: Option<String>,
    /// Properties container
    pub properties: Properties,
    /// Unique display ID
    pub id: DisplayId,
    /// Display info
    pub info: DisplayInfo,
}

impl BaseDisplay {
    /// Create a new base display
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        let name = name.into();
        let type_name = type_name.into();
        let entity_path = format!("/{}", name.to_lowercase().replace(' ', "_"));

        let info = DisplayInfo::new(
            &type_name,
            &name,
            "",
            "dviz",
        );

        Self {
            name,
            type_name,
            entity_path,
            enabled: true,
            status: DisplayStatus::Ok,
            status_message: None,
            properties: Properties::new(),
            id: DisplayId::new(),
            info,
        }
    }

    /// Create with custom entity path
    pub fn with_entity_path(mut self, path: impl Into<String>) -> Self {
        self.entity_path = path.into();
        self
    }

    /// Set the display name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Set status with message
    pub fn set_status(&mut self, status: DisplayStatus, message: Option<String>) {
        self.status = status;
        self.status_message = message;
    }

    /// Clear status (set to Ok)
    pub fn clear_status(&mut self) {
        self.status = DisplayStatus::Ok;
        self.status_message = None;
    }

    /// Add a property
    pub fn add_property(&mut self, name: &str, value: impl Into<PropertyValue>) {
        self.properties.set(name, value);
    }
}

impl Default for BaseDisplay {
    fn default() -> Self {
        Self::new("Display", "unknown")
    }
}

/// Macro to implement common Display trait methods
#[macro_export]
macro_rules! impl_display_base {
    ($type:ty) => {
        impl $type {
            /// Get display name
            pub fn name(&self) -> &str {
                &self.base.name
            }

            /// Get display type name
            pub fn type_name(&self) -> &str {
                &self.base.type_name
            }

            /// Get entity path
            pub fn entity_path(&self) -> &str {
                &self.base.entity_path
            }

            /// Check if enabled
            pub fn is_enabled(&self) -> bool {
                self.base.enabled
            }

            /// Set enabled state
            pub fn set_enabled(&mut self, enabled: bool) {
                self.base.enabled = enabled;
            }

            /// Get current status
            pub fn status(&self) -> dviz_core::DisplayStatus {
                self.base.status.clone()
            }

            /// Get status message
            pub fn status_message(&self) -> Option<&str> {
                self.base.status_message.as_deref()
            }

            /// Set status
            pub fn set_status(&mut self, status: dviz_core::DisplayStatus, message: Option<String>) {
                self.base.set_status(status, message);
            }

            /// Clear status
            pub fn clear_status(&mut self) {
                self.base.clear_status();
            }

            /// Get display ID
            pub fn id(&self) -> dviz_core::DisplayId {
                self.base.id
            }

            /// Get properties
            pub fn properties(&self) -> &dviz_core::Properties {
                &self.base.properties
            }

            /// Get mutable properties
            pub fn properties_mut(&mut self) -> &mut dviz_core::Properties {
                &mut self.base.properties
            }
        }
    };
}

/// Context for display updates
pub struct DisplayUpdateContext<'a> {
    /// The Rerun recording stream
    pub stream: &'a rerun::RecordingStream,
    /// The transform buffer for looking up transforms
    pub transform_buffer: &'a dviz_transform::TransformBuffer,
    /// Fixed frame ID
    pub fixed_frame: &'a dviz_core::FrameId,
    /// Current time
    pub current_time: &'a dviz_core::Timestamp,
    /// Wall clock delta time
    pub wall_dt: Duration,
}

impl<'a> DisplayUpdateContext<'a> {
    /// Create new display context
    pub fn new(
        stream: &'a rerun::RecordingStream,
        transform_buffer: &'a dviz_transform::TransformBuffer,
        fixed_frame: &'a dviz_core::FrameId,
        current_time: &'a dviz_core::Timestamp,
        wall_dt: Duration,
    ) -> Self {
        Self {
            stream,
            transform_buffer,
            fixed_frame,
            current_time,
            wall_dt,
        }
    }

    /// Get the recording stream
    pub fn recording_stream(&self) -> &rerun::RecordingStream {
        self.stream
    }

    /// Look up transform for a frame to fixed frame
    pub fn lookup_transform(&self, frame: &str) -> Option<dviz_core::types::Transform> {
        let frame_id = dviz_core::FrameId::new(frame);
        self.transform_buffer
            .lookup_transform(self.fixed_frame, &frame_id, self.current_time)
            .ok()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_display_creation() {
        let base = BaseDisplay::new("Test Display", "test_display");
        assert_eq!(base.name, "Test Display");
        assert_eq!(base.type_name, "test_display");
        assert_eq!(base.entity_path, "/test_display");
        assert!(base.enabled);
        assert_eq!(base.status, DisplayStatus::Ok);
    }

    #[test]
    fn test_base_display_with_entity_path() {
        let base = BaseDisplay::new("Test", "test")
            .with_entity_path("/custom/path");
        assert_eq!(base.entity_path, "/custom/path");
    }

    #[test]
    fn test_base_display_status() {
        let mut base = BaseDisplay::new("Test", "test");

        base.set_status(DisplayStatus::Warning, Some("Low data rate".into()));
        assert_eq!(base.status, DisplayStatus::Warning);
        assert_eq!(base.status_message.as_deref(), Some("Low data rate"));

        base.clear_status();
        assert_eq!(base.status, DisplayStatus::Ok);
        assert!(base.status_message.is_none());
    }

    #[test]
    fn test_base_display_properties() {
        let mut base = BaseDisplay::new("Test", "test");
        base.add_property("alpha", 1.0f64);
        assert!(base.properties.get("alpha").is_some());
    }
}
