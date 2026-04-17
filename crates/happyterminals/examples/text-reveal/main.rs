//! Text reveal over a live 3D scene — happyterminals hero example.
//!
//! A rotating bunny mesh renders continuously; title, tagline, and a
//! "press [space] to continue" cue reveal over it using tachyonfx text
//! effects bounded to the title region. The unexpected combination —
//! GPU-quality effects driving pure-text composition on top of an
//! ASCII-rendered 3D model — is the paradigm this library exists to show.
//!
//! Features exercised:
//! - OBJ mesh loading (`bunny.obj`) + `OrbitCamera` + `Renderer::draw`
//! - tachyonfx text effects via `TachyonAdapter::with_area` (`fade_from`, `sweep_in`, `coalesce`)
//! - `Pipeline::run_frame` + `Pipeline::reset` for interactive replay
//! - `InputMap` action dispatch for Space / Tab bindings
//! - `Grid::put_str` layered between mesh render and pipeline apply
//!
//! Controls:
//! - Space: replay the text reveal from the start
//! - Tab: swap to the next reveal effect (`fade_from` -> `sweep_in` -> `coalesce`)
//! - Ctrl-C or Q: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example text-reveal -p happyterminals
//!
//! Why this exists:
//! Pre-repo Discord testing validated two distinct "unexpected" reactions —
//! the bunny (a 3D mesh in a terminal) and physics-y particles. text-reveal
//! composes the third unexpected thing on top of the first: animated text
//! effects rendered as part of the same scene as the 3D mesh, blended into
//! the same Grid cells.

use std::time::Duration;

use happyterminals::prelude::*;
use happyterminals_pipeline::TachyonAdapter;
use happyterminals_renderer::Renderer;

const BUNNY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/bunny.obj"
);
const ROTATION_SPEED: f32 = 0.6; // radians per second — slow, cinematic
const REVEAL_DURATION_MS: u64 = 900;
const FOV: f32 = std::f32::consts::FRAC_PI_4;

type RevealFn = fn(Rect) -> TachyonAdapter;

fn fade_reveal(rect: Rect) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(Duration::from_millis(REVEAL_DURATION_MS));
    TachyonAdapter::with_area(
        tachyonfx::fx::fade_from(Color::Black, Color::Reset, tfx_dur),
        rect,
    )
}

fn sweep_reveal(rect: Rect) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(Duration::from_millis(REVEAL_DURATION_MS));
    TachyonAdapter::with_area(
        tachyonfx::fx::sweep_in(
            tachyonfx::Motion::LeftToRight,
            8, // gradient length
            0, // randomness
            Color::DarkGray,
            tfx_dur,
        ),
        rect,
    )
}

fn coalesce_reveal(rect: Rect) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(Duration::from_millis(REVEAL_DURATION_MS));
    TachyonAdapter::with_area(tachyonfx::fx::coalesce(tfx_dur), rect)
}

const REVEAL_EFFECTS: &[RevealFn] = &[fade_reveal, sweep_reveal, coalesce_reveal];

/// Title rect: centered horizontally, 4 rows tall, near the top so the bunny
/// has room to render below.
fn title_rect(grid_area: Rect) -> Rect {
    let width = 44u16.min(grid_area.width.saturating_sub(2));
    let x = grid_area.x + grid_area.width.saturating_sub(width) / 2;
    let y = grid_area.y + 4;
    Rect::new(x, y, width, 4)
}

fn make_pipeline(effect_idx: usize, rect: Rect) -> Pipeline {
    Pipeline::new().with(REVEAL_EFFECTS[effect_idx](rect))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load bunny once, up front (propagates cleanly if asset missing).
    let (bunny, _stats) = load_obj(BUNNY_PATH)?;

    // Camera: slow orbit, elevation slightly above horizon.
    let mut camera = OrbitCamera {
        azimuth: 0.0,
        elevation: 0.3,
        distance: 4.0,
        ..OrbitCamera::default()
    };

    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();

    // Input: default viewer controls + two new actions for Space and Tab.
    let mut input_map = InputMap::new();
    register_default_actions(&mut input_map);
    input_map.register_action("replay_reveal", ActionValueType::Bool);
    input_map.register_action("swap_effect", ActionValueType::Bool);
    let mut ctx = default_viewer_context();
    ctx.bind(
        "replay_reveal",
        Binding::Key(crossterm::event::KeyCode::Char(' ')),
        vec![],
    );
    ctx.bind(
        "swap_effect",
        Binding::Key(crossterm::event::KeyCode::Tab),
        vec![],
    );
    input_map.push_context(ctx);

    // Mutable state kept across frames.
    let mut effect_idx: usize = 0;
    let mut current_title_rect = Rect::new(0, 0, 0, 0);
    let mut pipeline = Pipeline::new(); // rebuilt on first frame once we know grid size

    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let tagline_style = Style::default().fg(Color::Cyan);
    let cue_style = Style::default().fg(Color::Yellow);

    run_with_input(
        move |grid, _input_signals, imap| {
            // 1. Lazy-init / resize-aware title rect + pipeline.
            let new_rect = title_rect(grid.area);
            if new_rect != current_title_rect {
                current_title_rect = new_rect;
                pipeline = make_pipeline(effect_idx, current_title_rect);
            }

            // 2. Advance bunny rotation (time-based — ~33ms/frame assumed).
            camera.azimuth += ROTATION_SPEED * 0.033;

            // 3. Draw bunny into the grid.
            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                fov_y: FOV,
                ..Projection::default()
            };
            renderer.draw(grid, &bunny, &camera, &projection, &shading);

            // 4. Write title text INSIDE the bounded rect, EVERY frame.
            //    Cells are overwritten -> effect sees fresh target each tick.
            let tx = current_title_rect.x;
            let ty = current_title_rect.y;
            grid.put_str(tx, ty, "HAPPY TERMINALS", title_style);
            grid.put_str(tx, ty + 1, "GPU-quality effects on text", tagline_style);
            grid.put_str(tx, ty + 3, ">  press [space] to continue", cue_style);

            // 5. Apply the bounded reveal pipeline to animate the text cells.
            pipeline.run_frame(grid, Duration::from_millis(33));

            // 6. Handle replay (Space) — reset pipeline so effect plays again.
            if let Some(sig) = imap.action_state("replay_reveal") {
                if sig.untracked() == ActionState::JustPressed {
                    pipeline.reset();
                }
            }

            // 7. Handle effect swap (Tab) — cycle and rebuild.
            if let Some(sig) = imap.action_state("swap_effect") {
                if sig.untracked() == ActionState::JustPressed {
                    effect_idx = (effect_idx + 1) % REVEAL_EFFECTS.len();
                    pipeline = make_pipeline(effect_idx, current_title_rect);
                }
            }
        },
        FrameSpec {
            title: Some("happyterminals - Text Reveal".into()),
            ..FrameSpec::default()
        },
        input_map,
    )
    .await
}
