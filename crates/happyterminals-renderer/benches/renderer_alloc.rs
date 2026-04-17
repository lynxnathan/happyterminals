//! Criterion benchmark for `Renderer::draw()` and `Renderer::draw_particles()` hot paths.
//!
//! Measures the rendering time and verifies the zero-allocation property
//! of the draw loop after warmup. Covers three scales:
//! - 12-triangle cube: baseline / low-count cost model.
//! - ~5000-triangle Stanford bunny: real-world mesh at REND-09 scale.
//! - 500-particle emitter: pool-based particles with z-buffer composition.

// Benches are harness binaries; panicking at bench setup (before the hot loop)
// is acceptable — an unloadable fixture is a configuration failure, not a
// runtime error to recover from.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use criterion::{criterion_group, criterion_main, Criterion};
use happyterminals_core::{Grid, Rect};
use happyterminals_renderer::{
    Cube, OrbitCamera, ParticleEmitter, Projection, Renderer, ShadingRamp, load_obj,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use ratatui_core::style::Color;

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

/// Bench rendering 500 pool-based particles with z-buffer composition.
///
/// Uses a deterministic RNG for reproducible results. Validates that
/// renderer buffer capacities and emitter pool capacity remain stable
/// after warmup (REND-09 particle path).
fn bench_draw_particles(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);

    let mut emitter = ParticleEmitter::new(500);
    emitter.origin = glam::Vec3::new(0.0, 2.0, 0.0);
    emitter.spread = glam::Vec3::new(2.0, 0.5, 2.0);
    emitter.gravity = glam::Vec3::new(0.0, -2.0, 0.0);
    emitter.spawn_rate = 50.0;
    emitter.life_range = (3.0, 5.0);
    emitter.color_start = Color::White;
    emitter.color_end = Color::Rgb(180, 200, 255);

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
    let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
    let mut renderer = Renderer::new();
    let cube_mesh = Cube::mesh();

    // Warmup: run 10 frames to fill pool and stabilize renderer buffers.
    for _ in 0..10 {
        emitter.update(0.016, &mut rng);
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
    }

    // Record capacities after warmup
    let cap_z = renderer.z_buffer_capacity();
    let cap_chars = renderer.cell_chars_capacity();
    let cap_colors = renderer.cell_colors_capacity();
    let cap_pool = emitter.particles.capacity();

    c.bench_function("renderer_draw_particles_80x24", |b| {
        b.iter(|| {
            emitter.update(0.016, &mut rng);
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
            renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
        });
    });

    // Post-bench capacity assertions (REND-09)
    assert_eq!(
        renderer.z_buffer_capacity(),
        cap_z,
        "z_buffer capacity must not grow during particle bench"
    );
    assert_eq!(
        renderer.cell_chars_capacity(),
        cap_chars,
        "cell_chars capacity must not grow during particle bench"
    );
    assert_eq!(
        renderer.cell_colors_capacity(),
        cap_colors,
        "cell_colors capacity must not grow during particle bench"
    );
    assert_eq!(
        emitter.particles.capacity(),
        cap_pool,
        "particle pool capacity must not grow during bench"
    );
}

criterion_group!(benches, bench_draw, bench_draw_mesh_bunny, bench_draw_particles, bench_alloc_stability);
criterion_main!(benches);

/// Comprehensive 3-path allocation stability validation.
///
/// Validates that all renderer paths (cube, OBJ mesh, particles) maintain
/// stable buffer capacities after warmup across 100 frames each.
fn alloc_stability_all_paths() {
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

    // ── Path 1: Cube (12 triangles) ─────────────────────────────────
    {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        let cap_z = renderer.z_buffer_capacity();
        let cap_chars = renderer.cell_chars_capacity();

        for _ in 0..100 {
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        }

        assert_eq!(renderer.z_buffer_capacity(), cap_z, "Cube: z_buffer grew");
        assert_eq!(renderer.cell_chars_capacity(), cap_chars, "Cube: cell_chars grew");
    }

    // ── Path 2: Bunny mesh (~5000 triangles) ────────────────────────
    {
        let bunny_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/bunny.obj"
        );
        let (bunny_mesh, _stats) =
            load_obj(bunny_path).expect("bunny.obj must load for alloc test");

        let (center, radius) = bunny_mesh.bounding_sphere();
        let bunny_camera = OrbitCamera {
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            distance: radius * 2.5,
            target: center,
        };

        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();

        renderer.draw(&mut grid, &bunny_mesh, &bunny_camera, &projection, &shading);
        let cap_z = renderer.z_buffer_capacity();
        let cap_chars = renderer.cell_chars_capacity();

        for _ in 0..100 {
            renderer.draw(&mut grid, &bunny_mesh, &bunny_camera, &projection, &shading);
        }

        assert_eq!(renderer.z_buffer_capacity(), cap_z, "Bunny: z_buffer grew");
        assert_eq!(renderer.cell_chars_capacity(), cap_chars, "Bunny: cell_chars grew");
    }

    // ── Path 3: Particles (500-pool emitter) ────────────────────────
    {
        let mut rng = StdRng::seed_from_u64(42);
        let mut emitter = ParticleEmitter::new(500);
        emitter.origin = glam::Vec3::new(0.0, 2.0, 0.0);
        emitter.spread = glam::Vec3::new(2.0, 0.5, 2.0);
        emitter.gravity = glam::Vec3::new(0.0, -2.0, 0.0);
        emitter.spawn_rate = 50.0;
        emitter.life_range = (3.0, 5.0);
        emitter.color_start = Color::White;
        emitter.color_end = Color::Rgb(180, 200, 255);

        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let mut renderer = Renderer::new();
        let cube_mesh = Cube::mesh();

        emitter.update(0.016, &mut rng);
        renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
        renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);

        let cap_z = renderer.z_buffer_capacity();
        let cap_chars = renderer.cell_chars_capacity();
        let cap_colors = renderer.cell_colors_capacity();
        let cap_pool = emitter.particles.capacity();

        for _ in 0..100 {
            emitter.update(0.016, &mut rng);
            renderer.draw(&mut grid, &cube_mesh, &camera, &projection, &shading);
            renderer.draw_particles(&mut grid, &emitter, &camera, &projection, &shading);
        }

        assert_eq!(renderer.z_buffer_capacity(), cap_z, "Particles: z_buffer grew");
        assert_eq!(renderer.cell_chars_capacity(), cap_chars, "Particles: cell_chars grew");
        assert_eq!(renderer.cell_colors_capacity(), cap_colors, "Particles: cell_colors grew");
        assert_eq!(emitter.particles.capacity(), cap_pool, "Particles: pool grew");
    }
}

// ── Integration: 3-path validation as bench entry ───────────────────

/// Wrapper bench function that runs the 3-path stability check, exercised
/// by criterion's `--test` mode.
fn bench_alloc_stability(c: &mut Criterion) {
    alloc_stability_all_paths();
    // Dummy bench so criterion has something to report for this group entry.
    c.bench_function("alloc_stability_3path_check", |b| {
        b.iter(|| {
            // Intentionally empty — the actual validation ran above.
        });
    });
}
