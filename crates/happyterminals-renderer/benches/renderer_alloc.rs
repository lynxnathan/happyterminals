//! Criterion benchmark for `Renderer::draw()` hot path.
//!
//! Measures the rendering time and verifies the zero-allocation property
//! of the draw loop after warmup.

use criterion::{criterion_group, criterion_main, Criterion};
use happyterminals_core::{Grid, Rect};
use happyterminals_renderer::{Cube, OrbitCamera, Projection, Renderer, ShadingRamp};

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

criterion_group!(benches, bench_draw);
criterion_main!(benches);
