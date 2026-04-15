//! Placeholder bench for `Renderer::draw()` zero-allocation gate.
//! Implementation lands in Phase 1.3 Plan 02 when the rasterizer is wired.

use criterion::{criterion_group, criterion_main, Criterion};

fn placeholder(_c: &mut Criterion) {
    // Bench body added when Renderer::draw() exists.
}

criterion_group!(benches, placeholder);
criterion_main!(benches);
