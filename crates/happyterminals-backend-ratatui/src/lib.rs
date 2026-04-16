//! # happyterminals-backend-ratatui
//!
//! Runtime backend for happyterminals. Drives a `tokio::select!` loop between
//! a frame ticker and `crossterm::EventStream`. Provides `TerminalGuard`
//! (RAII + panic hook) that restores the terminal on panic or early return.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod color;
pub mod event;
pub mod frame_spec;
pub mod guard;
pub mod run;

pub use color::{detect_color_mode, detect_color_mode_from_real_env, ColorMode, EnvProvider, RealEnv};
pub use event::{InputEvent, InputSignals};
pub use frame_spec::FrameSpec;
pub use guard::{install_panic_hook, TerminalGuard};
pub use run::{run, run_scene, run_with_input};

// Re-export input types for consumers
// Re-export input types for consumers
pub use happyterminals_input::{
    self as input,
    ActionState, ActionValue, ActionValueType,
    Binding, DragAxis, InputContext, InputMap, InputModifier, ScrollDirection,
    default_viewer_context,
};
pub use happyterminals_input::defaults::register_default_actions;
