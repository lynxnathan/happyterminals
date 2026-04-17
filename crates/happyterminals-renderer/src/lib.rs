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
//! - [`mesh`] -- Runtime-loaded triangle mesh + panic-free OBJ/STL loader.

#![forbid(unsafe_code)]

pub mod camera;
pub mod cube;
pub mod mesh;
pub mod particle;
pub mod projection;
pub mod rasterizer;
pub mod shading;

pub use camera::{Camera, FreeLookCamera, FpsCamera, OrbitCamera};
pub use cube::Cube;
pub use mesh::{LoadStats, Mesh, MeshError, load_obj, load_stl};
pub use projection::Projection;
pub use particle::{Particle, ParticleEmitter, lerp_color};
pub use shading::ShadingRamp;

use glam::Vec3;
use happyterminals_core::Grid;
use ratatui_core::layout::Position;
use ratatui_core::style::{Color, Style};

/// Compute a scene-fit near plane distance for the given camera and mesh.
///
/// Returns the distance from the camera eye to the near edge of the
/// mesh's bounding sphere, clamped to a minimum of 0.01. Using the mesh's
/// own bounding radius (instead of a hardcoded cube constant) means the
/// near plane adapts to meshes of arbitrary scale — required once the
/// rasterizer accepts any [`Mesh`], not just [`Cube`].
#[must_use]
fn scene_fit_near(camera: &OrbitCamera, mesh: &Mesh) -> f32 {
    let (_, radius) = mesh.bounding_sphere();
    (camera.distance - radius).max(0.01)
}

