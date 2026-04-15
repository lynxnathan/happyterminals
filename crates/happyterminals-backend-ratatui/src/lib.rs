//! # happyterminals-backend-ratatui
//!
//! Runtime backend for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! Drives a `tokio::select!` loop between a frame ticker and `crossterm::EventStream`.
//! Provides `TerminalGuard` (RAII + panic hook) that restores the terminal — cursor
//! visibility, raw mode, alternate-screen buffer, mouse capture, and SGR state — on
//! panic or early return. Ctrl-C never leaves a trashed shell.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.1.
