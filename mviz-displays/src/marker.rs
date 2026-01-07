//! Marker Display
//!
//! Displays visualization markers with lifetime management.

use std::collections::HashMap;
use mviz_core::{
    DisplayStatus, Marker, MarkerAction, MarkerArray,
};
use mviz_rerun_bridge::MarkerCoreAdapter;
use crate::base::{BaseDisplay, DisplayUpdateContext};

/// Key for identifying markers (namespace + id)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MarkerKey {
    pub ns: String,
    pub id: i32,
}

impl MarkerKey {
    pub fn new(ns: impl Into<String>, id: i32) -> Self {
        Self { ns: ns.into(), id }
    }

    pub fn from_marker(marker: &Marker) -> Self {
        Self::new(&marker.ns, marker.id)
    }
}

/// State for a cached marker
#[derive(Debug)]
struct MarkerState {
    /// The marker data
    marker: Marker,
    /// When the marker was last updated
    last_update: std::time::Instant,
    /// Whether the marker has expired
    expired: bool,
}

/// Marker display properties
#[derive(Debug, Clone)]
pub struct MarkerProperties {
    /// Filter by namespace (None = show all)
    pub namespace_filter: Option<String>,
}

impl Default for MarkerProperties {
    fn default() -> Self {
        Self {
            namespace_filter: None,
        }
    }
}

/// Marker display with lifetime management
pub struct MarkerDisplay {
    /// Base display state
    pub base: BaseDisplay,
    /// Display properties
    pub props: MarkerProperties,
    /// Cached markers by key
    marker_cache: HashMap<MarkerKey, MarkerState>,
    /// Statistics
    active_count: usize,
    expired_count: usize,
}

// Implement common display methods
crate::impl_display_base!(MarkerDisplay);

impl MarkerDisplay {
    /// Create a new marker display
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = BaseDisplay::new(name, "marker");

        // Add properties
        base.add_property("namespace_filter", "");

