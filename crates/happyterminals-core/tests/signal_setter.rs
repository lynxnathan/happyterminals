//! Integration test: `SignalSetter` cross-thread writes become visible after
//! `drain_setter_queue()` on the owning thread — REACT-08.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests may unwrap on asserted invariants"
)]

use std::thread;

use happyterminals_core::{Signal, create_root};

#[test]
fn setter_writes_visible_after_drain_on_owning_thread() {
    let (sig, owner) = create_root(|| Signal::new(0i32));

    // Send handle: an mpsc queue is held in the Signal; the setter is Send.
    let setter = sig.setter();

    let handle = thread::spawn(move || {
        setter.set(42);
    });
    handle.join().expect("setter thread panicked");

    // Before drain, the write is queued but not applied.
    assert_eq!(
        sig.untracked(),
        0,
        "SignalSetter write applied before drain_setter_queue() was called — \
         that would violate the mpsc-queue-on-render-thread contract"
    );

    sig.drain_setter_queue();

    assert_eq!(
        sig.untracked(),
        42,
        "SignalSetter write NOT visible after drain_setter_queue() ran on \
         the owning thread; queue drain is broken (REACT-08)"
    );

    owner.dispose();
}
