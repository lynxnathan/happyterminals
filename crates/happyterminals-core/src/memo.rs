//! `Memo<T>` — a cached reactive derived value with always-on equality-skip.
//!
//! A memo recomputes only when a tracked signal or memo read inside its
//! closure changes. Downstream effects and memos only re-run when the new
//! value is `!=` ([`PartialEq`]) the previous one.
//!
//! # Threading
//!
//! [`Memo<T>`] is **single-threaded**: `!Send + !Sync`. It wraps
//! `reactive_graph::computed::Memo`; the user-provided compute closure may
//! capture non-`Send` data (e.g. `Rc`, `RefCell`) because we route it through
//! an internal `SendWrapper` shell (`runtime::wrap_local_fn`) that panics if
//! dereferenced from another thread — matching our single-threaded promise.
//!
//! # Known deviation from CONTEXT.md §"API surface"
//!
//! `CONTEXT.md` specifies `impl<T: Clone + PartialEq + 'static> Memo<T>`
//! (no `Send + Sync` bound on `T`). `reactive_graph 0.2.13`'s
//! `computed::Memo<T>` — the type we delegate to — requires
//! `T: Send + Sync + 'static` on the outer type itself (the `SyncStorage`
//! default). The `SendWrapper` trick only relaxes the compute **closure**'s
//! bound, not the generic parameter's.
//!
//! Rather than fork or re-implement the memo path, we accept the narrower
//! bound `T: Clone + PartialEq + Send + Sync + 'static` for v0.0.0. In
//! practice this rules out only `Memo<Rc<_>>` / `Memo<RefCell<_>>`, which
//! users can replace with `Memo<Arc<_>>`. Revisiting this (e.g. a custom
//! `LocalStorage`-backed memo) is tracked as an open question in the phase
//! SUMMARY.
//!
//! # `PartialEq` contract
//!
//! `PartialEq` for `T` should be `O(1)` or `O(small)`. For large payloads,
//! prefer `Memo<Arc<Big>>` so the skip compares `Arc` pointers, not contents.

use std::marker::PhantomData;

use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::traits::{Get, GetUntracked};

use crate::runtime::wrap_local_fn;

/// A cached reactive derived value.
///
/// See the [module-level documentation](crate::memo) for the `PartialEq`
/// contract and the known `Send + Sync` bound on `T`.
///
/// **Single-threaded.** `Memo<T>` is `!Send + !Sync`. Use
/// [`crate::signal::Signal::setter`] for cross-thread writes into the
/// signals this memo reads from.
pub struct Memo<T>
where
    T: Send + Sync + 'static,
{
    // Fields are implementation details; public methods carry the docs.
    inner: RgMemo<T>,
    _not_send: PhantomData<*const ()>,
}

impl<T> Clone for Memo<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            _not_send: PhantomData,
        }
    }
}

impl<T> Memo<T>
where
    // Known deviation from CONTEXT.md: `Send + Sync` comes from
    // `reactive_graph::computed::Memo<T>`'s own bound. See the module doc.
    T: Clone + PartialEq + Send + Sync + 'static,
{
    /// Creates a new memo from a compute closure.
    ///
    /// The closure is called lazily on first [`Memo::get`] / [`Memo::untracked`]
    /// read and re-run whenever any tracked signal or memo read inside it
    /// changes. Dependents are only notified when the resulting `T` is `!=`
    /// ([`PartialEq`]) the previous value.
    ///
    /// The closure may capture non-`Send` values (e.g. an `Rc<Signal<_>>`
    /// clone); it is wrapped in a `SendWrapper` shell that pins it to the
    /// current thread.
    #[track_caller]
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        Self {
            inner: RgMemo::new(wrap_local_fn(f)),
            _not_send: PhantomData,
        }
    }

    /// Returns the current cached value, subscribing the active observer.
    #[must_use]
    pub fn get(&self) -> T {
        self.inner.get()
    }

    /// Returns the current cached value **without** subscribing the active
    /// observer.
    #[must_use]
    pub fn untracked(&self) -> T {
        self.inner.get_untracked()
    }
}
