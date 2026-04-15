//! # happyterminals-renderer
//!
//! Fresh ASCII 3D rasterizer for [happyterminals](https://github.com/lynxnathan/happyterminals):
//! perspective projection with configurable cell aspect ratio, reversed-Z buffer,
//! configurable ASCII shading ramp, OBJ/STL mesh loading, and particle infrastructure.
//!
//! Not a fork of any existing renderer -- see the "fresh implementation" decision
//! in `.eclusa/PROJECT.md` Key Decisions.
//!
//! ## Modules
//!
//! - [`projection`] -- Perspective projection with cell aspect ratio correction.
//! - [`cube`] -- Unit cube primitive (8 vertices, 12 triangles, 6 face normals).
//! - [`shading`] -- ASCII shading ramp mapping `NdotL` to characters.
//! - [`camera`] -- Orbit camera converting spherical coordinates to view matrix.
//! - [`rasterizer`] -- Triangle rasterizer with half-space edge functions.

#![forbid(unsafe_code)]

pub mod camera;
pub mod cube;
pub mod projection;
pub mod rasterizer;
pub mod shading;

pub use camera::OrbitCamera;
pub use cube::Cube;
pub use projection::Projection;
pub use shading::ShadingRamp;

use glam::Vec3;
use happyterminals_core::Grid;
use ratatui_core::layout::Position;

/// Bounding radius of the unit cube: sqrt(3)/2.
const CUBE_BOUNDING_RADIUS: f32 = 0.866_025_4;

/// Compute a scene-fit near plane distance for the given camera.
///
/// Returns the distance from the camera eye to the near edge of the
/// cube's bounding sphere, clamped to a minimum of 0.01.
#[must_use]
fn scene_fit_near(camera: &OrbitCamera) -> f32 {
    (camera.distance - CUBE_BOUNDING_RADIUS).max(0.01)
}

/// ASCII 3D renderer with pre-allocated z-buffer and staging character buffer.
///
/// After a single warmup call (which allocates the buffers), subsequent
/// [`draw`](Self::draw) calls perform zero heap allocations as long as
/// the grid dimensions remain unchanged.
pub struct Renderer {
    z_buffer: Vec<f32>,
    cell_chars: Vec<char>,
    last_width: u16,
    last_height: u16,
}

impl Renderer {
    /// Create a new renderer with empty buffers.
    ///
    /// The first call to [`draw`](Self::draw) allocates the z-buffer and
    /// staging buffer; subsequent calls reuse them.
    #[must_use]
    pub fn new() -> Self {
        Self {
            z_buffer: Vec::new(),
            cell_chars: Vec::new(),
            last_width: 0,
            last_height: 0,
        }
    }

