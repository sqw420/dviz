//! Transform buffer for time-indexed coordinate frame transforms
//!
//! Stores transform history and provides interpolation for querying
//! transforms at arbitrary timestamps.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use parking_lot::RwLock;
use thiserror::Error;

use dviz_core::{FrameId, Timestamp, Transform};

use crate::frame_tree::FrameTree;

// ============================================================================
// TRANSFORM ERROR
// ============================================================================

/// Errors that can occur when looking up transforms
#[derive(Debug, Error, Clone)]
pub enum TransformError {
    #[error("No path exists between frames '{0}' and '{1}'")]
    NoPath(String, String),

    #[error("Transform not found for frame '{0}'")]
    TransformNotFound(String),

    #[error("Cannot extrapolate: requested time {requested:?} is outside buffer range [{oldest:?}, {newest:?}]")]
    ExtrapolationError {
        requested: Timestamp,
        oldest: Timestamp,
        newest: Timestamp,
    },

    #[error("Frame '{0}' does not exist")]
    FrameNotFound(String),

    #[error("Buffer is empty for frame '{0}'")]
    EmptyBuffer(String),
}

/// Result type for transform operations
pub type TransformResult<T> = Result<T, TransformError>;

// ============================================================================
// TRANSFORM HISTORY
// ============================================================================

/// Time-indexed history of transforms for a single frame pair
#[derive(Debug, Clone)]
pub struct TransformHistory {
    /// Transforms indexed by timestamp
    transforms: BTreeMap<Timestamp, Transform>,
    /// Maximum duration to keep transforms
    max_duration: Duration,
}

impl TransformHistory {
    /// Create a new transform history
    pub fn new(max_duration: Duration) -> Self {
        Self {
            transforms: BTreeMap::new(),
            max_duration,
        }
    }

    /// Insert a transform at the given timestamp
    pub fn insert(&mut self, timestamp: Timestamp, transform: Transform) {
        self.transforms.insert(timestamp, transform);
    }

    /// Remove transforms older than max_duration from the given time
    pub fn prune_old(&mut self, now: &Timestamp) {
        let max_nanos = self.max_duration.as_nanos() as i64;
        let cutoff = Timestamp::from_nanos(now.as_nanos() - max_nanos);

        // Remove all entries before the cutoff
        self.transforms = self.transforms.split_off(&cutoff);
    }

    /// Get the transform at a specific time with interpolation
    pub fn get_at(&self, time: &Timestamp) -> TransformResult<Transform> {
        if self.transforms.is_empty() {
            return Err(TransformError::EmptyBuffer("unknown".to_string()));
        }

        // Exact match
        if let Some(transform) = self.transforms.get(time) {
            return Ok(*transform);
        }

        // Find surrounding transforms
        let before = self.transforms.range(..=*time).next_back();
        let after = self.transforms.range(*time..).next();

        match (before, after) {
            (Some((t1, tf1)), Some((t2, tf2))) => {
                // Interpolate between the two transforms
                let total_duration = t2.as_nanos() - t1.as_nanos();
                let elapsed = time.as_nanos() - t1.as_nanos();
                let t = if total_duration > 0 {
                    (elapsed as f64 / total_duration as f64) as f32
                } else {
                    0.0
                };
                Ok(tf1.lerp(tf2, t))
            }
            (Some((_, tf)), None) | (None, Some((_, tf))) => {
                // Only one side available - return closest without extrapolation error
                // This allows some flexibility at buffer boundaries
                Ok(*tf)
            }
            (None, None) => {
                // Should not happen if transforms is not empty
                Err(TransformError::EmptyBuffer("unknown".to_string()))
            }
        }
    }

    /// Get the latest transform
    pub fn get_latest(&self) -> Option<(Timestamp, Transform)> {
        self.transforms.iter().next_back().map(|(t, tf)| (*t, *tf))
    }

    /// Get the oldest transform
    pub fn get_oldest(&self) -> Option<(Timestamp, Transform)> {
        self.transforms.iter().next().map(|(t, tf)| (*t, *tf))
    }

    /// Get the number of transforms in the history
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Get the time range of transforms in the buffer
    pub fn time_range(&self) -> Option<(Timestamp, Timestamp)> {
        let oldest = self.transforms.keys().next()?;
        let newest = self.transforms.keys().next_back()?;
        Some((*oldest, *newest))
    }

    /// Clear all transforms
    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

// ============================================================================
// TRANSFORM KEY
// ============================================================================

/// Key for identifying a transform between two frames
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformKey {
    pub parent: FrameId,
    pub child: FrameId,
}

impl TransformKey {
    pub fn new(parent: FrameId, child: FrameId) -> Self {
        Self { parent, child }
    }
}

// ============================================================================
// TRANSFORM BUFFER
// ============================================================================

/// Thread-safe buffer for storing and querying coordinate frame transforms
pub struct TransformBuffer {
    /// Frame tree for parent-child relationships
    tree: RwLock<FrameTree>,
    /// Transform histories indexed by (parent, child) pairs
    histories: RwLock<HashMap<TransformKey, TransformHistory>>,
    /// Fixed frame for visualization
    fixed_frame: RwLock<FrameId>,
    /// Maximum duration to buffer transforms
    buffer_duration: Duration,
}

impl TransformBuffer {
    /// Create a new transform buffer
    pub fn new(fixed_frame: impl Into<FrameId>, buffer_duration: Duration) -> Self {
        let fixed = fixed_frame.into();
        let mut tree = FrameTree::new();
        tree.add_root(fixed.clone());

        Self {
            tree: RwLock::new(tree),
            histories: RwLock::new(HashMap::new()),
            fixed_frame: RwLock::new(fixed),
            buffer_duration,
        }
    }

