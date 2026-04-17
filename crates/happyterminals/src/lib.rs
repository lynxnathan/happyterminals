//! # happyterminals
//!
//! Meta crate for the [happyterminals](https://github.com/lynxnathan/happyterminals)
//! scene manager. Re-exports a curated public surface so most users can write:
//!
//! ```ignore
//! use happyterminals::prelude::*;
//! ```

/// Curated re-export module.
///
/// Contains all the types needed to build a happyterminals application:
/// reactive primitives (Signal, Memo, Effect), Grid/Cell, run loop,
/// scene DSL, renderer types, and style types.
pub mod prelude {
    // Reactive core (from Phase 1.0)
    pub use happyterminals_core::{
        Signal, SignalSetter, Memo, Effect, Owner,
        create_root, on_cleanup, batch,
    };

    // Grid + Cell (from Phase 1.1, Plan 01)
    pub use happyterminals_core::{Grid, Cell};

    // Style types (re-exported through core)
    pub use happyterminals_core::{Color, Modifier, Style, Rect};

    // Backend (from Phase 1.1, Plan 02-03)
    pub use happyterminals_backend_ratatui::{run, run_scene, run_scenes, run_with_input, FrameSpec, InputEvent, InputSignals};
    pub use happyterminals_backend_ratatui::{TerminalGuard, install_panic_hook};

    // Color-mode control surface (from Phase 2.2)
    pub use happyterminals_backend_ratatui::ColorMode;

    // Input action system (from Phase 2.3)
    pub use happyterminals_input::{
        InputMap, InputContext, Binding, DragAxis, ScrollDirection,
        ActionValue, ActionValueType, ActionState,
        InputModifier,
        default_viewer_context,
    };
    pub use happyterminals_input::defaults::register_default_actions;

    // Camera types (from Phase 2.3)
    pub use happyterminals_renderer::{Camera, FreeLookCamera, FpsCamera};

    // Scene types (from Phase 1.4, Plan 01)
    pub use happyterminals_scene::{
        Scene, SceneIr, SceneNode, SceneGraph, SceneError,
        CameraConfig, NodeId, NodeKind, PropValue, TransitionManager,
    };

    // DSL builder (from Phase 1.4, Plan 02)
    pub use happyterminals_dsl::{scene, SceneBuilder};
    pub use happyterminals_dsl::node_builder::{
        CubeBuilder, GroupBuilder, LayerBuilder,
    };

    // JSON recipe loader (from Phase 3.2) + sandbox surface (from Phase 3.3)
    pub use happyterminals_dsl::json::{
        load_recipe, load_recipe_sandboxed, recipe_schema, scene_ir_to_recipe,
        RecipeError, SandboxConfig,
    };
    pub use happyterminals_dsl::sandbox::EffectRegistry;

    // Renderer types
    pub use happyterminals_renderer::{OrbitCamera, Projection, ShadingRamp};
    pub use happyterminals_renderer::{Cube, LoadStats, Mesh, MeshError, load_obj, load_stl};
    pub use happyterminals_renderer::{Particle, ParticleEmitter, lerp_color};

    // Pipeline
    pub use happyterminals_pipeline::{Pipeline, Fx};
    pub use happyterminals_pipeline::effects;

    // Math
    pub use glam::vec3;
}

#[cfg(test)]
mod tests {
    #[test]
    fn prelude_reexports_compile() {
        // Verify all prelude types are accessible. The nested helper `fn` is
        // declared before any statements (clippy::items_after_statements).
        use crate::prelude::*;
        // Signal, Memo, Effect etc. require reactive runtime context
        // so we just verify the types exist.
        fn _check_types() {
            let _ = std::any::type_name::<Signal<i32>>();
            let _ = std::any::type_name::<Memo<i32>>();
            let _ = std::any::type_name::<FrameSpec>();
            let _ = std::any::type_name::<InputEvent>();
            let _ = std::any::type_name::<InputSignals>();
            let _ = std::any::type_name::<TerminalGuard>();
            let _ = std::any::type_name::<Scene>();
            let _ = std::any::type_name::<SceneBuilder>();
            let _ = std::any::type_name::<CubeBuilder>();
            let _ = std::any::type_name::<OrbitCamera>();
            let _ = std::any::type_name::<Pipeline>();
            let _ = std::any::type_name::<Mesh>();
            let _ = std::any::type_name::<LoadStats>();
            let _ = std::any::type_name::<MeshError>();
            let _ = std::any::type_name::<Cube>();
            let _ = std::any::type_name::<ColorMode>();
            let _ = std::any::type_name::<InputMap>();
            let _ = std::any::type_name::<FreeLookCamera>();
            let _ = std::any::type_name::<FpsCamera>();
            let _ = std::any::type_name::<Particle>();
            let _ = std::any::type_name::<ParticleEmitter>();
            let _ = std::any::type_name::<SandboxConfig>();
            let _ = std::any::type_name::<EffectRegistry>();
            let _ = std::any::type_name::<RecipeError>();
        }
        let _: fn() -> Grid = || Grid::new(Rect::new(0, 0, 80, 24));
        let _: fn() -> FrameSpec = FrameSpec::default;
    }
}
