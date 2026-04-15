//! Crate error types.
//!
//! Most runtime-path errors (cycle detection, `on_cleanup` outside scope)
//! panic rather than returning a Result — they are programmer errors, not
//! recoverable conditions. This enum is a placeholder for future fallible
//! surface area; as of v0.0.0 only `NotInitialized` exists.

use thiserror::Error;

/// Errors that can surface from the reactive core.
#[derive(Error, Debug)]
pub enum CoreError {
    /// The reactive runtime's thread-local state is not initialized on
    /// the current thread. Typically caused by calling reactive API from
    /// outside the render thread's scope.
    ///
    /// Placeholder variant: not constructed in v0.0.0. Future fallible
    /// surface area will populate this enum; `#[allow(dead_code)]` keeps
    /// the crate compatible with a project-wide `deny(dead_code)` upgrade.
    #[allow(dead_code)]
    #[error("reactive runtime is not initialized on this thread")]
    NotInitialized,
}

// Compile-time assertion: CoreError is Send + Sync (required for cross-boundary
// use at the Python binding layer in M4).
#[allow(dead_code)]
fn _assert_core_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CoreError>();
}
