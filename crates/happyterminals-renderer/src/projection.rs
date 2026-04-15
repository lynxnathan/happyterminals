//! Perspective projection with cell aspect ratio correction.
//!
//! Terminal cells are typically ~2x taller than wide. The [`Projection`] struct
//! accounts for this by dividing the pixel aspect ratio by `cell_aspect`,
//! producing an `effective_aspect` that prevents vertical stretching.
//!
//! Uses [`glam::Mat4::perspective_infinite_reverse_rh`] for reversed-Z depth:
//! near maps to 1.0, infinity maps to 0.0.

use glam::Mat4;

/// Perspective projection parameters for terminal rendering.
///
/// The key insight is that `effective_aspect = (viewport_w / viewport_h) / cell_aspect`.
/// With a default `cell_aspect` of 2.0, an 80x24 viewport has effective aspect 1.667
/// rather than the naive 3.333.
#[derive(Debug, Clone)]
pub struct Projection {
    /// Vertical field of view in radians (default: PI/4).
    pub fov_y: f32,
    /// Cell aspect ratio -- how many times taller a cell is than wide (default: 2.0).
    pub cell_aspect: f32,
    /// Viewport width in cells.
    pub viewport_w: u16,
    /// Viewport height in cells.
    pub viewport_h: u16,
}

impl Projection {
    /// Compute the projection matrix with cell aspect correction and reversed-Z.
    ///
    /// The returned matrix uses infinite reversed-Z perspective:
    /// near plane maps to depth 1.0, infinity maps to depth 0.0.
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        let pixel_aspect = f32::from(self.viewport_w) / f32::from(self.viewport_h);
        let effective_aspect = pixel_aspect / self.cell_aspect;

        Mat4::perspective_infinite_reverse_rh(self.fov_y, effective_aspect, 0.1)
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self {
            fov_y: std::f32::consts::FRAC_PI_4,
            cell_aspect: 2.0,
            viewport_w: 80,
            viewport_h: 24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_matrix_has_aspect_correction() {
        let proj = Projection {
            fov_y: std::f32::consts::FRAC_PI_4,
            cell_aspect: 2.0,
            viewport_w: 80,
            viewport_h: 24,
        };
        let mat = proj.matrix();
        // element [0][0] != element [1][1] because aspect != 1.0
        let col0 = mat.col(0);
        let col1 = mat.col(1);
        assert!(
            (col0.x - col1.y).abs() > f32::EPSILON,
            "Projection should have different x and y scaling due to aspect correction"
        );
    }

    #[test]
    fn effective_aspect_is_pixel_aspect_divided_by_cell_aspect() {
        let proj = Projection {
            fov_y: std::f32::consts::FRAC_PI_4,
            cell_aspect: 2.0,
            viewport_w: 80,
            viewport_h: 24,
        };
        let pixel_aspect = 80.0_f32 / 24.0;
        let effective_aspect = pixel_aspect / 2.0;
        // effective_aspect should be ~1.667, not ~3.333
        assert!(
            (effective_aspect - 1.666_666_7).abs() < 0.001,
            "effective_aspect = {effective_aspect}, expected ~1.667"
        );

        // The projection matrix x-scale should reflect effective_aspect
        let mat = proj.matrix();
        let x_scale = mat.col(0).x;
        let y_scale = mat.col(1).y;
        // x_scale = y_scale / effective_aspect (from the perspective formula)
        let expected_ratio = y_scale / effective_aspect;
        assert!(
            (x_scale - expected_ratio).abs() < 0.001,
            "x_scale={x_scale}, expected y_scale/effective_aspect={expected_ratio}"
        );
    }

    #[test]
    fn default_projection_has_expected_values() {
        let proj = Projection::default();
        assert!((proj.fov_y - std::f32::consts::FRAC_PI_4).abs() < f32::EPSILON);
        assert!((proj.cell_aspect - 2.0).abs() < f32::EPSILON);
        assert_eq!(proj.viewport_w, 80);
        assert_eq!(proj.viewport_h, 24);
    }

    #[test]
    fn reversed_z_near_maps_to_one() {
        let proj = Projection::default();
        let mat = proj.matrix();
        // In reversed-Z infinite perspective, mat[2][2] should be 0.0
        // and mat[3][2] should be z_near (0.1)
        let col2 = mat.col(2);
        let col3 = mat.col(3);
        assert!(
            col2.z.abs() < f32::EPSILON,
            "mat[2][2] should be ~0 for reversed-Z infinite, got {}",
            col2.z
        );
        assert!(
            (col3.z - 0.1).abs() < 0.001,
            "mat[3][2] should be z_near=0.1 for reversed-Z, got {}",
            col3.z
        );
    }
}
