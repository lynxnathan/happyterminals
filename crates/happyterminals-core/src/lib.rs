//! # happyterminals-core
//!
//! Foundational types for the [happyterminals](https://github.com/lynxnathan/happyterminals)
//! scene manager:
//!
//! - **Reactive primitives** — `Signal<T>`, `Memo<T>`, `Effect`, `Owner`, wrapping
//!   [`reactive_graph`] behind a happyterminals-owned public surface.
//! - **Grid buffer** — grapheme-cluster-aware cell grid, newtyped over
//!   [`ratatui_core::buffer::Buffer`] (compatibility verified in Phase 1.1).
//!
//! Phase 0 scaffolding — no public types yet. Implementations land in Phase 1.0
//! (reactive primitives) and Phase 1.1 (Grid).
//!
//! See `.eclusa/PROJECT.md` and `.eclusa/ROADMAP.md` for the full design.
