//! `Owner`, [`create_root`], [`on_cleanup`] — the scope tree of the reactive
//! core.
//!
//! An [`Owner`] is an RAII handle over `reactive_graph::owner::Owner`. It owns
//! every [`crate::effect::Effect`], [`crate::memo::Memo`], and [`on_cleanup`]
//! callback registered inside its scope; disposing the owner cascades cleanup
//! to all of them.
//!
//! # Threading
//!
//! [`Owner`] is **single-threaded** (`!Send + !Sync`). The underlying
//! `reactive_graph` owner uses a thread-local current-owner slot; moving the
//! handle across threads would desynchronize that slot.
//!
//! # `on_cleanup` out-of-scope behavior
//!
//! [`on_cleanup`] panics in debug builds when called outside an active owner
//! scope, and falls back to an `eprintln!` warning plus a dropped callback in
//! release. `reactive_graph`'s own `on_cleanup` silently drops the callback
//! in both modes; our debug-panic prevents that footgun from reaching
//! production. A later phase may upgrade the release path to `tracing::warn!`
//! when a subscriber is wired by the backend — see PLAN 01.0-03 §decisions.

use std::marker::PhantomData;

use reactive_graph::owner::Owner as RgOwner;
use send_wrapper::SendWrapper;

/// A reactive scope handle.
///
/// Disposing an [`Owner`] cascades cleanup to every [`crate::effect::Effect`],
/// [`crate::memo::Memo`], and [`on_cleanup`] callback registered inside it.
///
/// `Owner` is `!Send + !Sync` — see the module-level doc.
#[must_use = "an Owner must be held for the duration of its scope; dropping \
              one immediately disposes every Effect, Memo, and on_cleanup \
              callback registered inside"]
pub struct Owner {
    inner: RgOwner,
    _not_send: PhantomData<*const ()>,
}

impl Owner {
    /// Runs `f` with `self` set as the current reactive owner. Signals,
    /// memos, and effects created inside `f` register under this owner.
    pub fn run_in<R>(&self, f: impl FnOnce() -> R) -> R {
        self.inner.with(f)
    }

    /// Disposes this owner, running cleanup callbacks and dropping every
    /// [`crate::effect::Effect`] / [`crate::memo::Memo`] registered inside
    /// its scope.
    ///
    /// Taking `self` by value prevents double-dispose through the type
    /// system; however, `reactive_graph::owner::Owner::cleanup` is
    /// idempotent by construction (it drains `Vec`-based cleanup and node
    /// lists), so accidental double-drops via `Drop` after explicit dispose
    /// are harmless.
    pub fn dispose(self) {
        self.inner.cleanup();
    }
}

/// Creates a new root reactive scope, runs `f` inside it, and returns the
/// value plus the owner handle.
///
/// Drop or [`Owner::dispose`] the returned owner to clean up every
/// [`crate::effect::Effect`] / [`crate::memo::Memo`] / [`on_cleanup`]
/// callback allocated inside `f`.
///
/// Implementation note: `reactive_graph::owner::Owner::new_root` is gated
/// behind the `hydration` feature in 0.2.13 — not a feature we enable — so we
/// use [`reactive_graph::owner::Owner::new`] instead. At top-level (no
/// current owner set on the thread) `new()` returns a parentless owner,
/// which is exactly a root. If `create_root` is called from inside another
/// [`Owner::run_in`] scope it will produce a child of that scope, not a
/// true root — callers who need a detached root must invoke `create_root`
/// outside any existing scope.
#[track_caller]
pub fn create_root<R>(f: impl FnOnce() -> R) -> (R, Owner) {
    let inner = RgOwner::new();
    let result = inner.with(f);
    (
        result,
        Owner {
            inner,
            _not_send: PhantomData,
        },
    )
}

/// Registers `f` to run when the current owner is disposed.
///
/// # Panics
///
/// In **debug builds** (`cfg(debug_assertions)`), panics if called outside an
/// active owner scope. In release builds, logs a warning to stderr and
/// silently drops `f` — matching the release-mode philosophy of not crashing
/// on programmer errors that were caught during development.
///
/// # Send/Sync bound
///
/// `reactive_graph::owner::on_cleanup` requires `FnOnce() + Send + Sync +
/// 'static`. We preserve `CONTEXT.md`'s narrower public bound
/// (`FnOnce() + 'static`) by wrapping `f` in a [`SendWrapper`] — the cleanup
/// will only ever run on the owning thread, so the `Send + Sync` erasure is
/// safe. If invoked from another thread the `SendWrapper` deref would panic
/// first; this matches our single-threaded runtime contract.
#[track_caller]
#[allow(
    clippy::manual_assert,
    reason = "cfg-gated panic path cannot be expressed as a single assert! \
              without duplicating the cfg(debug_assertions) branch"
)]
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    if !owner_scope_active() {
        #[cfg(debug_assertions)]
        {
            #[allow(
                clippy::panic,
                reason = "intentional debug-only panic: on_cleanup outside an \
                          owner scope is a programmer error caught loudly \
                          during development"
            )]
            {
                panic!("happyterminals_core::on_cleanup called outside an owner scope");
            }
        }
        #[cfg(not(debug_assertions))]
        {
            eprintln!(
                "[happyterminals_core::on_cleanup] warning: called outside \
                 owner scope; callback dropped"
            );
            return;
        }
    }

    // Wrap the FnOnce in a SendWrapper<Option<F>> so the outer closure can
    // `.take()` the callback exactly once when the owner is cleaned up, even
    // though reactive_graph's cleanup-list entry is typed as
    // `Box<dyn FnOnce() + Send + Sync>` and we need to move the inner
    // `FnOnce` out to invoke it. `SendWrapper::take(self)` would consume the
    // whole wrapper, but the cleanup closure must be an `FnOnce` that
    // captures the wrapper by move — the `Option::take()` pattern inside a
    // `RefCell` would overcomplicate things; the closure-level move captures
    // work because reactive_graph invokes the cleanup callback at most once.
    let wrapped = SendWrapper::new(f);
    reactive_graph::owner::on_cleanup(move || {
        // `SendWrapper::take` panics if invoked from another thread. Since
        // cleanup runs during `Owner::cleanup`, which by our single-threaded
        // contract always runs on the owning thread, this is safe. `take`
        // consumes the wrapper and yields the inner `FnOnce`, which we then
        // invoke exactly once.
        let f = wrapped.take();
        f();
    });
}

/// Scope-check helper. Returns `true` if a reactive [`Owner`] is currently
/// active on this thread.
///
/// Uses `reactive_graph::owner::Owner::current()` — confirmed present in
/// 0.2.13 by spike A (see `runtime::__spike_owner_current_exists`).
fn owner_scope_active() -> bool {
    RgOwner::current().is_some()
}
