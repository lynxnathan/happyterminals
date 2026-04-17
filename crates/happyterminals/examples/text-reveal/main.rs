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

const BUNNY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/bunny.obj"
);
const AUTO_ROTATION_SPEED: f32 = 0.22; // rad/s — slow + cinematic, not a kitchen-timer spin
const INITIAL_CAMERA_DISTANCE: f32 = 2.8; // close enough that the bunny owns the frame
const INITIAL_CAMERA_ELEVATION: f32 = 0.4; // slightly above horizon — shows the bunny's back
const ELEVATION_SWAY_AMPLITUDE: f32 = 0.1; // radians — subtle nodding, not a coaster
const ELEVATION_SWAY_FREQ: f32 = 0.3; // rad/s — slow breath
const FRAME_DT: f32 = 0.033; // ~30fps cadence assumed by the tick closure
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

/// Title rect: centered horizontally, 4 rows tall, anchored tight to the top
/// of the grid so the bunny owns the central/vertical frame real estate.
/// The title reads as a title card overlay, not as a co-star.
fn title_rect(grid_area: Rect) -> Rect {
    let width = 44u16.min(grid_area.width.saturating_sub(2));
    let x = grid_area.x + grid_area.width.saturating_sub(width) / 2;
    let y = grid_area.y + 1;
    Rect::new(x, y, width, 4)
}

/// Compute the cinematic elevation sway at a given elapsed time. Oscillates
/// around a base elevation so the bunny has a sense of breath rather than
/// spinning at a fixed camera height.
#[inline]
#[must_use]
pub fn elevation_at(base_elev: f32, elapsed_secs: f32) -> f32 {
    base_elev + ELEVATION_SWAY_AMPLITUDE * (elapsed_secs * ELEVATION_SWAY_FREQ).sin()
}

/// Project a world-space axis direction into the gizmo's screen-space offset
/// from the origin cell. Uses the camera's view rotation only (translation
/// ignored) so the gizmo shows pure orientation, screen-pinned to a corner.
/// Returns (dx, dy) in cell units relative to the gizmo origin.
#[inline]
#[must_use]
pub fn project_axis_to_screen(camera: &OrbitCamera, axis: glam::Vec3, radius: f32) -> (f32, f32) {
    let view = camera.view_matrix();
    let cam = view.transform_vector3(axis);
    // cam.x = screen right, cam.y = world up (flip for screen coords which grow down),
    // cam.z = depth (ignored — gizmo is a 2D projection of orientation).
    (cam.x * radius, -cam.y * radius)
}

