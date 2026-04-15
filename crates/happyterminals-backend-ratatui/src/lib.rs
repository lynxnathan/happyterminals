//! # happyterminals-backend-ratatui
//!
//! Runtime backend for happyterminals. Drives a `tokio::select!` loop between
//! a frame ticker and `crossterm::EventStream`. Provides `TerminalGuard`
//! (RAII + panic hook) that restores the terminal on panic or early return.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod guard;
pub mod event;
pub mod frame_spec;

pub use guard::{install_panic_hook, TerminalGuard};
