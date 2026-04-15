//! Tests for Task 2: SceneGraph, Scene, TransitionManager.

use std::cell::Cell;
use std::rc::Rc;

use happyterminals_core::{Owner, create_root, on_cleanup};
use happyterminals_renderer::camera::OrbitCamera;
use happyterminals_scene::{
    CameraConfig, NodeId, NodeKind, Scene, SceneError, SceneGraph, SceneIr, SceneNode,
    Transform, TransitionManager,
};

fn make_layer(z: i16) -> SceneNode {
    SceneNode {
        id: NodeId::next(),
        kind: NodeKind::Layer { z_order: z },
        transform: Transform::default(),
        props: Default::default(),
        children: vec![],
        pipeline: None,
    }
}

fn make_cube() -> SceneNode {
    SceneNode {
        id: NodeId::next(),
        kind: NodeKind::Cube,
        transform: Transform::default(),
        props: Default::default(),
        children: vec![],
        pipeline: None,
    }
}

#[test]
fn scene_graph_sorted_layers_ascending_z_order() {
    let ir = SceneIr::new(vec![make_layer(5), make_layer(-1), make_layer(3)]);
    let graph = SceneGraph::new(&ir);
    let layers = graph.sorted_layers();
    let z_orders: Vec<i16> = layers
        .iter()
        .map(|n| match n.kind {
            NodeKind::Layer { z_order } => z_order,
            _ => panic!("expected Layer"),
        })
        .collect();
    assert_eq!(z_orders, vec![-1, 3, 5]);
}

#[test]
fn scene_graph_sorted_layers_excludes_non_layers() {
    let ir = SceneIr::new(vec![
        make_layer(1),
        make_cube(), // not a layer
        make_layer(0),
    ]);
    let graph = SceneGraph::new(&ir);
    let layers = graph.sorted_layers();
    assert_eq!(layers.len(), 2);
}

#[test]
fn scene_wraps_ir_and_camera() {
    let ir = SceneIr::new(vec![make_layer(0)]);
    let camera = CameraConfig::Orbit(OrbitCamera::default());
    let scene = Scene::new(ir, camera, None);
    assert!(scene.is_ok());
    let scene = scene.unwrap_or_else(|e| panic!("unexpected error: {e}"));
    assert_eq!(scene.ir().nodes().len(), 1);
    match scene.camera() {
        CameraConfig::Orbit(cam) => {
            assert!((cam.distance - 5.0).abs() < f32::EPSILON);
        }
    }
}

#[test]
fn scene_ir_accessor_returns_reference() {
    let ir = SceneIr::new(vec![make_layer(0)]);
    let camera = CameraConfig::default();
    let scene = Scene::new(ir, camera, None).unwrap_or_else(|e| panic!("{e}"));
    let _ir_ref: &SceneIr = scene.ir();
    let _cam_ref: &CameraConfig = scene.camera();
}

#[test]
fn scene_rejects_empty_ir() {
    let ir = SceneIr::new(vec![]);
    let camera = CameraConfig::default();
    let result = Scene::new(ir, camera, None);
    assert!(matches!(result, Err(SceneError::EmptyScene)));
}

#[test]
fn scene_rejects_duplicate_ids() {
    let shared_id = NodeId::next();
    let ir = SceneIr::new(vec![
        SceneNode {
            id: shared_id,
            kind: NodeKind::Layer { z_order: 0 },
            transform: Transform::default(),
            props: Default::default(),
            children: vec![SceneNode {
                id: shared_id, // duplicate!
                kind: NodeKind::Cube,
                transform: Transform::default(),
                props: Default::default(),
                children: vec![],
                pipeline: None,
            }],
            pipeline: None,
        },
    ]);
    let camera = CameraConfig::default();
    let result = Scene::new(ir, camera, None);
    assert!(matches!(result, Err(SceneError::DuplicateId { .. })));
}

#[test]
fn transition_manager_new_has_no_scene() {
    let tm = TransitionManager::new();
    assert!(tm.current().is_none());
}

#[test]
fn transition_manager_set_scene_stores_scene() {
    let (_result, owner) = create_root(|| {});
    let ir = SceneIr::new(vec![make_layer(0)]);
    let camera = CameraConfig::default();
    let scene = Scene::new(ir, camera, None).unwrap_or_else(|e| panic!("{e}"));

    let mut tm = TransitionManager::new();
    tm.set_scene(scene, owner);
    assert!(tm.current().is_some());
}

#[test]
fn transition_manager_disposes_old_owner() {
    let cleaned_up = Rc::new(Cell::new(false));

    // Create old scene with cleanup callback
    let flag = Rc::clone(&cleaned_up);
    let (_result, old_owner) = create_root(move || {
        on_cleanup(move || {
            flag.set(true);
        });
    });

    let ir1 = SceneIr::new(vec![make_layer(0)]);
    let scene1 = Scene::new(ir1, CameraConfig::default(), None)
        .unwrap_or_else(|e| panic!("{e}"));

    let mut tm = TransitionManager::new();
    tm.set_scene(scene1, old_owner);
    assert!(!cleaned_up.get(), "cleanup should not have run yet");

    // Now transition to a new scene -- old owner should be disposed
    let (_result, new_owner) = create_root(|| {});
    let ir2 = SceneIr::new(vec![make_layer(0)]);
    let scene2 = Scene::new(ir2, CameraConfig::default(), None)
        .unwrap_or_else(|e| panic!("{e}"));
    tm.set_scene(scene2, new_owner);

    assert!(cleaned_up.get(), "old owner's on_cleanup should have run");
}

#[test]
fn transition_manager_take_consumes() {
    let (_result, owner) = create_root(|| {});
    let ir = SceneIr::new(vec![make_layer(0)]);
    let scene = Scene::new(ir, CameraConfig::default(), None)
        .unwrap_or_else(|e| panic!("{e}"));

    let mut tm = TransitionManager::new();
    tm.set_scene(scene, owner);
    let taken = tm.take();
    assert!(taken.is_some());
    assert!(tm.current().is_none());
}
