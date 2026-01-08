//! Robot Model Display
//!
//! Visualizes robot models loaded from URDF files.

use crate::base::{BaseDisplay, DisplayUpdateContext};
use glam::Vec3;
use mviz_core::display::{DisplayError, DisplayInfo, PropertyMeta, PropertyValue};
use mviz_urdf::{
    parse_urdf, parse_urdf_string, Geometry, MeshData, RobotDescription,
    mesh_loader::primitives,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Robot model properties
#[derive(Debug, Clone)]
pub struct RobotModelProperties {
    /// Source of robot description
    pub source: RobotSource,
    /// Whether to show visual geometry
    pub visual_enabled: bool,
    /// Whether to show collision geometry
    pub collision_enabled: bool,
    /// Alpha transparency (0-1)
    pub alpha: f32,
    /// TF prefix for looking up link transforms
    pub tf_prefix: String,
}

impl Default for RobotModelProperties {
    fn default() -> Self {
        Self {
            source: RobotSource::None,
            visual_enabled: true,
            collision_enabled: false,
            alpha: 1.0,
            tf_prefix: String::new(),
        }
    }
}

/// Source of robot description
#[derive(Debug, Clone)]
pub enum RobotSource {
    /// No robot loaded
    None,
    /// Load from URDF file path
    File(PathBuf),
    /// Load from URDF string
    String(String),
}

/// Robot model display
pub struct RobotModelDisplay {
    base: BaseDisplay,
    properties: RobotModelProperties,
    /// Loaded robot description
    robot: Option<RobotDescription>,
    /// Cached meshes for each link
    link_meshes: HashMap<String, Vec<CachedMesh>>,
    /// Joint positions
    joint_positions: HashMap<String, f32>,
}

/// Cached mesh for a link
struct CachedMesh {
    /// Mesh data
    mesh: MeshData,
    /// Local transform from link origin
    local_transform: mviz_core::types::Transform,
    /// Color RGBA
    color: [f32; 4],
    /// Is visual (true) or collision (false)
    is_visual: bool,
}

impl RobotModelDisplay {
    /// Create a new robot model display
    pub fn new(name: &str) -> Self {
        let mut base = BaseDisplay::new(name, "robot_model");
        base.entity_path = "robot".to_string();

        // Add properties
        base.add_property("visual_enabled", PropertyValue::Bool(true));
        base.add_property("collision_enabled", PropertyValue::Bool(false));
        base.add_property("alpha", PropertyValue::Float(1.0));
        base.add_property("tf_prefix", PropertyValue::String(String::new()));

        Self {
            base,
            properties: RobotModelProperties::default(),
            robot: None,
            link_meshes: HashMap::new(),
            joint_positions: HashMap::new(),
        }
    }

    /// Load robot from URDF file
    pub fn load_urdf_file(&mut self, path: &str) -> Result<(), DisplayError> {
        let robot = parse_urdf(path)
            .map_err(|e| DisplayError::InitializationFailed(format!("Failed to parse URDF: {}", e)))?;

        self.properties.source = RobotSource::File(PathBuf::from(path));
        self.set_robot(robot)?;
        Ok(())
    }

    /// Load robot from URDF string
    pub fn load_urdf_string(&mut self, urdf: &str) -> Result<(), DisplayError> {
        let robot = parse_urdf_string(urdf)
            .map_err(|e| DisplayError::InitializationFailed(format!("Failed to parse URDF: {}", e)))?;

        self.properties.source = RobotSource::String(urdf.to_string());
        self.set_robot(robot)?;
        Ok(())
    }

    /// Set the robot description
    fn set_robot(&mut self, robot: RobotDescription) -> Result<(), DisplayError> {
        self.link_meshes.clear();
        self.joint_positions.clear();

        // Cache meshes for each link
        for (link_name, link) in &robot.links {
            let mut meshes = Vec::new();

            // Visual geometry
            if let Some(visual) = &link.visual {
                let mesh = self.geometry_to_mesh(&visual.geometry)?;
                let color = visual.material
                    .as_ref()
                    .and_then(|m| m.color)
                    .unwrap_or([0.8, 0.8, 0.8, 1.0]);

                meshes.push(CachedMesh {
                    mesh,
                    local_transform: visual.origin,
                    color,
                    is_visual: true,
                });
            }

            // Collision geometry
            if let Some(collision) = &link.collision {
                let mesh = self.geometry_to_mesh(&collision.geometry)?;

                meshes.push(CachedMesh {
                    mesh,
                    local_transform: collision.origin,
                    color: [1.0, 0.5, 0.0, 0.5], // Orange for collision
                    is_visual: false,
                });
            }

            if !meshes.is_empty() {
                self.link_meshes.insert(link_name.clone(), meshes);
            }
        }

        // Initialize joint positions to zero
        for joint_name in robot.joint_names() {
            self.joint_positions.insert(joint_name.to_string(), 0.0);
        }

        self.robot = Some(robot);
        Ok(())
    }

    /// Convert geometry to mesh data
    fn geometry_to_mesh(&self, geometry: &Geometry) -> Result<MeshData, DisplayError> {
        match geometry {
            Geometry::Box { size } => {
                Ok(primitives::create_box(*size * 0.5))
            }
            Geometry::Cylinder { radius, length } => {
                Ok(primitives::create_cylinder(*radius, *length, 16))
            }
            Geometry::Sphere { radius } => {
                Ok(primitives::create_sphere(*radius, 16, 32))
            }
            Geometry::Mesh { filename, scale } => {
                // Try to load mesh file
                let loader = mviz_urdf::mesh_loader::MultiFormatMeshLoader::new();
                let path = std::path::Path::new(filename);

                if path.exists() {
                    loader.load_scaled(path, *scale)
                        .map_err(|e| DisplayError::InitializationFailed(format!("Failed to load mesh: {}", e)))
                } else {
                    // Return placeholder cube if mesh not found
                    log::warn!("Mesh file not found: {}, using placeholder", filename);
                    Ok(primitives::create_box(Vec3::splat(0.1)))
                }
            }
        }
    }

    /// Set a joint position
    pub fn set_joint_position(&mut self, joint_name: &str, position: f32) {
        if let Some(robot) = &self.robot {
            if let Some(joint) = robot.joints.get(joint_name) {
                // Clamp to limits if present
                let clamped = if let Some(limits) = &joint.limits {
                    limits.clamp(position)
                } else {
                    position
                };
                self.joint_positions.insert(joint_name.to_string(), clamped);
            }
        }
    }

    /// Set all joint positions from a map
    pub fn set_joint_positions(&mut self, positions: &HashMap<String, f32>) {
        for (name, pos) in positions {
            self.set_joint_position(name, *pos);
        }
    }

    /// Get current joint position
    pub fn get_joint_position(&self, joint_name: &str) -> Option<f32> {
        self.joint_positions.get(joint_name).copied()
    }

    /// Get robot description if loaded
    pub fn robot(&self) -> Option<&RobotDescription> {
        self.robot.as_ref()
    }

    /// Set visual geometry enabled
    pub fn set_visual_enabled(&mut self, enabled: bool) {
        self.properties.visual_enabled = enabled;
        self.base.properties.set("visual_enabled", PropertyValue::Bool(enabled));
    }

    /// Set collision geometry enabled
    pub fn set_collision_enabled(&mut self, enabled: bool) {
        self.properties.collision_enabled = enabled;
        self.base.properties.set("collision_enabled", PropertyValue::Bool(enabled));
    }

    /// Set alpha transparency
    pub fn set_alpha(&mut self, alpha: f32) {
        self.properties.alpha = alpha.clamp(0.0, 1.0);
        self.base.properties.set("alpha", PropertyValue::Float(self.properties.alpha as f64));
    }

    /// Set TF prefix for transform lookups
    pub fn set_tf_prefix(&mut self, prefix: &str) {
        self.properties.tf_prefix = prefix.to_string();
        self.base.properties.set("tf_prefix", PropertyValue::String(prefix.to_string()));
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), DisplayError> {
        if !self.base.enabled {
            return Ok(());
        }

        let _robot = match &self.robot {
            Some(r) => r,
            None => return Ok(()), // No robot loaded
        };

        let rec = ctx.recording_stream();

        // For each link, compute transform and log meshes
        for (link_name, meshes) in &self.link_meshes {
            // Build frame name with optional prefix
            let frame_name = if self.properties.tf_prefix.is_empty() {
                link_name.clone()
            } else {
                format!("{}/{}", self.properties.tf_prefix, link_name)
            };

            // Try to get transform from TF buffer
            let link_transform = ctx.lookup_transform(&frame_name)
                .unwrap_or(mviz_core::types::Transform::IDENTITY);

            for cached_mesh in meshes {
                // Skip based on visual/collision settings
                if cached_mesh.is_visual && !self.properties.visual_enabled {
                    continue;
                }
                if !cached_mesh.is_visual && !self.properties.collision_enabled {
                    continue;
                }

                // Compute final transform
                let mesh_transform = link_transform * cached_mesh.local_transform;

                // Create entity path
                let entity_path = format!(
                    "{}/{}/{}",
                    &self.base.entity_path,
                    link_name,
                    if cached_mesh.is_visual { "visual" } else { "collision" }
                );

                // Log mesh to Rerun
                self.log_mesh(
                    rec,
                    &entity_path,
                    &cached_mesh.mesh,
                    &mesh_transform,
                    cached_mesh.color,
                )?;
            }
        }

        Ok(())
    }

    /// Log a mesh to Rerun
    fn log_mesh(
        &self,
        rec: &rerun::RecordingStream,
        entity_path: &str,
        mesh: &MeshData,
        transform: &mviz_core::types::Transform,
        color: [f32; 4],
    ) -> Result<(), DisplayError> {
        if mesh.is_empty() {
            return Ok(());
        }

        // Transform vertices
        let positions: Vec<[f32; 3]> = mesh.vertices
            .iter()
            .map(|v| {
                let transformed = transform.transform_point(*v);
                [transformed.x, transformed.y, transformed.z]
            })
            .collect();

        // Create triangle indices
        let indices: Vec<[u32; 3]> = mesh.indices
            .chunks(3)
            .filter_map(|chunk| {
                if chunk.len() == 3 {
                    Some([chunk[0], chunk[1], chunk[2]])
                } else {
                    None
                }
            })
            .collect();

        // Apply alpha to color
        let final_color = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * self.properties.alpha * 255.0) as u8,
        ];

        // Log as Mesh3D
        let mesh3d = rerun::Mesh3D::new(positions)
            .with_triangle_indices(indices)
            .with_albedo_factor(rerun::Rgba32::from_unmultiplied_rgba(
                final_color[0],
                final_color[1],
                final_color[2],
                final_color[3],
            ));

        rec.log(entity_path, &mesh3d)
            .map_err(|e| DisplayError::RenderError(e.to_string()))?;

        Ok(())
    }

    /// Get display info
    pub fn info(&self) -> DisplayInfo {
        DisplayInfo::new(
            "robot_model",
            "Robot Model",
            "Visualizes robot models from URDF files",
            "mviz",
        )
    }

    /// Get property metadata
    pub fn property_meta(&self) -> Vec<PropertyMeta> {
        vec![
            PropertyMeta::new("visual_enabled", "Visual", "Show visual geometry", true),
            PropertyMeta::new("collision_enabled", "Collision", "Show collision geometry", false),
            PropertyMeta::new("alpha", "Alpha", "Transparency (0-1)", 1.0f64),
            PropertyMeta::new("tf_prefix", "TF Prefix", "TF prefix for frame lookups", ""),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_URDF: &str = r#"<?xml version="1.0"?>
<robot name="test_robot">
  <link name="base_link">
    <visual>
      <geometry>
        <box size="1.0 0.5 0.2"/>
      </geometry>
    </visual>
  </link>
  <link name="link1">
    <visual>
      <geometry>
        <cylinder radius="0.1" length="0.5"/>
      </geometry>
    </visual>
  </link>
  <joint name="joint1" type="revolute">
    <parent link="base_link"/>
    <child link="link1"/>
    <origin xyz="0.5 0 0.1"/>
    <axis xyz="0 0 1"/>
    <limit lower="-1.57" upper="1.57" effort="10" velocity="1.0"/>
  </joint>
</robot>
"#;

    #[test]
    fn test_robot_model_display_creation() {
        let display = RobotModelDisplay::new("Test Robot");
        assert!(display.robot().is_none());
        assert!(display.link_meshes.is_empty());
    }

    #[test]
    fn test_load_urdf_string() {
        let mut display = RobotModelDisplay::new("Test Robot");
        display.load_urdf_string(TEST_URDF).unwrap();

        assert!(display.robot().is_some());
        let robot = display.robot().unwrap();
        assert_eq!(robot.name, "test_robot");
        assert_eq!(robot.links.len(), 2);
        assert_eq!(robot.joints.len(), 1);

        // Check meshes were created
        assert!(display.link_meshes.contains_key("base_link"));
        assert!(display.link_meshes.contains_key("link1"));
    }

    #[test]
    fn test_joint_positions() {
        let mut display = RobotModelDisplay::new("Test Robot");
        display.load_urdf_string(TEST_URDF).unwrap();

        // Initial position should be 0
        assert_eq!(display.get_joint_position("joint1"), Some(0.0));

        // Set position within limits
        display.set_joint_position("joint1", 1.0);
        assert_eq!(display.get_joint_position("joint1"), Some(1.0));

        // Position should be clamped to limits
        display.set_joint_position("joint1", 5.0);
        let pos = display.get_joint_position("joint1").unwrap();
        assert!((pos - 1.57).abs() < 0.01); // Clamped to upper limit
    }

    #[test]
    fn test_geometry_to_mesh() {
        let display = RobotModelDisplay::new("Test");

        // Box
        let box_geom = Geometry::Box { size: Vec3::ONE };
        let mesh = display.geometry_to_mesh(&box_geom).unwrap();
        assert!(!mesh.is_empty());

        // Cylinder
        let cyl_geom = Geometry::Cylinder { radius: 0.5, length: 1.0 };
        let mesh = display.geometry_to_mesh(&cyl_geom).unwrap();
        assert!(!mesh.is_empty());

        // Sphere
        let sphere_geom = Geometry::Sphere { radius: 0.5 };
        let mesh = display.geometry_to_mesh(&sphere_geom).unwrap();
        assert!(!mesh.is_empty());
    }

    #[test]
    fn test_properties() {
        let mut display = RobotModelDisplay::new("Test");

        display.set_visual_enabled(false);
        assert!(!display.properties.visual_enabled);

        display.set_collision_enabled(true);
        assert!(display.properties.collision_enabled);

        display.set_alpha(0.5);
        assert!((display.properties.alpha - 0.5).abs() < 0.01);

        display.set_tf_prefix("robot1");
        assert_eq!(display.properties.tf_prefix, "robot1");
    }
}