        Self {
            base,
            props: MarkerProperties::default(),
            marker_cache: HashMap::new(),
            active_count: 0,
            expired_count: 0,
        }
    }

    /// Process a single marker
    pub fn process_marker(&mut self, marker: Marker) {
        let key = MarkerKey::from_marker(&marker);

        match marker.action {
            MarkerAction::Add | MarkerAction::Modify => {
                self.marker_cache.insert(key, MarkerState {
                    marker,
                    last_update: std::time::Instant::now(),
                    expired: false,
                });
            }
            MarkerAction::Delete => {
                self.marker_cache.remove(&key);
            }
            MarkerAction::DeleteAll => {
                // Delete all markers in this namespace
                if marker.ns.is_empty() {
                    self.marker_cache.clear();
                } else {
                    self.marker_cache.retain(|k, _| k.ns != marker.ns);
                }
            }
        }
    }

    /// Process a marker array
    pub fn process_marker_array(&mut self, array: MarkerArray) {
        for marker in array.markers {
            self.process_marker(marker);
        }
    }

    /// Clear all markers in a namespace
    pub fn clear_namespace(&mut self, ns: &str) {
        self.marker_cache.retain(|k, _| k.ns != ns);
    }

    /// Clear all markers
    pub fn clear_all(&mut self) {
        self.marker_cache.clear();
    }

    /// Check and expire old markers based on lifetime
    fn check_lifetimes(&mut self) {
        let now = std::time::Instant::now();

        for state in self.marker_cache.values_mut() {
            if state.marker.lifetime_secs > 0.0 {
                let age = now.duration_since(state.last_update);
                let lifetime = std::time::Duration::from_secs_f32(state.marker.lifetime_secs);
                state.expired = age > lifetime;
            }
        }
    }

    /// Get active markers (not expired and matching filter)
    fn _active_markers(&self) -> impl Iterator<Item = &Marker> {
        self.marker_cache
            .values()
            .filter(|state| {
                if state.expired {
                    return false;
                }

                // Apply namespace filter
                if let Some(ref filter) = self.props.namespace_filter {
                    if !filter.is_empty() && state.marker.ns != *filter {
                        return false;
                    }
                }

                true
            })
            .map(|state| &state.marker)
    }

    /// Set namespace filter
    pub fn set_namespace_filter(&mut self, filter: Option<String>) {
        self.props.namespace_filter = filter;
    }

    /// Get marker count
    pub fn marker_count(&self) -> usize {
        self.marker_cache.len()
    }

    /// Get active marker count
    pub fn active_marker_count(&self) -> usize {
        self.active_count
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Check marker lifetimes
        self.check_lifetimes();

        // Log all active markers
        self.active_count = 0;
        self.expired_count = 0;

        for (key, state) in &self.marker_cache {
            if state.expired {
                self.expired_count += 1;
                continue;
            }

            // Apply namespace filter
            if let Some(ref filter) = self.props.namespace_filter {
                if !filter.is_empty() && state.marker.ns != *filter {
                    continue;
                }
            }

            // Log the marker
            if let Err(e) = MarkerCoreAdapter::log(
                ctx.stream,
                self.entity_path(),
                &state.marker,
            ) {
                log::warn!("Failed to log marker {}:{}: {}", key.ns, key.id, e);
            }

            self.active_count += 1;
        }

        // Remove expired markers
        self.marker_cache.retain(|_, state| !state.expired);

        // Update status
        self.set_status(
            DisplayStatus::Ok,
            Some(format!("{} active, {} expired", self.active_count, self.expired_count)),
        );

        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self, _ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        self.marker_cache.clear();
        self.active_count = 0;
        self.expired_count = 0;
        Ok(())
    }

    /// Reset the display
    pub fn reset(&mut self) {
        self.props = MarkerProperties::default();
        self.marker_cache.clear();
        self.active_count = 0;
        self.expired_count = 0;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mviz_core::{Color, MarkerType, Timestamp};
    use glam::Vec3;

    fn test_marker(ns: &str, id: i32) -> Marker {
        Marker {
            ns: ns.to_string(),
            id,
            marker_type: MarkerType::Cube,
            action: MarkerAction::Add,
            position: Vec3::ZERO,
            orientation: glam::Quat::IDENTITY,
            scale: 1.0.into(),
            color: Color::RED,
            lifetime_secs: 0.0,
            frame_id: Default::default(),
            timestamp: Timestamp::now(),
            points: Vec::new(),
            colors: Vec::new(),
            text: String::new(),
            mesh_resource: String::new(),
            mesh_use_embedded_materials: false,
        }
    }

    #[test]
    fn test_marker_display_creation() {
        let display = MarkerDisplay::new("Markers");
        assert_eq!(display.name(), "Markers");
        assert_eq!(display.type_name(), "marker");
        assert!(display.is_enabled());
    }

    #[test]
    fn test_marker_process_add() {
        let mut display = MarkerDisplay::new("Test");

        display.process_marker(test_marker("ns1", 1));
        display.process_marker(test_marker("ns1", 2));
        display.process_marker(test_marker("ns2", 1));

        assert_eq!(display.marker_count(), 3);
    }

    #[test]
    fn test_marker_process_delete() {
        let mut display = MarkerDisplay::new("Test");

        display.process_marker(test_marker("ns1", 1));
        display.process_marker(test_marker("ns1", 2));
        assert_eq!(display.marker_count(), 2);

        let mut delete = test_marker("ns1", 1);
        delete.action = MarkerAction::Delete;
        display.process_marker(delete);

        assert_eq!(display.marker_count(), 1);
    }

    #[test]
    fn test_marker_process_delete_all() {
        let mut display = MarkerDisplay::new("Test");

        display.process_marker(test_marker("ns1", 1));
        display.process_marker(test_marker("ns1", 2));
        display.process_marker(test_marker("ns2", 1));
        assert_eq!(display.marker_count(), 3);

        let mut delete_all = test_marker("ns1", 0);
        delete_all.action = MarkerAction::DeleteAll;
        display.process_marker(delete_all);

        // Should only delete ns1 markers
        assert_eq!(display.marker_count(), 1);
    }

    #[test]
    fn test_marker_clear_namespace() {
        let mut display = MarkerDisplay::new("Test");

        display.process_marker(test_marker("ns1", 1));
        display.process_marker(test_marker("ns1", 2));
        display.process_marker(test_marker("ns2", 1));

        display.clear_namespace("ns1");
        assert_eq!(display.marker_count(), 1);
    }

    #[test]
    fn test_marker_clear_all() {
        let mut display = MarkerDisplay::new("Test");

        display.process_marker(test_marker("ns1", 1));
        display.process_marker(test_marker("ns2", 1));

        display.clear_all();
        assert_eq!(display.marker_count(), 0);
    }

    #[test]
    fn test_marker_key() {
        let key1 = MarkerKey::new("ns", 1);
        let key2 = MarkerKey::new("ns", 1);
        let key3 = MarkerKey::new("ns", 2);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
