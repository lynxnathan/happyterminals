//! Text reveal over a live 3D scene — happyterminals hero example.
//!
//! A rotating bunny mesh renders continuously; title, tagline, and a
//! "press [space] to replay" cue reveal over it using tachyonfx text
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
//! - Left-drag: orbit the bunny (pauses auto-rotate that frame)
//! - Right-drag: pan the camera target
//! - Scroll: zoom (adjust camera distance)
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
const AUTO_ROTATION_SPEED: f32 = 0.6; // radians per second — slow, cinematic
const REVEAL_DURATION_MS: u64 = 1800; // long enough to see the animation land
const ZOOM_SENSITIVITY: f32 = 0.5;
const MIN_CAMERA_DISTANCE: f32 = 1.5;
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

/// Apply orbit/pan/zoom deltas from the `InputMap` to the camera, then auto-rotate
/// only when the user isn't actively orbiting this frame. Returns the zoom-delta
/// applied (test hook — lets integration tests observe state flow without a terminal).
fn apply_camera_inputs(
    camera: &mut OrbitCamera,
    imap: &InputMap,
    viewport_w: u16,
    viewport_h: u16,
    auto_rotate_delta: f32,
) -> (f32, glam::Vec2) {
    let vw = f32::from(viewport_w.max(1));
    let vh = f32::from(viewport_h.max(1));

    let world_h = 2.0 * camera.distance * (FOV / 2.0).tan();
    let world_w = world_h * (vw / vh);
    let orbit_per_cell = std::f32::consts::PI / vw;
    let pan_per_cell_x = world_w / vw;
    let pan_per_cell_y = world_h / vh;

    let mut orbit_delta = glam::Vec2::ZERO;
    if let Some(orbit_sig) = imap.action_axis2d("orbit") {
        orbit_delta = orbit_sig.untracked();
        camera.azimuth -= orbit_delta.x * orbit_per_cell;
        camera.elevation += orbit_delta.y * orbit_per_cell;
    }

    let mut zoom_delta = 0.0;
    if let Some(zoom_sig) = imap.action_axis1d("zoom") {
        zoom_delta = zoom_sig.untracked();
        camera.distance =
            (camera.distance - zoom_delta * ZOOM_SENSITIVITY).max(MIN_CAMERA_DISTANCE);
    }

    if let Some(pan_sig) = imap.action_axis2d("pan") {
        let delta = pan_sig.untracked();
        let right = glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
        let up = glam::Vec3::Y;
        camera.target -=
            right * delta.x * pan_per_cell_x + up * (-delta.y) * pan_per_cell_y;
    }

    // Auto-rotate only when the user isn't actively orbiting — preserves the
    // "cinematic slow spin" when idle, yields to user drag when engaged.
    if orbit_delta == glam::Vec2::ZERO {
        camera.azimuth += auto_rotate_delta;
    }

    (zoom_delta, orbit_delta)
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

    // Input: default viewer controls (orbit/pan/zoom/quit) + two new actions for Space and Tab.
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

            // 2. Camera: apply orbit/pan/zoom from input, then auto-rotate when idle.
            let auto_rotate_this_frame = AUTO_ROTATION_SPEED * 0.033;
            apply_camera_inputs(
                &mut camera,
                imap,
                grid.area.width,
                grid.area.height,
                auto_rotate_this_frame,
            );

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
            grid.put_str(tx, ty + 3, ">  press [space] to replay", cue_style);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent, MouseEventKind};
    use happyterminals_core::create_root;

    /// Build the input map exactly like `main()` does — the structure under test.
    fn build_input_map() -> InputMap {
        let mut input_map = InputMap::new();
        register_default_actions(&mut input_map);
        input_map.register_action("replay_reveal", ActionValueType::Bool);
        input_map.register_action("swap_effect", ActionValueType::Bool);
        let mut ctx = default_viewer_context();
        ctx.bind(
            "replay_reveal",
            Binding::Key(KeyCode::Char(' ')),
            vec![],
        );
        ctx.bind(
            "swap_effect",
            Binding::Key(KeyCode::Tab),
            vec![],
        );
        input_map.push_context(ctx);
        input_map
    }

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn scroll_up_event() -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn scroll_down_event() -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn space_fires_replay_reveal_action() {
        let (_r, _owner) = create_root(|| {
            let mut imap = build_input_map();
            imap.dispatch(&key_event(KeyCode::Char(' ')));
            let state = imap
                .action_state("replay_reveal")
                .expect("replay_reveal registered");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }

    #[test]
    fn tab_fires_swap_effect_action() {
        let (_r, _owner) = create_root(|| {
            let mut imap = build_input_map();
            imap.dispatch(&key_event(KeyCode::Tab));
            let state = imap
                .action_state("swap_effect")
                .expect("swap_effect registered");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }

    #[test]
    fn scroll_up_zooms_in_reducing_camera_distance() {
        let (_r, _owner) = create_root(|| {
            let mut imap = build_input_map();
            imap.dispatch(&scroll_up_event());

            let mut cam = OrbitCamera {
                distance: 4.0,
                ..OrbitCamera::default()
            };
            let (zoom_delta, _orbit) = apply_camera_inputs(&mut cam, &imap, 80, 24, 0.0);
            assert!(zoom_delta.abs() > 0.0, "scroll-up must produce non-zero zoom delta");
            assert!(
                cam.distance < 4.0,
                "scroll-up should reduce camera.distance; got {}",
                cam.distance
            );
        });
    }

    #[test]
    fn scroll_down_zooms_out_increasing_camera_distance() {
        let (_r, _owner) = create_root(|| {
            let mut imap = build_input_map();
            imap.dispatch(&scroll_down_event());

            let mut cam = OrbitCamera {
                distance: 4.0,
                ..OrbitCamera::default()
            };
            let (zoom_delta, _orbit) = apply_camera_inputs(&mut cam, &imap, 80, 24, 0.0);
            assert!(zoom_delta.abs() > 0.0, "scroll-down must produce non-zero zoom delta");
            assert!(
                cam.distance > 4.0,
                "scroll-down should increase camera.distance; got {}",
                cam.distance
            );
        });
    }

    #[test]
    fn zoom_clamps_at_min_distance() {
        let (_r, _owner) = create_root(|| {
            let mut imap = build_input_map();
            // Many scroll-ups at min distance should not go below MIN_CAMERA_DISTANCE.
            for _ in 0..50 {
                imap.dispatch(&scroll_up_event());
            }
            let mut cam = OrbitCamera {
                distance: MIN_CAMERA_DISTANCE + 0.1,
                ..OrbitCamera::default()
            };
            let _ = apply_camera_inputs(&mut cam, &imap, 80, 24, 0.0);
            assert!(
                cam.distance >= MIN_CAMERA_DISTANCE,
                "camera.distance must clamp at {}, got {}",
                MIN_CAMERA_DISTANCE,
                cam.distance
            );
        });
    }

    #[test]
    fn idle_frame_auto_rotates_azimuth() {
        let (_r, _owner) = create_root(|| {
            let imap = build_input_map(); // no dispatch — no orbit input
            let mut cam = OrbitCamera {
                azimuth: 0.0,
                ..OrbitCamera::default()
            };
            let auto_delta = 0.02;
            let _ = apply_camera_inputs(&mut cam, &imap, 80, 24, auto_delta);
            assert!(
                (cam.azimuth - auto_delta).abs() < 1e-6,
                "idle frame should auto-rotate by exactly {} rad; got {}",
                auto_delta,
                cam.azimuth
            );
        });
    }

    #[test]
    fn replay_reveal_action_checked_before_firing() {
        // Before any input, replay_reveal should NOT be JustPressed.
        let (_r, _owner) = create_root(|| {
            let imap = build_input_map();
            let state = imap
                .action_state("replay_reveal")
                .expect("replay_reveal registered");
            assert_ne!(
                state.untracked(),
                ActionState::JustPressed,
                "replay_reveal must be idle before any key dispatch"
            );
        });
    }

    #[test]
    fn pipeline_reset_restarts_effect_animation() {
        // After the effect has run to completion on a fully-drawn text cell, pipeline.reset()
        // followed by one more frame must produce a visibly different cell state than the
        // fully-completed state (because the reveal is mid-animation again).
        use happyterminals_core::Grid;
        use ratatui_core::layout::Position;

        let (_r, _owner) = create_root(|| {
            let rect = Rect::new(0, 0, 20, 4);
            let mut pipeline = make_pipeline(0, rect); // fade_reveal
            let mut grid = Grid::new(rect);
            let text_style = Style::default().fg(Color::White);

            // Run enough frames to complete the reveal.
            let frames_to_complete = (REVEAL_DURATION_MS / 33) as usize + 5;
            for _ in 0..frames_to_complete {
                grid.put_str(0, 0, "HELLO WORLD", text_style);
                pipeline.run_frame(&mut grid, Duration::from_millis(33));
            }

            // Capture the completed cell at (0,0) — fg should be white (reveal done).
            let completed_fg = grid
                .cell(Position::new(0, 0))
                .expect("cell exists")
                .fg;

            // Reset + re-write + one frame: the cell should be mid-animation again,
            // which for fade_reveal means a different fg than the completed state.
            pipeline.reset();
            grid.put_str(0, 0, "HELLO WORLD", text_style);
            pipeline.run_frame(&mut grid, Duration::from_millis(33));

            let mid_animation_fg = grid
                .cell(Position::new(0, 0))
                .expect("cell exists")
                .fg;

            assert_ne!(
                completed_fg, mid_animation_fg,
                "pipeline.reset() must restart the effect — post-reset frame should not match completed-state cell"
            );
        });
    }
}
