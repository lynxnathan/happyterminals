//! Scene transitions — cycle through 3 scenes using dissolve, slide-left,
//! and fade-to-black effects (Phase 3.1 deliverable).
//!
//! Each scene is an independent reactive Owner; when the transition completes,
//! the outgoing Owner is disposed cleanly so no reactive effects leak. The
//! active scene's OrbitCamera remains fully interactive (orbit/zoom/pan) even
//! during a transition.
//!
//! Features exercised:
//! - TransitionManager with 3 named transition effects
//! - run_scenes backend entry point (transition-aware loop)
//! - InputMap action `next_scene` bound to Tab
//! - create_root / Owner-per-scene for clean reactive disposal
//! - OrbitCamera interactive controls routed through the active scene
//!
//! Controls:
//! - Tab: transition to the next scene (cycles dissolve → slide-left → fade-to-black)
//! - Left-drag: orbit the active scene's camera
//! - Scroll: zoom
//! - Right-drag: pan
//! - Q or Ctrl-C: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example transitions -p happyterminals

use std::time::Duration;

use happyterminals::prelude::*;

const SCENES: &[(&str, f32, f32, &str)] = &[
    ("Front view", 0.0, 0.3, "dissolve"),
    ("Side view", 1.2, 0.1, "slide-left"),
    ("Top view", 0.5, 1.2, "fade-to-black"),
];

const TRANSITION_DURATION: Duration = Duration::from_millis(600);
const FOV: f32 = std::f32::consts::FRAC_PI_4;

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
            .unwrap_or_else(|e| unreachable!("static scene: {e}"))
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input_map = InputMap::new();

    // Register all standard viewer actions + scene cycling
    register_default_actions(&mut input_map);
    input_map.register_action("next_scene", ActionValueType::Bool);

    let mut ctx = default_viewer_context();
    ctx.bind(
        "next_scene",
        Binding::Key(KeyCode::Tab),
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
        move |_dt, _input_signals, imap, transition_manager| {
            // Scene cycling via Tab
            if let Some(next_sig) = imap.action_state("next_scene") {
                if next_sig.untracked() == ActionState::JustPressed
                    && !transition_manager.is_transitioning()
                {
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
            }

            // Interactive camera controls on the active scene
            if let Some(cam_config) = transition_manager.current_camera_mut() {
                if let Some(orbit) = cam_config.as_orbit_mut() {
                    // Viewport-relative sensitivity (same as model-viewer)
                    let orbit_per_cell = std::f32::consts::PI / 80.0; // approximate

                    if let Some(orbit_sig) = imap.action_axis2d("orbit") {
                        let delta = orbit_sig.untracked();
                        orbit.azimuth -= delta.x * orbit_per_cell;
                        orbit.elevation += delta.y * orbit_per_cell;
                    }

                    if let Some(zoom_sig) = imap.action_axis1d("zoom") {
                        let delta = zoom_sig.untracked();
                        orbit.distance = (orbit.distance - delta * 0.5).max(0.5);
                    }

                    if let Some(pan_sig) = imap.action_axis2d("pan") {
                        let delta = pan_sig.untracked();
                        let world_h = 2.0 * orbit.distance * (FOV / 2.0).tan();
                        let pan_per_cell = world_h / 40.0; // approximate
                        let right = glam::Vec3::new(
                            orbit.azimuth.cos(), 0.0, -orbit.azimuth.sin(),
                        );
                        let up = glam::Vec3::Y;
                        orbit.target -=
                            (right * delta.x + up * (-delta.y)) * pan_per_cell;
                    }
                }
            }
        },
    )
    .await
}
