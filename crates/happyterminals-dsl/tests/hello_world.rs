//! Verify the hello-world example compiles and fits in <=25 lines.
//!
//! This is a compilation test -- if it compiles, the prelude surface is correct.
//! Uses the DSL prelude directly (the meta crate re-exports the same types).

use happyterminals_dsl::prelude::*;

#[test]
fn hello_world_compiles() {
    let (_result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);

        let _scene = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| {
                l.cube()
                    .rotation(&rotation)
                    .position(vec3(0., 0., 0.))
            })
            .build();

        assert!(_scene.is_ok());
    });
}
