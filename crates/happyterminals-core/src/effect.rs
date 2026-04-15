//! `Effect` — the side-effect runner over
//! `reactive_graph::effect::ImmediateEffect::new_mut_scoped`.
//!
//! An [`Effect`] re-runs whenever any signal or memo read inside its closure
//! changes. Effects are owned by the current reactive scope: they are
//! disposed automatically when the scope's [`crate::owner::Owner`] is
//! disposed or dropped. Dropping the returned [`Effect`] handle is a no-op
//! on disposal because, in `reactive_graph 0.2.13`, `new_mut_scoped` returns
//! `()` — the effect's lifetime is already plumbed into the owner tree. We
//! model [`Effect`] as a zero-sized marker that carries the
//! `!Send + !Sync` phantom and keeps the public API stable across future
//! `reactive_graph` shape changes (where an opaque handle may reappear).
//!
//! # Cycle detection
//!
//! If the closure (directly or transitively) causes itself to re-enter while
//! still running, `reactive_graph` panics from inside the `FnMut` Mutex
//! `try_lock` — this IS our cycle detection. The first propagation tick is
//! where the panic surfaces; the runtime never stack-overflows.
//!
//! # Threading
//!
//! [`Effect`] is **single-threaded** (`!Send + !Sync`). The user closure may
//! capture non-`Send` data (e.g. `Rc`, `RefCell`) because we route it through
//! an internal `SendWrapper`-based shim (`runtime::wrap_local_fnmut`) that
//! satisfies `ImmediateEffect::new_mut_scoped`'s `FnMut + Send + Sync +
//! 'static` bound while panicking if the closure is ever invoked from another
//! thread — matching our single-threaded promise.

use std::marker::PhantomData;

use reactive_graph::effect::ImmediateEffect;

use crate::runtime::wrap_local_fnmut;

/// A side-effect that re-runs whenever any tracked signal or memo it
/// reads changes.
///
/// # Lifecycle
///
/// Effects run once immediately at construction, and again on each
/// dependency change. They are disposed automatically when their owning
/// [`crate::owner::Owner`] is disposed or dropped; no explicit cleanup is
/// required. [`Effect`] itself is a zero-sized marker — see the module-level
/// doc for the rationale.
///
/// # Cycle detection
///
/// If the closure (directly or transitively) causes itself to re-enter while
/// still running, the runtime panics with `reactive_graph 0.2.13`'s default
/// recursion message. The first propagation tick is where this surfaces —
/// never a stack overflow.
///
/// # Single-threaded
///
/// `Effect` is `!Send + !Sync`. Cross-thread writes queue through
/// [`crate::signal::SignalSetter`] and are drained on the render thread.
pub struct Effect {
    _not_send: PhantomData<*const ()>,
}

impl Effect {
    /// Constructs a new Effect that runs `f` once immediately, then whenever
    /// any signal or memo read inside `f` changes.
    ///
    /// # Panics
    ///
    /// Panics if the effect recursively re-enters itself. See the type-level
    /// documentation on cycle detection.
    #[track_caller]
    #[must_use]
    pub fn new(f: impl FnMut() + 'static) -> Self {
        let wrapped = wrap_local_fnmut(f);
        // `ImmediateEffect::new_mut_scoped` returns `()` in reactive_graph
        // 0.2.13; the effect's disposal is plumbed into the current owner
        // scope via an internal `on_cleanup(move || effect.dispose())` call.
        ImmediateEffect::new_mut_scoped(wrapped);
        Self {
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod smoke {
    //! In-file smoke test: Effect re-runs exactly twice when a dependency
    //! changes once. Validates REACT-03 wiring end-to-end within Plan
    //! 01.0-03's own gate. Formal integration tests (`tests/diamond.rs`,
    //! `tests/disposal.rs`, etc.) are owned by Plan 01.0-05.

    use std::cell::Cell;
    use std::rc::Rc;

    use crate::effect::Effect;
    use crate::owner::create_root;
    use crate::signal::Signal;

    #[test]
    fn smoke_effect_reruns_twice_on_one_dep_change() {
        let runs = Rc::new(Cell::new(0u32));
        let runs_e = Rc::clone(&runs);

        let (handles, owner) = create_root(|| {
            let s = Signal::new(0i32);
            let s_clone = s.clone();
            // The Effect handle is a ZST; disposal is plumbed through the
            // current owner's cleanup list by `new_mut_scoped` internally,
            // so returning it alongside `s` is purely decorative — the
            // effect will fire on `s.set(...)` regardless of whether we keep
            // the handle. We still return it to mirror the intended user
            // pattern (hold the handle for the duration of the scope).
            let eff = Effect::new(move || {
                let _ = s_clone.get();
                runs_e.set(runs_e.get() + 1);
            });
            (s, eff)
        });

        let (s, _eff) = handles;

        assert_eq!(
            runs.get(),
            1,
            "Effect did not run once at construction (got {})",
            runs.get()
        );

        // Single dependency change → exactly one propagation re-run.
        s.set(42);

        assert_eq!(
            runs.get(),
            2,
            "Effect did not re-run exactly once after a single set() \
             (expected 2 total runs, got {})",
            runs.get()
        );

        owner.dispose();
    }
}
