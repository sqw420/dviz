//! Mesh Loader
//!
//! Loads mesh files (STL, OBJ) for robot visualization.

use glam::Vec3;
use std::path::Path;
use thiserror::Error;

/// Mesh loading errors
#[derive(Debug, Error)]
pub enum MeshError {
    #[error("Failed to read mesh file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse mesh: {0}")]
    ParseError(String),

    #[error("Unsupported mesh format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid mesh data: {0}")]
    InvalidMesh(String),
}

/// Mesh data for visualization
#[derive(Debug, Clone)]
pub struct MeshData {
    /// Vertex positions
    pub vertices: Vec<Vec3>,
    /// Triangle indices (3 per triangle)
    pub indices: Vec<u32>,
    /// Vertex normals (optional)
    pub normals: Option<Vec<Vec3>>,
    /// Texture coordinates (optional)
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Vertex colors (optional)
    pub colors: Option<Vec<[f32; 4]>>,
}

impl MeshData {
    /// Create empty mesh data
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: None,
            uvs: None,
            colors: None,
        }
    }

    /// Create mesh with capacity
    pub fn with_capacity(vertex_count: usize, index_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count),
            indices: Vec::with_capacity(index_count),
            normals: None,
            uvs: None,
            colors: None,
        }
    }

    /// Number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if mesh is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Apply scale to vertices
    pub fn apply_scale(&mut self, scale: Vec3) {
        for vertex in &mut self.vertices {
            *vertex *= scale;
        }
    }

    /// Apply transform to mesh
    pub fn apply_transform(&mut self, transform: &dviz_core::types::Transform) {
        for vertex in &mut self.vertices {
            *vertex = transform.transform_point(*vertex);
        }
        if let Some(normals) = &mut self.normals {
            for normal in normals {
                *normal = transform.transform_vector(*normal).normalize();
            }
        }
    }

    /// Compute normals from triangles (flat shading)
    pub fn compute_flat_normals(&mut self) {
        let mut normals = vec![Vec3::ZERO; self.vertices.len()];

        for tri in self.indices.chunks(3) {
            if tri.len() < 3 {
                continue;
            }
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            if i0 >= self.vertices.len() || i1 >= self.vertices.len() || i2 >= self.vertices.len() {
                continue;
            }

            let v0 = self.vertices[i0];
            let v1 = self.vertices[i1];
            let v2 = self.vertices[i2];

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2).normalize();

            if normal.is_finite() {
                normals[i0] += normal;
                normals[i1] += normal;
                normals[i2] += normal;
            }
        }

        // Normalize accumulated normals
        for normal in &mut normals {
            if normal.length_squared() > 0.0 {
                *normal = normal.normalize();
            } else {
                *normal = Vec3::Z;
            }
        }

        self.normals = Some(normals);
    }

    /// Get bounding box (min, max)
    pub fn bounding_box(&self) -> (Vec3, Vec3) {
        if self.vertices.is_empty() {
            return (Vec3::ZERO, Vec3::ZERO);
        }

        let mut min = self.vertices[0];
        let mut max = self.vertices[0];

        for v in &self.vertices {
            min = min.min(*v);
            max = max.max(*v);
        }

        (min, max)
    }

    /// Get center of bounding box
    pub fn center(&self) -> Vec3 {
        let (min, max) = self.bounding_box();
        (min + max) * 0.5
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for mesh loaders
pub trait MeshLoader: Send + Sync {
    /// Load mesh from file
    fn load(&self, path: &Path) -> Result<MeshData, MeshError>;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];

    /// Check if this loader supports the given file
    fn supports(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            self.supported_extensions().iter().any(|e| *e == ext_str)
        } else {
            false
        }
    }
}

/// STL file loader
pub struct StlLoader;

impl StlLoader {
    /// Create a new STL loader
    pub fn new() -> Self {
        Self
    }
}

impl Default for StlLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshLoader for StlLoader {
    fn load(&self, path: &Path) -> Result<MeshData, MeshError> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(path)?;

        let stl = stl_io::read_stl(&mut file)
            .map_err(|e| MeshError::ParseError(e.to_string()))?;

        // Convert STL to indexed mesh
        // stl_io stores vertices separately and faces reference them by index
        let mut mesh = MeshData::with_capacity(stl.vertices.len(), stl.faces.len() * 3);

