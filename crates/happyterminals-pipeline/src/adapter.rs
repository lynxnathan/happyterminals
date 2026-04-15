//! TachyonAdapter: bridges tachyonfx effects into our [`Effect`] trait system.
//!
//! Wraps a [`tachyonfx::Effect`](crate::Fx) (the tachyonfx effect struct) and
//! implements our [`Effect`] trait so any tachyonfx shader can participate in a
//! [`Pipeline`](crate::Pipeline).

// Implementation will be added in the GREEN phase.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use happyterminals_core::Grid;
    use ratatui_core::layout::Rect;

    use crate::adapter::TachyonAdapter;
    use crate::effect::{Effect, EffectState};

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
        // 200ms effect, advance 250ms worth of time => done
        let tfx_effect = tachyonfx::fx::dissolve(200);
        let mut adapter = TachyonAdapter::new(tfx_effect);
        let mut grid = make_grid();

        // Advance enough time for the effect to complete
        for _ in 0..20 {
            adapter.apply(&mut grid, Duration::from_millis(16));
        }
        // 20 * 16ms = 320ms > 200ms => should be done
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

        // Run to completion
        for _ in 0..10 {
            adapter.apply(&mut grid, Duration::from_millis(16));
        }
        assert!(adapter.is_done());

        // Reset and verify not done
        adapter.reset();
        assert!(!adapter.is_done());
    }

    #[test]
    fn adapter_name_delegates() {
        let tfx_effect = tachyonfx::fx::dissolve(500);
        let adapter = TachyonAdapter::new(tfx_effect);
        // tachyonfx dissolve reports its name
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
