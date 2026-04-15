//! Integration test: `batch()` coalesces multiple signal writes into one
//! propagation pass — REACT-05.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::cell::Cell;
use std::rc::Rc;

use happyterminals_core::{Effect, Signal, batch, create_root};

#[test]
fn batch_coalesces_two_sets_into_one_propagation() {
    let runs = Rc::new(Cell::new(0u32));
    let runs_e = Rc::clone(&runs);

    let ((a, b, _eff), owner) = create_root(|| {
        let a = Signal::new(0i32);
        let b = Signal::new(0i32);

        let a_e = a.clone();
        let b_e = b.clone();
        let eff = Effect::new(move || {
            let _ = a_e.get();
            let _ = b_e.get();
            runs_e.set(runs_e.get() + 1);
        });

        (a, b, eff)
    });

    assert_eq!(runs.get(), 1, "effect did not run at construction");

    batch(|| {
        a.set(1);
        b.set(2);
    });

    assert_eq!(
        runs.get(),
        2,
        "batch(|| {{ a.set(1); b.set(2); }}) caused {} effect runs (expected 2: \
         init + 1 coalesced propagation). batch() is not coalescing the writes \
         (REACT-05).",
        runs.get()
    );

    owner.dispose();
}
