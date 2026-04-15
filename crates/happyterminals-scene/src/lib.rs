//! # happyterminals-scene
//!
//! Scene IR and scene-graph types for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! One intermediate representation (`SceneIr`) is the target of every front-end —
//! Rust builder, JSON recipes, and (future) Python. The scene graph supports
//! layered composition with explicit z-order and signal-driven prop bindings.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.4;
//! full `TransitionManager` in Phase 3.1.
