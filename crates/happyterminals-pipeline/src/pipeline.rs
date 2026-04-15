//! Sequential effect pipeline executor.

use std::time::Duration;

use happyterminals_core::Grid;

use crate::effect::{Effect, EffectState};

/// Sequential effect pipeline. Effects run in Vec order; each sees the Grid
/// after all prior effects have mutated it (per D-06).
///
/// # Invariant
///
/// Effects mutate Grid **only** through `Pipeline::run_frame()`. Do not pass
/// `&mut Grid` to effects outside of this method (per D-10, PIPE-07).
#[derive(Debug)]
pub struct Pipeline {
    effects: Vec<Box<dyn Effect>>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Builder: append an effect to the end of the chain.
    #[must_use]
    pub fn with(mut self, effect: impl Effect + 'static) -> Self {
        self.effects.push(Box::new(effect));
        self
    }

    /// Append a boxed effect (for runtime construction per D-03).
    #[must_use]
    pub fn with_boxed(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    /// Process one frame. Applies all non-done effects in sequence (D-06).
    /// Returns `Done` when every effect is done (D-07).
    pub fn run_frame(&mut self, grid: &mut Grid, dt: Duration) -> EffectState {
        let mut all_done = true;
        for effect in &mut self.effects {
            if !effect.is_done() {
                let state = effect.apply(grid, dt);
                if state == EffectState::Running {
                    all_done = false;
                }
            }
        }
        if all_done {
            EffectState::Done
        } else {
            EffectState::Running
        }
    }

    /// Reset all effects for replay (D-07).
    pub fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }

    /// Number of effects in the pipeline.
    #[must_use]
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Whether the pipeline has no effects.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::layout::Rect;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Mock effect that counts frames and reports Done after `done_after` calls.
    /// Records call order via a shared log.
    #[derive(Debug, Clone)]
    struct MockEffect {
        label: &'static str,
        done_after: u32,
        frame_count: u32,
        call_log: Rc<RefCell<Vec<(&'static str, u32)>>>,
    }

    impl MockEffect {
        fn new(label: &'static str, done_after: u32, log: Rc<RefCell<Vec<(&'static str, u32)>>>) -> Self {
            Self {
                label,
                done_after,
                frame_count: 0,
                call_log: log,
            }
        }
    }

    impl Effect for MockEffect {
        fn apply(&mut self, _grid: &mut Grid, _dt: Duration) -> EffectState {
            self.frame_count += 1;
            self.call_log.borrow_mut().push((self.label, self.frame_count));

            if self.frame_count >= self.done_after {
                EffectState::Done
            } else {
                EffectState::Running
            }
        }

        fn is_done(&self) -> bool {
            self.frame_count >= self.done_after
        }

        fn reset(&mut self) {
            self.frame_count = 0;
        }

        fn name(&self) -> &'static str {
            self.label
        }
    }

    fn make_grid() -> Grid {
        Grid::new(Rect::new(0, 0, 10, 5))
    }

    fn dt() -> Duration {
        Duration::from_millis(16)
    }

    #[test]
    fn pipeline_new_is_empty() {
        let p = Pipeline::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn pipeline_with_accepts_impl_effect() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let p = Pipeline::new().with(MockEffect::new("a", 1, log));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn pipeline_with_boxed_accepts_box_dyn_effect() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let boxed: Box<dyn Effect> = Box::new(MockEffect::new("a", 1, log));
        let p = Pipeline::new().with_boxed(boxed);
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn pipeline_run_frame_sequential_order() {
        // Two effects: "a" (done after 3) and "b" (done after 2)
        // Both should be called each frame in order a, b
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipeline::new()
            .with(MockEffect::new("a", 3, Rc::clone(&log)))
            .with(MockEffect::new("b", 2, Rc::clone(&log)));

        let mut grid = make_grid();

        // Frame 1: both called
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Running);

        let entries = log.borrow().clone();
        assert_eq!(entries, vec![("a", 1), ("b", 1)]);
        log.borrow_mut().clear();

        // Frame 2: both called, b finishes
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Running); // a still running

        let entries = log.borrow().clone();
        assert_eq!(entries, vec![("a", 2), ("b", 2)]);
        log.borrow_mut().clear();

        // Frame 3: only a called (b is done, skipped), a finishes
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Done);

        let entries = log.borrow().clone();
        assert_eq!(entries, vec![("a", 3)]);
    }

    #[test]
    fn pipeline_skips_done_effects() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipeline::new()
            .with(MockEffect::new("fast", 1, Rc::clone(&log)))
            .with(MockEffect::new("slow", 3, Rc::clone(&log)));

        let mut grid = make_grid();

        // Frame 1: both called, "fast" finishes
        p.run_frame(&mut grid, dt());
        log.borrow_mut().clear();

        // Frame 2: only "slow" called
        p.run_frame(&mut grid, dt());
        let entries = log.borrow().clone();
        assert_eq!(entries, vec![("slow", 2)]);
    }

    #[test]
    fn pipeline_returns_done_when_all_done() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipeline::new()
            .with(MockEffect::new("a", 1, Rc::clone(&log)));

        let mut grid = make_grid();
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Done);
    }

    #[test]
    fn pipeline_returns_running_when_any_running() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipeline::new()
            .with(MockEffect::new("a", 1, Rc::clone(&log)))
            .with(MockEffect::new("b", 2, Rc::clone(&log)));

        let mut grid = make_grid();
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Running);
    }

    #[test]
    fn pipeline_reset_resets_all_effects() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipeline::new()
            .with(MockEffect::new("a", 1, Rc::clone(&log)))
            .with(MockEffect::new("b", 1, Rc::clone(&log)));

        let mut grid = make_grid();

        // Run to completion
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Done);

        // Reset and run again
        p.reset();
        log.borrow_mut().clear();

        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Done);

        let entries = log.borrow().clone();
        assert_eq!(entries, vec![("a", 1), ("b", 1)]);
    }

    #[test]
    fn pipeline_empty_returns_done() {
        let mut p = Pipeline::new();
        let mut grid = make_grid();
        let state = p.run_frame(&mut grid, dt());
        assert_eq!(state, EffectState::Done);
    }

    #[test]
    fn pipeline_default_is_new() {
        let p = Pipeline::default();
        assert!(p.is_empty());
    }
}
