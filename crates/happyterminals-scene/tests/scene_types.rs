//! Tests for Task 1: Scene crate foundation types.

use happyterminals_core::{Signal, Memo, create_root};
use happyterminals_scene::{
    NodeId, NodeKind, SceneNode, PropValue, SceneIr, CameraConfig, SceneError, Transform,
};
use happyterminals_renderer::camera::OrbitCamera;

#[test]
fn node_id_next_returns_unique_ids() {
    let a = NodeId::next();
    let b = NodeId::next();
    let c = NodeId::next();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn scene_node_can_hold_children() {
    let parent = SceneNode {
        id: NodeId::next(),
        kind: NodeKind::Group,
        transform: Transform::default(),
        props: Default::default(),
        children: vec![
            SceneNode {
                id: NodeId::next(),
                kind: NodeKind::Cube,
                transform: Transform::default(),
                props: Default::default(),
                children: vec![],
                pipeline: None,
            },
        ],
        pipeline: None,
    };
    assert_eq!(parent.children.len(), 1);
}

#[test]
fn prop_value_static_stores_and_retrieves_f32() {
    let pv = PropValue::Static(Box::new(42.0_f32));
    let val = pv.get::<f32>();
    assert_eq!(val, Some(&42.0_f32));
}

#[test]
fn prop_value_reactive_stores_signal_f32() {
    let (_result, _owner) = create_root(|| {
        let sig = Signal::new(3.14_f32);
        let pv = PropValue::Reactive(Box::new(sig));
        let val = pv.read::<f32>();
        assert_eq!(val, Some(3.14_f32));
    });
}

#[test]
fn prop_value_reactive_stores_memo_f32() {
    let (_result, _owner) = create_root(|| {
        let sig = Signal::new(10.0_f32);
        let sig2 = sig.clone();
        let memo = Memo::new(move || sig2.get() * 2.0);
        let pv = PropValue::Reactive(Box::new(memo));
        let val = pv.read::<f32>();
        assert_eq!(val, Some(20.0_f32));
    });
}

#[test]
fn scene_ir_holds_root_children() {
    let ir = SceneIr::new(vec![
        SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Layer { z_order: 0 },
            transform: Transform::default(),
            props: Default::default(),
            children: vec![],
            pipeline: None,
        },
    ]);
    assert_eq!(ir.nodes().len(), 1);
}

#[test]
fn camera_config_orbit_wraps_orbit_camera() {
    let cam = CameraConfig::Orbit(OrbitCamera::default());
    match cam {
        CameraConfig::Orbit(inner) => {
            assert!((inner.distance - 5.0).abs() < f32::EPSILON);
        }
        _ => panic!("expected Orbit variant"),
    }
}

#[test]
fn scene_error_display_missing_camera() {
    let err = SceneError::MissingCamera;
    let msg = format!("{err}");
    assert!(msg.contains("camera"), "expected 'camera' in: {msg}");
}

#[test]
fn scene_error_display_empty_scene() {
    let err = SceneError::EmptyScene;
    let msg = format!("{err}");
    assert!(msg.contains("empty"), "expected 'empty' in: {msg}");
}

#[test]
fn scene_error_display_duplicate_id() {
    let id = NodeId::next();
    let err = SceneError::DuplicateId { node_id: id };
    let msg = format!("{err}");
    assert!(msg.contains("duplicate"), "expected 'duplicate' in: {msg}");
}

#[test]
fn scene_error_display_invalid_binding() {
    let id = NodeId::next();
    let err = SceneError::InvalidBinding {
        node_id: id,
        prop_name: "rotation".into(),
        reason: "wrong type".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("rotation"), "expected 'rotation' in: {msg}");
}
