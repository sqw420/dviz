//! Robot Model Types
//!
//! Data structures representing a robot parsed from URDF.

use glam::{Quat, Vec3};
use dviz_core::types::Transform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete robot description parsed from URDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotDescription {
    /// Robot name
    pub name: String,
    /// Links by name
    pub links: HashMap<String, Link>,
    /// Joints by name
    pub joints: HashMap<String, Joint>,
    /// Root link name (link with no parent)
    pub root_link: Option<String>,
    /// Materials defined in URDF
    pub materials: HashMap<String, Material>,
}

impl RobotDescription {
    /// Create a new empty robot description
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            links: HashMap::new(),
            joints: HashMap::new(),
            root_link: None,
            materials: HashMap::new(),
        }
    }

    /// Get all link names
    pub fn link_names(&self) -> Vec<&str> {
        self.links.keys().map(|s| s.as_str()).collect()
    }

    /// Get all joint names
    pub fn joint_names(&self) -> Vec<&str> {
        self.joints.keys().map(|s| s.as_str()).collect()
    }

    /// Find the parent link of a given link
    pub fn parent_link(&self, link_name: &str) -> Option<&str> {
        for joint in self.joints.values() {
            if joint.child_link == link_name {
                return Some(&joint.parent_link);
            }
        }
        None
    }

    /// Find the joint connecting a child link to its parent
    pub fn parent_joint(&self, link_name: &str) -> Option<&Joint> {
        for joint in self.joints.values() {
            if joint.child_link == link_name {
                return Some(joint);
            }
        }
        None
    }

    /// Get child links of a given link
    pub fn child_links(&self, link_name: &str) -> Vec<&str> {
        let mut children = Vec::new();
        for joint in self.joints.values() {
            if joint.parent_link == link_name {
                children.push(joint.child_link.as_str());
            }
        }
        children
    }

    /// Get kinematic chain from root to a link
    pub fn chain_to_link<'a>(&'a self, link_name: &'a str) -> Vec<&'a str> {
        let mut chain = Vec::new();
        let mut current = link_name;

        while let Some(parent) = self.parent_link(current) {
            chain.push(current);
            current = parent;
        }
        chain.push(current); // Add root
        chain.reverse();
        chain
    }
}

/// Robot link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Link name
    pub name: String,
    /// Visual geometry (for rendering)
    pub visual: Option<Visual>,
    /// Collision geometry (for physics)
    pub collision: Option<Collision>,
    /// Inertial properties
    pub inertial: Option<Inertial>,
}

impl Link {
    /// Create a new link
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            visual: None,
            collision: None,
            inertial: None,
        }
    }
}

/// Visual properties of a link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visual {
    /// Name (optional)
    pub name: Option<String>,
    /// Transform from link origin
    pub origin: Transform,
    /// Geometry shape
    pub geometry: Geometry,
    /// Material for coloring
    pub material: Option<Material>,
}

/// Collision properties of a link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collision {
    /// Name (optional)
    pub name: Option<String>,
    /// Transform from link origin
    pub origin: Transform,
    /// Geometry shape
    pub geometry: Geometry,
}

/// Inertial properties of a link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inertial {
    /// Transform of center of mass
    pub origin: Transform,
    /// Mass in kg
    pub mass: f32,
    /// Inertia matrix (ixx, ixy, ixz, iyy, iyz, izz)
    pub inertia: [f32; 6],
}

/// Geometry shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    /// Box with half-extents
    Box { size: Vec3 },
    /// Cylinder along Z axis
    Cylinder { radius: f32, length: f32 },
    /// Sphere
    Sphere { radius: f32 },
    /// Mesh from file
    Mesh { filename: String, scale: Vec3 },
}

impl Geometry {
    /// Create a box geometry
    pub fn box_shape(x: f32, y: f32, z: f32) -> Self {
        Geometry::Box { size: Vec3::new(x, y, z) }
    }

    /// Create a cylinder geometry
    pub fn cylinder(radius: f32, length: f32) -> Self {
        Geometry::Cylinder { radius, length }
    }

    /// Create a sphere geometry
    pub fn sphere(radius: f32) -> Self {
        Geometry::Sphere { radius }
    }

    /// Create a mesh geometry
    pub fn mesh(filename: &str) -> Self {
        Geometry::Mesh {
            filename: filename.to_string(),
            scale: Vec3::ONE,
        }
    }

    /// Create a mesh geometry with scale
    pub fn mesh_scaled(filename: &str, scale: Vec3) -> Self {
        Geometry::Mesh {
            filename: filename.to_string(),
            scale,
        }
    }
}

/// Material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Material name
    pub name: String,
    /// RGBA color (0-1 range)
    pub color: Option<[f32; 4]>,
    /// Texture filename
    pub texture: Option<String>,
}

impl Material {
    /// Create a colored material
    pub fn color(name: &str, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            name: name.to_string(),
            color: Some([r, g, b, a]),
            texture: None,
        }
    }

    /// Create a textured material
    pub fn textured(name: &str, texture_file: &str) -> Self {
        Self {
            name: name.to_string(),
            color: None,
            texture: Some(texture_file.to_string()),
        }
    }
}

/// Robot joint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// Joint name
    pub name: String,
    /// Joint type
    pub joint_type: JointType,
    /// Parent link name
    pub parent_link: String,
    /// Child link name
    pub child_link: String,
    /// Transform from parent link to joint
    pub origin: Transform,
    /// Joint axis (for revolute/prismatic)
    pub axis: Vec3,
    /// Joint limits
    pub limits: Option<JointLimits>,
    /// Mimic joint (copies another joint's motion)
    pub mimic: Option<MimicJoint>,
}

