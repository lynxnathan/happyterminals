//! Effect trait and lifecycle state for the pipeline executor.

use std::time::Duration;

use happyterminals_core::Grid;

/// Lifecycle state returned by [`Effect::apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectState {
    /// The effect is still producing changes.
    Running,
    /// The effect has completed its work.
    Done,
}

/// A composable visual effect that mutates a [`Grid`] over time.
///
/// Effects are stateful: they track internal progress (timers, counters) and
/// report completion via [`EffectState`]. The pipeline executor calls
/// [`apply`](Effect::apply) once per frame for each running effect.
///
/// # Contract
///
/// - `apply` MUST be idempotent per frame (calling twice with the same `dt`
///   should not advance twice).
/// - `is_done` MUST return `true` after `apply` returns `EffectState::Done`.
/// - `reset` restores the effect to its initial state for replay.
pub trait Effect: std::fmt::Debug {
    /// Apply this effect to `grid` for one frame of duration `dt`.
    fn apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState;

    /// Whether this effect has completed.
    fn is_done(&self) -> bool;

    /// Reset the effect to its initial state, allowing replay.
    fn reset(&mut self);

    /// A human-readable name for debugging and JSON round-trip.
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::layout::Rect;

    /// Mock effect that runs for `n` frames then reports Done.
    #[derive(Debug)]
    struct CountdownEffect {
        remaining: u32,
        initial: u32,
    }

    impl CountdownEffect {
        fn new(frames: u32) -> Self {
            Self {
                remaining: frames,
                initial: frames,
            }
        }
    }

    impl Effect for CountdownEffect {
        fn apply(&mut self, _grid: &mut Grid, _dt: Duration) -> EffectState {
            if self.remaining == 0 {
                return EffectState::Done;
            }
            self.remaining -= 1;
            if self.remaining == 0 {
                EffectState::Done
            } else {
                EffectState::Running
            }
        }

        fn is_done(&self) -> bool {
            self.remaining == 0
        }

        fn reset(&mut self) {
            self.remaining = self.initial;
        }

        fn name(&self) -> &'static str {
            "countdown"
        }
    }

    #[test]
    fn effect_state_running_ne_done() {
        assert_ne!(EffectState::Running, EffectState::Done);
    }

    #[test]
    fn mock_effect_runs_then_done() {
        let mut effect = CountdownEffect::new(2);
        let mut grid = Grid::new(Rect::new(0, 0, 10, 5));
        let dt = Duration::from_millis(16);

        assert!(!effect.is_done());
        assert_eq!(effect.apply(&mut grid, dt), EffectState::Running);
        assert!(!effect.is_done());
        assert_eq!(effect.apply(&mut grid, dt), EffectState::Done);
        assert!(effect.is_done());
    }

    #[test]
    fn mock_effect_reset() {
        let mut effect = CountdownEffect::new(1);
        let mut grid = Grid::new(Rect::new(0, 0, 10, 5));
        let dt = Duration::from_millis(16);

        assert_eq!(effect.apply(&mut grid, dt), EffectState::Done);
        assert!(effect.is_done());

        effect.reset();
        assert!(!effect.is_done());
        assert_eq!(effect.apply(&mut grid, dt), EffectState::Done);
    }

    #[test]
    fn mock_effect_name() {
        let effect = CountdownEffect::new(1);
        assert_eq!(effect.name(), "countdown");
    }
}
