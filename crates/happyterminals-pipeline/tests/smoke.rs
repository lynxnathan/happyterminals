//! Smoke tests: exercise all 10 wired tachyonfx effects end-to-end.

use std::time::Duration;

use happyterminals_core::Grid;
use happyterminals_pipeline::effects;
use happyterminals_pipeline::{Effect, EffectState, Pipeline};
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

const DT: Duration = Duration::from_millis(16);

/// Run a pipeline for up to `max_frames`, returning the final state.
fn run_pipeline(pipeline: &mut Pipeline, grid: &mut Grid, max_frames: usize) -> EffectState {
    let mut state = EffectState::Running;
    for _ in 0..max_frames {
        state = pipeline.run_frame(grid, DT);
        if state == EffectState::Done {
            break;
        }
    }
    state
}

#[test]
fn test_dissolve() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::dissolve(Duration::from_millis(500)));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_fade_from() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::fade_from(
        Color::Black,
        Color::Black,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_fade_to() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::fade_to(
        Color::White,
        Color::White,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_sweep_in() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::sweep_in(
        Motion::LeftToRight,
        5,
        Color::Black,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_slide_in() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::slide_in(
        Motion::UpToDown,
        5,
        Color::Black,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_coalesce() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::coalesce(Duration::from_millis(500)));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_hsl_shift() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::hsl_shift(
        [30.0, 10.0, 10.0],
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_evolve() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::evolve(
        EvolveSymbolSet::Circles,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_darken() {
    let mut grid = make_grid();
    let mut pipeline =
        Pipeline::new().with(effects::darken(0.5, Duration::from_millis(500)));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_paint() {
    let mut grid = make_grid();
    let mut pipeline = Pipeline::new().with(effects::paint(
        Color::Red,
        Color::Blue,
        Duration::from_millis(500),
    ));
    let state = run_pipeline(&mut pipeline, &mut grid, 60);
    assert_eq!(state, EffectState::Done);
}

#[test]
fn test_pipeline_10_effects_combined() {
    let mut grid = make_grid();
    let dur = Duration::from_millis(500);

    let mut pipeline = Pipeline::new()
        .with(effects::dissolve(dur))
        .with(effects::fade_from(Color::Black, Color::Black, dur))
        .with(effects::fade_to(Color::White, Color::White, dur))
        .with(effects::sweep_in(Motion::LeftToRight, 5, Color::Black, dur))
        .with(effects::slide_in(Motion::UpToDown, 5, Color::Black, dur))
        .with(effects::coalesce(dur))
        .with(effects::hsl_shift([30.0, 10.0, 10.0], dur))
        .with(effects::evolve(EvolveSymbolSet::Circles, dur))
        .with(effects::darken(0.5, dur))
        .with(effects::paint(Color::Red, Color::Blue, dur));

    let state = run_pipeline(&mut pipeline, &mut grid, 120);
    assert_eq!(state, EffectState::Done);
}
