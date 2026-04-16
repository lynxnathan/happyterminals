//! `InputMap` dispatch engine with context stack resolution.
//!
//! The `InputMap` is the central hub of the input system. It holds a registry
//! of actions (each backed by reactive signals) and a stack of
//! [`InputContext`]s. When a crossterm event arrives via [`InputMap::dispatch`],
//! the engine walks the context stack top-down, fires the first matching
//! action, and sets the appropriate signal.

// Implementation in Task 2.