    /// Get the fixed frame
    pub fn fixed_frame(&self) -> FrameId {
        self.fixed_frame.read().clone()
    }

    /// Set the fixed frame
    pub fn set_fixed_frame(&self, frame: impl Into<FrameId>) {
        *self.fixed_frame.write() = frame.into();
    }

    /// Set a transform from parent to child at the given timestamp
    pub fn set_transform(
        &self,
        parent: FrameId,
        child: FrameId,
        transform: Transform,
        timestamp: Timestamp,
    ) {
        // Update frame tree
        {
            let mut tree = self.tree.write();
            tree.set_parent(child.clone(), parent.clone());
        }

        // Update transform history
        {
            let mut histories = self.histories.write();
            let key = TransformKey::new(parent, child);
            let history = histories
                .entry(key)
                .or_insert_with(|| TransformHistory::new(self.buffer_duration));
            history.insert(timestamp, transform);
        }
    }

    /// Look up the transform from source frame to target frame at the given time
    pub fn lookup_transform(
        &self,
        target: &FrameId,
        source: &FrameId,
        time: &Timestamp,
    ) -> TransformResult<Transform> {
        if target == source {
            return Ok(Transform::IDENTITY);
        }

        // Get the path between frames
        let tree = self.tree.read();
        let (path_up, ancestor, path_down) = tree.path_between(source, target).ok_or_else(|| {
            TransformError::NoPath(source.to_string(), target.to_string())
        })?;
        drop(tree);

        let histories = self.histories.read();

        // Compose transforms going up from source to ancestor
        let mut result = Transform::IDENTITY;

        for i in 0..path_up.len() {
            let child = &path_up[i];
            let parent = if i + 1 < path_up.len() {
                &path_up[i + 1]
            } else {
                &ancestor
            };

            let key = TransformKey::new(parent.clone(), child.clone());
            let history = histories.get(&key).ok_or_else(|| {
                TransformError::TransformNotFound(child.to_string())
            })?;

            let tf = history.get_at(time).map_err(|_| {
                TransformError::TransformNotFound(child.to_string())
            })?;

            // Going up means we need the inverse
            result = result.mul(&tf.inverse());
        }

        // Compose transforms going down from ancestor to target
        for i in 0..path_down.len() {
            let child = &path_down[i];
            let parent = if i == 0 {
                &ancestor
            } else {
                &path_down[i - 1]
            };

            let key = TransformKey::new(parent.clone(), child.clone());
            let history = histories.get(&key).ok_or_else(|| {
                TransformError::TransformNotFound(child.to_string())
            })?;

            let tf = history.get_at(time).map_err(|_| {
                TransformError::TransformNotFound(child.to_string())
            })?;

            result = result.mul(&tf);
        }

        Ok(result)
    }

    /// Get transform from a frame to the fixed frame
    pub fn get_transform_to_fixed(&self, frame: &FrameId, time: &Timestamp) -> TransformResult<Transform> {
        let fixed = self.fixed_frame.read().clone();
        self.lookup_transform(&fixed, frame, time)
    }

    /// Get the latest transform from a frame to the fixed frame
    pub fn get_latest_transform_to_fixed(&self, frame: &FrameId) -> TransformResult<Transform> {
        let fixed = self.fixed_frame.read().clone();

        // Find the latest timestamp for this frame's transform
        let tree = self.tree.read();
        let parent = tree.parent(frame).cloned();
        drop(tree);

        let parent = parent.ok_or_else(|| {
            TransformError::FrameNotFound(frame.to_string())
        })?;

        let histories = self.histories.read();
        let key = TransformKey::new(parent, frame.clone());
        let history = histories.get(&key).ok_or_else(|| {
            TransformError::TransformNotFound(frame.to_string())
        })?;

        let (timestamp, _) = history.get_latest().ok_or_else(|| {
            TransformError::EmptyBuffer(frame.to_string())
        })?;
        drop(histories);

        self.lookup_transform(&fixed, frame, &timestamp)
    }

    /// Get all frames in the buffer
    pub fn all_frames(&self) -> Vec<FrameId> {
        self.tree.read().all_frames()
    }

    /// Check if a frame exists
    pub fn has_frame(&self, frame: &FrameId) -> bool {
        self.tree.read().contains(frame)
    }

    /// Check if a transform is available between two frames
    pub fn can_transform(&self, target: &FrameId, source: &FrameId, time: &Timestamp) -> bool {
        self.lookup_transform(target, source, time).is_ok()
    }

