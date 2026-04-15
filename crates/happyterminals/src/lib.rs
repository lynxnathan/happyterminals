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
    pub use happyterminals_backend_ratatui::{run, FrameSpec, InputEvent, InputSignals};
    pub use happyterminals_backend_ratatui::{TerminalGuard, install_panic_hook};

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

    // Renderer types
    pub use happyterminals_renderer::{OrbitCamera, Projection, ShadingRamp};

    // Pipeline
    pub use happyterminals_pipeline::{Pipeline, Fx};

    // Math
    pub use glam::vec3;
}

#[cfg(test)]
mod tests {
    #[test]
    fn prelude_reexports_compile() {
        // Verify all prelude types are accessible
        use crate::prelude::*;
        let _: fn() -> Grid = || Grid::new(Rect::new(0, 0, 80, 24));
        let _: fn() -> FrameSpec = FrameSpec::default;
        // Signal, Memo, Effect etc. require reactive runtime context
        // so we just verify the types exist
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
        }
    }
}
