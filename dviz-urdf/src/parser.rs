//! URDF Parser
//!
//! Parses URDF XML files and converts to MViz robot types.

use crate::robot::{
    Collision, Geometry, Inertial, Joint, JointLimits, JointType,
    Link, Material, MimicJoint, RobotDescription, Visual,
};
use glam::{Quat, Vec3};
use dviz_core::types::Transform;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// URDF parsing errors
#[derive(Debug, Error)]
pub enum UrdfError {
    #[error("Failed to read URDF file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse URDF: {0}")]
    ParseError(String),

    #[error("Invalid URDF structure: {0}")]
    InvalidStructure(String),

    #[error("Missing required element: {0}")]
    MissingElement(String),
}

/// Parse URDF from a file path
pub fn parse_urdf<P: AsRef<Path>>(path: P) -> Result<RobotDescription, UrdfError> {
    let content = std::fs::read_to_string(path)?;
    parse_urdf_string(&content)
}

/// Parse URDF from a string
pub fn parse_urdf_string(urdf_xml: &str) -> Result<RobotDescription, UrdfError> {
    let urdf_robot = urdf_rs::read_from_string(urdf_xml)
        .map_err(|e| UrdfError::ParseError(e.to_string()))?;

    convert_robot(&urdf_robot)
}

/// Convert urdf_rs::Robot to our RobotDescription
fn convert_robot(urdf: &urdf_rs::Robot) -> Result<RobotDescription, UrdfError> {
    let mut robot = RobotDescription::new(&urdf.name);

    // Convert materials first (they may be referenced by links)
    for material in &urdf.materials {
        let mat = convert_material(material);
        robot.materials.insert(mat.name.clone(), mat);
    }

    // Convert links
    for link in &urdf.links {
        let converted = convert_link(link, &robot.materials)?;
        robot.links.insert(converted.name.clone(), converted);
    }

    // Convert joints
    for joint in &urdf.joints {
        let converted = convert_joint(joint)?;
        robot.joints.insert(converted.name.clone(), converted);
    }

    // Find root link (link with no parent joint)
    let child_links: std::collections::HashSet<_> = robot.joints.values()
        .map(|j| j.child_link.as_str())
        .collect();

    for link_name in robot.links.keys() {
        if !child_links.contains(link_name.as_str()) {
            robot.root_link = Some(link_name.clone());
            break;
        }
    }

    Ok(robot)
}

/// Convert urdf_rs::Link to our Link
fn convert_link(
    link: &urdf_rs::Link,
    materials: &HashMap<String, Material>,
) -> Result<Link, UrdfError> {
    let mut converted = Link::new(&link.name);

    // Convert visual (urdf-rs uses Vec, take first if any)
    if let Some(visual) = link.visual.first() {
        converted.visual = Some(convert_visual(visual, materials)?);
    }

    // Convert collision (urdf-rs uses Vec, take first if any)
    if let Some(collision) = link.collision.first() {
        converted.collision = Some(convert_collision(collision)?);
    }

    // Convert inertial
    converted.inertial = Some(convert_inertial(&link.inertial)?);

    Ok(converted)
}

/// Convert urdf_rs::Visual to our Visual
fn convert_visual(
    visual: &urdf_rs::Visual,
    materials: &HashMap<String, Material>,
) -> Result<Visual, UrdfError> {
    let origin = convert_pose(&visual.origin);
    let geometry = convert_geometry(&visual.geometry)?;

    // Handle material: either inline or reference
    let material = visual.material.as_ref().map(|mat| {
        if mat.color.is_some() || mat.texture.is_some() {
            // Inline material definition
            convert_material(mat)
        } else {
            // Reference to global material
            materials.get(&mat.name).cloned().unwrap_or_else(|| Material {
                name: mat.name.clone(),
                color: Some([0.8, 0.8, 0.8, 1.0]), // Default gray
                texture: None,
            })
        }
    });

    Ok(Visual {
        name: visual.name.clone(),
        origin,
        geometry,
        material,
    })
}

/// Convert urdf_rs::Collision to our Collision
fn convert_collision(collision: &urdf_rs::Collision) -> Result<Collision, UrdfError> {
    let origin = convert_pose(&collision.origin);
    let geometry = convert_geometry(&collision.geometry)?;

    Ok(Collision {
        name: collision.name.clone(),
        origin,
        geometry,
    })
}

