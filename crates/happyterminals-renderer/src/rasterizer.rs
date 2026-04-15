//! Triangle rasterizer using half-space edge functions.
//!
//! Provides the low-level rasterization primitives for the ASCII 3D renderer:
//! vertex projection, edge function evaluation, triangle rasterization with
//! reversed-Z depth testing, and backface culling.

use glam::{Mat4, Vec3, Vec4};

/// Compute the signed area of the parallelogram formed by edge (v0->v1) and point p.
///
/// Returns positive if p is on the left side of the edge (inside for CCW winding),
/// zero if on the edge, negative if on the right side.
#[inline]
#[must_use]
#[allow(clippy::similar_names)]
pub fn edge_function(v0x: f32, v0y: f32, v1x: f32, v1y: f32, px: f32, py: f32) -> f32 {
    (v1x - v0x) * (py - v0y) - (v1y - v0y) * (px - v0x)
}

/// Project a 3D vertex to screen coordinates using an MVP matrix.
///
/// Returns `Some((screen_x, screen_y, ndc_z))` or `None` if the vertex is behind
/// the camera (clip.w <= 0).
///
/// `half_w` and `half_h` are half the grid dimensions in cells.
#[inline]
#[must_use]
pub fn project_vertex(pos: Vec3, mvp: &Mat4, half_w: f32, half_h: f32) -> Option<(f32, f32, f32)> {
    let clip = *mvp * Vec4::new(pos.x, pos.y, pos.z, 1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let inv_w = 1.0 / clip.w;
    let ndc_x = clip.x * inv_w;
    let ndc_y = clip.y * inv_w;
    let ndc_z = clip.z * inv_w;
    let screen_x = (ndc_x + 1.0) * half_w;
    let screen_y = (1.0 - ndc_y) * half_h;
    Some((screen_x, screen_y, ndc_z))
}

/// Rasterize a single triangle into the cell character and z-buffer arrays.
///
/// Uses the half-space (edge function) method. For each cell in the triangle's
/// bounding box, tests all three edge functions. If all are >= 0 (inside for CCW
/// winding), performs barycentric depth interpolation and a reversed-Z depth test
/// (depth > current wins, since closer objects have higher depth in reversed-Z).
///
/// # Arguments
/// * `sv` - Three screen-space vertices as (`screen_x`, `screen_y`, `ndc_z`)
/// * `z_buffer` - Mutable slice of depth values (row-major, initialized to 0.0 for reversed-Z far)
/// * `grid_w` - Grid width in cells
/// * `grid_h` - Grid height in cells
/// * `shade_char` - The ASCII character to write for this triangle
/// * `cell_chars` - Mutable slice of characters (row-major, same size as `z_buffer`)
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::similar_names
)]
pub fn rasterize_triangle(
    sv: &[(f32, f32, f32); 3],
    z_buffer: &mut [f32],
    grid_w: usize,
    grid_h: usize,
    shade_char: char,
    cell_chars: &mut [char],
) {
    let (x0, y0, z0) = sv[0];
    let (x1, y1, z1) = sv[1];
    let (x2, y2, z2) = sv[2];

    // Bounding box clamped to grid
    let min_x = x0.min(x1).min(x2).max(0.0) as usize;
    let max_x = (x0.max(x1).max(x2).ceil() as usize).min(grid_w.saturating_sub(1));
    let min_y = y0.min(y1).min(y2).max(0.0) as usize;
    let max_y = (y0.max(y1).max(y2).ceil() as usize).min(grid_h.saturating_sub(1));

    // Total area (for barycentric normalization)
    let area = edge_function(x0, y0, x1, y1, x2, y2);
    if area.abs() < f32::EPSILON {
        return; // Degenerate triangle
    }
    let inv_area = 1.0 / area;
    // After Y-flip in projection, CCW triangles become CW in screen space,
    // making the area negative. We accept both windings by checking all edge
    // functions have the same sign as the total area.
    let sign_positive = area > 0.0;

    for py in min_y..=max_y {
        let py_f = py as f32 + 0.5;
        for px in min_x..=max_x {
            let px_f = px as f32 + 0.5;

            let w0 = edge_function(x1, y1, x2, y2, px_f, py_f);
            let w1 = edge_function(x2, y2, x0, y0, px_f, py_f);
            let w2 = edge_function(x0, y0, x1, y1, px_f, py_f);

            let inside = if sign_positive {
                w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
            } else {
                w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
            };

            if inside {
                // Barycentric depth interpolation
                let bary0 = w0 * inv_area;
                let bary1 = w1 * inv_area;
                let bary2 = w2 * inv_area;
                let depth = bary0 * z0 + bary1 * z1 + bary2 * z2;

                let idx = py * grid_w + px;
                // Reversed-Z: greater depth = closer to camera
                if depth > z_buffer[idx] {
                    z_buffer[idx] = depth;
                    cell_chars[idx] = shade_char;
                }
            }
        }
    }
}

