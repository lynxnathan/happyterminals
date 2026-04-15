//! `TachyonAdapter`: bridges tachyonfx effects into our [`Effect`] trait system (D-04).
//!
//! Wraps a [`tachyonfx::Effect`](crate::Fx) and implements our [`Effect`] trait
//! so any tachyonfx shader can participate in a [`Pipeline`](crate::Pipeline).
//!
//! Duration conversion: our `std::time::Duration` dt is converted to
//! `tachyonfx::Duration` (a `{milliseconds: u32}` struct) via `From`.
//! The adapter passes `grid.buffer_mut()` to the tachyonfx effect's `process()`.

use std::time::Duration;

use happyterminals_core::Grid;
use ratatui_core::layout::Rect;

use crate::effect::{Effect, EffectState};
use crate::Fx;

/// Adapts any tachyonfx [`Effect`](crate::Fx) into our [`Effect`] trait (D-04).
///
/// Duration conversion: `std::time::Duration` is converted to
/// `tachyonfx::Duration` via `From`. The adapter passes `grid.buffer_mut()`
/// to the tachyonfx effect's `process()`.
#[derive(Debug)]
pub struct TachyonAdapter {
    fx: Fx,
    area: Option<Rect>,
    completed: bool,
}

impl TachyonAdapter {
    /// Create a new adapter wrapping a tachyonfx effect.
    /// Uses the full grid area by default.
    #[must_use]
    pub fn new(fx: Fx) -> Self {
        Self {
            fx,
            area: None,
            completed: false,
        }
    }

    /// Create a new adapter with a custom area override.
    #[must_use]
    pub fn with_area(fx: Fx, area: Rect) -> Self {
        Self {
            fx,
            area: Some(area),
            completed: false,
        }
    }
}

impl Effect for TachyonAdapter {
    fn apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState {
        if self.completed {
            return EffectState::Done;
        }

        let tfx_dt = tachyonfx::Duration::from(dt);
        let area = self.area.unwrap_or(grid.area);
        let buf = grid.buffer_mut();
        self.fx.process(tfx_dt, buf, area);

        if self.fx.done() {
            self.completed = true;
            EffectState::Done
        } else {
            EffectState::Running
        }
    }

    fn is_done(&self) -> bool {
        self.completed
    }

    fn reset(&mut self) {
        self.fx.reset();
        self.completed = false;
    }

    fn name(&self) -> &'static str {
        self.fx.name()
    }
}

impl std::fmt::Display for TachyonAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TachyonAdapter({})", self.fx.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> Grid {
        Grid::new(Rect::new(0, 0, 80, 24))
    }

    #[test]
    fn adapter_wraps_dissolve_returns_running() {
        let tfx_effect = tachyonfx::fx::dissolve(500);
        let mut adapter = TachyonAdapter::new(tfx_effect);
        let mut grid = make_grid();
        let state = adapter.apply(&mut grid, Duration::from_millis(16));
        assert_eq!(state, EffectState::Running);
    }

    #[test]
    fn adapter_forwards_dt_correctly() {
        let tfx_effect = tachyonfx::fx::dissolve(200);
        let mut adapter = TachyonAdapter::new(tfx_effect);
        let mut grid = make_grid();

        for _ in 0..20 {
            adapter.apply(&mut grid, Duration::from_millis(16));
        }
        assert!(adapter.is_done());
    }

    #[test]
    fn adapter_is_done_initially_false() {
        let tfx_effect = tachyonfx::fx::dissolve(500);
        let adapter = TachyonAdapter::new(tfx_effect);
        assert!(!adapter.is_done());
    }

    #[test]
    fn adapter_reset_allows_replay() {
        let tfx_effect = tachyonfx::fx::dissolve(100);
        let mut adapter = TachyonAdapter::new(tfx_effect);
        let mut grid = make_grid();

        for _ in 0..10 {
            adapter.apply(&mut grid, Duration::from_millis(16));
        }
        assert!(adapter.is_done());

        adapter.reset();
        assert!(!adapter.is_done());
    }

    #[test]
    fn adapter_name_delegates() {
        let tfx_effect = tachyonfx::fx::dissolve(500);
        let adapter = TachyonAdapter::new(tfx_effect);
        let name = adapter.name();
        assert!(!name.is_empty());
    }

    #[test]
    fn adapter_as_box_dyn_effect() {
        let tfx_effect = tachyonfx::fx::dissolve(500);
        let adapter = TachyonAdapter::new(tfx_effect);
        let _boxed: Box<dyn Effect> = Box::new(adapter);
    }
}
