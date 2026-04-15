//! # happyterminals-core
//!
//! Single-threaded fine-grained reactive runtime wrapping `reactive_graph 0.2.13`
//! behind a happyterminals-owned public API.
//!
//! ## Threading contract
//!
//! `Signal<T>`, `Memo<T>`, `Effect`, and `Owner` are all `!Send + !Sync` — they
//! live on a single "render thread." Cross-thread writes go through
//! [`SignalSetter`], which queues updates into an mpsc channel that the render
//! thread drains once per frame tick via `Signal::drain_setter_queue`.
//!
//! ## `PartialEq` on `Memo`
//!
//! `Memo<T>` requires `T: PartialEq`. The memo only notifies its dependents
//! when the new value is `!=` the old — equality-skip is always on.
//!
//! ## Cycle detection
//!
//! If an Effect directly or transitively causes itself to re-enter while
//! still running, the runtime panics. Cycles are programmer errors, not
//! recoverable conditions. The exact panic message is defined by
//! `reactive_graph` v0.2.13 in this crate release; a custom labelled
//! message may be added in a later phase.
//!
//! ## Name collision warning
//!
//! `reactive_graph::wrappers::write::SignalSetter` is **unrelated** to
//! [`SignalSetter`] in this crate. The former is a synchronous signal-mutator
//! handle; ours is an mpsc-queue for cross-thread writes drained on the
//! render thread. We never re-export `reactive_graph`'s type.
//!
//! [`SignalSetter`]: crate::signal

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod batch;
pub mod clock;
pub mod effect;
pub mod error;
pub mod memo;
pub mod owner;
pub mod rng;
pub mod signal;

mod runtime; // private helpers
