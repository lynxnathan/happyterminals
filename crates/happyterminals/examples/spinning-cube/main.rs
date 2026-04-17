//! Spinning cube — the M1 exit artifact and the shortest possible "hello world"
//! for the happyterminals stack.
//!
//! A single rotating cube driven by a reactive `Signal<f32>` updated each tick,
//! with a `coalesce` pipeline effect layered on top. Roughly 40 LOC total — the
//! smallest thing that exercises every layer of the framework.
//!
//! Features exercised:
//! - Reactive core: Signal, create_root, Owner
//! - Scene builder DSL: scene().camera().layer().pipeline()
//! - Rust-native OrbitCamera
//! - Pipeline with tachyonfx `coalesce` effect
//! - run_scene backend entry point
//!
//! Run from the workspace root:
//!
//!     cargo run --example spinning-cube -p happyterminals
//!
//! Why this exists:
//! The smallest artifact that proves every layer works together — signal,
//! scene, camera, renderer, effect pipeline, Grid, terminal. If this fails,
//! something in the stack is broken.

use happyterminals::prelude::*;
use std::time::Duration;

const ROTATION_SPEED: f32 = 1.5; // radians per second

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);
        let rotation_writer = rotation.clone();

        let scene = scene()
            .camera(OrbitCamera {
                azimuth: 0.0,
                elevation: 0.4,
                distance: 4.0,
                ..OrbitCamera::default()
            })
            .layer(0, |l| l.cube().rotation(&rotation))
            .pipeline(Pipeline::new().with(effects::coalesce(Duration::from_secs(1))))
            .build()?;

        Ok::<_, Box<dyn std::error::Error>>((scene, rotation_writer))
    });

    let (scene, rotation) = result?;

    run_scene(
        scene,
        FrameSpec {
            title: Some("happyterminals - Spinning Cube".into()),
            ..FrameSpec::default()
        },
        |dt, _input| {
            let current = rotation.untracked();
            rotation.set(current + dt.as_secs_f32() * ROTATION_SPEED);
        },
    )
    .await
}
