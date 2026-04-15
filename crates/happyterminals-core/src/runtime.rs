//! Internal runtime glue: the `SendWrapper` helper used by `Effect`, and
//! small compile-time spikes that pin down MEDIUM-confidence API details
//! from `reactive_graph` 0.2.13.
//!
//! Nothing here is public. The spikes exist to fail fast at build time if
//! `reactive_graph`'s shape drifts from what `RESEARCH.md` assumes.
//!
//! ## Spike outcomes (Plan 01.0-01, 2026-04-14)
//!
//! - **Spike A (`Owner::current()`):** path used = `reactive_graph::owner::Owner::current()`.
//!   Returns `Option<Owner>`. No fallback needed; 01.0-03 consumes this directly
//!   via [`__spike_owner_current_exists`].
//! - **Spike B (`SendWrapper<RefCell<FnMut>>`):** path used =
//!   `send_wrapper::SendWrapper` as a **direct dependency** of this crate.
//!   `reactive_graph` does not re-export `send_wrapper` at
//!   `reactive_graph::send_wrapper_ext::SendWrapper` in 0.2.13, so the research
//!   note's preferred transitive path does not apply. We add `send_wrapper = "0.6"`
//!   to `crates/happyterminals-core/Cargo.toml` (not workspace-level — this is a
//!   core-crate-internal concern). [`wrap_local_fnmut`] compiles and flows
//!   through `ImmediateEffect::new_mut_scoped`'s `Send + Sync + 'static` bound
//!   without further adjustment.
//!
//!   **Secondary finding:** `ImmediateEffect::new_mut_scoped` in 0.2.13 returns
//!   `()` (not an `ImmediateEffect` handle). The effect runs once synchronously
//!   and is kept alive through the owner tree / observer graph internally; the
//!   caller does not get a handle to drop. `Effect::new` (Plan 01.0-03) will
//!   therefore be a zero-sized handle that relies on the current Owner scope
//!   for disposal, matching the single-threaded owner-tree model. If we need an
//!   explicit disposable handle later we will revisit (likely by keeping our
//!   own `Rc<Cell<bool>>` shutdown flag consulted from inside the closure).

use std::cell::RefCell;

use send_wrapper::SendWrapper;

/// Wraps a non-`Send` `FnMut` in a `Send + Sync` shell that panics if
/// ever invoked from another thread. Matches our single-threaded promise.
///
/// Used by `Effect::new` in 01.0-03 and potentially `Memo::new` in 01.0-02.
pub(crate) fn wrap_local_fnmut<F: FnMut() + 'static>(f: F) -> impl FnMut() + Send + Sync + 'static {
    let cell = SendWrapper::new(RefCell::new(f));
    move || {
        (cell.borrow_mut())();
    }
}

/// Spike A: confirm `Owner::current()` exists and returns an Option.
#[doc(hidden)]
#[allow(dead_code)]
pub(crate) fn __spike_owner_current_exists() -> bool {
    reactive_graph::owner::Owner::current().is_some()
}

/// Spike B: confirm `wrap_local_fnmut` output satisfies
/// `ImmediateEffect::new_mut_scoped`'s bounds. The call returns `()` in
/// reactive_graph 0.2.13 — we bind it to `_` to prove only the line compiles.
#[doc(hidden)]
#[allow(dead_code)]
pub(crate) fn __spike_immediate_effect_accepts_wrapped_fnmut() {
    let wrapped = wrap_local_fnmut(|| {});
    let _: () = reactive_graph::effect::ImmediateEffect::new_mut_scoped(wrapped);
}