/// Returns `true` if the face should be culled (back-facing).
///
/// A face is back-facing when its normal points away from the camera,
/// i.e., the dot product of the face normal and the direction from
/// surface to camera is <= 0.
#[inline]
#[must_use]
pub fn backface_cull(face_normal: Vec3, camera_dir: Vec3) -> bool {
    // camera_dir points FROM camera TO target (into the scene).
    // Face is back-facing if normal points in the same direction as camera_dir.
    face_normal.dot(camera_dir) >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_function_positive_for_left_side() {
        // Edge from (0,0) to (1,0), point at (0.5, 1.0) is on the left
        let val = edge_function(0.0, 0.0, 1.0, 0.0, 0.5, 1.0);
        assert!(val > 0.0, "Point on left of edge should be positive, got {val}");
    }

    #[test]
    fn edge_function_negative_for_right_side() {
        let val = edge_function(0.0, 0.0, 1.0, 0.0, 0.5, -1.0);
        assert!(val < 0.0, "Point on right of edge should be negative, got {val}");
    }

    #[test]
    fn edge_function_zero_on_edge() {
        let val = edge_function(0.0, 0.0, 1.0, 0.0, 0.5, 0.0);
        assert!(val.abs() < f32::EPSILON, "Point on edge should be zero, got {val}");
    }

    #[test]
    fn project_vertex_identity_mvp_center() {
        // With identity MVP, a vertex at origin projects to screen center
        let mvp = Mat4::IDENTITY;
        let result = project_vertex(Vec3::ZERO, &mvp, 5.0, 5.0);
        // clip = (0,0,0,1), ndc = (0,0,0), screen = (5, 5, 0)
        let (sx, sy, _sz) = result.map_or_else(|| panic!("Should not be None"), |v| v);
        assert!((sx - 5.0).abs() < 0.01, "screen_x should be 5.0 (center), got {sx}");
        assert!((sy - 5.0).abs() < 0.01, "screen_y should be 5.0 (center), got {sy}");
    }

    #[test]
    fn project_vertex_behind_camera_returns_none() {
        // Create a matrix that makes clip.w <= 0 for vertex at (0,0,10)
        // A look_at from origin looking at +Z with the vertex behind at (0,0,-10)
        // Using a simple projection: reversed-Z perspective
        let proj = Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::FRAC_PI_4,
            1.0,
            0.1,
        );
        let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
        let mvp = proj * view;
        // Vertex at (0, 0, 10) is behind the camera (camera looks at -Z)
        let result = project_vertex(Vec3::new(0.0, 0.0, 10.0), &mvp, 40.0, 12.0);
        assert!(result.is_none(), "Vertex behind camera should return None");
    }

    #[test]
    fn rasterize_triangle_writes_to_center_cells() {
        let grid_w = 10;
        let grid_h = 10;
        let size = grid_w * grid_h;
        let mut z_buffer = vec![0.0_f32; size];
        let mut cell_chars = vec![' '; size];

        // A triangle covering roughly the center of the grid
        let sv = [
            (5.0, 2.0, 0.5),
            (8.0, 8.0, 0.5),
            (2.0, 8.0, 0.5),
        ];

        rasterize_triangle(&sv, &mut z_buffer, grid_w, grid_h, '#', &mut cell_chars);

        // Count non-space characters
        let filled: usize = cell_chars.iter().filter(|&&c| c != ' ').count();
        assert!(filled > 0, "Triangle should have written some characters, got 0");
    }

    #[test]
    fn backface_cull_facing_away() {
        // Face normal points at +Z, camera looks at -Z (camera_dir = -Z)
        // Normal dot camera_dir = (0,0,1) . (0,0,-1) = -1 < 0 -> not culled (visible)
        assert!(!backface_cull(Vec3::Z, Vec3::NEG_Z), "Front face should NOT be culled");
    }

    #[test]
    fn backface_cull_facing_camera() {
        // Face normal points at -Z, camera looks at -Z (camera_dir = -Z)
        // Normal dot camera_dir = (0,0,-1) . (0,0,-1) = 1 >= 0 -> culled
        assert!(backface_cull(Vec3::NEG_Z, Vec3::NEG_Z), "Back face should be culled");
    }
}