/// Convert urdf_rs::Inertial to our Inertial
fn convert_inertial(inertial: &urdf_rs::Inertial) -> Result<Inertial, UrdfError> {
    let origin = convert_pose(&inertial.origin);

    Ok(Inertial {
        origin,
        mass: inertial.mass.value as f32,
        inertia: [
            inertial.inertia.ixx as f32,
            inertial.inertia.ixy as f32,
            inertial.inertia.ixz as f32,
            inertial.inertia.iyy as f32,
            inertial.inertia.iyz as f32,
            inertial.inertia.izz as f32,
        ],
    })
}

/// Convert urdf_rs::Geometry to our Geometry
fn convert_geometry(geom: &urdf_rs::Geometry) -> Result<Geometry, UrdfError> {
    match geom {
        urdf_rs::Geometry::Box { size } => {
            Ok(Geometry::Box {
                size: Vec3::new(size[0] as f32, size[1] as f32, size[2] as f32),
            })
        }
        urdf_rs::Geometry::Cylinder { radius, length } => {
            Ok(Geometry::Cylinder {
                radius: *radius as f32,
                length: *length as f32,
            })
        }
        urdf_rs::Geometry::Sphere { radius } => {
            Ok(Geometry::Sphere { radius: *radius as f32 })
        }
        urdf_rs::Geometry::Mesh { filename, scale } => {
            let scale_vec = scale.map(|s| Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32))
                .unwrap_or(Vec3::ONE);
            Ok(Geometry::Mesh {
                filename: filename.clone(),
                scale: scale_vec,
            })
        }
        _ => Err(UrdfError::InvalidStructure("Unknown geometry type".to_string())),
    }
}

/// Convert urdf_rs::Material to our Material
fn convert_material(material: &urdf_rs::Material) -> Material {
    let color = material.color.as_ref().map(|c| {
        [c.rgba[0] as f32, c.rgba[1] as f32, c.rgba[2] as f32, c.rgba[3] as f32]
    });

    let texture = material.texture.as_ref().map(|t| t.filename.clone());

    Material {
        name: material.name.clone(),
        color,
        texture,
    }
}

/// Convert urdf_rs::Joint to our Joint
fn convert_joint(joint: &urdf_rs::Joint) -> Result<Joint, UrdfError> {
    let joint_type = match joint.joint_type {
        urdf_rs::JointType::Fixed => JointType::Fixed,
        urdf_rs::JointType::Revolute => JointType::Revolute,
        urdf_rs::JointType::Continuous => JointType::Continuous,
        urdf_rs::JointType::Prismatic => JointType::Prismatic,
        urdf_rs::JointType::Floating => JointType::Floating,
        urdf_rs::JointType::Planar => JointType::Planar,
        urdf_rs::JointType::Spherical => JointType::Floating, // Treat spherical as floating
    };

    let origin = convert_pose(&joint.origin);

    let axis = Vec3::new(
        joint.axis.xyz[0] as f32,
        joint.axis.xyz[1] as f32,
        joint.axis.xyz[2] as f32,
    );

    // Create limits for joints that have them (revolute, prismatic)
    let limits = match joint.joint_type {
        urdf_rs::JointType::Revolute | urdf_rs::JointType::Prismatic => {
            Some(JointLimits {
                lower: joint.limit.lower as f32,
                upper: joint.limit.upper as f32,
                effort: joint.limit.effort as f32,
                velocity: joint.limit.velocity as f32,
            })
        }
        _ => None,
    };

    let mimic = joint.mimic.as_ref().map(|m| MimicJoint {
        joint: m.joint.clone(),
        multiplier: m.multiplier.unwrap_or(1.0) as f32,
        offset: m.offset.unwrap_or(0.0) as f32,
    });

    Ok(Joint {
        name: joint.name.clone(),
        joint_type,
        parent_link: joint.parent.link.clone(),
        child_link: joint.child.link.clone(),
        origin,
        axis,
        limits,
        mimic,
    })
}