        // Copy vertices
        for vertex in &stl.vertices {
            mesh.vertices.push(Vec3::new(vertex[0], vertex[1], vertex[2]));
        }

        // Copy face indices
        for face in &stl.faces {
            mesh.indices.push(face.vertices[0] as u32);
            mesh.indices.push(face.vertices[1] as u32);
            mesh.indices.push(face.vertices[2] as u32);
        }

        // Compute normals
        mesh.compute_flat_normals();

        Ok(mesh)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["stl"]
    }
}

/// Multi-format mesh loader that delegates to specialized loaders
pub struct MultiFormatMeshLoader {
    loaders: Vec<Box<dyn MeshLoader>>,
}

impl MultiFormatMeshLoader {
    /// Create a new multi-format loader with default loaders
    pub fn new() -> Self {
        Self {
            loaders: vec![
                Box::new(StlLoader::new()),
            ],
        }
    }

    /// Add a custom loader
    pub fn add_loader(&mut self, loader: Box<dyn MeshLoader>) {
        self.loaders.push(loader);
    }

    /// Load mesh from file, auto-detecting format
    pub fn load(&self, path: &Path) -> Result<MeshData, MeshError> {
        for loader in &self.loaders {
            if loader.supports(path) {
                return loader.load(path);
            }
        }

        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        Err(MeshError::UnsupportedFormat(ext))
    }

    /// Load mesh from file with scale
    pub fn load_scaled(&self, path: &Path, scale: Vec3) -> Result<MeshData, MeshError> {
        let mut mesh = self.load(path)?;
        mesh.apply_scale(scale);
        Ok(mesh)
    }

    /// Get all supported extensions
    pub fn supported_extensions(&self) -> Vec<String> {
        self.loaders.iter()
            .flat_map(|l| l.supported_extensions().iter().map(|s| s.to_string()))
            .collect()
    }
}

impl Default for MultiFormatMeshLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve package:// URLs used in ROS/URDF
pub fn resolve_package_url(url: &str, package_paths: &[&str]) -> Option<String> {
    if !url.starts_with("package://") {
        return Some(url.to_string());
    }

    let rest = &url[10..]; // Remove "package://"
    let parts: Vec<&str> = rest.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }

    let package_name = parts[0];
    let relative_path = parts[1];

    // Search in package paths
    for base_path in package_paths {
        let full_path = format!("{}/{}/{}", base_path, package_name, relative_path);
        if Path::new(&full_path).exists() {
            return Some(full_path);
        }
    }

    None
}

/// Create primitive mesh shapes
pub mod primitives {
    use super::*;

    /// Create a box mesh
    pub fn create_box(half_extents: Vec3) -> MeshData {
        let hx = half_extents.x;
        let hy = half_extents.y;
        let hz = half_extents.z;

        let vertices = vec![
            // Front face
            Vec3::new(-hx, -hy, hz), Vec3::new(hx, -hy, hz),
            Vec3::new(hx, hy, hz), Vec3::new(-hx, hy, hz),
            // Back face
            Vec3::new(-hx, -hy, -hz), Vec3::new(-hx, hy, -hz),
            Vec3::new(hx, hy, -hz), Vec3::new(hx, -hy, -hz),
            // Top face
            Vec3::new(-hx, hy, -hz), Vec3::new(-hx, hy, hz),
            Vec3::new(hx, hy, hz), Vec3::new(hx, hy, -hz),
            // Bottom face
            Vec3::new(-hx, -hy, -hz), Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, -hy, hz), Vec3::new(-hx, -hy, hz),
            // Right face
            Vec3::new(hx, -hy, -hz), Vec3::new(hx, hy, -hz),
            Vec3::new(hx, hy, hz), Vec3::new(hx, -hy, hz),
            // Left face
            Vec3::new(-hx, -hy, -hz), Vec3::new(-hx, -hy, hz),
            Vec3::new(-hx, hy, hz), Vec3::new(-hx, hy, -hz),
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3,       // Front
            4, 5, 6, 4, 6, 7,       // Back
            8, 9, 10, 8, 10, 11,    // Top
            12, 13, 14, 12, 14, 15, // Bottom
            16, 17, 18, 16, 18, 19, // Right
            20, 21, 22, 20, 22, 23, // Left
        ];

