//! # happyterminals-pipeline
//!
//! Effect pipeline for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! Defines the [`Effect`] trait (`apply(&mut self, grid: &mut Grid, dt: Duration)`),
//! the [`Pipeline`](pipeline::Pipeline) executor (`Vec<Box<dyn Effect>>` so JSON
//! recipes and Python can construct pipelines at runtime), and in Plan 02 the
//! `TachyonAdapter` — which wraps any [`tachyonfx`] shader as one of our `Effect`
//! trait objects.
//!
//! To disambiguate the two `Effect` names, `tachyonfx::Effect` is re-exported
//! as [`Fx`] in our public surface. See `.eclusa/research/PITFALLS.md` section 16.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod adapter;
pub mod effect;
pub mod effects;
pub mod error;
pub mod pipeline;

pub use adapter::TachyonAdapter;
pub use effect::{Effect, EffectState};
pub use error::PipelineError;
pub use pipeline::Pipeline;

// D-02: Fx alias — tachyonfx::Effect renamed to avoid clash with our Effect trait
pub use tachyonfx::Effect as Fx;
