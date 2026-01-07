//! Frame tree for coordinate frame hierarchy
//!
//! Manages parent-child relationships between coordinate frames.

use std::collections::{HashMap, HashSet};

use mviz_core::FrameId;

// ============================================================================
// FRAME NODE
// ============================================================================

/// Node in the frame tree representing a coordinate frame
#[derive(Debug, Clone, Default)]
pub struct FrameNode {
    /// Parent frame (None for root frames)
    pub parent: Option<FrameId>,
    /// Child frames
    pub children: HashSet<FrameId>,
}

impl FrameNode {
    /// Create a new frame node with no parent
    pub fn new() -> Self {
        Self {
            parent: None,
            children: HashSet::new(),
        }
    }

    /// Create a frame node with a parent
    pub fn with_parent(parent: FrameId) -> Self {
        Self {
            parent: Some(parent),
            children: HashSet::new(),
        }
    }

    /// Check if this is a root frame (no parent)
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Check if this frame has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

// ============================================================================
// FRAME TREE
// ============================================================================

/// Tree structure maintaining frame hierarchy
#[derive(Debug, Clone, Default)]
pub struct FrameTree {
    /// Map of frame IDs to their nodes
    nodes: HashMap<FrameId, FrameNode>,
}

impl FrameTree {
    /// Create a new empty frame tree
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Get the number of frames in the tree
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Check if a frame exists in the tree
    pub fn contains(&self, frame: &FrameId) -> bool {
        self.nodes.contains_key(frame)
    }

    /// Get a frame node
    pub fn get(&self, frame: &FrameId) -> Option<&FrameNode> {
        self.nodes.get(frame)
    }

    /// Get the parent of a frame
    pub fn parent(&self, frame: &FrameId) -> Option<&FrameId> {
        self.nodes.get(frame).and_then(|n| n.parent.as_ref())
    }

    /// Get the children of a frame
    pub fn children(&self, frame: &FrameId) -> Option<&HashSet<FrameId>> {
        self.nodes.get(frame).map(|n| &n.children)
    }

    /// Set the parent of a frame, handling reparenting
    ///
    /// If the child frame doesn't exist, it will be created.
    /// If the parent frame doesn't exist, it will be created as a root.
    /// If the child already has a parent, it will be removed from the old parent.
    pub fn set_parent(&mut self, child: FrameId, parent: FrameId) {
        // Ensure parent exists
        if !self.nodes.contains_key(&parent) {
            self.nodes.insert(parent.clone(), FrameNode::new());
        }

        // Get or create child node
        let old_parent = {
            let child_node = self.nodes.entry(child.clone()).or_insert_with(FrameNode::new);
            let old_parent = child_node.parent.clone();
            child_node.parent = Some(parent.clone());
            old_parent
        };

        // Remove from old parent's children
        if let Some(old_parent_id) = old_parent {
            if let Some(old_parent_node) = self.nodes.get_mut(&old_parent_id) {
                old_parent_node.children.remove(&child);
            }
        }

        // Add to new parent's children
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.insert(child);
        }
    }

    /// Add a root frame (frame with no parent)
    pub fn add_root(&mut self, frame: FrameId) {
        self.nodes.entry(frame).or_insert_with(FrameNode::new);
    }

    /// Remove a frame from the tree
    ///
    /// Children of the removed frame become roots.
    pub fn remove(&mut self, frame: &FrameId) -> Option<FrameNode> {
        if let Some(node) = self.nodes.remove(frame) {
            // Remove from parent's children
            if let Some(parent) = &node.parent {
                if let Some(parent_node) = self.nodes.get_mut(parent) {
                    parent_node.children.remove(frame);
                }
            }

            // Make children into roots
            for child in &node.children {
                if let Some(child_node) = self.nodes.get_mut(child) {
                    child_node.parent = None;
                }
            }

            Some(node)
        } else {
            None
        }
    }

    /// Get the path from a frame to the root
    ///
    /// Returns the frame IDs from the given frame up to (and including) the root.
    pub fn path_to_root(&self, frame: &FrameId) -> Vec<FrameId> {
        let mut path = Vec::new();
        let mut current = Some(frame.clone());

        while let Some(frame_id) = current {
            path.push(frame_id.clone());
            current = self.nodes.get(&frame_id).and_then(|n| n.parent.clone());
        }

        path
    }

    /// Find the common ancestor of two frames
    ///
    /// Returns None if the frames are in disconnected trees.
    pub fn common_ancestor(&self, frame_a: &FrameId, frame_b: &FrameId) -> Option<FrameId> {
        let path_a: HashSet<_> = self.path_to_root(frame_a).into_iter().collect();

        // Walk up from frame_b and find the first frame in path_a
        let mut current = Some(frame_b.clone());
        while let Some(frame_id) = current {
            if path_a.contains(&frame_id) {
                return Some(frame_id);
            }
            current = self.nodes.get(&frame_id).and_then(|n| n.parent.clone());
        }

        None
    }

