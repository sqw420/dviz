//! Core Type Adapters
//!
//! Adapters for converting dviz-core types to Rerun visualization types.

use rerun::RecordingStream;

use dviz_core::{
    ColorMode, Colormap, FrameId, Marker, MarkerArray, MarkerType, PointCloud, Transform,
};

use crate::RerunError;

// ============================================================================
// POINT CLOUD CORE ADAPTER
// ============================================================================

/// Adapter for logging dviz-core PointCloud to Rerun
pub struct PointCloudCoreAdapter;

impl PointCloudCoreAdapter {
    /// Log a PointCloud to Rerun
    pub fn log(
        stream: &RecordingStream,
        entity_path: &str,
        cloud: &PointCloud,
        color_mode: &ColorMode,
        point_radius: f32,
    ) -> Result<(), RerunError> {
        if cloud.is_empty() {
            return Ok(());
        }

        let positions: Vec<[f32; 3]> = cloud
            .positions
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect();

        let colors = Self::compute_colors(cloud, color_mode);

        let mut points3d = rerun::Points3D::new(positions).with_radii([point_radius]);

        if let Some(colors) = colors {
            points3d = points3d.with_colors(colors);
        }

        stream
            .log(entity_path, &points3d)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Compute colors for point cloud based on color mode
    pub fn compute_colors(cloud: &PointCloud, mode: &ColorMode) -> Option<Vec<rerun::Color>> {
        match mode {
            ColorMode::FlatColor(color) => {
                Some(vec![rerun::Color::from_unmultiplied_rgba(color.r, color.g, color.b, color.a); cloud.len()])
            }
            ColorMode::RGB => {
                cloud.colors.as_ref().map(|colors| {
                    colors
                        .iter()
                        .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                        .collect()
                })
            }
            ColorMode::AxisColor { axis, min, max, colormap } => {
                let colors: Vec<rerun::Color> = cloud
                    .positions
                    .iter()
                    .map(|p| {
                        let value = axis.get(*p);
                        let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
                        Self::colormap_lookup(colormap, t)
                    })
                    .collect();
                Some(colors)
            }
            ColorMode::Intensity { min, max, colormap } => {
                cloud.intensities.as_ref().map(|intensities| {
                    intensities
                        .iter()
                        .map(|&intensity| {
                            let t = ((intensity - min) / (max - min)).clamp(0.0, 1.0);
                            Self::colormap_lookup(colormap, t)
                        })
                        .collect()
                })
            }
        }
    }

    /// Look up color from colormap
    pub fn colormap_lookup(colormap: &Colormap, t: f32) -> rerun::Color {
        let color = colormap.sample(t);
        rerun::Color::from_unmultiplied_rgba(color.r, color.g, color.b, color.a)
    }
}

// ============================================================================
// TRANSFORM CORE ADAPTER
// ============================================================================

/// Adapter for logging dviz-core Transform to Rerun
pub struct TransformCoreAdapter;

impl TransformCoreAdapter {
    /// Log a Transform to Rerun
    pub fn log(
        stream: &RecordingStream,
        entity_path: &str,
        transform: &Transform,
    ) -> Result<(), RerunError> {
        let tf = rerun::Transform3D::from_translation_rotation(
            rerun::Vec3D::new(
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            rerun::Quaternion::from_xyzw([
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ]),
        );

        stream
            .log(entity_path, &tf)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Log coordinate frame axes (RGB for XYZ)
    pub fn log_frame_axes(
        stream: &RecordingStream,
        entity_path: &str,
        transform: &Transform,
        scale: f32,
    ) -> Result<(), RerunError> {
        let origin = transform.translation;

        // Rotate unit vectors by the transform's rotation
        let x_axis = transform.transform_vector(glam::Vec3::X) * scale;
        let y_axis = transform.transform_vector(glam::Vec3::Y) * scale;
        let z_axis = transform.transform_vector(glam::Vec3::Z) * scale;

        // Log X axis (red)
        let x_path = format!("{}/x", entity_path);
        stream
            .log(
                x_path.as_str(),
                &rerun::Arrows3D::from_vectors([[x_axis.x, x_axis.y, x_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(255, 0, 0)]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        // Log Y axis (green)
        let y_path = format!("{}/y", entity_path);
        stream
            .log(
                y_path.as_str(),
                &rerun::Arrows3D::from_vectors([[y_axis.x, y_axis.y, y_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(0, 255, 0)]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        // Log Z axis (blue)
        let z_path = format!("{}/z", entity_path);
        stream
            .log(
                z_path.as_str(),
                &rerun::Arrows3D::from_vectors([[z_axis.x, z_axis.y, z_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(0, 0, 255)]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Log all transforms in a frame tree
    pub fn log_all_frames(
        stream: &RecordingStream,
        base_path: &str,
        frames: &[(FrameId, Transform)],
        axis_scale: f32,
    ) -> Result<(), RerunError> {
        for (frame_id, transform) in frames {
            let path = format!("{}/{}", base_path, frame_id.as_str());
            Self::log_frame_axes(stream, &path, transform, axis_scale)?;
        }
        Ok(())
    }
}

// ============================================================================
// MARKER CORE ADAPTER
// ============================================================================

/// Adapter for logging dviz-core Marker to Rerun
pub struct MarkerCoreAdapter;

impl MarkerCoreAdapter {
    /// Log a single Marker to Rerun
    pub fn log(
        stream: &RecordingStream,
        base_path: &str,
        marker: &Marker,
    ) -> Result<(), RerunError> {
        let entity_path = if marker.ns.is_empty() {
            format!("{}/marker_{}", base_path, marker.id)
        } else {
            format!("{}/{}/marker_{}", base_path, marker.ns, marker.id)
        };

        let color = rerun::Color::from_unmultiplied_rgba(
            marker.color.r,
            marker.color.g,
            marker.color.b,
            marker.color.a,
        );

        match marker.marker_type {
            MarkerType::Arrow => {
                Self::log_arrow(stream, &entity_path, marker, color)?;
            }
            MarkerType::Cube => {
                Self::log_cube(stream, &entity_path, marker, color)?;
            }
            MarkerType::Sphere => {
                Self::log_sphere(stream, &entity_path, marker, color)?;
            }
            MarkerType::Cylinder => {
                // Rerun doesn't have native cylinders, approximate with ellipsoid
                Self::log_cylinder(stream, &entity_path, marker, color)?;
            }
            MarkerType::LineStrip => {
                Self::log_line_strip(stream, &entity_path, marker, color)?;
            }
            MarkerType::LineList => {
                Self::log_line_list(stream, &entity_path, marker, color)?;
            }
            MarkerType::CubeList => {
                Self::log_cube_list(stream, &entity_path, marker, color)?;
            }
            MarkerType::SphereList => {
                Self::log_sphere_list(stream, &entity_path, marker, color)?;
            }
            MarkerType::Points => {
                Self::log_points(stream, &entity_path, marker, color)?;
            }
            MarkerType::Text => {
                Self::log_text(stream, &entity_path, marker, color)?;
            }
            MarkerType::TriangleList => {
                Self::log_triangle_list(stream, &entity_path, marker, color)?;
            }
            MarkerType::MeshResource => {
                // Mesh resources not yet supported
                log::warn!("MeshResource markers not yet supported");
            }
        }

        Ok(())
    }

    /// Log a MarkerArray to Rerun
    pub fn log_array(
        stream: &RecordingStream,
        base_path: &str,
        array: &MarkerArray,
    ) -> Result<(), RerunError> {
        for marker in array.iter() {
            if !marker.is_delete() {
                Self::log(stream, base_path, marker)?;
            }
        }
        Ok(())
    }

    fn log_arrow(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let origin = [marker.position.x, marker.position.y, marker.position.z];

        // Arrow direction from orientation (forward is X)
        let forward = marker.orientation * glam::Vec3::X * marker.scale.x;

        stream
            .log(
                entity_path,
                &rerun::Arrows3D::from_vectors([[forward.x, forward.y, forward.z]])
                    .with_origins([origin])
                    .with_colors([color]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_cube(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let center = [marker.position.x, marker.position.y, marker.position.z];
        let half_sizes = [marker.scale.x / 2.0, marker.scale.y / 2.0, marker.scale.z / 2.0];

        stream
            .log(
                entity_path,
                &rerun::Boxes3D::from_centers_and_half_sizes([center], [half_sizes])
                    .with_quaternions([rerun::Quaternion::from_xyzw([
                        marker.orientation.x,
                        marker.orientation.y,
                        marker.orientation.z,
                        marker.orientation.w,
                    ])])
                    .with_colors([color]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_sphere(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        let center = [marker.position.x, marker.position.y, marker.position.z];
        let radii = [marker.scale.x / 2.0, marker.scale.y / 2.0, marker.scale.z / 2.0];

        stream
            .log(
                entity_path,
                &rerun::Ellipsoids3D::from_centers_and_half_sizes([center], [radii])
                    .with_colors([color]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_cylinder(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        // Approximate cylinder with ellipsoid (not perfect but visible)
        let center = [marker.position.x, marker.position.y, marker.position.z];
        let radii = [marker.scale.x / 2.0, marker.scale.y / 2.0, marker.scale.z / 2.0];

        stream
            .log(
                entity_path,
                &rerun::Ellipsoids3D::from_centers_and_half_sizes([center], [radii])
                    .with_colors([color]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_line_strip(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.is_empty() {
            return Ok(());
        }

        let points: Vec<[f32; 3]> = marker.points.iter().map(|p| [p.x, p.y, p.z]).collect();

        let mut line_strip = rerun::LineStrips3D::new([points]).with_colors([color]);

        if !marker.colors.is_empty() {
            let colors: Vec<rerun::Color> = marker
                .colors
                .iter()
                .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                .collect();
            line_strip = line_strip.with_colors(colors);
        }

        stream
            .log(entity_path, &line_strip)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_line_list(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.len() < 2 {
            return Ok(());
        }

        // Convert pairs of points to line segments
        let lines: Vec<Vec<[f32; 3]>> = marker
            .points
            .chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .map(|chunk| {
                vec![
                    [chunk[0].x, chunk[0].y, chunk[0].z],
                    [chunk[1].x, chunk[1].y, chunk[1].z],
                ]
            })
            .collect();

        stream
            .log(
                entity_path,
                &rerun::LineStrips3D::new(lines).with_colors([color]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_cube_list(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.is_empty() {
            return Ok(());
        }

        let centers: Vec<[f32; 3]> = marker.points.iter().map(|p| [p.x, p.y, p.z]).collect();
        let half_size = [marker.scale.x / 2.0, marker.scale.y / 2.0, marker.scale.z / 2.0];
        let half_sizes = vec![half_size; centers.len()];

        let mut boxes = rerun::Boxes3D::from_centers_and_half_sizes(centers, half_sizes);

        if !marker.colors.is_empty() {
            let colors: Vec<rerun::Color> = marker
                .colors
                .iter()
                .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                .collect();
            boxes = boxes.with_colors(colors);
        } else {
            boxes = boxes.with_colors([color]);
        }

        stream
            .log(entity_path, &boxes)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_sphere_list(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.is_empty() {
            return Ok(());
        }

        let centers: Vec<[f32; 3]> = marker.points.iter().map(|p| [p.x, p.y, p.z]).collect();
        let half_size = [marker.scale.x / 2.0, marker.scale.y / 2.0, marker.scale.z / 2.0];
        let half_sizes = vec![half_size; centers.len()];

        let mut ellipsoids = rerun::Ellipsoids3D::from_centers_and_half_sizes(centers, half_sizes);

        if !marker.colors.is_empty() {
            let colors: Vec<rerun::Color> = marker
                .colors
                .iter()
                .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                .collect();
            ellipsoids = ellipsoids.with_colors(colors);
        } else {
            ellipsoids = ellipsoids.with_colors([color]);
        }

        stream
            .log(entity_path, &ellipsoids)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_points(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.is_empty() {
            return Ok(());
        }

        let positions: Vec<[f32; 3]> = marker.points.iter().map(|p| [p.x, p.y, p.z]).collect();

        let mut points = rerun::Points3D::new(positions).with_radii([marker.scale.x / 2.0]);

        if !marker.colors.is_empty() {
            let colors: Vec<rerun::Color> = marker
                .colors
                .iter()
                .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                .collect();
            points = points.with_colors(colors);
        } else {
            points = points.with_colors([color]);
        }

        stream
            .log(entity_path, &points)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_text(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        _color: rerun::Color,
    ) -> Result<(), RerunError> {
        // Log text at position
        stream
            .log(
                entity_path,
                &rerun::TextLog::new(marker.text.as_str()),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        // Also log a point at the text position for reference
        let position = [marker.position.x, marker.position.y, marker.position.z];
        let path = format!("{}/position", entity_path);
        stream
            .log(
                path.as_str(),
                &rerun::Points3D::new([position])
                    .with_radii([marker.scale.x / 4.0])
                    .with_labels([marker.text.as_str()]),
            )
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    fn log_triangle_list(
        stream: &RecordingStream,
        entity_path: &str,
        marker: &Marker,
        color: rerun::Color,
    ) -> Result<(), RerunError> {
        if marker.points.len() < 3 {
            return Ok(());
        }

        let vertices: Vec<[f32; 3]> = marker.points.iter().map(|p| [p.x, p.y, p.z]).collect();

        // Generate triangle indices (every 3 vertices form a triangle)
        let indices: Vec<[u32; 3]> = (0..vertices.len() / 3)
            .map(|i| [(i * 3) as u32, (i * 3 + 1) as u32, (i * 3 + 2) as u32])
            .collect();

        let mut mesh = rerun::Mesh3D::new(vertices).with_triangle_indices(indices);

        if !marker.colors.is_empty() {
            let colors: Vec<rerun::Color> = marker
                .colors
                .iter()
                .map(|c| rerun::Color::from_unmultiplied_rgba(c.r, c.g, c.b, c.a))
                .collect();
            mesh = mesh.with_vertex_colors(colors);
        } else {
            mesh = mesh.with_vertex_colors([color]);
        }

        stream
            .log(entity_path, &mesh)
            .map_err(|e| RerunError::LogError(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use dviz_core::{Axis, Color};

    #[test]
    fn test_colormap_lookup() {
        let color = PointCloudCoreAdapter::colormap_lookup(&Colormap::Jet, 0.0);
        // Jet at 0.0 should be blue-ish
        let [r, _g, b, _a] = color.to_array();
        assert!(b > r, "At t=0.0, blue should be greater than red"); // Blue > Red

        let color = PointCloudCoreAdapter::colormap_lookup(&Colormap::Jet, 1.0);
        // Jet at 1.0 should be red-ish
        let [r, _g, b, _a] = color.to_array();
        assert!(r > b, "At t=1.0, red should be greater than blue"); // Red > Blue
    }

    #[test]
    fn test_compute_colors_flat() {
        let cloud = PointCloud::from_positions(
            vec![Vec3::ZERO, Vec3::ONE],
            "world",
        );

        let color = Color::RED;
        let colors = PointCloudCoreAdapter::compute_colors(&cloud, &ColorMode::FlatColor(color));

        assert!(colors.is_some());
        let colors = colors.unwrap();
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_compute_colors_by_axis() {
        let cloud = PointCloud::from_positions(
            vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 10.0)],
            "world",
        );

        let mode = ColorMode::AxisColor {
            axis: Axis::Z,
            min: 0.0,
            max: 10.0,
            colormap: Colormap::Jet,
        };
        let colors = PointCloudCoreAdapter::compute_colors(&cloud, &mode);

        assert!(colors.is_some());
        let colors = colors.unwrap();
        assert_eq!(colors.len(), 2);
        // First point (z=0) should be different from second (z=10)
        assert_ne!(colors[0], colors[1]);
    }
}
