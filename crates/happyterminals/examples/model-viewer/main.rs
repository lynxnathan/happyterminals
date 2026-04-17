//! Model viewer — load bunny / cow / teapot and orbit with mouse + keyboard.
//!
//! All input is routed through the InputMap action system (Phase 2.3). Cycle
//! models with Left/Right arrows; everything else mirrors a standard
//! orbit-camera DCC viewer.
//!
//! Features exercised:
//! - OBJ mesh loading via `load_obj` (tobj, Phase 2.1)
//! - OrbitCamera with mouse-drag + scroll-zoom + WASD pan
//! - InputMap action system (register_default_actions + default_viewer_context)
//! - Renderer::draw on `&dyn Camera` (Phase 3.1 refactor)
//! - ShadingRamp + Projection defaults
//!
//! Controls:
//! - Left-drag: orbit (azimuth + elevation)
//! - Scroll: zoom (adjust camera distance)
//! - WASD: pan camera target
//! - Left/Right arrows: cycle model
//! - Ctrl-C or Q: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example model-viewer -p happyterminals

use happyterminals::prelude::*;

const MODELS: &[(&str, &str)] = &[
    (
        "bunny",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/bunny.obj"
        ),
    ),
    (
        "cow",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/cow.obj"
        ),
    ),
    (
        "teapot",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/teapot.obj"
        ),
    ),
];

const ZOOM_SENSITIVITY: f32 = 0.5;
const FOV: f32 = std::f32::consts::FRAC_PI_4; // 45° — matches Projection::default()

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meshes: Vec<(String, Mesh)> = MODELS
        .iter()
        .map(|(name, path)| load_obj(path).map(|(m, _stats)| ((*name).to_string(), m)))
        .collect::<Result<_, _>>()?;

    let (result, _owner) = create_root(|| {
        let current = Signal::new(0_usize);
        current
    });
    let current = result;

    // Set up InputMap with default viewer bindings (orbit, pan, zoom, quit)
    let mut input_map = InputMap::new();
    register_default_actions(&mut input_map);

    // Register model cycling + WASD pan actions
    input_map.register_action("cycle_next", ActionValueType::Bool);
    input_map.register_action("cycle_prev", ActionValueType::Bool);
    input_map.register_action("pan_left", ActionValueType::Bool);
    input_map.register_action("pan_right", ActionValueType::Bool);
    input_map.register_action("pan_up", ActionValueType::Bool);
    input_map.register_action("pan_down", ActionValueType::Bool);

    // Build default context and add model cycling + WASD bindings
    let mut ctx = default_viewer_context();
    ctx.bind(
        "cycle_next",
        Binding::Key(KeyCode::Right),
        vec![],
    );
    ctx.bind(
        "cycle_prev",
        Binding::Key(KeyCode::Left),
        vec![],
    );
    ctx.bind(
        "pan_left",
        Binding::Key(KeyCode::Char('a')),
        vec![],
    );
    ctx.bind(
        "pan_right",
        Binding::Key(KeyCode::Char('d')),
        vec![],
    );
    ctx.bind(
        "pan_up",
        Binding::Key(KeyCode::Char('w')),
        vec![],
    );
    ctx.bind(
        "pan_down",
        Binding::Key(KeyCode::Char('s')),
        vec![],
    );
    input_map.push_context(ctx);

    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();

    // Camera state: orbit driven by input actions
    let mut camera = OrbitCamera {
        elevation: 0.4,
        ..OrbitCamera::default()
    };
    let mut last_idx: usize = 0;

    run_with_input(
        move |grid, _input_signals, imap| {
            let vw = grid.area.width.max(1) as f32;
            let vh = grid.area.height.max(1) as f32;

            // World-space size of the viewport at the model's depth.
            // This makes drag feel 1:1 with the model surface.
            let world_h = 2.0 * camera.distance * (FOV / 2.0).tan();
            let world_w = world_h * (vw / vh);

            // Orbit: full-width drag ≈ PI rotation (180°)
            let orbit_per_cell = std::f32::consts::PI / vw;

            // Pan: 1 cell of drag = 1 cell of world movement at model depth
            let pan_per_cell_x = world_w / vw;
            let pan_per_cell_y = world_h / vh;

            if let Some(orbit_sig) = imap.action_axis2d("orbit") {
                let delta = orbit_sig.untracked();
                camera.azimuth -= delta.x * orbit_per_cell;
                camera.elevation += delta.y * orbit_per_cell;
            }

            if let Some(zoom_sig) = imap.action_axis1d("zoom") {
                let delta = zoom_sig.untracked();
                camera.distance = (camera.distance - delta * ZOOM_SENSITIVITY).max(0.5);
            }

            if let Some(pan_sig) = imap.action_axis2d("pan") {
                let delta = pan_sig.untracked();
                let right =
                    glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
                let up = glam::Vec3::Y;
                camera.target -=
                    right * delta.x * pan_per_cell_x + up * (-delta.y) * pan_per_cell_y;
            }

            // WASD keyboard pan — one step per keypress (terminals don't send
            // Release events, so Held would persist forever).
            {
                let right =
                    glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
                let up = glam::Vec3::Y;
                let step_x = pan_per_cell_x * 3.0; // 3 cells per keypress
                let step_y = pan_per_cell_y * 3.0;
                if let Some(s) = imap.action_state("pan_left") {
                    if s.untracked() == ActionState::JustPressed {
                        camera.target -= right * step_x;
                    }
                }
                if let Some(s) = imap.action_state("pan_right") {
                    if s.untracked() == ActionState::JustPressed {
                        camera.target += right * step_x;
                    }
                }
                if let Some(s) = imap.action_state("pan_up") {
                    if s.untracked() == ActionState::JustPressed {
                        camera.target += up * step_y;
                    }
                }
                if let Some(s) = imap.action_state("pan_down") {
                    if s.untracked() == ActionState::JustPressed {
                        camera.target -= up * step_y;
                    }
                }
            }

            // Model cycling (consume-on-read pattern via JustPressed)
            if let Some(next_sig) = imap.action_state("cycle_next") {
                if next_sig.untracked() == ActionState::JustPressed {
                    let len = MODELS.len();
                    current.set((current.untracked() + 1) % len);
                }
            }
            if let Some(prev_sig) = imap.action_state("cycle_prev") {
                if prev_sig.untracked() == ActionState::JustPressed {
                    let len = MODELS.len();
                    current.set((current.untracked() + len - 1) % len);
                }
            }

            let idx = current.untracked();
            let (name, mesh) = &meshes[idx];

            // Auto-fit camera distance when model changes
            if idx != last_idx {
                let (_center, radius) = mesh.bounding_sphere();
                camera.distance = radius * 2.5;
                last_idx = idx;
            }

            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                ..Projection::default()
            };

            renderer.draw(grid, mesh, &camera, &projection, &shading);

            let label = format!(
                " Model: {name}  (L-drag=orbit, R-drag=pan, Scroll=zoom, WASD=pan, Arrows=cycle, Q=quit) "
            );
            grid.put_str(0, 0, &label, Style::default());
        },
        FrameSpec {
            title: Some("happyterminals - Model Viewer".into()),
            ..FrameSpec::default()
        },
        input_map,
    )
    .await
}
