//! Criterion benchmarks for pipeline overhead measurement (D-11, PIPE-05).
//!
//! Proves O(cells) pipeline overhead: 10 effects on 200x60 grid should scale
//! linearly with effect count, not quadratically with objects x effects.

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use happyterminals_core::Grid;
use happyterminals_pipeline::{effects, Pipeline};
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style};
use tachyonfx::fx::EvolveSymbolSet;
use tachyonfx::Motion;

fn make_grid() -> Grid {
    let mut grid = Grid::new(Rect::new(0, 0, 200, 60));
    for y in 0..60 {
        grid.put_str(0, y, &"X".repeat(200), Style::default());
    }
    grid
}

fn make_10fx_pipeline() -> Pipeline {
    let dur = Duration::from_millis(100);
    Pipeline::new()
        .with(effects::dissolve(dur))
        .with(effects::fade_from(Color::Black, Color::Black, dur))
        .with(effects::fade_to(Color::White, Color::White, dur))
        .with(effects::sweep_in(Motion::LeftToRight, 5, Color::Black, dur))
        .with(effects::slide_in(Motion::UpToDown, 5, Color::Black, dur))
        .with(effects::coalesce(dur))
        .with(effects::hsl_shift([30.0, 10.0, 10.0], dur))
        .with(effects::evolve(EvolveSymbolSet::Circles, dur))
        .with(effects::darken(0.5, dur))
        .with(effects::paint(Color::Red, Color::Blue, dur))
}

fn pipeline_benchmarks(c: &mut Criterion) {
    let dt = Duration::from_millis(16);

    c.bench_function("pipeline_10fx_200x60_single_frame", |b| {
        b.iter_batched(
            || (make_10fx_pipeline(), make_grid()),
            |(mut pipeline, mut grid)| {
                std::hint::black_box(pipeline.run_frame(&mut grid, dt));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    c.bench_function("pipeline_10fx_200x60_60_frames", |b| {
        b.iter_batched(
            || (make_10fx_pipeline(), make_grid()),
            |(mut pipeline, mut grid)| {
                for _ in 0..60 {
                    std::hint::black_box(pipeline.run_frame(&mut grid, dt));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    c.bench_function("pipeline_1fx_200x60_single_frame", |b| {
        b.iter_batched(
            || {
                let pipeline =
                    Pipeline::new().with(effects::dissolve(Duration::from_millis(100)));
                (pipeline, make_grid())
            },
            |(mut pipeline, mut grid)| {
                std::hint::black_box(pipeline.run_frame(&mut grid, dt));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, pipeline_benchmarks);
criterion_main!(benches);
