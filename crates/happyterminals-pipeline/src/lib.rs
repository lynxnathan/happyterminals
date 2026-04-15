//! # happyterminals-pipeline
//!
//! Effect pipeline for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! Defines the `Effect` trait (`apply(&mut self, grid: &mut Grid, dt: Duration)`),
//! the `Pipeline` executor (`Vec<Box<dyn Effect>>` so JSON recipes and Python can
//! construct pipelines at runtime), and `TachyonAdapter` — which wraps any
//! [`tachyonfx`] shader as one of our `Effect` trait objects.
//!
//! To disambiguate the two `Effect` names, `tachyonfx::Effect` is re-exported
//! as `Fx` in our public surface. See `.eclusa/research/PITFALLS.md` §16.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.2.
