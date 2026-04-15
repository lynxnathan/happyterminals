//! Smoke tests for the DSL builder chain.

use happyterminals_core::{create_root, Signal};
use happyterminals_dsl::scene;
use happyterminals_pipeline::Pipeline;
use happyterminals_renderer::OrbitCamera;
use happyterminals_scene::{NodeKind, PropValue, SceneError};

use glam::vec3;

#[test]
fn minimal_valid_scene() {
    let (_result, _owner) = create_root(|| {
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube())
            .build();
        assert!(s.is_ok());
    });
}

#[test]
fn missing_camera_returns_error() {
    let (_result, _owner) = create_root(|| {
        let s = scene().layer(0, |l| l.cube()).build();
        assert!(matches!(s, Err(SceneError::MissingCamera)));
    });
}

#[test]
fn empty_scene_returns_error() {
    let (_result, _owner) = create_root(|| {
        let s = scene().camera(OrbitCamera::default()).build();
        assert!(matches!(s, Err(SceneError::EmptyScene)));
    });
}

#[test]
fn cube_position_sets_transform() {
    let (_result, _owner) = create_root(|| {
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube().position(vec3(1., 2., 3.)))
            .build()
            .unwrap();

        let layer = &s.ir().nodes()[0];
        let cube = &layer.children[0];
        assert!((cube.transform.position.x - 1.0).abs() < f32::EPSILON);
        assert!((cube.transform.position.y - 2.0).abs() < f32::EPSILON);
        assert!((cube.transform.position.z - 3.0).abs() < f32::EPSILON);
    });
}

#[test]
fn cube_rotation_signal_stores_reactive_prop() {
    let (_result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);

        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube().rotation(&rotation))
            .build()
            .unwrap();

        let layer = &s.ir().nodes()[0];
        let cube = &layer.children[0];
        assert!(
            matches!(cube.props.get("rotation"), Some(PropValue::Reactive(_))),
            "rotation prop should be Reactive"
        );
    });
}

#[test]
fn layer_z_order_is_set() {
    let (_result, _owner) = create_root(|| {
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(5, |l| l.cube())
            .build()
            .unwrap();

        let layer = &s.ir().nodes()[0];
        assert!(
            matches!(layer.kind, NodeKind::Layer { z_order: 5 }),
            "Layer should have z_order=5, got {:?}",
            layer.kind
        );
    });
}

#[test]
fn layer_with_multiple_children() {
    let (_result, _owner) = create_root(|| {
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube().cube())
            .build()
            .unwrap();

        let layer = &s.ir().nodes()[0];
        assert_eq!(layer.children.len(), 2, "Layer should have 2 cube children");
    });
}

#[test]
fn group_builder_wraps_children() {
    let (_result, _owner) = create_root(|| {
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| {
                l.group(|g| g.cube().cube())
            })
            .build()
            .unwrap();

        let layer = &s.ir().nodes()[0];
        assert_eq!(layer.children.len(), 1, "Layer should have 1 group child");
        let group = &layer.children[0];
        assert!(
            matches!(group.kind, NodeKind::Group),
            "Child should be a Group, got {:?}",
            group.kind
        );
        assert_eq!(group.children.len(), 2, "Group should have 2 cube children");
    });
}

#[test]
fn scene_effect_stores_pipeline() {
    let (_result, _owner) = create_root(|| {
        let pipeline = Pipeline::new();
        let s = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube())
            .pipeline(pipeline)
            .build()
            .unwrap();

        assert!(s.pipeline().is_some(), "Scene should have a pipeline");
    });
}