    /// Get the path between two frames
    ///
    /// Returns (path_up, common_ancestor, path_down) where:
    /// - path_up: frames from source to common ancestor (exclusive of ancestor)
    /// - common_ancestor: the common ancestor frame
    /// - path_down: frames from common ancestor to target (exclusive of ancestor)
    pub fn path_between(
        &self,
        source: &FrameId,
        target: &FrameId,
    ) -> Option<(Vec<FrameId>, FrameId, Vec<FrameId>)> {
        let ancestor = self.common_ancestor(source, target)?;

        // Path from source to ancestor
        let mut path_up = Vec::new();
        let mut current = Some(source.clone());
        while let Some(frame_id) = current {
            if frame_id == ancestor {
                break;
            }
            path_up.push(frame_id.clone());
            current = self.nodes.get(&frame_id).and_then(|n| n.parent.clone());
        }

        // Path from target to ancestor (we'll reverse this)
        let mut path_down = Vec::new();
        let mut current = Some(target.clone());
        while let Some(frame_id) = current {
            if frame_id == ancestor {
                break;
            }
            path_down.push(frame_id.clone());
            current = self.nodes.get(&frame_id).and_then(|n| n.parent.clone());
        }
        path_down.reverse();

        Some((path_up, ancestor, path_down))
    }

    /// Get all frames in the tree
    pub fn all_frames(&self) -> Vec<FrameId> {
        self.nodes.keys().cloned().collect()
    }

    /// Get all root frames (frames with no parent)
    pub fn root_frames(&self) -> Vec<FrameId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.is_root())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check if there's a path from source to target
    pub fn has_path(&self, source: &FrameId, target: &FrameId) -> bool {
        self.common_ancestor(source, target).is_some()
    }

    /// Get the depth of a frame (distance from root)
    pub fn depth(&self, frame: &FrameId) -> usize {
        self.path_to_root(frame).len().saturating_sub(1)
    }

    /// Clear all frames from the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Iterate over all frame nodes
    pub fn iter(&self) -> impl Iterator<Item = (&FrameId, &FrameNode)> {
        self.nodes.iter()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(s: &str) -> FrameId {
        FrameId::new(s)
    }

    #[test]
    fn test_empty_tree() {
        let tree = FrameTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_add_root() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));

        assert!(tree.contains(&frame("world")));
        assert_eq!(tree.len(), 1);
        assert!(tree.get(&frame("world")).unwrap().is_root());
    }

    #[test]
    fn test_set_parent() {
        let mut tree = FrameTree::new();
        tree.set_parent(frame("base_link"), frame("world"));

        assert!(tree.contains(&frame("world")));
        assert!(tree.contains(&frame("base_link")));
        assert_eq!(tree.parent(&frame("base_link")), Some(&frame("world")));
        assert!(tree.children(&frame("world")).unwrap().contains(&frame("base_link")));
    }

    #[test]
    fn test_reparenting() {
        let mut tree = FrameTree::new();
        tree.set_parent(frame("child"), frame("parent1"));
        tree.set_parent(frame("child"), frame("parent2"));

        assert_eq!(tree.parent(&frame("child")), Some(&frame("parent2")));
        assert!(!tree.children(&frame("parent1")).unwrap().contains(&frame("child")));
        assert!(tree.children(&frame("parent2")).unwrap().contains(&frame("child")));
    }

    #[test]
    fn test_path_to_root() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));
        tree.set_parent(frame("sensor"), frame("base_link"));

        let path = tree.path_to_root(&frame("sensor"));
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], frame("sensor"));
        assert_eq!(path[1], frame("base_link"));
        assert_eq!(path[2], frame("odom"));
        assert_eq!(path[3], frame("world"));
    }

    #[test]
    fn test_common_ancestor() {
        let mut tree = FrameTree::new();
        // Build tree:
        //         world
        //        /     \
        //     odom    map
        //      |
        //   base_link
        //    /    \
        // left   right
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("map"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));
        tree.set_parent(frame("left"), frame("base_link"));
        tree.set_parent(frame("right"), frame("base_link"));

        // Same subtree
        assert_eq!(
            tree.common_ancestor(&frame("left"), &frame("right")),
            Some(frame("base_link"))
        );

        // Different subtrees under world
        assert_eq!(
            tree.common_ancestor(&frame("left"), &frame("map")),
            Some(frame("world"))
        );

        // Direct ancestor
        assert_eq!(
            tree.common_ancestor(&frame("base_link"), &frame("world")),
            Some(frame("world"))
        );
    }

    #[test]
    fn test_path_between() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));
        tree.set_parent(frame("left"), frame("base_link"));
        tree.set_parent(frame("right"), frame("base_link"));

        let (up, ancestor, down) = tree.path_between(&frame("left"), &frame("right")).unwrap();

        assert_eq!(ancestor, frame("base_link"));
        assert_eq!(up, vec![frame("left")]);
        assert_eq!(down, vec![frame("right")]);
    }

    #[test]
    fn test_remove_frame() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));

        tree.remove(&frame("odom"));

        assert!(!tree.contains(&frame("odom")));
        assert!(!tree.children(&frame("world")).unwrap().contains(&frame("odom")));
        // base_link should now be a root
        assert!(tree.get(&frame("base_link")).unwrap().is_root());
    }

    #[test]
    fn test_depth() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));

        assert_eq!(tree.depth(&frame("world")), 0);
        assert_eq!(tree.depth(&frame("odom")), 1);
        assert_eq!(tree.depth(&frame("base_link")), 2);
    }

    #[test]
    fn test_all_frames() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.set_parent(frame("odom"), frame("world"));
        tree.set_parent(frame("base_link"), frame("odom"));

        let frames = tree.all_frames();
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn test_root_frames() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.add_root(frame("map"));
        tree.set_parent(frame("odom"), frame("world"));

        let roots = tree.root_frames();
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_has_path() {
        let mut tree = FrameTree::new();
        tree.add_root(frame("world"));
        tree.add_root(frame("isolated"));
        tree.set_parent(frame("base_link"), frame("world"));

        assert!(tree.has_path(&frame("base_link"), &frame("world")));
        assert!(!tree.has_path(&frame("base_link"), &frame("isolated")));
    }
}
