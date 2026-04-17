//! # happyterminals-scene
//!
//! Scene IR and scene-graph types for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! One intermediate representation ([`SceneIr`]) is the target of every front-end --
//! Rust builder, JSON recipes, and (future) Python. The scene graph supports
//! layered composition with explicit z-order and signal-driven prop bindings.

#![forbid(unsafe_code)]

pub mod camera;
pub mod easing;
pub mod error;
pub mod graph;
pub mod ir;
pub mod node;
pub mod prop;
pub mod scene;
pub mod transition;
pub mod transition_effect;

// Re-export key types at the crate root.
pub use camera::CameraConfig;
pub use error::SceneError;
pub use graph::SceneGraph;
pub use ir::SceneIr;
pub use node::{NodeId, NodeKind, PropMap, SceneNode, Transform};
pub use prop::{AnyReactive, PropValue};
pub use scene::Scene;
pub use easing::{ease_in_out, ease_out_cubic, linear};
pub use transition::TransitionManager;
pub use transition_effect::{Dissolve, FadeToBlack, SlideLeft, TransitionEffect};
