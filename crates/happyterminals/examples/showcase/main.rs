//! Showcase example — the public demo.
//!
//! Cycle between four 3D objects via an on-screen menu, money particles
//! rain continuously through the scene, and your typed message appears
//! as floating text below the model. All pure ASCII + ANSI.
//!
//! Features exercised:
//! - `load_obj` — three meshes (bunny / cow / teapot) plus built-in `Cube`
//! - `ParticleEmitter` with a custom single-char shading ramp (`'$'`) and gravity
//! - `InputMap` action system for menu navigation + effect cycling
//! - `InputSignals.last_key` for raw text input
//! - Reveal pipeline cycling (`fade_from` / `sweep_in` / `coalesce` / `evolve`)
//! - Heat-map colorization + axis gizmo
//!
//! Controls:
//! - ↑ / ↓: navigate menu · Enter: select (swaps the 3D object + replays reveal)
//! - Type letters: append to your message · Backspace: delete · Ctrl+U: clear
//! - Tab: cycle reveal effect · F5: replay current reveal
//! - Scroll: zoom (adaptive) · L-drag: orbit · R-drag: pan
//! - Ctrl-C or Q: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example showcase -p happyterminals
//!
//! Why this exists:
//! text-reveal proved the hero frame (3D + effects on text). showcase proves
//! the framework supports interactive apps: model selection, text input,
//! continuous physics particles, reveal transitions, orbital camera — all
//! running inside a single character-stream device.

use std::time::Duration;

use happyterminals::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

const BUNNY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/bunny.obj"
);
const COW_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/cow.obj"
);
const TEAPOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/teapot.obj"
);

const MENU_ITEMS: &[&str] = &["bunny", "cow", "teapot", "cube"];
const AUTO_ROTATION_SPEED: f32 = 0.30; // rad/s — slightly faster than text-reveal for showcase energy
const FRAME_DT: f32 = 0.033;
const INITIAL_CAMERA_DISTANCE: f32 = 3.5;
const ZOOM_FACTOR_PER_TICK: f32 = 0.88;
const MIN_CAMERA_DISTANCE: f32 = 0.5;
const MAX_CAMERA_DISTANCE: f32 = 25.0;
const FOV: f32 = std::f32::consts::FRAC_PI_4;
const REVEAL_DURATION_MS: u64 = 1200;
const MESSAGE_MAX_LEN: usize = 48;

#[must_use]
fn heat_color_for_symbol(sym: &str) -> Option<Color> {
    let mut chars = sym.chars();
    let first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    match first {
        '.' => Some(Color::Rgb(40, 20, 80)),
        ',' => Some(Color::Rgb(40, 60, 120)),
        '\'' => Some(Color::Rgb(30, 90, 150)),
        ':' => Some(Color::Rgb(30, 130, 160)),
        ';' => Some(Color::Rgb(60, 170, 140)),
        '!' => Some(Color::Rgb(130, 200, 100)),
        '+' => Some(Color::Rgb(200, 220, 80)),
        '*' => Some(Color::Rgb(240, 220, 80)),
        '=' => Some(Color::Rgb(250, 180, 60)),
        '#' => Some(Color::Rgb(255, 130, 50)),
        // '$' is NOT in the palette — money particles keep their green color
        // (set by the ParticleEmitter start/end gradient).
        '@' => Some(Color::Rgb(255, 50, 100)),
        _ => None,
    }
}

fn colorize_heat_map(grid: &mut Grid) {
    use ratatui_core::layout::Position;
    let area = grid.area;
    let buf = grid.buffer_mut();
    for y in area.y..(area.y + area.height) {
        for x in area.x..(area.x + area.width) {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                if let Some(color) = heat_color_for_symbol(cell.symbol()) {
                    cell.set_fg(color);
                }
            }
        }
    }
}

