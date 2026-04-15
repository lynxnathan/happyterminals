//! `Memo<T>` — a cached reactive derived value with always-on equality-skip.
//!
//! Recomputes lazily on `.get()` — only when a tracked dependency has changed
//! since the last read. Downstream observers re-run only when the recomputed
//! value differs from the cached one ([`PartialEq`] equality-skip).
//!
//! # Implementation
//!
//! Custom lazy-pull wrapper built on [`Signal`] + [`Effect`](crate::effect::Effect):
//!
//! - An internal "version" [`Signal<u64>`] tracks dependency invalidation.
//!   An [`Effect`](crate::effect::Effect) watches the user's compute function's dependencies and
//!   bumps the version each time they change.
//! - `.get()` checks the version: if it has changed since the last read,
//!   the compute function re-runs and the result is cached. Downstream
//!   observers are notified via a separate "value" signal ONLY when the
//!   result differs from the previous cache (equality-skip).
//! - This lazy-pull design avoids the diamond thundering-herd problem
//!   inherent to eager-push implementations.
//!
//! ## Why custom
//!
//! `reactive_graph 0.2.13`'s `MemoInner::mark_subscribers_check` holds a
//! read lock on `reactivity` while iterating subscribers. If a subscriber
//! is an `ImmediateEffect`, it fires synchronously and calls back into the
//! Memo's `update_if_necessary`, which wants a write lock on the same
//! `RwLock` → same-thread deadlock. See `01.0-05-SUMMARY.md` for details.
//!
//! # Threading
//!
//! `!Send + !Sync`. See [`crate`]-level docs for the threading model.

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::signal::Signal;

/// A cached reactive derived value.
///
/// **Single-threaded.** `Memo<T>` is `!Send + !Sync`.
pub struct Memo<T>
where
    T: Clone + PartialEq + 'static,
{
    /// Rc-shared state for lazy recompute.
    state: Rc<MemoState<T>>,
    /// The signal downstream observers `.track()` when they call `.get()`.
    /// Only `.set()` when the value actually changes (equality-skip).
    value_signal: Signal<Option<T>>,
    _not_send: PhantomData<*const ()>,
}

struct MemoState<T> {
    compute: Box<dyn Fn() -> T>,
    /// Monotonically increasing version bumped by the driver Effect
    /// each time a tracked dependency changes.
    dirty_version: Cell<u64>,
    /// The version at which we last recomputed.
    read_version: Cell<u64>,
}

impl<T> Clone for Memo<T>
where
    T: Clone + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
            value_signal: self.value_signal.clone(),
            _not_send: PhantomData,
        }
    }
}

impl<T> Memo<T>
where
    T: Clone + PartialEq + 'static,
{
    /// Creates a new memo from a compute closure.
    ///
    /// The closure runs eagerly once at construction to populate the
    /// cache. Subsequent recomputes are lazy — only triggered by `.get()`
    /// when the dependency version has advanced.
    #[track_caller]
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        // Eagerly compute initial value.
        let initial = f();
        let value_signal: Signal<Option<T>> = Signal::new(Some(initial));

        let state = Rc::new(MemoState {
            compute: Box::new(f),
            dirty_version: Cell::new(0),
            read_version: Cell::new(0),
        });

        // Driver Effect: runs the compute fn to establish dep tracking,
        // then bumps dirty_version. We don't use the computed value here —
        // the real recompute happens lazily in `.get()`. The purpose of
        // this Effect is ONLY to subscribe to the compute fn's deps so
        // we get notified when they change.
        let state_for_effect = Rc::clone(&state);
        let vs_for_effect = value_signal.clone();
        let driver = crate::effect::Effect::new(move || {
            // Run compute to register deps. Discard the result.
            let _ = (state_for_effect.compute)();
            // Bump version so the next `.get()` knows to recompute.
            let old_dv = state_for_effect.dirty_version.get();
            let new_dv = old_dv + 1;
            state_for_effect.dirty_version.set(new_dv);
            // Notify downstream observers by poking the value signal.
            // The actual recompute + equality-skip happens lazily when
            // they call `.get()` → `ensure_fresh()`. We use a no-op
            // `update` to trigger notifications without changing the
            // value — downstream Effects will re-run and call
            // Memo::get(), which does the real work.
            if old_dv > 0 {
                // Skip the poke on the initial Effect run (version 0→1)
                // since the init value is already correct.
                vs_for_effect.update(|_| {});
            }
        });
        // Effect is a ZST; dropping is a no-op. Disposal is via the owner tree.
        #[allow(
            clippy::drop_non_drop,
            reason = "explicit intent: Effect is ZST, disposed by owner"
        )]
        drop(driver);

        // The driver's initial run bumped dirty_version to 1.
        // Set read_version to 1 too (we already have the initial value).
        state.read_version.set(state.dirty_version.get());

        Self {
            state,
            value_signal,
            _not_send: PhantomData,
        }
    }

    /// Recompute if dirty, update `value_signal` only if changed.
    fn ensure_fresh(&self) {
        let dv = self.state.dirty_version.get();
        let rv = self.state.read_version.get();
        if dv != rv {
            let new_val = (self.state.compute)();
            self.state.read_version.set(dv);

            // Equality-skip: only propagate if value actually changed.
            let current = self.value_signal.untracked();
            let changed = match current {
                Some(ref prev) => *prev != new_val,
                None => true,
            };
            if changed {
                self.value_signal.set(Some(new_val));
            }
        }
    }

    /// Returns the current value, subscribing the active observer.
    #[must_use]
    #[allow(clippy::expect_used, reason = "cache empty = programmer error")]
    pub fn get(&self) -> T {
        self.ensure_fresh();
        self.value_signal
            .get()
            .expect("Memo cache empty after recompute")
    }

    /// Returns the current value **without** subscribing.
    #[must_use]
    #[allow(clippy::expect_used, reason = "see Memo::get")]
    pub fn untracked(&self) -> T {
        self.ensure_fresh();
        self.value_signal
            .untracked()
            .expect("Memo cache empty after recompute")
    }
}
