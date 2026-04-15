//! Model viewer -- Phase 2.1 exit artifact.
//!
//! Loads bunny / cow / teapot at startup, cycles them with Left / Right
//! arrow keys, renders each with bounding-sphere auto-fit camera distance.
//! Ctrl-C exits cleanly (`TerminalGuard` restores the terminal).
//!
//! Run from the workspace root:
//!
//!     cargo run --example model-viewer -p happyterminals
//!
//! Uses the low-level `run(render_fn, spec)` entry point rather than
//! `run_scene` -- the viewer manages its own 3-model state outside the
//! scene graph (Path A per phase 2.1 CONTEXT / RESEARCH).

use crossterm::event::KeyCode;
use happyterminals::prelude::*;
use happyterminals_renderer::Renderer;

// CARGO_MANIFEST_DIR is the crate root (crates/happyterminals), so the models
// directory lives two levels up. Using concat! keeps these paths resolved at
// compile time, independent of the cwd the example is launched from.
const MODELS: &[(&str, &str)] = &[
    ("bunny", concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/bunny.obj")),
    ("cow", concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/cow.obj")),
    ("teapot", concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/teapot.obj")),
];

const ROTATION_SPEED: f32 = 1.0; // rad/s -- gentler than spinning-cube

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Pre-load all three meshes once up front. Loading in the render loop
    // would violate the zero-per-frame-alloc discipline (see REND-09).
    let meshes: Vec<(String, Mesh)> = MODELS
        .iter()
        .map(|(name, path)| load_obj(path).map(|(m, _stats)| ((*name).to_string(), m)))
        .collect::<Result<_, _>>()?;

    let (result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);
        let current = Signal::new(0_usize);
        (rotation, current)
    });
    let (rotation, current) = result;

    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();
    let start = std::time::Instant::now();

    run(
        move |grid, input| {
            // Advance rotation from wall-clock for smooth motion across resizes.
            let elapsed = start.elapsed().as_secs_f32();
            rotation.set(elapsed * ROTATION_SPEED);

            // Consume-and-reset debounce: without the `set(None)` a held key
            // would re-cycle on every frame (PITFALLS §8).
            if let Some(InputEvent::Key { code, .. }) = input.last_key.untracked() {
                let len = MODELS.len();
                match code {
                    KeyCode::Right => current.set((current.untracked() + 1) % len),
                    KeyCode::Left => current.set((current.untracked() + len - 1) % len),
                    _ => {}
                }
                input.last_key.set(None);
            }

            let idx = current.untracked();
            let (name, mesh) = &meshes[idx];
            let (_center, radius) = mesh.bounding_sphere();
            let camera = OrbitCamera {
                azimuth: rotation.untracked(),
                elevation: 0.4,
                distance: radius * 2.5,
                ..OrbitCamera::default()
            };
            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                ..Projection::default()
            };

            renderer.draw(grid, mesh, &camera, &projection, &shading);

            // In-grid overlay for the current model name. Using a static
            // FrameSpec.title keeps us out of run.rs (constraint #7).
            let label = format!(" Model: {name}  (Left/Right to cycle, Ctrl-C to exit) ");
            grid.put_str(0, 0, &label, Style::default());
        },
        FrameSpec {
            title: Some("happyterminals - Model Viewer".into()),
            ..FrameSpec::default()
        },
    )
    .await
}
