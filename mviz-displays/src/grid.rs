//! Grid Display
//!
//! Displays a reference grid on a specified plane.

use glam::Vec3;
use mviz_core::{Color, FrameId};
use crate::base::{BaseDisplay, DisplayUpdateContext};

/// Grid plane orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridPlane {
    /// XY plane (Z up)
    #[default]
    XY,
    /// XZ plane (Y up)
    XZ,
    /// YZ plane (X up)
    YZ,
}

impl GridPlane {
    /// Get the normal vector for this plane
    pub fn normal(&self) -> Vec3 {
        match self {
            GridPlane::XY => Vec3::Z,
            GridPlane::XZ => Vec3::Y,
            GridPlane::YZ => Vec3::X,
        }
    }

    /// Get basis vectors for this plane
    pub fn basis(&self) -> (Vec3, Vec3) {
        match self {
            GridPlane::XY => (Vec3::X, Vec3::Y),
            GridPlane::XZ => (Vec3::X, Vec3::Z),
            GridPlane::YZ => (Vec3::Y, Vec3::Z),
        }
    }
}

/// Grid display properties
#[derive(Debug, Clone)]
pub struct GridProperties {
    /// Number of cells in each direction
    pub cell_count: u32,
    /// Size of each cell in meters
    pub cell_size: f32,
    /// Grid color
    pub color: Color,
    /// Line alpha (transparency)
    pub alpha: f32,
    /// Line width
    pub line_width: f32,
    /// Reference frame
    pub frame: FrameId,
    /// Offset from frame origin
    pub offset: Vec3,
    /// Grid plane orientation
    pub plane: GridPlane,
}

impl Default for GridProperties {
    fn default() -> Self {
        Self {
            cell_count: 10,
            cell_size: 1.0,
            color: Color::GRAY,
            alpha: 0.5,
            line_width: 1.0,
            frame: FrameId::default(),
            offset: Vec3::ZERO,
            plane: GridPlane::XY,
        }
    }
}

impl GridProperties {
    /// Calculate total grid size
    pub fn total_size(&self) -> f32 {
        self.cell_count as f32 * self.cell_size
    }
}

/// Grid display for reference visualization
pub struct GridDisplay {
    /// Base display state
    pub base: BaseDisplay,
    /// Grid properties
    pub props: GridProperties,
    /// Whether grid needs to be redrawn
    dirty: bool,
}

// Implement common display methods
crate::impl_display_base!(GridDisplay);

impl GridDisplay {
    /// Create a new grid display
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = BaseDisplay::new(name, "grid");

        // Add properties
        base.add_property("cell_count", 10i64);
        base.add_property("cell_size", 1.0f64);
        base.add_property("alpha", 0.5f64);

        Self {
            base,
            props: GridProperties::default(),
            dirty: true,
        }
    }

    /// Set the cell count
    pub fn set_cell_count(&mut self, count: u32) {
        if self.props.cell_count != count {
            self.props.cell_count = count;
            self.dirty = true;
        }
    }

    /// Set the cell size
    pub fn set_cell_size(&mut self, size: f32) {
        if (self.props.cell_size - size).abs() > f32::EPSILON {
            self.props.cell_size = size;
            self.dirty = true;
        }
    }

    /// Set the grid color
    pub fn set_color(&mut self, color: Color) {
        if self.props.color != color {
            self.props.color = color;
            self.dirty = true;
        }
    }

    /// Set the grid plane
    pub fn set_plane(&mut self, plane: GridPlane) {
        if self.props.plane != plane {
            self.props.plane = plane;
            self.dirty = true;
        }
    }

    /// Set the reference frame
    pub fn set_frame(&mut self, frame: FrameId) {
        if self.props.frame != frame {
            self.props.frame = frame;
            self.dirty = true;
        }
    }

    /// Generate grid line segments
    pub fn generate_grid_lines(&self) -> Vec<Vec<[f32; 3]>> {
        let mut lines = Vec::new();
        let (axis1, axis2) = self.props.plane.basis();
        let half_size = self.props.total_size() / 2.0;
        let cell_size = self.props.cell_size;
        let count = self.props.cell_count as i32;

        // Generate lines along first axis
        for i in -count / 2..=count / 2 {
            let offset = i as f32 * cell_size;
            let start = self.props.offset + axis2 * offset - axis1 * half_size;
            let end = self.props.offset + axis2 * offset + axis1 * half_size;
            lines.push(vec![
                [start.x, start.y, start.z],
                [end.x, end.y, end.z],
            ]);
        }

        // Generate lines along second axis
        for i in -count / 2..=count / 2 {
            let offset = i as f32 * cell_size;
            let start = self.props.offset + axis1 * offset - axis2 * half_size;
            let end = self.props.offset + axis1 * offset + axis2 * half_size;
            lines.push(vec![
                [start.x, start.y, start.z],
                [end.x, end.y, end.z],
            ]);
        }

        lines
    }

    /// Log the grid to Rerun
    pub fn log_grid(&self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        let lines = self.generate_grid_lines();

        let color = rerun::Color::from_unmultiplied_rgba(
            self.props.color.r,
            self.props.color.g,
            self.props.color.b,
            (self.props.alpha * 255.0) as u8,
        );

        ctx.stream
            .log_static(
                self.entity_path(),
                &rerun::LineStrips3D::new(lines)
                    .with_colors([color])
                    .with_radii([self.props.line_width * 0.001]), // Convert to meters
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Only redraw if dirty (grid is static)
        if self.dirty {
            self.log_grid(ctx)?;
            self.dirty = false;
        }

        Ok(())
    }

    /// Force redraw on next update
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Initialize the display
    pub fn initialize(&mut self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        self.dirty = true;
        self.update(ctx)
    }

    /// Reset the display
    pub fn reset(&mut self) {
        self.props = GridProperties::default();
        self.dirty = true;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_properties_default() {
        let props = GridProperties::default();
        assert_eq!(props.cell_count, 10);
        assert_eq!(props.cell_size, 1.0);
        assert_eq!(props.total_size(), 10.0);
    }

    #[test]
    fn test_grid_plane_basis() {
        let (x, y) = GridPlane::XY.basis();
        assert_eq!(x, Vec3::X);
        assert_eq!(y, Vec3::Y);

        let (x, z) = GridPlane::XZ.basis();
        assert_eq!(x, Vec3::X);
        assert_eq!(z, Vec3::Z);
    }

    #[test]
    fn test_grid_display_creation() {
        let grid = GridDisplay::new("Ground Grid");
        assert_eq!(grid.name(), "Ground Grid");
        assert_eq!(grid.type_name(), "grid");
        assert!(grid.is_enabled());
    }

    #[test]
    fn test_grid_line_generation() {
        let mut grid = GridDisplay::new("Test Grid");
        grid.props.cell_count = 2;
        grid.props.cell_size = 1.0;

        let lines = grid.generate_grid_lines();
        // For a 2x2 grid centered at origin:
        // Lines at -1, 0, 1 on each axis = 3 lines per direction = 6 total
        assert_eq!(lines.len(), 6);
    }

    #[test]
    fn test_grid_dirty_tracking() {
        let mut grid = GridDisplay::new("Test");
        assert!(grid.dirty);

        grid.dirty = false;
        grid.set_cell_count(20);
        assert!(grid.dirty);

        grid.dirty = false;
        grid.set_cell_size(2.0);
        assert!(grid.dirty);
    }
}
