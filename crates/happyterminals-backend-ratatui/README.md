# happyterminals-backend-ratatui

Runtime backend for [happyterminals](https://github.com/lynxnathan/happyterminals).
Drives a `tokio::select!` loop between a frame ticker and `crossterm::EventStream`.
Provides `TerminalGuard` (RAII + panic hook) that restores the terminal (cursor,
raw mode, alternate screen, mouse capture, SGR state) on panic or early return —
Ctrl-C never leaves a trashed shell.

Dual-licensed under MIT OR Apache-2.0.
