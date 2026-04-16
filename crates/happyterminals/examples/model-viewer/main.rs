//! Model viewer -- Phase 2.3 upgrade.
//!
//! Loads bunny / cow / teapot at startup. Controls:
//! - Left-drag: orbit (azimuth + elevation)
//! - Scroll: zoom (adjust camera distance)
//! - WASD: pan camera target
//! - Left/Right arrows: cycle model
//! - Ctrl-C or Q: quit
//!
//! All input is routed through the InputMap action system.
//!
//! Run from the workspace root:
//!
//!     cargo run --example model-viewer -p happyterminals

use happyterminals::prelude::*;
use happyterminals_renderer::Renderer;

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

/// Shared drag sensitivity — orbit and pan use the same value so they
/// feel like the same "grab and move" gesture at the same speed.
const DRAG_SENSITIVITY: f32 = 0.01;
const ZOOM_SENSITIVITY: f32 = 0.5;

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
        Binding::Key(crossterm::event::KeyCode::Right),
        vec![],
    );
    ctx.bind(
        "cycle_prev",
        Binding::Key(crossterm::event::KeyCode::Left),
        vec![],
    );
    ctx.bind(
        "pan_left",
        Binding::Key(crossterm::event::KeyCode::Char('a')),
        vec![],
    );
    ctx.bind(
        "pan_right",
        Binding::Key(crossterm::event::KeyCode::Char('d')),
        vec![],
    );
    ctx.bind(
        "pan_up",
        Binding::Key(crossterm::event::KeyCode::Char('w')),
        vec![],
    );
    ctx.bind(
        "pan_down",
        Binding::Key(crossterm::event::KeyCode::Char('s')),
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
            // Orbit (left-drag) and pan (right-drag) share DRAG_SENSITIVITY
            // so they feel like the same "grab" gesture at the same speed.
            let sens = DRAG_SENSITIVITY * camera.distance;

            if let Some(orbit_sig) = imap.action_axis2d("orbit") {
                let delta = orbit_sig.untracked();
                camera.azimuth -= delta.x * sens;
                camera.elevation += delta.y * sens;
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
                camera.target -= (right * delta.x + up * (-delta.y)) * sens;
            }

            // WASD keyboard pan (supplementary)
            {
                let right =
                    glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
                let up = glam::Vec3::Y;
                let step = sens;
                if let Some(s) = imap.action_state("pan_left") {
                    if matches!(s.untracked(), ActionState::JustPressed | ActionState::Held(_)) {
                        camera.target -= right * step;
                    }
                }
                if let Some(s) = imap.action_state("pan_right") {
                    if matches!(s.untracked(), ActionState::JustPressed | ActionState::Held(_)) {
                        camera.target += right * step;
                    }
                }
                if let Some(s) = imap.action_state("pan_up") {
                    if matches!(s.untracked(), ActionState::JustPressed | ActionState::Held(_)) {
                        camera.target += up * step;
                    }
                }
                if let Some(s) = imap.action_state("pan_down") {
                    if matches!(s.untracked(), ActionState::JustPressed | ActionState::Held(_)) {
                        camera.target -= up * step;
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
