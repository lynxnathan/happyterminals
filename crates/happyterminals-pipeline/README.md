# happyterminals-pipeline

Effect pipeline for [happyterminals](https://github.com/lynxnathan/happyterminals).
Defines the `Effect` trait (`apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState`),
the `Pipeline` executor (a `Vec<Box<dyn Effect>>` so JSON recipes and Python can
construct pipelines at runtime), and `TachyonAdapter` which wraps any `tachyonfx`
shader as one of our `Effect` trait objects.

`tachyonfx::Effect` is aliased as `Fx` in our public surface to disambiguate
the two `Effect` names.

Dual-licensed under MIT OR Apache-2.0.