/// Convert urdf_rs::Pose to our Transform
fn convert_pose(pose: &urdf_rs::Pose) -> Transform {
    let translation = Vec3::new(
        pose.xyz[0] as f32,
        pose.xyz[1] as f32,
        pose.xyz[2] as f32,
    );

    // URDF uses RPY (roll, pitch, yaw) in radians
    let roll = pose.rpy[0] as f32;
    let pitch = pose.rpy[1] as f32;
    let yaw = pose.rpy[2] as f32;

    // Convert RPY to quaternion
    // Order: first roll (X), then pitch (Y), then yaw (Z)
    let rotation = Quat::from_euler(glam::EulerRot::XYZ, roll, pitch, yaw);

    Transform::new(translation, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_URDF: &str = r#"<?xml version="1.0"?>
<robot name="simple_robot">
  <material name="blue">
    <color rgba="0.0 0.0 0.8 1.0"/>
  </material>

  <link name="base_link">
    <visual>
      <geometry>
        <box size="1.0 0.5 0.2"/>
      </geometry>
      <material name="blue"/>
    </visual>
  </link>

  <link name="link1">
    <visual>
      <origin xyz="0 0 0.5" rpy="0 0 0"/>
      <geometry>
        <cylinder radius="0.1" length="1.0"/>
      </geometry>
    </visual>
  </link>

  <joint name="joint1" type="revolute">
    <parent link="base_link"/>
    <child link="link1"/>
    <origin xyz="0 0 0.1" rpy="0 0 0"/>
    <axis xyz="0 0 1"/>
    <limit lower="-1.57" upper="1.57" effort="10" velocity="1.0"/>
  </joint>
</robot>
"#;

    #[test]
    fn test_parse_simple_urdf() {
        let robot = parse_urdf_string(SIMPLE_URDF).unwrap();

        assert_eq!(robot.name, "simple_robot");
        assert_eq!(robot.links.len(), 2);
        assert_eq!(robot.joints.len(), 1);
        assert_eq!(robot.root_link, Some("base_link".to_string()));
    }

    #[test]
    fn test_parse_joint() {
        let robot = parse_urdf_string(SIMPLE_URDF).unwrap();

        let joint = robot.joints.get("joint1").unwrap();
        assert_eq!(joint.joint_type, JointType::Revolute);
        assert_eq!(joint.parent_link, "base_link");
        assert_eq!(joint.child_link, "link1");
        assert_eq!(joint.axis, Vec3::Z);

        let limits = joint.limits.as_ref().unwrap();
        assert!((limits.lower - (-1.57)).abs() < 0.01);
        assert!((limits.upper - 1.57).abs() < 0.01);
    }

    #[test]
    fn test_parse_geometry() {
        let robot = parse_urdf_string(SIMPLE_URDF).unwrap();

        let base_link = robot.links.get("base_link").unwrap();
        let visual = base_link.visual.as_ref().unwrap();

        if let Geometry::Box { size } = &visual.geometry {
            assert_eq!(size.x, 1.0);
            assert_eq!(size.y, 0.5);
            assert_eq!(size.z, 0.2);
        } else {
            panic!("Expected box geometry");
        }

        let link1 = robot.links.get("link1").unwrap();
        let visual1 = link1.visual.as_ref().unwrap();

        if let Geometry::Cylinder { radius, length } = &visual1.geometry {
            assert_eq!(*radius, 0.1);
            assert_eq!(*length, 1.0);
        } else {
            panic!("Expected cylinder geometry");
        }
    }

    #[test]
    fn test_parse_material() {
        let robot = parse_urdf_string(SIMPLE_URDF).unwrap();

        let material = robot.materials.get("blue").unwrap();
        let color = material.color.unwrap();
        assert_eq!(color[0], 0.0);
        assert_eq!(color[1], 0.0);
        assert_eq!(color[2], 0.8);
        assert_eq!(color[3], 1.0);
    }

    #[test]
    fn test_convert_pose() {
        let pose = urdf_rs::Pose {
            xyz: urdf_rs::Vec3([1.0, 2.0, 3.0]),
            rpy: urdf_rs::Vec3([0.0, 0.0, std::f64::consts::FRAC_PI_2]),
        };

        let transform = convert_pose(&pose);
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));

        // 90 degree yaw rotation
        let rotated = transform.rotation * Vec3::X;
        assert!((rotated.x - 0.0).abs() < 1e-5);
        assert!((rotated.y - 1.0).abs() < 1e-5);
    }

#[test]
fn test_car_urdf_fk() {
    use std::collections::HashMap;
    let robot = parse_urdf("/home/demo/dviz/car.urdf").unwrap();
    
    // Simulate FK BFS
    let mut link_world: HashMap<String, (f32,f32,f32)> = HashMap::new();
    let root = robot.root_link.clone().unwrap();
    let mut queue = vec![root.clone()];
    link_world.insert(root, (0.0,0.0,0.0));
    while let Some(link_name) = queue.first().cloned() {
        queue.remove(0);
        let (px,py,pz) = *link_world.get(&link_name).unwrap();
        for child in robot.child_links(&link_name) {
            if let Some(joint) = robot.parent_joint(child) {
                let t = joint.origin.translation;
                let child_t = (px+t.x, py+t.y, pz+t.z);
                eprintln!("{} -> {}: {:?}", link_name, child, child_t);
                link_world.insert(child.to_string(), child_t);
            }
            queue.push(child.to_string());
        }
    }
    eprintln!("body: {:?}", link_world.get("body"));
    eprintln!("right_rear_wheel: {:?}", link_world.get("right_rear_wheel"));
}
}