fn draw_axis_gizmo(grid: &mut Grid, camera: &OrbitCamera, corner: (u16, u16), radius: u16) {
    let (ox, oy) = corner;
    let grid_right = grid.area.x + grid.area.width;
    let grid_bottom = grid.area.y + grid.area.height;
    let r = f32::from(radius);
    grid.put_str(ox, oy, "+", Style::default().fg(Color::DarkGray));
    let view = camera.view_matrix();
    for (axis, label, color) in [
        (glam::Vec3::X, "X", Color::Red),
        (glam::Vec3::Y, "Y", Color::Green),
        (glam::Vec3::Z, "Z", Color::Blue),
    ] {
        let cam = view.transform_vector3(axis);
        let col_f = (f32::from(ox) + cam.x * r).round();
        let row_f = (f32::from(oy) - cam.y * r).round();
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

fn apply_camera_inputs(
    camera: &mut OrbitCamera,
    imap: &InputMap,
    viewport_w: u16,
    viewport_h: u16,
    auto_rotate_delta: f32,
) {
    let vw = f32::from(viewport_w.max(1));
    let vh = f32::from(viewport_h.max(1));
    let world_h = 2.0 * camera.distance * (FOV / 2.0).tan();
    let world_w = world_h * (vw / vh);
    let orbit_per_cell = std::f32::consts::PI / vw;
    let pan_per_cell_x = world_w / vw;
    let pan_per_cell_y = world_h / vh;

    let mut orbit_delta = glam::Vec2::ZERO;
    if let Some(s) = imap.action_axis2d("orbit") {
        orbit_delta = s.untracked();
        camera.azimuth -= orbit_delta.x * orbit_per_cell;
        camera.elevation += orbit_delta.y * orbit_per_cell;
    }
    if let Some(s) = imap.action_axis1d("zoom") {
        let delta = s.untracked();
        camera.distance = (camera.distance * ZOOM_FACTOR_PER_TICK.powf(delta))
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    }
    if let Some(s) = imap.action_axis2d("pan") {
        let delta = s.untracked();
        let right = glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
        let up = glam::Vec3::Y;
        camera.target -=
            right * delta.x * pan_per_cell_x + up * (-delta.y) * pan_per_cell_y;
    }
    if orbit_delta == glam::Vec2::ZERO {
        camera.azimuth += auto_rotate_delta;
    }
}

fn make_pipeline(effect_idx: usize) -> Pipeline {
    match effect_idx % 4 {
        0 => Pipeline::new().with(effects::fade_from(
            Color::Black,
            Color::Reset,
            Duration::from_millis(REVEAL_DURATION_MS),
        )),
        1 => Pipeline::new().with(effects::sweep_in(
            tachyonfx::Motion::LeftToRight,
            8,
            Color::DarkGray,
            Duration::from_millis(REVEAL_DURATION_MS),
        )),
        2 => Pipeline::new().with(effects::coalesce(Duration::from_millis(REVEAL_DURATION_MS))),
        _ => Pipeline::new().with(effects::evolve(
            tachyonfx::fx::EvolveSymbolSet::Circles,
            Duration::from_millis(REVEAL_DURATION_MS * 2),
        )),
    }
}

fn load_meshes() -> Vec<(String, Mesh)> {
    let mut meshes = Vec::new();
    for (name, path) in [("bunny", BUNNY_PATH), ("cow", COW_PATH), ("teapot", TEAPOT_PATH)] {
        match load_obj(path) {
            Ok((mesh, _stats)) => meshes.push((name.to_string(), mesh)),
            Err(e) => eprintln!("warning: failed to load {name}: {e}"),
        }
    }
    meshes.push(("cube".to_string(), Cube::mesh()));
    meshes
}

#[allow(clippy::too_many_lines)] // demo main + tick closure are naturally colocated
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meshes = load_meshes();
    if meshes.is_empty() {
        return Err("no meshes loaded — check examples/models/".into());
    }

    let mut camera = OrbitCamera {
        azimuth: 0.0,
        elevation: 0.35,
        distance: INITIAL_CAMERA_DISTANCE,
        ..OrbitCamera::default()
    };

    let mut renderer = Renderer::new();
    let shading_mesh = ShadingRamp::default();
    // Single-char ramp — every alive particle renders as '$'.
    let money_ramp = ShadingRamp {
        ramp: &['$'],
        light_dir: glam::Vec3::new(1.0, 1.0, 1.0).normalize(),
    };

    // Money particles — rain from above with gravity, tinted green.
    let mut emitter = ParticleEmitter::new(500);
    emitter.origin = glam::Vec3::new(0.0, 4.5, 0.0);
    emitter.spread = glam::Vec3::new(5.5, 0.4, 3.0);
    emitter.gravity = glam::Vec3::new(0.0, -3.0, 0.0);
    emitter.spawn_rate = 32.0;
    emitter.life_range = (2.5, 4.0);
    emitter.color_start = Color::Rgb(80, 255, 120); // vivid new-money green
    emitter.color_end = Color::Rgb(20, 70, 30);     // dimming as it falls

    let mut rng = SmallRng::from_rng(&mut rand::rng());

    // Input: default viewer (orbit/pan/zoom/quit) + menu + effect cycling.
    let mut input_map = InputMap::new();
    register_default_actions(&mut input_map);
    input_map.register_action("menu_up", ActionValueType::Bool);
    input_map.register_action("menu_down", ActionValueType::Bool);
    input_map.register_action("menu_select", ActionValueType::Bool);
    input_map.register_action("cycle_effect", ActionValueType::Bool);
    input_map.register_action("replay_reveal", ActionValueType::Bool);

    let mut ctx = default_viewer_context();
    ctx.bind("menu_up", Binding::Key(KeyCode::Up), vec![]);
    ctx.bind("menu_down", Binding::Key(KeyCode::Down), vec![]);
    ctx.bind("menu_select", Binding::Key(KeyCode::Enter), vec![]);
    ctx.bind("cycle_effect", Binding::Key(KeyCode::Tab), vec![]);
    ctx.bind("replay_reveal", Binding::Key(KeyCode::F(5)), vec![]);
    input_map.push_context(ctx);

    // Mutable state across frames.
    let mut current_model: usize = 0;
    let mut menu_highlight: usize = 0;
    let mut message: String = String::new();
    let mut effect_idx: usize = 0;
    let mut pipeline = make_pipeline(effect_idx);

    run_with_input(
        move |grid, input_signals, imap| {
            // 1. Raw text input — read and CLEAR last_key so same key can fire again.
            if let Some(InputEvent::Key { code, modifiers }) = input_signals.last_key.untracked() {
                let handled = match code {
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL)
                        && c.is_ascii()
                        && !c.is_control()
                        && c != 'q'  // reserved for quit by default_viewer_context
                    => {
                        if message.chars().count() < MESSAGE_MAX_LEN {
                            message.push(c);
                        }
                        true
                    }
                    KeyCode::Backspace => {
                        message.pop();
                        true
                    }
                    KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                        message.clear();
                        true
                    }
                    _ => false,
                };
                if handled {
                    input_signals.last_key.set(None);
                }
            }

            // 2. Menu + reveal controls.
            if let Some(s) = imap.action_state("menu_up") {
                if s.untracked() == ActionState::JustPressed {
                    menu_highlight = (menu_highlight + MENU_ITEMS.len() - 1) % MENU_ITEMS.len();
                }
            }
            if let Some(s) = imap.action_state("menu_down") {
                if s.untracked() == ActionState::JustPressed {
                    menu_highlight = (menu_highlight + 1) % MENU_ITEMS.len();
                }
            }
            if let Some(s) = imap.action_state("menu_select") {
                if s.untracked() == ActionState::JustPressed {
                    current_model = menu_highlight;
                    pipeline.reset();
                }
            }
            if let Some(s) = imap.action_state("cycle_effect") {
                if s.untracked() == ActionState::JustPressed {
                    effect_idx = (effect_idx + 1) % 4;
                    pipeline = make_pipeline(effect_idx);
                }
            }
            if let Some(s) = imap.action_state("replay_reveal") {
                if s.untracked() == ActionState::JustPressed {
                    pipeline.reset();
                }
            }

            // 3. Camera — orbit/pan/zoom from input, auto-rotate when idle.
            let auto_rotate = AUTO_ROTATION_SPEED * FRAME_DT;
            apply_camera_inputs(
                &mut camera,
                imap,
                grid.area.width,
                grid.area.height,
                auto_rotate,
            );

            // 4. Render current model.
            let (_name, mesh) = &meshes[current_model.min(meshes.len() - 1)];
            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                fov_y: FOV,
                ..Projection::default()
            };
            renderer.draw(grid, mesh, &camera, &projection, &shading_mesh);

            // 5. Update + draw money particles (uses single-char '$' ramp).
            emitter.update(FRAME_DT, &mut rng);
            renderer.draw_particles(grid, &emitter, &camera, &projection, &money_ramp);

            // 6. Heat-map mesh cells — '$' particles are NOT in the palette so they keep green.
            colorize_heat_map(grid);

            // 7. Overlays: title, menu panel, message, prompt.
            let title_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
            grid.put_str(
                grid.area.x + 1,
                grid.area.y,
                "HAPPY TERMINALS — Showcase",
                title_style,
            );

            // Menu panel — top-right.
            let menu_w = 13u16;
            let menu_x = grid.area.x + grid.area.width.saturating_sub(menu_w + 1);
            let menu_y = grid.area.y + 2;
            let menu_border = Style::default().fg(Color::Rgb(180, 180, 200));
            grid.put_str(menu_x, menu_y, "┌─ Models ──┐", menu_border);
            for (i, name) in MENU_ITEMS.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let y = menu_y + 1 + i as u16;
                if y >= grid.area.y + grid.area.height {
                    break;
                }
                let prefix = if i == menu_highlight { ">" } else { " " };
                let marker = if i == current_model { "*" } else { " " };
                let row = format!("│{prefix}{marker} {name:<7}│");
                let style = if i == menu_highlight {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i == current_model {
                    Style::default().fg(Color::Cyan)
                } else {
                    menu_border
                };
                grid.put_str(menu_x, y, &row, style);
            }
            #[allow(clippy::cast_possible_truncation)]
            let menu_bottom_y = menu_y + 1 + MENU_ITEMS.len() as u16;
            grid.put_str(menu_x, menu_bottom_y, "└───────────┘", menu_border);

            // Floating gold message — appears in the scene when user types.
            if !message.is_empty() {
                let msg_y = grid.area.y + grid.area.height.saturating_sub(5);
                #[allow(clippy::cast_possible_truncation)]
                let msg_len = message.chars().count() as u16;
                let msg_x = grid.area.x
                    + grid.area.width.saturating_sub(msg_len) / 2;
                let msg_style = Style::default()
                    .fg(Color::Rgb(255, 220, 100))
                    .add_modifier(Modifier::BOLD);
                grid.put_str(msg_x, msg_y, &message, msg_style);
            }

            // Bottom: hint + prompt.
            let hint_y = grid.area.y + grid.area.height.saturating_sub(2);
            let prompt_y = grid.area.y + grid.area.height.saturating_sub(1);
            let hint = "↑↓=menu  ⏎=pick  Tab=fx  F5=replay  scroll=zoom  drag=orbit  ^C=quit";
            grid.put_str(
                grid.area.x,
                hint_y,
                hint,
                Style::default().fg(Color::DarkGray),
            );
            let prompt = format!("> type: {message}_");
            grid.put_str(
                grid.area.x,
                prompt_y,
                &prompt,
                Style::default().fg(Color::Rgb(180, 180, 180)),
            );

            // 8. Apply the full-scene reveal pipeline — animates bunny + particles + UI together.
            pipeline.run_frame(grid, Duration::from_millis(33));

            // 9. Gizmo AFTER pipeline so it remains a stable orientation anchor.
            let gizmo_radius = 3u16;
            let gizmo_x = grid.area.x + grid.area.width.saturating_sub(gizmo_radius + 2);
            let gizmo_y = grid.area.y + grid.area.height.saturating_sub(gizmo_radius + 4);
            draw_axis_gizmo(grid, &camera, (gizmo_x, gizmo_y), gizmo_radius);
        },
        FrameSpec {
            title: Some("happyterminals - Showcase".into()),
            ..FrameSpec::default()
        },
        input_map,
    )
    .await
}