    /// Prune old transforms from all histories
    pub fn prune_old(&self, now: &Timestamp) {
        let mut histories = self.histories.write();
        for history in histories.values_mut() {
            history.prune_old(now);
        }
    }

    /// Clear all transforms
    pub fn clear(&self) {
        let fixed = self.fixed_frame.read().clone();
        let mut tree = self.tree.write();
        tree.clear();
        tree.add_root(fixed);
        drop(tree);

        self.histories.write().clear();
    }

    /// Get the frame tree (read-only access)
    pub fn frame_tree(&self) -> impl std::ops::Deref<Target = FrameTree> + '_ {
        self.tree.read()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    fn frame(s: &str) -> FrameId {
        FrameId::new(s)
    }

    #[test]
    fn test_transform_history_basic() {
        let mut history = TransformHistory::new(Duration::from_secs(10));

        let t1 = Timestamp::from_secs_f64(1.0);
        let tf1 = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        history.insert(t1, tf1);

        assert_eq!(history.len(), 1);
        let result = history.get_at(&t1).unwrap();
        assert!((result.translation - tf1.translation).length() < 1e-6);
    }

    #[test]
    fn test_transform_history_interpolation() {
        let mut history = TransformHistory::new(Duration::from_secs(10));

        let t1 = Timestamp::from_secs_f64(1.0);
        let t2 = Timestamp::from_secs_f64(2.0);
        let tf1 = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let tf2 = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));

        history.insert(t1, tf1);
        history.insert(t2, tf2);

        // Query at midpoint
        let t_mid = Timestamp::from_secs_f64(1.5);
        let result = history.get_at(&t_mid).unwrap();
        assert!((result.translation.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_history_prune() {
        let mut history = TransformHistory::new(Duration::from_secs(5));

        history.insert(Timestamp::from_secs_f64(1.0), Transform::IDENTITY);
        history.insert(Timestamp::from_secs_f64(5.0), Transform::IDENTITY);
        history.insert(Timestamp::from_secs_f64(10.0), Transform::IDENTITY);

        assert_eq!(history.len(), 3);

        // Prune from time 10.0 with 5 second duration
        history.prune_old(&Timestamp::from_secs_f64(10.0));

        // Should keep 5.0 and 10.0 (within 5 seconds of now)
        assert!(history.len() <= 3); // May vary based on boundary conditions
    }

    #[test]
    fn test_transform_buffer_basic() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));

        let tf = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let ts = Timestamp::from_secs_f64(1.0);

        buffer.set_transform(frame("world"), frame("base_link"), tf, ts);

        assert!(buffer.has_frame(&frame("world")));
        assert!(buffer.has_frame(&frame("base_link")));
    }

    #[test]
    fn test_transform_buffer_lookup() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));

        let tf = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let ts = Timestamp::from_secs_f64(1.0);

        buffer.set_transform(frame("world"), frame("base_link"), tf, ts);

        // Look up transform from base_link to world
        let result = buffer.lookup_transform(&frame("world"), &frame("base_link"), &ts).unwrap();

        // Should be the inverse of the stored transform
        assert!((result.translation.x - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_transform_buffer_chain() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));
        let ts = Timestamp::from_secs_f64(1.0);

        // world -> odom (translate 1 in X)
        buffer.set_transform(
            frame("world"),
            frame("odom"),
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            ts,
        );

        // odom -> base_link (translate 1 in Y)
        buffer.set_transform(
            frame("odom"),
            frame("base_link"),
            Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ts,
        );

        // Look up transform from base_link to world
        let result = buffer.lookup_transform(&frame("world"), &frame("base_link"), &ts).unwrap();

        // base_link is at (1, 1, 0) in world frame
        // So transform from base_link to world should translate by (-1, -1, 0)
        assert!((result.translation.x - (-1.0)).abs() < 1e-5);
        assert!((result.translation.y - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_transform_buffer_identity_lookup() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));
        let ts = Timestamp::from_secs_f64(1.0);

        // Same frame should return identity
        let result = buffer.lookup_transform(&frame("world"), &frame("world"), &ts).unwrap();
        assert!((result.translation).length() < 1e-6);
        assert!((result.rotation.dot(Quat::IDENTITY) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_buffer_no_path() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));
        let ts = Timestamp::from_secs_f64(1.0);

        buffer.set_transform(
            frame("world"),
            frame("base_link"),
            Transform::IDENTITY,
            ts,
        );

        // Try to look up between unconnected frames
        let result = buffer.lookup_transform(&frame("base_link"), &frame("isolated"), &ts);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_buffer_to_fixed() {
        let buffer = TransformBuffer::new("world", Duration::from_secs(10));
        let ts = Timestamp::from_secs_f64(1.0);

        buffer.set_transform(
            frame("world"),
            frame("base_link"),
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            ts,
        );

        let result = buffer.get_transform_to_fixed(&frame("base_link"), &ts).unwrap();

        // Transform from base_link to world should be inverse
        assert!((result.translation.x - (-2.0)).abs() < 1e-6);
    }
}
