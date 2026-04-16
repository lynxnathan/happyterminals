//! Snapshot matrix for Phase 2.2 color-mode downsampling.
//!
//! Fixtures: 2 scenes (unit cube + Stanford bunny) × 4 modes = 8 snapshots.
//!
//! We snapshot the POST-DOWNSAMPLE `Buffer` state (NOT raw ANSI bytes)
//! per RESEARCH §Pitfall 5 — `ratatui::Buffer`'s `Debug` impl is
//! deterministic given identical input, while ANSI-byte ordering
//! depends on diff-emission order and terminal state.
//!
//! ## Why we inject synthetic RGB colors
//!
//! The current renderer (Phase 1.3 + 2.1) writes shading-ramp *symbols*
//! into the grid but leaves cell colors at their default `Color::Reset`.
//! That means a pure-rendered scene produces identical buffer state
//! under all four color modes — the snapshots would not discriminate
//! modes, weakening their value as a regression gate.
//!
//! To keep the matrix meaningful while staying inside the plan's
//! "no renderer changes" constraint, we inject a deterministic
//! position-derived `Color::Rgb(...)` onto every non-space cell's
//! foreground *before* calling [`downsample`]. The injection is a pure
//! function of `(x, y)` — no wall-clock, no RNG — so determinism is
//! preserved (Pitfall 5), and each mode now produces visibly
//! different snapshots:
//!
//! - `TrueColor`   → RGB preserved verbatim.
//! - `Palette256`  → `Color::Indexed(n)` per xterm quantization.
//! - `Ansi16`      → named 16-color variants.
//! - `Mono`        → all fg/bg collapsed to `Color::Reset`.
//!
//! Snapshot tests call [`downsample`] directly with an explicit
//! [`ColorMode`] so CI env state (`NO_COLOR`, `COLORTERM`, `TERM`)
//! is irrelevant to determinism (Pitfall 4 closed by construction).

use happyterminals_backend_ratatui::color::{downsample, ColorMode};
use happyterminals_core::{Grid, Rect};
use happyterminals_renderer::{Cube, Mesh, OrbitCamera, Projection, Renderer, ShadingRamp};
use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::style::Color;
use std::ops::Deref;

/// Absolute path to the Stanford bunny OBJ, resolved at compile time so
/// the test runs from any cwd.
const BUNNY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/bunny.obj"
);

// Small fixed viewport keeps each snapshot tractable (~40×12 = 480 cells)
// and eliminates terminal-size variance between dev machines.
const W: u16 = 40;
const H: u16 = 12;

/// Deterministic position-derived RGB color. No wall-clock, no RNG.
///
/// The (x, y) mapping produces a horizontal red/green gradient with a
/// vertical blue ramp — coarse enough that the 256-cube and 16-named
/// quantizers produce distinct outputs at different cells.
#[allow(clippy::cast_possible_truncation)]
fn color_for(col_x: u16, row_y: u16) -> Color {
    // Linear-ish spread across the viewport. Each component is bounded
    // by construction to 0..=255, so the `as u8` casts cannot truncate.
    let max_x = u32::from(W.saturating_sub(1).max(1));
    let max_y = u32::from(H.saturating_sub(1).max(1));
    let red = ((u32::from(col_x) * 255) / max_x) as u8;
    let green = ((u32::from(W - 1 - col_x) * 255) / max_x) as u8;
    let blue = ((u32::from(row_y) * 255) / max_y) as u8;
    Color::Rgb(red, green, blue)
}

/// Builds a deterministic one-frame scene: rasterizes `mesh` into a
/// 40×12 `Grid`, clones the underlying `Buffer`, and paints every
/// non-space cell with a position-derived RGB foreground.
fn render_scene(mesh: &Mesh) -> Buffer {
    let mut grid = Grid::new(Rect::new(0, 0, W, H));
    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();
    let (center, radius) = mesh.bounding_sphere();
    let camera = OrbitCamera {
        azimuth: std::f32::consts::FRAC_PI_4,
        elevation: std::f32::consts::FRAC_PI_6,
        distance: radius * 2.5,
        target: center,
    };
    let projection = Projection {
        viewport_w: W,
        viewport_h: H,
        ..Projection::default()
    };
    renderer.draw(&mut grid, mesh, &camera, &projection, &shading);
    // Deref Grid → &Buffer → clone to an owned Buffer for mutation.
    let mut buf = grid.deref().clone();

    // Inject deterministic RGB fg on every non-space cell so the
    // snapshot matrix discriminates between modes (see module docs).
    for row_y in 0..H {
        for col_x in 0..W {
            if let Some(cell) = buf.cell_mut(Position::new(col_x, row_y)) {
                if cell.symbol() != " " {
                    cell.set_fg(color_for(col_x, row_y));
                }
            }
        }
    }
    buf
}

fn build_cube_scene() -> Buffer {
    render_scene(&Cube::mesh())
}

fn build_bunny_scene() -> Buffer {
    let (mesh, _stats) = happyterminals_renderer::load_obj(BUNNY_PATH)
        .unwrap_or_else(|e| panic!("bunny.obj must load for snapshot test: {e}"));
    render_scene(&mesh)
}

// --- cube × 4 modes --------------------------------------------------

#[test]
fn cube_truecolor() {
    let mut buf = build_cube_scene();
    downsample(&mut buf, ColorMode::TrueColor);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn cube_palette256() {
    let mut buf = build_cube_scene();
    downsample(&mut buf, ColorMode::Palette256);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn cube_ansi16() {
    let mut buf = build_cube_scene();
    downsample(&mut buf, ColorMode::Ansi16);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn cube_mono() {
    let mut buf = build_cube_scene();
    downsample(&mut buf, ColorMode::Mono);
    insta::assert_debug_snapshot!(buf);
}

// --- bunny × 4 modes -------------------------------------------------

#[test]
fn bunny_truecolor() {
    let mut buf = build_bunny_scene();
    downsample(&mut buf, ColorMode::TrueColor);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn bunny_palette256() {
    let mut buf = build_bunny_scene();
    downsample(&mut buf, ColorMode::Palette256);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn bunny_ansi16() {
    let mut buf = build_bunny_scene();
    downsample(&mut buf, ColorMode::Ansi16);
    insta::assert_debug_snapshot!(buf);
}

#[test]
fn bunny_mono() {
    let mut buf = build_bunny_scene();
    downsample(&mut buf, ColorMode::Mono);
    insta::assert_debug_snapshot!(buf);
}
