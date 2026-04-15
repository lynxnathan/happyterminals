//! Injectable RNG abstraction.
//!
//! The minimal API (`gen_u64`, `gen_f32`) keeps downstream crates free
//! of any `rand` semver dependency. Production uses [`ThreadRng`]
//! (rand 0.9's thread-local generator); tests use
//! `test_util::SeededRng` (rand's `SmallRng` seeded from a `u64`).

// `Rng as RandRng` is needed for the `.random::<T>()` extension method to
// resolve on `rand::rngs::ThreadRng` / `SmallRng`. Clippy's
// `unused_imports` lint can false-flag it because the trait is never
// named — only its methods are invoked. Do NOT remove `RandRng`.
use rand::rngs::ThreadRng as RandThreadRng;
#[allow(unused_imports)]
use rand::{Rng as RandRng, RngCore, rng};

/// Injectable RNG abstraction.
pub trait Rng: 'static {
    /// Returns a uniform random `u64`.
    fn gen_u64(&mut self) -> u64;

    /// Returns a uniform random `f32` in `[0, 1)`.
    fn gen_f32(&mut self) -> f32;
}

/// Production RNG — wraps [`rand::rng`] (rand 0.9's thread-local generator).
///
/// `!Send` (inherited from `rand::rngs::ThreadRng`).
pub struct ThreadRng {
    inner: RandThreadRng,
}

impl Default for ThreadRng {
    fn default() -> Self {
        Self { inner: rng() }
    }
}

impl Rng for ThreadRng {
    fn gen_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn gen_f32(&mut self) -> f32 {
        self.inner.random::<f32>()
    }
}

#[cfg(feature = "test-util")]
pub mod test_util {
    //! Test-only RNG utilities. Gated behind the `test-util` feature so
    //! production builds can't accidentally consume them.

    use rand::rngs::SmallRng;
    // Same `unused_imports` preemption rationale as the parent module —
    // `Rng as RandRng` brings the `.random::<T>()` method into scope.
    #[allow(unused_imports)]
    use rand::{Rng as RandRng, RngCore, SeedableRng};

    use super::Rng;

    /// Seeded deterministic RNG for snapshot-stable tests.
    ///
    /// Wraps rand's `SmallRng`. Byte-deterministic across runs for a given seed.
    pub struct SeededRng {
        inner: SmallRng,
    }

    impl SeededRng {
        /// Constructs a new `SeededRng` from a `u64` seed.
        #[must_use]
        pub fn new(seed: u64) -> Self {
            Self {
                inner: SmallRng::seed_from_u64(seed),
            }
        }
    }

    impl Rng for SeededRng {
        fn gen_u64(&mut self) -> u64 {
            self.inner.next_u64()
        }

        fn gen_f32(&mut self) -> f32 {
            self.inner.random::<f32>()
        }
    }
}
