//! Scene-level 1-cell bytes test (DEMO-04 hard gate).
//!
//! Proves that changing one reactive signal in a full Scene (with a cube bound
//! to a rotation signal) produces minimal ANSI output -- not a full-buffer
//! repaint. This extends the SharedBuf pattern from `one_cell_change.rs`.

use std::cell::RefCell;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;

use happyterminals_core::{Grid, Signal, create_root};
use happyterminals_dsl::scene;
use happyterminals_renderer::{Cube, Mesh, OrbitCamera, Projection, Renderer, ShadingRamp};
use happyterminals_scene::node::{NodeKind, SceneNode};
use happyterminals_scene::CameraConfig;
use ratatui::Terminal;
use ratatui_crossterm::CrosstermBackend;

/// A `Write` wrapper around `Rc<RefCell<Vec<u8>>>` so we can share the byte
/// sink between the `CrosstermBackend` and our test assertions.
#[derive(Clone)]
struct SharedBuf(Rc<RefCell<Vec<u8>>>);

impl SharedBuf {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
    fn len(&self) -> usize {
        self.0.borrow().len()
    }
    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Inline rendering helper: walk nodes and render cubes.
fn render_scene_nodes(
    nodes: &[SceneNode],
    grid: &mut Grid,
    renderer: &mut Renderer,
    camera: &mut OrbitCamera,
    projection: &Projection,
    shading: &ShadingRamp<'_>,
    cube_mesh: &Mesh,
) {
    for node in nodes {
        match &node.kind {
            NodeKind::Cube => {
                if let Some(prop) = node.props.get("rotation") {
                    if let Some(angle) = prop.read_untracked::<f32>() {
                        camera.azimuth = angle;
                    }
                }
                renderer.draw(grid, cube_mesh, camera, projection, shading);
            }
            NodeKind::Layer { .. } | NodeKind::Group => {
                render_scene_nodes(&node.children, grid, renderer, camera, projection, shading, cube_mesh);
            }
            NodeKind::Custom(_) => {}
        }
    }
}

#[test]
fn scene_one_cell_change_minimal_bytes() {
    let (_result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);

        let built_scene = scene()
            .camera(OrbitCamera {
                elevation: 0.3,
                ..OrbitCamera::default()
            })
            .layer(0, |l| l.cube().rotation(&rotation))
            .build()
            .unwrap();

        let (ir, camera_config, _pipeline) = built_scene.into_parts();
        let CameraConfig::Orbit(mut camera) = camera_config;
        let mut renderer = Renderer::new();
        let projection = Projection {
            viewport_w: 80,
            viewport_h: 24,
            ..Projection::default()
        };
        let shading = ShadingRamp::default();
        let cube_mesh = Cube::mesh();

        let buf = SharedBuf::new();
        let backend = CrosstermBackend::new(buf.clone());
        let mut terminal = Terminal::new(backend).unwrap();

        // Frame 1: initial render
        terminal
            .draw(|frame| {
                let mut grid = Grid::new(frame.area());
                render_scene_nodes(
                    ir.nodes(),
                    &mut grid,
                    &mut renderer,
                    &mut camera,
                    &projection,
                    &shading,
                    &cube_mesh,
                );
                *frame.buffer_mut() = grid.deref().clone();
            })
            .unwrap();

        buf.clear();

        // Tiny signal change
        rotation.set(0.001);

        // Frame 2: re-render with nudged rotation
        terminal
            .draw(|frame| {
                let mut grid = Grid::new(frame.area());
                render_scene_nodes(
                    ir.nodes(),
                    &mut grid,
                    &mut renderer,
                    &mut camera,
                    &projection,
                    &shading,
                    &cube_mesh,
                );
                *frame.buffer_mut() = grid.deref().clone();
            })
            .unwrap();

        let delta = buf.len();
        assert!(
            delta <= 50,
            "Expected <= 50 bytes for 1-cell signal change, got {delta}"
        );
        assert!(delta > 0, "Expected some bytes for signal change, got 0");
    });
}
