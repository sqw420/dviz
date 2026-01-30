//! MViz URDF - URDF parsing and robot model loading
//!
//! This crate provides:
//! - URDF parsing with conversion to MViz types
//! - Mesh loading for STL files
//! - Robot model representation for visualization

pub mod parser;
pub mod mesh_loader;
pub mod robot;

pub use parser::{parse_urdf, parse_urdf_string, UrdfError};
pub use mesh_loader::{MeshData, MeshLoader, StlLoader};
pub use robot::{
    RobotDescription, Link, Visual, Collision, Inertial,
    Geometry, Material, Joint, JointType, JointLimits,
};
