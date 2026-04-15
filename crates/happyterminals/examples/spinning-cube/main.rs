//! Spinning cube demo -- M1 exit artifact.
//! Proves the full stack: signal -> camera -> renderer -> pipeline -> grid -> terminal.

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