        let mut mesh = MeshData { vertices, indices, normals: None, uvs: None, colors: None };
        mesh.compute_flat_normals();
        mesh
    }

    /// Create a cylinder mesh
    pub fn create_cylinder(radius: f32, length: f32, segments: usize) -> MeshData {
        let half_length = length * 0.5;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Create vertices for the cylinder
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            // Top vertex
            vertices.push(Vec3::new(x, y, half_length));
            // Bottom vertex
            vertices.push(Vec3::new(x, y, -half_length));
        }

        // Side faces
        for i in 0..segments {
            let i0 = (i * 2) as u32;
            let i1 = (i * 2 + 1) as u32;
            let i2 = (i * 2 + 2) as u32;
            let i3 = (i * 2 + 3) as u32;

            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }

        // Top cap center
        let top_center_idx = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, 0.0, half_length));

        // Bottom cap center
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, 0.0, -half_length));

        // Top cap triangles
        for i in 0..segments {
            let i0 = (i * 2) as u32;
            let i2 = (i * 2 + 2) as u32;
            indices.extend_from_slice(&[top_center_idx, i0, i2]);
        }

        // Bottom cap triangles
        for i in 0..segments {
            let i1 = (i * 2 + 1) as u32;
            let i3 = (i * 2 + 3) as u32;
            indices.extend_from_slice(&[bottom_center_idx, i3, i1]);
        }

        let mut mesh = MeshData { vertices, indices, normals: None, uvs: None, colors: None };
        mesh.compute_flat_normals();
        mesh
    }

    /// Create a sphere mesh
    pub fn create_sphere(radius: f32, lat_segments: usize, lon_segments: usize) -> MeshData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for lat in 0..=lat_segments {
            let theta = (lat as f32 / lat_segments as f32) * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=lon_segments {
                let phi = (lon as f32 / lon_segments as f32) * std::f32::consts::TAU;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = sin_phi * sin_theta;
                let z = cos_theta;

                vertices.push(Vec3::new(x * radius, y * radius, z * radius));
            }
        }

        for lat in 0..lat_segments {
            for lon in 0..lon_segments {
                let first = (lat * (lon_segments + 1) + lon) as u32;
                let second = first + (lon_segments + 1) as u32;

                indices.extend_from_slice(&[
                    first, second, first + 1,
                    second, second + 1, first + 1,
                ]);
            }
        }

        let mut mesh = MeshData { vertices, indices, normals: None, uvs: None, colors: None };
        mesh.compute_flat_normals();
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_data_basic() {
        let mut mesh = MeshData::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);

        mesh.vertices.push(Vec3::ZERO);
        mesh.vertices.push(Vec3::X);
        mesh.vertices.push(Vec3::Y);
        mesh.indices.extend_from_slice(&[0, 1, 2]);

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_bounding_box() {
        let mut mesh = MeshData::new();
        mesh.vertices = vec![
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];

        let (min, max) = mesh.bounding_box();
        assert_eq!(min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_apply_scale() {
        let mut mesh = MeshData::new();
        mesh.vertices = vec![Vec3::ONE, Vec3::new(2.0, 2.0, 2.0)];

        mesh.apply_scale(Vec3::new(2.0, 3.0, 4.0));

        assert_eq!(mesh.vertices[0], Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(mesh.vertices[1], Vec3::new(4.0, 6.0, 8.0));
    }

    #[test]
    fn test_create_box() {
        let mesh = primitives::create_box(Vec3::ONE);

        assert_eq!(mesh.vertex_count(), 24); // 6 faces * 4 vertices
        assert_eq!(mesh.triangle_count(), 12); // 6 faces * 2 triangles
        assert!(mesh.normals.is_some());
    }

    #[test]
    fn test_create_sphere() {
        let mesh = primitives::create_sphere(1.0, 8, 16);

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_create_cylinder() {
        let mesh = primitives::create_cylinder(0.5, 1.0, 16);

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_resolve_package_url() {
        // Non-package URL should pass through
        let url = "/path/to/mesh.stl";
        assert_eq!(resolve_package_url(url, &[]), Some(url.to_string()));

        // Package URL with no matching path
        let pkg_url = "package://my_robot/meshes/base.stl";
        assert_eq!(resolve_package_url(pkg_url, &[]), None);
    }

    #[test]
    fn test_stl_loader_extensions() {
        let loader = StlLoader::new();
        assert!(loader.supports(Path::new("mesh.stl")));
        assert!(loader.supports(Path::new("mesh.STL")));
        assert!(!loader.supports(Path::new("mesh.obj")));
    }
}
