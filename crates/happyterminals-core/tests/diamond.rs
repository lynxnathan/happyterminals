//! Integration test: diamond-dependency graph fires the tip effect exactly once per propagation — REACT-02, REACT-10.
//!
//! Structure:
//!
//! ```text
//!     a
//!    / \
//!   b   c   (Memos)
//!    \ /
//!    eff    (Effect reads both)
//! ```
//!
//! After construction the effect has run once (init). After a single
//! `a.set(42)` the effect must run exactly one more time — totaling two.
//! A counter of 3 would indicate a thundering-herd regression (both Memos
//! fired the Effect independently instead of being batched by
//! `reactive_graph`'s two-phase propagation).

#![allow(
    clippy::unwrap_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::cell::Cell;
use std::rc::Rc;

use happyterminals_core::{Effect, Memo, Signal, create_root};

// BLOCKER: `ImmediateEffect` + `Memo` in `reactive_graph 0.2.13` deadlocks on
// signal propagation. When `Signal::set()` walks the subscriber graph, the
// synchronous recursion path holds `MemoInner.reactivity.read()` across the
// call-chain that eventually re-enters `MemoInner.update_if_necessary()` —
// which wants `reactivity.write()` on the same lock (pthread rwlock,
// write-after-read same-thread = deadlock).
//
// Reproduced with raw `reactive_graph` types (no our wrappers) at the same
// scale. This is a pre-existing incompatibility between the arena-backed
// `Memo` and the synchronous `ImmediateEffect` path; reactive_graph's own
// test suite never exercises the combination (see
// `reactive_graph-0.2.13/tests/memo.rs` — all async `Effect`, no
// `ImmediateEffect`).
//
// Escalated as a Phase 1.0 architectural blocker; see
// `.eclusa/phases/01.0-reactive-core/01.0-05-SUMMARY.md` §"Deferred Issues".
// The test body stays compiled so the blocker is visible in-file, but
// `#[ignore]` keeps `cargo test` green until the fix lands.
#[test]
#[ignore = "BLOCKER: ImmediateEffect + Memo deadlocks in reactive_graph 0.2.13 — see SUMMARY.md"]
fn diamond_fires_effect_once_per_propagation() {
    let ((), owner) = create_root(|| {
        let a = Signal::new(0i32);

        // Each Memo/Effect closure is `impl Fn() + 'static`, so it cannot
        // capture `a` (or `b`/`c`) by reference. `Signal<T>: Clone` is cheap
        // (shared Rc-backed state), so we clone into each closure's scope.
        let b = {
            let a = a.clone();
            Memo::new(move || a.get())
        };
        let c = {
            let a = a.clone();
            Memo::new(move || a.get() * 2)
        };

        let runs = Rc::new(Cell::new(0u32));
        let runs_e = Rc::clone(&runs);

        let _eff_guard = {
            let b = b.clone();
            let c = c.clone();
            Effect::new(move || {
                let _ = b.get();
                let _ = c.get();
                runs_e.set(runs_e.get() + 1);
            })
        };

        assert_eq!(
            runs.get(),
            1,
            "effect did not run at construction (got {})",
            runs.get()
        );

        a.set(42);

        assert_eq!(
            runs.get(),
            2,
            "effect fired {} times (expected 2: init + 1 propagation); \
             3 would indicate thundering herd (two Memo updates each \
             independently re-firing the Effect instead of being coalesced \
             by reactive_graph's two-phase propagation)",
            runs.get()
        );
    });
    owner.dispose();
}
