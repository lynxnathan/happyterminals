//! Prelude re-exports for single-import hello world.
//!
//! ```ignore
//! use happyterminals_dsl::prelude::*;
//! ```

// DSL entry point + builder types
pub use crate::node_builder::{CubeBuilder, GroupBuilder, LayerBuilder};
pub use crate::{scene, SceneBuilder};

// Scene types
pub use happyterminals_scene::{
    CameraConfig, Scene, SceneError, SceneIr, TransitionManager,
};

// Reactive core
pub use happyterminals_core::{
    Signal, Memo, Effect, Owner,
    batch, create_root, on_cleanup,
};

// Renderer types
pub use happyterminals_renderer::{OrbitCamera, Projection, ShadingRamp};

// Pipeline
pub use happyterminals_pipeline::{Pipeline, Fx};

// JSON recipe loader
pub use crate::json::{load_recipe, recipe_schema, scene_ir_to_recipe, RecipeError};

// Math
pub use glam::vec3;
