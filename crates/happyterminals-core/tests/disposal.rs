//! Integration test: `Owner` disposal runs `on_cleanup` and stops `Effect` reruns — REACT-04.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::cell::Cell;
use std::rc::Rc;

use happyterminals_core::{Effect, Signal, create_root, on_cleanup};

#[test]
fn on_cleanup_runs_on_owner_dispose() {
    let cleanups = Rc::new(Cell::new(0u32));
    let cleanups_c = Rc::clone(&cleanups);

    let ((), owner) = create_root(|| {
        on_cleanup(move || {
            cleanups_c.set(cleanups_c.get() + 1);
        });
    });

    assert_eq!(cleanups.get(), 0, "on_cleanup fired before dispose");
    owner.dispose();
    assert_eq!(cleanups.get(), 1, "on_cleanup did not fire after dispose");
}

#[test]
fn effect_stops_running_after_owner_dispose() {
    let runs = Rc::new(Cell::new(0u32));
    let runs_e = Rc::clone(&runs);

    let ((sig, _eff_guard), owner) = create_root(|| {
        let s = Signal::new(0i32);
        let s_clone = s.clone();
        let eff = Effect::new(move || {
            let _ = s_clone.get();
            runs_e.set(runs_e.get() + 1);
        });
        (s, eff)
    });

    assert_eq!(runs.get(), 1, "effect did not run at construction");
    sig.set(1);
    assert_eq!(runs.get(), 2, "effect did not re-run on set()");

    owner.dispose();

    // After disposal, further sets must NOT wake the effect.
    sig.set(2);
    sig.set(3);
    assert_eq!(
        runs.get(),
        2,
        "effect kept re-running after owner.dispose() (got {} runs; expected 2)",
        runs.get()
    );
}
