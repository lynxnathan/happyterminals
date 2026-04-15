//! [`SceneBuilder`] -- the top-level builder for constructing a [`Scene`].
//!
//! Uses the consuming-self pattern: each method takes `self` by value and
//! returns `Self`, enabling fluent chains like:
//!
//! ```ignore
//! scene().camera(cam).layer(0, |l| l.cube()).build()?
//! ```

use happyterminals_pipeline::Pipeline;
use happyterminals_scene::node::SceneNode;
use happyterminals_scene::{CameraConfig, Scene, SceneError, SceneIr};

use crate::node_builder::LayerBuilder;

/// Top-level scene builder.
///
/// Accumulates layers, camera config, and an optional pipeline, then produces
/// a validated [`Scene`] via [`build()`](Self::build).
pub struct SceneBuilder {
    camera: Option<CameraConfig>,
    layers: Vec<SceneNode>,
    pipeline: Option<Pipeline>,
}

impl SceneBuilder {
    /// Create a new empty scene builder.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            camera: None,
            layers: Vec::new(),
            pipeline: None,
        }
    }

    /// Set the camera for the scene.
    ///
    /// Accepts any type that converts into [`CameraConfig`], including
    /// [`OrbitCamera`](happyterminals_renderer::OrbitCamera) directly.
    #[must_use]
    pub fn camera(mut self, cam: impl Into<CameraConfig>) -> Self {
        self.camera = Some(cam.into());
        self
    }

    /// Add a compositing layer with the given z-order.
    ///
    /// The closure receives a [`LayerBuilder`] and returns any type implementing
    /// `Into<LayerBuilder>` (including `LayerBuilder` itself or `CubeBuilder`).
    #[must_use]
    pub fn layer<R: Into<LayerBuilder>>(
        mut self,
        z_order: i16,
        f: impl FnOnce(LayerBuilder) -> R,
    ) -> Self {
        let lb = LayerBuilder::new(z_order);
        let lb: LayerBuilder = f(lb).into();
        self.layers.push(lb.into_node());
        self
    }

    /// Set the scene-level effect pipeline.
    #[must_use]
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Validate and build the final [`Scene`].
    ///
    /// # Errors
    ///
    /// - [`SceneError::MissingCamera`] if no camera was set.
    /// - [`SceneError::EmptyScene`] if no layers were added.
    /// - [`SceneError::DuplicateId`] if any node IDs collide.
    pub fn build(self) -> Result<Scene, SceneError> {
        let camera = self.camera.ok_or(SceneError::MissingCamera)?;
        let ir = SceneIr::new(self.layers);
        Scene::new(ir, camera, self.pipeline)
    }
}
