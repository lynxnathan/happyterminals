//! Compile-time assertions: Send/Sync surface of the reactive core — REACT-08.
//!
//! If any of these change silently (e.g., a refactor accidentally makes
//! `Signal<T>: Send`), this file fails to compile and the test suite blocks
//! the PR. This is the "reactive threading tattoo" enforced at compile
//! time.

use happyterminals_core::{Effect, Memo, Owner, Signal, SignalSetter};
use static_assertions::{assert_impl_all, assert_not_impl_any};

// Reactive core types MUST NOT be Send or Sync.
assert_not_impl_any!(Signal<i32>: Send, Sync);
assert_not_impl_any!(Memo<i32>: Send, Sync);
assert_not_impl_any!(Effect: Send, Sync);
assert_not_impl_any!(Owner: Send, Sync);

// SignalSetter<T: Send> IS Send (that's its whole purpose).
assert_impl_all!(SignalSetter<i32>: Send);

// SignalSetter<T> implementing Sync is intentionally unasserted either
// direction for T=i32 in v0.0.0 — the inner `mpsc::Sender<T>` is Sync
// when T is Send + Sync, so the compile-time stance follows T. The
// important guarantee for our mpsc-queued-writes design is that
// SignalSetter is Send (asserted above); Sync is incidental.

#[test]
fn compile_time_send_sync_assertions_hold() {
    // If this file compiles, the assertions hold. This empty test exists
    // so `cargo test` reports a passing result.
}
