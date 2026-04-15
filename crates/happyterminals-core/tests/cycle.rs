//! Integration test: self-cycling `Effect` panics (does not stack-overflow) — REACT-06.
//!
//! An Effect that reads `s` and then writes `s = s + 1` recursively triggers
//! itself. `reactive_graph`'s `ImmediateEffect` uses a `Mutex::try_lock`
//! guard inside its `FnMut` dispatcher; re-entry fails the try-lock and
//! panics. The runtime never stack-overflows.
//!
//! Message shape intentionally unchecked — see
//! `.eclusa/phases/01.0-reactive-core/01.0-CONTEXT.md` §"Cycle detection"
//! (amended 2026-04-14). Different `reactive_graph` releases may reword the
//! panic message; we only assert that a panic occurred.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use happyterminals_core::{Effect, Signal, create_root};

#[test]
fn self_cycling_effect_panics_not_overflows() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let ((), owner) = create_root(|| {
            let s = Signal::new(0i32);
            let s_eff = s.clone();
            let _eff_guard = Effect::new(move || {
                let v = s_eff.get();
                // Writing the signal we just read inside the same Effect is
                // a direct self-cycle. reactive_graph's re-entry guard must
                // fire instead of recursing forever.
                s_eff.set(v + 1);
            });
        });
        // If cycle detection did NOT fire we'd never reach here — the
        // `Effect::new` call triggers the initial run which performs the
        // self-set. `owner.dispose()` is only reached on the no-panic path.
        owner.dispose();
    }));

    assert!(
        result.is_err(),
        "self-cycling Effect did NOT panic — reactive_graph's cycle \
         detection failed or was bypassed. This violates REACT-06. \
         See RESEARCH §\"Integration test: cycle detection\" for the \
         expected behavior; escalate to Phase 1.0 exit review."
    );
}
