//! Scene transitions demo -- Phase 3.1 deliverable.
//!
//! Cycles through 3 scenes via Tab key, each using a different transition
//! effect:
//!   Scene 1 -> 2: "dissolve"
//!   Scene 2 -> 3: "slide-left"
//!   Scene 3 -> 1: "fade-to-black"
//!
//! Each scene renders a cube from a different camera angle so the transition
//! effects are clearly visible. Demonstrates `run_scenes()` with
//! `TransitionManager`, two-buffer blending, and owner disposal.
//!
//! Run from the workspace root:
//!
//!     cargo run --example transitions -p happyterminals

use std::time::Duration;

use happyterminals::prelude::*;

/// Scene definitions: (label, azimuth, elevation, effect to NEXT scene).
const SCENES: &[(&str, f32, f32, &str)] = &[
    ("Front view", 0.0, 0.3, "dissolve"),
    ("Side view", 1.2, 0.1, "slide-left"),
    ("Top view", 0.5, 1.2, "fade-to-black"),
];

const TRANSITION_DURATION: Duration = Duration::from_millis(600);

/// Build a scene containing a single cube with the given camera angles.
fn make_scene(azimuth: f32, elevation: f32) -> (Scene, Owner) {
    create_root(|| {
        scene()
            .camera(OrbitCamera {
                azimuth,
                elevation,
                distance: 4.0,
                ..OrbitCamera::default()
            })
            .layer(0, LayerBuilder::cube)
            .build()
            .unwrap_or_else(|e| unreachable!("static scene definition is always valid: {e}"))
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input_map = InputMap::new();
    input_map.register_action("quit", ActionValueType::Bool);
    input_map.register_action("next_scene", ActionValueType::Bool);

    let mut ctx = InputContext::new("transitions");
    ctx.bind(
        "quit",
        Binding::Key(crossterm::event::KeyCode::Char('q')),
        vec![],
    );
    ctx.bind(
        "next_scene",
        Binding::Key(crossterm::event::KeyCode::Tab),
        vec![],
    );
    input_map.push_context(ctx);

    let (_label, az, el, _effect) = SCENES[0];
    let (initial_scene, initial_owner) = make_scene(az, el);
    let mut tm = TransitionManager::new();
    tm.set_scene(initial_scene, initial_owner);

    let mut scene_idx: usize = 0;

    run_scenes(
        tm,
        FrameSpec {
            title: Some("happyterminals - Transitions".into()),
            ..FrameSpec::default()
        },
        input_map,
        move |_dt, input_signals, transition_manager| {
            // Check for Tab key to trigger next scene transition.
            // Collapse the two if-let patterns to satisfy clippy::collapsible_match.
            if let Some(InputEvent::Key {
                code: crossterm::event::KeyCode::Tab,
                ..
            }) = input_signals.last_key.untracked()
            {
                if !transition_manager.is_transitioning() {
                    let (_label, _az, _el, effect) = SCENES[scene_idx];
                    let next_idx = (scene_idx + 1) % SCENES.len();
                    let (_next_label, next_az, next_el, _) = SCENES[next_idx];

                    let (next_scene, next_owner) = make_scene(next_az, next_el);
                    let _ = transition_manager.transition_to(
                        next_scene,
                        next_owner,
                        effect,
                        TRANSITION_DURATION,
                    );
                    scene_idx = next_idx;
                }
                input_signals.last_key.set(None);
            }
        },
    )
    .await
}
