//! Injectable monotonic-clock abstraction.
//!
//! Production code uses [`SystemClock`]. Tests use `test_util::ManualClock`
//! (available with the `test-util` feature).
//!
//! Clocks are passed as arguments — never global state. This keeps
//! snapshot/property tests deterministic.

use std::time::Instant;

/// Injectable monotonic-clock abstraction.
pub trait Clock: 'static {
    /// Returns the current moment, monotonic.
    fn now(&self) -> Instant;
}

/// Production Clock — wraps [`std::time::Instant::now`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(feature = "test-util")]
pub mod test_util {
    //! Test-only clock utilities. Gated behind the `test-util` feature so
    //! production builds can't accidentally consume them.

    use super::Clock;
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    /// A test-only clock. Time advances only via explicit [`advance`][Self::advance].
    ///
    /// Uses interior mutability (`Cell<Instant>`) so tests can hold `&dyn Clock`
    /// without needing mut references — [`advance`][Self::advance] takes `&self`.
    pub struct ManualClock {
        now: Cell<Instant>,
    }

    impl ManualClock {
        /// Starts the clock at `start`.
        #[must_use]
        pub fn new(start: Instant) -> Self {
            Self {
                now: Cell::new(start),
            }
        }

        /// Advances the clock by `d`. Idempotent with respect to ordering
        /// (repeated calls accumulate).
        pub fn advance(&self, d: Duration) {
            self.now.set(self.now.get() + d);
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Instant {
            self.now.get()
        }
    }
}
