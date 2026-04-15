//! Integration test: `.untracked()` reads do NOT subscribe the active observer — REACT-07.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::cell::Cell;
use std::rc::Rc;

use happyterminals_core::{Effect, Signal, create_root};

#[test]
fn untracked_read_does_not_subscribe() {
    let runs = Rc::new(Cell::new(0u32));
    let runs_e = Rc::clone(&runs);

    let ((tracked, untracked, _eff_guard), owner) = create_root(|| {
        let tracked = Signal::new(0i32);
        let untracked = Signal::new(0i32);

        let t_e = tracked.clone();
        let u_e = untracked.clone();
        let eff = Effect::new(move || {
            // Only `t_e.get()` subscribes; `u_e.untracked()` does not.
            let _ = t_e.get();
            let _ = u_e.untracked();
            runs_e.set(runs_e.get() + 1);
        });

        (tracked, untracked, eff)
    });

    assert_eq!(runs.get(), 1, "effect did not run at construction");

    // Mutating an untracked-only signal should NOT re-run the effect.
    untracked.set(100);
    untracked.set(200);
    assert_eq!(
        runs.get(),
        1,
        "effect re-ran after mutation of a signal read only via .untracked() \
         (got {} runs; expected 1) — untracked() is subscribing when it \
         shouldn't (REACT-07)",
        runs.get()
    );

    // Mutating the tracked signal DOES re-run the effect.
    tracked.set(1);
    assert_eq!(
        runs.get(),
        2,
        "effect did not re-run after a set() on the tracked signal"
    );

    owner.dispose();
}
