//! [`Scene`] -- a validated, sealed wrapper around [`SceneIr`].
//!
//! Users cannot construct a `Scene` directly; it is created via
//! [`Scene::new`] which validates invariants (non-empty, no duplicate IDs).
//! The DSL crate's `SceneBuilder::build()` calls `Scene::new`.

use std::collections::HashSet;

use happyterminals_pipeline::Pipeline;

use crate::camera::CameraConfig;
use crate::error::SceneError;
use crate::ir::SceneIr;
use crate::node::{NodeId, SceneNode};

/// A validated scene ready for rendering.
///
/// Guarantees:
/// - At least one root node exists.
/// - All node IDs are unique within the tree.
///
/// Constructed via [`Scene::new`]. `Scene` is not `Clone` -- it owns the IR tree.
pub struct Scene {
    ir: SceneIr,
    camera: CameraConfig,
    pipeline: Option<Pipeline>,
}

impl Scene {
    /// Creates a new validated scene.
    ///
    /// # Errors
    ///
    /// - [`SceneError::EmptyScene`] if `ir` has no root children.
    /// - [`SceneError::DuplicateId`] if any two nodes share an ID.
    pub fn new(
        ir: SceneIr,
        camera: CameraConfig,
        pipeline: Option<Pipeline>,
    ) -> Result<Self, SceneError> {
        // Validate: not empty
        if ir.nodes().is_empty() {
            return Err(SceneError::EmptyScene);
        }

        // Validate: no duplicate IDs (O(n) via HashSet)
        let mut seen = HashSet::new();
        collect_ids_recursive(ir.nodes(), &mut seen)?;

        Ok(Self {
            ir,
            camera,
            pipeline,
        })
    }

    /// Returns a reference to the scene's IR tree.
    #[must_use]
    pub fn ir(&self) -> &SceneIr {
        &self.ir
    }

    /// Returns a reference to the scene's camera configuration.
    #[must_use]
    pub fn camera(&self) -> &CameraConfig {
        &self.camera
    }

    /// Returns a reference to the scene-level pipeline, if any.
    #[must_use]
    pub fn pipeline(&self) -> Option<&Pipeline> {
        self.pipeline.as_ref()
    }

    /// Consume the scene and return its parts for mutable access.
    ///
    /// Used by `run_scene()` which needs mutable ownership of the camera
    /// and pipeline while the IR tree stays immutable.
    #[must_use]
    pub fn into_parts(self) -> (SceneIr, CameraConfig, Option<Pipeline>) {
        (self.ir, self.camera, self.pipeline)
    }
}

/// Recursively collect all node IDs, returning an error on the first duplicate.
fn collect_ids_recursive(
    nodes: &[SceneNode],
    seen: &mut HashSet<NodeId>,
) -> Result<(), SceneError> {
    for node in nodes {
        if !seen.insert(node.id) {
            return Err(SceneError::DuplicateId { node_id: node.id });
        }
        collect_ids_recursive(&node.children, seen)?;
    }
    Ok(())
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("ir", &self.ir)
            .field("camera", &self.camera)
            .field("pipeline_present", &self.pipeline.is_some())
            .finish()
    }
}
