//! Criterion benchmark for `Renderer::draw()` hot path.
//!
//! Measures the rendering time and verifies the zero-allocation property
//! of the draw loop after warmup. Covers two scales:
//! - 12-triangle cube: baseline / low-count cost model.
//! - ~5000-triangle Stanford bunny: real-world mesh at REND-09 scale.

// Benches are harness binaries; panicking at bench setup (before the hot loop)
// is acceptable — an unloadable fixture is a configuration failure, not a
// runtime error to recover from.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use criterion::{criterion_group, criterion_main, Criterion};
use happyterminals_core::{Grid, Rect};
use happyterminals_renderer::{Cube, OrbitCamera, Projection, Renderer, ShadingRamp, load_obj};

fn bench_draw(c: &mut Criterion) {
    let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
    let camera = OrbitCamera {
        azimuth: std::f32::consts::FRAC_PI_4,
        elevation: std::f32::consts::FRAC_PI_6,
        distance: 5.0,
        target: glam::Vec3::ZERO,
    };
    let projection = Projection {
        viewport_w: 80,
        viewport_h: 24,
        ..Projection::default()
    };
    let shading = ShadingRamp::default();
    let mut renderer = Renderer::new();
    let cube_mesh = Cube::mesh();

    // Warmup: allocate buffers once
    renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);

    c.bench_function("renderer_draw_80x24", |b| {
        b.iter(|| {
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        });
    });
}

/// Bench rasterizing a loaded Stanford bunny (~5000 triangles) — proves
/// `Renderer::draw` scales beyond the 12-triangle cube while preserving the
/// warmup-once, zero-per-frame-allocation invariant (REND-09 at real-world
/// mesh scale). VALIDATION must-have #6.
fn bench_draw_mesh_bunny(c: &mut Criterion) {
    let bunny_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/models/bunny.obj"
    );
    let (bunny_mesh, _stats) =
        load_obj(bunny_path).expect("bunny.obj must load for bench");

    // Auto-fit camera to bounding sphere so the bunny fits the 80x24 frame.
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

    // Warmup: first draw allocates z-buffer + cell_chars buffers.
    renderer.draw(&mut grid, &bunny_mesh, &camera, &projection, &shading);

    c.bench_function("renderer_draw_bunny_80x24", |b| {
        b.iter(|| {
            renderer.draw(&mut grid, &bunny_mesh, &camera, &projection, &shading);
        });
    });
}

criterion_group!(benches, bench_draw, bench_draw_mesh_bunny);
criterion_main!(benches);