/// Draw a small axis gizmo at `corner` (cell position). Shows X (red), Y (green),
/// Z (blue) axes projected from the camera's current orientation.
/// Debug aid for orbit + "3D world on a teletyper" in action.
fn draw_axis_gizmo(grid: &mut Grid, camera: &OrbitCamera, corner: (u16, u16), radius: u16) {
    let (ox, oy) = corner;
    let grid_right = grid.area.x + grid.area.width;
    let grid_bottom = grid.area.y + grid.area.height;
    let r = f32::from(radius);

    // Draw origin marker — a dim '+' to anchor the eye.
    grid.put_str(ox, oy, "+", Style::default().fg(Color::DarkGray));

    let axes = [
        (glam::Vec3::X, "X", Color::Red),
        (glam::Vec3::Y, "Y", Color::Green),
        (glam::Vec3::Z, "Z", Color::Blue),
    ];

    for (axis, label, color) in axes {
        let (dx, dy) = project_axis_to_screen(camera, axis, r);
        // Tip position in cell coords — round to nearest cell.
        let col_f = (f32::from(ox) + dx).round();
        let row_f = (f32::from(oy) + dy).round();
        if col_f < 0.0 || row_f < 0.0 {
            continue;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let col = col_f as u16;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let row = row_f as u16;
        if col >= grid_right || row >= grid_bottom {
            continue;
        }
        grid.put_str(
            col,
            row,
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );
    }
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

    // Camera: slow orbit, slightly above horizon, close enough that the bunny
    // dominates the frame. Elevation here is the *commanded* base — a subtle
    // sinusoidal sway is added at render time so the motion reads as "alive"
    // rather than "spinning prop".
    let mut camera = OrbitCamera {
        azimuth: 0.0,
        elevation: INITIAL_CAMERA_ELEVATION,
        distance: INITIAL_CAMERA_DISTANCE,
        ..OrbitCamera::default()
    };
    let mut commanded_elevation = INITIAL_CAMERA_ELEVATION;
    let mut elapsed_secs: f32 = 0.0;

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
        Binding::Key(KeyCode::Char(' ')),
        vec![],
    );
    ctx.bind(
        "swap_effect",
        Binding::Key(KeyCode::Tab),
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
            //    camera.elevation here is the *commanded* value (changed only by user
            //    orbit input). The render-time elevation adds a subtle sinusoidal sway
            //    so the bunny feels alive rather than spinning at a locked height.
            elapsed_secs += FRAME_DT;
            let auto_rotate_this_frame = AUTO_ROTATION_SPEED * FRAME_DT;
            let (_zoom, orbit_delta) = apply_camera_inputs(
                &mut camera,
                imap,
                grid.area.width,
                grid.area.height,
                auto_rotate_this_frame,
            );
            if orbit_delta.y != 0.0 {
                // User drag-orbited vertically — re-anchor the sway around the new base.
                commanded_elevation = camera.elevation;
            }
            let sway_elevation = elevation_at(commanded_elevation, elapsed_secs);

            // 3. Draw bunny into the grid — use swayed elevation, restore commanded after.
            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                fov_y: FOV,
                ..Projection::default()
            };
            let render_elevation_was = camera.elevation;
            camera.elevation = sway_elevation;
            renderer.draw(grid, &bunny, &camera, &projection, &shading);

            // 3b. Axis gizmo (debug + hero — "3D orientation indicator in a terminal").
            //     Positioned in the bottom-right corner; reflects the CURRENT render
            //     elevation so it visually tracks orbit/sway in real time.
            let gizmo_radius = 3u16;
            let gizmo_x = grid.area.x + grid.area.width.saturating_sub(gizmo_radius + 2);
            let gizmo_y = grid.area.y + grid.area.height.saturating_sub(gizmo_radius + 2);
            draw_axis_gizmo(grid, &camera, (gizmo_x, gizmo_y), gizmo_radius);

            camera.elevation = render_elevation_was;

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
    use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState, MouseEvent, MouseEventKind};
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
    fn title_rect_anchors_tight_to_top_for_bunny_hero_frame() {
        // The title must sit as close to the top edge as possible (row 1)
        // so the bunny owns the central/vertical frame. If this drifts
        // back to y+4 the composition loses its hero quality.
        let grid_area = Rect::new(0, 0, 80, 24);
        let rect = title_rect(grid_area);
        assert_eq!(rect.y, grid_area.y + 1, "title must anchor at grid.y + 1 (tight to top)");
        assert_eq!(rect.height, 4, "title height should stay at 4 rows");
    }

    #[test]
    fn initial_camera_is_close_enough_for_hero_composition() {
        // If the bunny doesn't fill the frame, it reads as a prop instead of
        // the hero. This test pins the close-camera choice.
        assert!(
            INITIAL_CAMERA_DISTANCE <= 3.0,
            "initial camera distance must be <= 3.0 for hero-scale bunny; got {}",
            INITIAL_CAMERA_DISTANCE
        );
        assert!(
            INITIAL_CAMERA_DISTANCE >= MIN_CAMERA_DISTANCE,
            "initial distance must be >= MIN_CAMERA_DISTANCE"
        );
    }

    #[test]
    fn auto_rotation_is_cinematic_not_spinning() {
        // A "spinning prop" feel comes from azimuth rotation > ~0.4 rad/s.
        // Cap the auto-rotation at the cinematic range.
        assert!(
            AUTO_ROTATION_SPEED <= 0.3,
            "AUTO_ROTATION_SPEED must be <= 0.3 rad/s for cinematic feel; got {}",
            AUTO_ROTATION_SPEED
        );
    }

    #[test]
    fn elevation_sway_oscillates_around_base() {
        // At elapsed=0 the sway contribution is zero — we start at the base.
        let base = 0.4_f32;
        assert!((elevation_at(base, 0.0) - base).abs() < 1e-6);

        // After a quarter-period the sway reaches its positive peak.
        let quarter_period = (std::f32::consts::FRAC_PI_2) / ELEVATION_SWAY_FREQ;
        let peak = elevation_at(base, quarter_period);
        assert!(
            (peak - (base + ELEVATION_SWAY_AMPLITUDE)).abs() < 1e-4,
            "at quarter-period elevation should be base + amplitude; got {}",
            peak
        );

        // Sway amplitude must be subtle (≤ 0.15 rad ≈ 8.6°) — anything more
        // feels like a rollercoaster, not a breath.
        assert!(
            ELEVATION_SWAY_AMPLITUDE <= 0.15,
            "elevation sway amplitude must stay subtle (<= 0.15 rad); got {}",
            ELEVATION_SWAY_AMPLITUDE
        );
    }

    #[test]
    fn axis_gizmo_projects_x_to_positive_x_screen_when_camera_faces_neg_z() {
        // OrbitCamera default: azimuth=0, elevation=0 → looking down -Z axis.
        // At this pose, world +X should project to camera's +X (screen right),
        // so dx > 0 and dy ≈ 0.
        let cam = OrbitCamera::default();
        let (dx, dy) = project_axis_to_screen(&cam, glam::Vec3::X, 3.0);
        assert!(dx > 2.0, "world +X should project well to the right; dx={}", dx);
        assert!(dy.abs() < 0.5, "world +X should have near-zero screen-y; dy={}", dy);
    }

    #[test]
    fn axis_gizmo_projects_y_to_negative_y_screen_when_camera_is_horizontal() {
        // World +Y is up; screen y grows DOWN, so world +Y should project to
        // negative dy (i.e., above the gizmo origin).
        let cam = OrbitCamera::default(); // azimuth=0, elevation=0
        let (dx, dy) = project_axis_to_screen(&cam, glam::Vec3::Y, 3.0);
        assert!(dy < -2.0, "world +Y should project upward (negative screen-y); dy={}", dy);
        assert!(dx.abs() < 0.5, "world +Y should have near-zero screen-x; dx={}", dx);
    }

    #[test]
    fn axis_gizmo_projection_responds_to_azimuth_rotation() {
        // Rotating camera azimuth by PI/2 should swap screen-X/Z axis projections.
        let cam_zero = OrbitCamera {
            azimuth: 0.0,
            elevation: 0.0,
            ..OrbitCamera::default()
        };
        let cam_quarter = OrbitCamera {
            azimuth: std::f32::consts::FRAC_PI_2,
            elevation: 0.0,
            ..OrbitCamera::default()
        };
        let (dx0, _) = project_axis_to_screen(&cam_zero, glam::Vec3::X, 3.0);
        let (dx1, _) = project_axis_to_screen(&cam_quarter, glam::Vec3::X, 3.0);
        assert!(
            (dx0 - dx1).abs() > 1.0,
            "gizmo must change with azimuth; dx went from {} to {}",
            dx0,
            dx1
        );
    }

    #[test]
    fn axis_gizmo_draw_stays_within_grid_bounds() {
        // Even at an arbitrary camera pose, the gizmo writes must not panic
        // or reach out-of-bounds cells. Compile-time check by exercising a
        // drawing call — Grid::put_str silently clips but we also check our
        // own bounds-guarding.
        use happyterminals_core::Grid;
        let grid_area = Rect::new(0, 0, 20, 10);
        let mut grid = Grid::new(grid_area);
        let cam = OrbitCamera {
            azimuth: 0.7,
            elevation: 0.5,
            distance: 3.0,
            ..OrbitCamera::default()
        };
        // Origin near the corner — tips could push past edges without our guard.
        draw_axis_gizmo(&mut grid, &cam, (18, 8), 3);
        // No panic means bounds-guarding works. Additionally: the '+' origin marker
        // must be at (18,8).
        use ratatui_core::layout::Position;
        let origin_cell = grid.cell(Position::new(18, 8)).expect("origin in bounds");
        assert_eq!(origin_cell.symbol(), "+");
    }

    #[test]
    fn elevation_sway_is_continuous_not_stepped() {
        // Frame-to-frame elevation change from sway must be small — no jumps.
        let base = 0.4_f32;
        let mut prev = elevation_at(base, 0.0);
        for i in 1..300 {
            let t = (i as f32) * FRAME_DT;
            let cur = elevation_at(base, t);
            let jump = (cur - prev).abs();
            assert!(
                jump < 0.05,
                "frame-to-frame elevation jump must be < 0.05; got {} at t={}",
                jump,
                t
            );
            prev = cur;
        }
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