impl Joint {
    /// Create a fixed joint
    pub fn fixed(name: &str, parent: &str, child: &str) -> Self {
        Self {
            name: name.to_string(),
            joint_type: JointType::Fixed,
            parent_link: parent.to_string(),
            child_link: child.to_string(),
            origin: Transform::IDENTITY,
            axis: Vec3::Z,
            limits: None,
            mimic: None,
        }
    }

    /// Create a revolute joint
    pub fn revolute(name: &str, parent: &str, child: &str, axis: Vec3) -> Self {
        Self {
            name: name.to_string(),
            joint_type: JointType::Revolute,
            parent_link: parent.to_string(),
            child_link: child.to_string(),
            origin: Transform::IDENTITY,
            axis,
            limits: None,
            mimic: None,
        }
    }

    /// Set joint origin
    pub fn with_origin(mut self, origin: Transform) -> Self {
        self.origin = origin;
        self
    }

    /// Set joint limits
    pub fn with_limits(mut self, limits: JointLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Compute transform for a given joint position
    pub fn transform_at(&self, position: f32) -> Transform {
        match self.joint_type {
            JointType::Fixed => self.origin,
            JointType::Revolute | JointType::Continuous => {
                let rotation = Quat::from_axis_angle(self.axis, position);
                Transform::new(self.origin.translation, self.origin.rotation * rotation)
            }
            JointType::Prismatic => {
                let translation = self.origin.translation + self.axis * position;
                Transform::new(translation, self.origin.rotation)
            }
            JointType::Floating | JointType::Planar => {
                // These require 6DOF/3DOF input, return origin for now
                self.origin
            }
        }
    }
}

/// Joint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JointType {
    /// Fixed joint (no motion)
    Fixed,
    /// Revolute joint (rotation with limits)
    Revolute,
    /// Continuous joint (rotation without limits)
    Continuous,
    /// Prismatic joint (linear motion)
    Prismatic,
    /// Floating joint (6 DOF)
    Floating,
    /// Planar joint (motion in XY plane)
    Planar,
}

/// Joint limits
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JointLimits {
    /// Lower limit (radians or meters)
    pub lower: f32,
    /// Upper limit (radians or meters)
    pub upper: f32,
    /// Maximum effort (Nm or N)
    pub effort: f32,
    /// Maximum velocity (rad/s or m/s)
    pub velocity: f32,
}

impl JointLimits {
    /// Create joint limits
    pub fn new(lower: f32, upper: f32, effort: f32, velocity: f32) -> Self {
        Self { lower, upper, effort, velocity }
    }

    /// Clamp a joint position to limits
    pub fn clamp(&self, position: f32) -> f32 {
        position.clamp(self.lower, self.upper)
    }
}

/// Mimic joint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimicJoint {
    /// Name of the joint to mimic
    pub joint: String,
    /// Multiplier
    pub multiplier: f32,
    /// Offset
    pub offset: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_description() {
        let mut robot = RobotDescription::new("test_robot");

        robot.links.insert("base_link".to_string(), Link::new("base_link"));
        robot.links.insert("link1".to_string(), Link::new("link1"));
        robot.joints.insert("joint1".to_string(), Joint::fixed("joint1", "base_link", "link1"));
        robot.root_link = Some("base_link".to_string());

        assert_eq!(robot.link_names().len(), 2);
        assert_eq!(robot.joint_names().len(), 1);
        assert_eq!(robot.parent_link("link1"), Some("base_link"));
        assert_eq!(robot.child_links("base_link"), vec!["link1"]);
    }

    #[test]
    fn test_revolute_joint_transform() {
        let joint = Joint::revolute("j1", "p", "c", Vec3::Z);

        // Zero position
        let t0 = joint.transform_at(0.0);
        assert!((t0.rotation.w - 1.0).abs() < 1e-6);

        // 90 degree rotation
        let t90 = joint.transform_at(std::f32::consts::FRAC_PI_2);
        assert!((t90.rotation.z - (std::f32::consts::FRAC_PI_4).sin()).abs() < 1e-5);
    }

    #[test]
    fn test_geometry_constructors() {
        let box_geom = Geometry::box_shape(1.0, 2.0, 3.0);
        if let Geometry::Box { size } = box_geom {
            assert_eq!(size, Vec3::new(1.0, 2.0, 3.0));
        } else {
            panic!("Expected box geometry");
        }

        let cyl = Geometry::cylinder(0.5, 1.0);
        if let Geometry::Cylinder { radius, length } = cyl {
            assert_eq!(radius, 0.5);
            assert_eq!(length, 1.0);
        } else {
            panic!("Expected cylinder geometry");
        }
    }

    #[test]
    fn test_chain_to_link() {
        let mut robot = RobotDescription::new("arm");

        robot.links.insert("base".to_string(), Link::new("base"));
        robot.links.insert("link1".to_string(), Link::new("link1"));
        robot.links.insert("link2".to_string(), Link::new("link2"));
        robot.links.insert("link3".to_string(), Link::new("link3"));

        robot.joints.insert("j1".to_string(), Joint::fixed("j1", "base", "link1"));
        robot.joints.insert("j2".to_string(), Joint::fixed("j2", "link1", "link2"));
        robot.joints.insert("j3".to_string(), Joint::fixed("j3", "link2", "link3"));
        robot.root_link = Some("base".to_string());

        let chain = robot.chain_to_link("link3");
        assert_eq!(chain, vec!["base", "link1", "link2", "link3"]);
    }
}