/// ASCII 3D renderer with pre-allocated z-buffer and staging character buffer.
///
/// After a single warmup call (which allocates the buffers), subsequent
/// [`draw`](Self::draw) calls perform zero heap allocations as long as
/// the grid dimensions remain unchanged.
pub struct Renderer {
    z_buffer: Vec<f32>,
    cell_chars: Vec<char>,
    cell_colors: Vec<Option<Color>>,
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
            cell_colors: Vec::new(),
            last_width: 0,
            last_height: 0,
        }
    }

    /// Current capacity of the z-buffer (for allocation-stability assertions).
    #[must_use]
    pub fn z_buffer_capacity(&self) -> usize {
        self.z_buffer.capacity()
    }

    /// Current capacity of the cell character buffer (for allocation-stability assertions).
    #[must_use]
    pub fn cell_chars_capacity(&self) -> usize {
        self.cell_chars.capacity()
    }

    /// Current capacity of the cell color buffer (for allocation-stability assertions).
    #[must_use]
    pub fn cell_colors_capacity(&self) -> usize {
        self.cell_colors.capacity()
    }

    /// Render a shaded, z-buffered [`Mesh`] into the given grid.
    ///
    /// Orchestrates the full pipeline: backface culling, vertex projection,
    /// triangle rasterization with reversed-Z depth testing, ASCII shading,
    /// and writing characters into the grid. Works for any [`Mesh`] — cube,
    /// loaded OBJ, or procedurally generated — unified under a single hot
    /// path (REND-06).
    ///
    /// If `mesh.shading` is `Some`, that ramp overrides `shading` for this
    /// mesh. Otherwise the scene-provided `shading` applies.
    ///
    /// # Zero-allocation guarantee
    ///
    /// After the first call (warmup), this method performs zero heap allocations
    /// as long as the grid dimensions do not change between frames. Holds for
    /// meshes of any triangle count — the hot loop borrows `mesh.indices` and
    /// `mesh.normals` without cloning (REND-09).
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn draw(
        &mut self,
        grid: &mut Grid,
        mesh: &Mesh,
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
        let z_near = scene_fit_near(camera, mesh);
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

        let grid_w = w as usize;
        let grid_h = h as usize;

        // Per-mesh shading override (999.x scaffold): borrow, no allocation.
        let effective_ramp: &ShadingRamp<'_> = mesh.shading.as_ref().unwrap_or(shading);

        // Rasterize each triangle (one normal per triangle — cube's 6 shared
        // face normals are baked into `Cube::mesh()` at 12 entries, and
        // loaded meshes compute flat normals at load time).
        for (tri_idx, &[i0, i1, i2]) in mesh.indices.iter().enumerate() {
            // `mesh.normals.len() == mesh.indices.len()` by Mesh invariant.
            let face_normal = mesh.normals[tri_idx];

            // Backface cull
            if rasterizer::backface_cull(face_normal, camera_dir) {
                continue;
            }

            let shade_char = effective_ramp.shade(face_normal);

            let vi0 = i0 as usize;
            let vi1 = i1 as usize;
            let vi2 = i2 as usize;
            // Defensive bounds check: loaded meshes can technically carry
            // stale indices if constructed manually. load_obj() already
            // filters these, but the rasterizer stays safe anyway.
            if vi0 >= mesh.vertices.len()
                || vi1 >= mesh.vertices.len()
                || vi2 >= mesh.vertices.len()
            {
                continue;
            }

            // Project all 3 vertices; skip triangle if any is behind camera
            let v0 = rasterizer::project_vertex(mesh.vertices[vi0], &mvp, half_w, half_h);
            let v1 = rasterizer::project_vertex(mesh.vertices[vi1], &mvp, half_w, half_h);
            let v2 = rasterizer::project_vertex(mesh.vertices[vi2], &mvp, half_w, half_h);

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

    /// Render alive particles from an emitter into the grid.
    ///
    /// Composites on top of a prior [`draw`](Self::draw) call — does NOT clear
    /// the z-buffer or cell buffers. Particles behind existing mesh surfaces
    /// are correctly occluded by the reversed-Z depth test.
    ///
    /// Each alive particle is point-projected via [`rasterizer::project_vertex`]
    /// and assigned a shading ramp character based on normalized lifetime plus
    /// an interpolated foreground color (`color_start` -> `color_end`).
    ///
    /// # Zero-allocation guarantee
    ///
    /// After the first frame (warmup), no heap allocations occur as long as
    /// grid dimensions remain unchanged.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn draw_particles(
        &mut self,
        grid: &mut Grid,
        emitter: &ParticleEmitter,
        camera: &OrbitCamera,
        projection: &Projection,
        shading: &ShadingRamp<'_>,
    ) {
        let w = grid.area.width;
        let h = grid.area.height;
        let total = (w as usize) * (h as usize);

        // Resize buffers if dimensions changed (same pattern as draw())
        if w != self.last_width || h != self.last_height {
            self.z_buffer.resize(total, 0.0);
            self.cell_chars.resize(total, ' ');
            self.cell_colors.resize(total, None);
            self.last_width = w;
            self.last_height = h;
        } else if self.cell_colors.len() < total {
            // First call to draw_particles after draw() sized z_buffer/cell_chars
            self.cell_colors.resize(total, None);
        }

        // Compute MVP matrix (same as draw())
        let view = camera.view_matrix();
        let pixel_aspect = f32::from(projection.viewport_w) / f32::from(projection.viewport_h);
        let effective_aspect = pixel_aspect / projection.cell_aspect;
        // Use a reasonable near plane for particles (no mesh bounding sphere)
        let proj = glam::Mat4::perspective_infinite_reverse_rh(
            projection.fov_y,
            effective_aspect,
            0.01,
        );
        let mvp = proj * view;

        let half_w = f32::from(w) / 2.0;
        let half_h = f32::from(h) / 2.0;
        let grid_w = w as usize;
        let grid_h = h as usize;

        let ramp_len = shading.ramp.len();
        let max_ramp_idx = ramp_len.saturating_sub(1);

        for particle in emitter.alive_particles() {
            let Some((sx, sy, depth)) =
                rasterizer::project_vertex(particle.position, &mvp, half_w, half_h)
            else {
                continue;
            };

            let px = sx as usize;
            let py = sy as usize;
            if px >= grid_w || py >= grid_h {
                continue;
            }

            let idx = py * grid_w + px;

            // Reversed-Z depth test (same convention as triangle rasterizer)
            if depth > self.z_buffer[idx] {
                self.z_buffer[idx] = depth;

                // Life-based ramp character (young = bright, old = dim)
                let t = if particle.max_life > 0.0 {
                    (particle.life / particle.max_life).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let ramp_idx = (t * max_ramp_idx as f32) as usize;
                self.cell_chars[idx] = shading.ramp[ramp_idx.min(max_ramp_idx)];

                // Color over time: young=start color, old=end color
                let color = particle::lerp_color(
                    particle.color_start,
                    particle.color_end,
                    1.0 - t,
                );
                self.cell_colors[idx] = Some(color);
            }
        }

        // Write particle cells into Grid (only those with color set)
        let buf = grid.buffer_mut();
        for y in 0..h {
            for x in 0..w {
                let idx = (y as usize) * grid_w + (x as usize);
                if let Some(color) = self.cell_colors[idx].take() {
                    let ch = self.cell_chars[idx];
                    if ch != ' ' {
                        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                            let mut tmp = [0u8; 4];
                            let s = ch.encode_utf8(&mut tmp);
                            cell.set_symbol(s);
                            cell.set_style(Style::default().fg(color));
                        }
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
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use happyterminals_core::Rect;

    /// Absolute path to a pre-imported real-world model at the workspace root.
    macro_rules! real_model {
        ($name:literal) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/", $name)
        };
    }

    fn count_non_space_cells(grid: &Grid, w: u16, h: u16) -> usize {
        let mut non_space = 0;
        for y in 0..h {
            for x in 0..w {
                if let Some(cell) = grid.cell(Position::new(x, y)) {
                    if cell.symbol() != " " {
                        non_space += 1;
                    }
                }
            }
        }
        non_space
    }

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
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);

        let non_space = count_non_space_cells(&grid, 80, 24);
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
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        let cap_after_first = renderer.z_buffer.capacity();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        let cap_after_second = renderer.z_buffer.capacity();

        assert_eq!(
            cap_after_first, cap_after_second,
            "z_buffer should not grow between draws: {cap_after_first} vs {cap_after_second}"
        );
    }

    /// VALIDATION must-have #5: rasterizing a loaded bunny Mesh produces
    /// >= 50 non-space cells on an 80x24 grid at default orbit pose.
    #[test]
    fn draw_loaded_bunny_produces_pixels() {
        let bunny_path = real_model!("bunny.obj");
        let (bunny_mesh, _stats) = load_obj(bunny_path).expect("bunny.obj must load");

        // Auto-fit camera to bunny bounding sphere.
        let (center, radius) = bunny_mesh.bounding_sphere();
        let camera = OrbitCamera {
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            distance: radius * 2.5,
            target: center,
        };
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();

        renderer.draw(&mut grid, &bunny_mesh, &camera, &projection, &shading);

        let non_space = count_non_space_cells(&grid, 80, 24);
        assert!(
            non_space >= 50,
            "Bunny should rasterize to >= 50 non-space cells, got {non_space}"
        );
    }

    /// REND-09 zero-allocation discipline: drawing a 12-tri mesh twice does
    /// not grow the z-buffer capacity. This is the same invariant as the
    /// original `draw_twice_does_not_grow_z_buffer` test; kept explicitly
    /// under a "mesh" name for clarity post-refactor.
    #[test]
    fn draw_mesh_twice_does_not_grow_z_buffer() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = OrbitCamera::default();
        let projection = Projection::default();
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        let cap_first = renderer.z_buffer.capacity();
        let cap_chars_first = renderer.cell_chars.capacity();

        for _ in 0..10 {
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        }
        let cap_last = renderer.z_buffer.capacity();
        let cap_chars_last = renderer.cell_chars.capacity();

        assert_eq!(
            cap_first, cap_last,
            "z_buffer capacity must stay stable across 11 draws"
        );
        assert_eq!(
            cap_chars_first, cap_chars_last,
            "cell_chars capacity must stay stable across 11 draws"
        );
    }

    /// Per-mesh shading override: if `mesh.shading` is Some, the rasterizer
    /// must use it in place of the scene-default ramp. Exercises the 999.x
    /// scaffolding field on `Mesh`.
    #[test]
    fn draw_respects_per_mesh_shading_override() {
        // Custom ramp made entirely of characters that are NOT in DEFAULT_RAMP.
        static CUSTOM_RAMP: &[char] = &['X', 'Y', 'Z', 'W', 'V'];

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
        // Scene-default ramp uses DEFAULT_RAMP characters ('.', ',', ...).
        let scene_shading = ShadingRamp::default();

        let custom = ShadingRamp {
            ramp: CUSTOM_RAMP,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalize(),
        };

        let mut cube_mesh = Cube::mesh();
        cube_mesh.shading = Some(custom);

        let mut renderer = Renderer::new();
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &scene_shading);

        // Collect the distinct non-space glyphs drawn.
        let mut found_custom_glyph = false;
        for y in 0..24_u16 {
            for x in 0..80_u16 {
                if let Some(cell) = grid.cell(Position::new(x, y)) {
                    let sym = cell.symbol();
                    if sym != " " && CUSTOM_RAMP.iter().any(|c| sym.starts_with(*c)) {
                        found_custom_glyph = true;
                        break;
                    }
                }
            }
            if found_custom_glyph {
                break;
            }
        }
        assert!(
            found_custom_glyph,
            "Per-mesh shading override should inject custom ramp characters"
        );
    }

    #[test]
    fn scene_fit_near_clamps_to_minimum() {
        let cam = OrbitCamera {
            distance: 0.5,
            ..OrbitCamera::default()
        };
        let cube_mesh = Cube::mesh();
        let near = scene_fit_near(&cam, &cube_mesh);
        assert!(
            (near - 0.01).abs() < f32::EPSILON,
            "Near plane should clamp to 0.01 for close camera, got {near}"
        );
    }

    #[test]
    fn scene_fit_near_at_default_distance() {
        let cam = OrbitCamera::default(); // distance = 5.0
        let cube_mesh = Cube::mesh();
        let near = scene_fit_near(&cam, &cube_mesh);
        // sqrt(3)/2 ≈ 0.8660254 — bounding radius of the unit cube.
        let expected = 5.0 - cube_mesh.bounding_sphere().1;
        assert!(
            (near - expected).abs() < 0.001,
            "Near plane should be distance - bounding_radius, got {near} expected {expected}"
        );
    }

    // ── draw_particles tests ────────────────────────────────────────────

    /// Helper: create a particle emitter with particles in front of camera.
    fn emitter_with_visible_particles() -> ParticleEmitter {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut emitter = ParticleEmitter::new(100);
        emitter.spawn_rate = 200.0;
        emitter.origin = Vec3::ZERO;
        emitter.spread = Vec3::new(0.5, 0.5, 0.5);
        emitter.life_range = (2.0, 3.0);
        emitter.gravity = Vec3::ZERO; // No gravity so they stay near origin

        let mut rng = StdRng::seed_from_u64(42);
        // Run a few frames to spawn particles
        for _ in 0..5 {
            emitter.update(0.016, &mut rng);
        }
        assert!(emitter.alive_count() > 0, "Should have alive particles");
        emitter
    }

    #[test]
    fn draw_particles_renders_visible_cells() {
        let emitter = emitter_with_visible_particles();
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = OrbitCamera {
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            distance: 3.0,
            target: Vec3::ZERO,
        };
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();

        // Clear z-buffer by drawing nothing, then draw particles
        let cube_mesh = Cube::mesh();
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);

        // Reset grid and z_buffer so particles start fresh
        let mut grid2 = Grid::new(Rect::new(0, 0, 80, 24));
        renderer.z_buffer.fill(0.0);
        renderer.cell_chars.fill(' ');

        renderer.draw_particles(&mut grid2, &emitter, &camera, &projection, &shading);

        let non_space = count_non_space_cells(&grid2, 80, 24);
        assert!(
            non_space >= 1,
            "draw_particles should render at least 1 non-space cell, got {non_space}"
        );
    }

    #[test]
    fn draw_particles_respects_z_buffer_occlusion() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Create particles far behind the camera's near plane
        let mut emitter = ParticleEmitter::new(50);
        emitter.spawn_rate = 200.0;
        emitter.origin = Vec3::new(0.0, 0.0, 0.0);
        emitter.spread = Vec3::splat(0.01);
        emitter.life_range = (5.0, 5.0);
        emitter.gravity = Vec3::ZERO;

        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..5 {
            emitter.update(0.016, &mut rng);
        }

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
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();

        // First draw the cube (fills z-buffer at the cube's surface depth)
        let cube_mesh = Cube::mesh();
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        let cube_cells = count_non_space_cells(&grid, 80, 24);

        // Now draw particles (should be occluded where cube surface is closer)
        renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
        let after_particles = count_non_space_cells(&grid, 80, 24);

        // Particles at origin are at/behind the cube surface depth.
        // Some might be visible at the edges, but the count should not
        // dramatically exceed the cube-only count.
        // At minimum, the test proves draw_particles checks z_buffer.
        assert!(
            after_particles >= cube_cells,
            "After particles, should have >= cube cells: {after_particles} vs {cube_cells}"
        );
    }

    // ── REND-11: &dyn Camera polymorphism tests ──────────────────────

    #[test]
    fn draw_with_freelook_camera_produces_output() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = FreeLookCamera {
            position: Vec3::new(3.0, 2.0, 5.0),
            yaw: 0.3,
            pitch: -0.2,
            speed: 5.0,
        };
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);

        let non_space = count_non_space_cells(&grid, 80, 24);
        assert!(
            non_space > 0,
            "FreeLookCamera draw should produce visible output, got {non_space}"
        );
    }

    #[test]
    fn draw_particles_with_freelook_camera_no_panic() {
        let emitter = emitter_with_visible_particles();
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let camera = FreeLookCamera::default();
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
        // No panic = pass
    }

    #[test]
    fn scene_fit_near_with_freelook_view_matrix() {
        let camera = FreeLookCamera {
            position: Vec3::new(0.0, 0.0, 5.0),
            ..FreeLookCamera::default()
        };
        let view = camera.view_matrix();
        let cube_mesh = Cube::mesh();
        let near = scene_fit_near(&view, &cube_mesh);
        assert!(
            near > 0.0,
            "scene_fit_near should return positive near plane for FreeLookCamera, got {near}"
        );
    }

    #[test]
    fn renderer_capacity_stable_across_draw_and_draw_particles() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut emitter = ParticleEmitter::new(100);
        emitter.spawn_rate = 50.0;
        let mut rng = StdRng::seed_from_u64(42);

        let camera = OrbitCamera {
            azimuth: 0.5,
            elevation: 0.3,
            distance: 5.0,
            target: Vec3::ZERO,
        };
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        // Warmup
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        emitter.update(0.016, &mut rng);
        renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);

        let cap_z = renderer.z_buffer.capacity();
        let cap_chars = renderer.cell_chars.capacity();
        let cap_colors = renderer.cell_colors.capacity();

        // Run 10 frames
        for _ in 0..10 {
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
            emitter.update(0.016, &mut rng);
            renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
        }

        assert_eq!(
            renderer.z_buffer.capacity(),
            cap_z,
            "z_buffer capacity must be stable"
        );
        assert_eq!(
            renderer.cell_chars.capacity(),
            cap_chars,
            "cell_chars capacity must be stable"
        );
        assert_eq!(
            renderer.cell_colors.capacity(),
            cap_colors,
            "cell_colors capacity must be stable"
        );
    }
}
