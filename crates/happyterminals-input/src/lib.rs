//! Godot/Unreal-inspired input action system with reactive signals.
//!
//! Actions are identified by string keys and typed as `Bool`, `Axis1D`, or
//! `Axis2D`. Each registered action owns a reactive `Signal` -- when raw
//! terminal events arrive, `InputMap::dispatch()` resolves bindings through
//! a context stack and sets the signal directly. No polling.

pub mod action;
pub mod binding;
pub mod context;
pub mod defaults;
pub mod drag;
pub mod input_map;
pub mod modifier;

pub use action::{ActionEntry, ActionState, ActionValue, ActionValueType};
pub use binding::{Binding, DragAxis, ScrollDirection};
pub use context::{FiredAction, InputContext};
pub use drag::{DragOutput, DragState, DragStateMachine};
pub use defaults::default_viewer_context;
pub use input_map::InputMap;
pub use modifier::InputModifier;
