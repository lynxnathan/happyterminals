//! `batch()` — coalesce multiple signal writes into a single propagation.
//!
//! Direct forward to `reactive_graph::effect::batch`. Only affects
//! synchronous effects (`ImmediateEffect` internally, which is what our
//! `Effect::new` in 01.0-03 will build on). Has no effect on user-provided
//! async work.

use reactive_graph::effect::batch as rg_batch;

/// Coalesces multiple signal writes into a single propagation pass.
///
/// Dependents of any signals written inside the closure run **once** after
/// the closure returns, with the combined new state — not once per write.
///
/// ```ignore
/// use happyterminals_core::signal::Signal;
/// use happyterminals_core::batch::batch;
///
/// let a = Signal::new(0);
/// let b = Signal::new(0);
/// batch(|| {
///     a.set(1);
///     b.set(2);
/// });
/// // Effects that read both a and b re-run exactly ONCE here.
/// ```
///
/// `batch` is transparent to the closure's return value: `R` is unbounded.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    rg_batch(f)
}