    /// Render a shaded, z-buffered cube into the given grid.
    ///
    /// Orchestrates the full pipeline: backface culling, vertex projection,
    /// triangle rasterization with reversed-Z depth testing, ASCII shading,
    /// and writing characters into the grid.
    ///
    /// # Zero-allocation guarantee
    ///
    /// After the first call (warmup), this method performs zero heap allocations
    /// as long as the grid dimensions do not change between frames.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn draw(
        &mut self,
        grid: &mut Grid,
        camera: &OrbitCamera,
        projection: &Projection,
        shading: &ShadingRamp<'_>,
    ) {
        let w = grid.area.width;
        let h = grid.area.height;
        let total = (w as usize) * (h as usize);

        // Resize buffers only if dimensions changed (zero-alloc after warmup)
        if w != self.last_width || h != self.last_height {
            self.z_buffer.resize(total, 0.0);
            self.cell_chars.resize(total, ' ');
            self.last_width = w;
            self.last_height = h;
        }

        // Clear buffers: reversed-Z far = 0.0, cells = space
        self.z_buffer.fill(0.0);
        self.cell_chars.fill(' ');

        // Compute matrices
        let view = camera.view_matrix();
        let z_near = scene_fit_near(camera);
        let pixel_aspect = f32::from(projection.viewport_w) / f32::from(projection.viewport_h);
        let effective_aspect = pixel_aspect / projection.cell_aspect;
        let proj = glam::Mat4::perspective_infinite_reverse_rh(
            projection.fov_y,
            effective_aspect,
            z_near,
        );
        let mvp = proj * view;

        let half_w = f32::from(w) / 2.0;
        let half_h = f32::from(h) / 2.0;

        // Camera direction (from eye toward target) for backface culling
        let eye = camera.target
            + Vec3::new(
                camera.distance * camera.elevation.cos() * camera.azimuth.sin(),
                camera.distance * camera.elevation.sin(),
                camera.distance * camera.elevation.cos() * camera.azimuth.cos(),
            );
        let camera_dir = (camera.target - eye).normalize();

        // Rasterize each face (2 triangles per face)
        let grid_w = w as usize;
        let grid_h = h as usize;

        for face in 0..6 {
            let face_normal = Cube::FACE_NORMALS[face];

            // Backface cull
            if rasterizer::backface_cull(face_normal, camera_dir) {
                continue;
            }

            let shade_char = shading.shade(face_normal);

            for tri_offset in 0..2 {
                let tri_idx = face * 2 + tri_offset;
                let indices = Cube::INDICES[tri_idx];

                // Project all 3 vertices; skip triangle if any is behind camera
                let v0 = rasterizer::project_vertex(Cube::VERTICES[indices[0]], &mvp, half_w, half_h);
                let v1 = rasterizer::project_vertex(Cube::VERTICES[indices[1]], &mvp, half_w, half_h);
                let v2 = rasterizer::project_vertex(Cube::VERTICES[indices[2]], &mvp, half_w, half_h);

                let (Some(sv0), Some(sv1), Some(sv2)) = (v0, v1, v2) else {
                    continue;
                };

                let sv = [sv0, sv1, sv2];
                rasterizer::rasterize_triangle(
                    &sv,
                    &mut self.z_buffer,
                    grid_w,
                    grid_h,
                    shade_char,
                    &mut self.cell_chars,
                );
            }
        }

        // Write cell_chars into Grid
        let buf = grid.buffer_mut();
        for y in 0..h {
            for x in 0..w {
                let idx = (y as usize) * grid_w + (x as usize);
                let ch = self.cell_chars[idx];
                if ch != ' ' {
                    if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                        // Use a stack-allocated tiny string to avoid heap alloc
                        let mut tmp = [0u8; 4];
                        let s = ch.encode_utf8(&mut tmp);
                        cell.set_symbol(s);
                    }
                }
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use happyterminals_core::Rect;

    #[test]
    fn draw_renders_visible_cube() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = OrbitCamera {
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            distance: 5.0,
            target: Vec3::ZERO,
        };
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();

        renderer.draw(&mut grid, &camera, &projection, &shading);

        // Count non-space cells
        let mut non_space = 0;
        for y in 0..24_u16 {
            for x in 0..80_u16 {
                if let Some(cell) = grid.cell(Position::new(x, y)) {
                    if cell.symbol() != " " {
                        non_space += 1;
                    }
                }
            }
        }
        assert!(
            non_space > 10,
            "Cube should be visible (non-space cells), got {non_space}"
        );
    }

    #[test]
    fn draw_twice_does_not_grow_z_buffer() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = OrbitCamera::default();
        let projection = Projection::default();
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();

        renderer.draw(&mut grid, &camera, &projection, &shading);
        let cap_after_first = renderer.z_buffer.capacity();

        renderer.draw(&mut grid, &camera, &projection, &shading);
        let cap_after_second = renderer.z_buffer.capacity();

        assert_eq!(
            cap_after_first, cap_after_second,
            "z_buffer should not grow between draws: {cap_after_first} vs {cap_after_second}"
        );
    }

    #[test]
    fn scene_fit_near_clamps_to_minimum() {
        let cam = OrbitCamera {
            distance: 0.5,
            ..OrbitCamera::default()
        };
        let near = scene_fit_near(&cam);
        assert!(
            (near - 0.01).abs() < f32::EPSILON,
            "Near plane should clamp to 0.01 for close camera, got {near}"
        );
    }

    #[test]
    fn scene_fit_near_at_default_distance() {
        let cam = OrbitCamera::default(); // distance = 5.0
        let near = scene_fit_near(&cam);
        let expected = 5.0 - CUBE_BOUNDING_RADIUS;
        assert!(
            (near - expected).abs() < 0.001,
            "Near plane should be distance - bounding_radius, got {near} expected {expected}"
        );
    }
}
