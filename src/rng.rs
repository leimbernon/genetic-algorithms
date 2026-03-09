//! Seedable random number generation for reproducible GA runs.
//!
//! This module provides a thread-local RNG seeding mechanism. When a seed is set
//! via [`set_seed`], subsequent calls to [`make_rng`] produce deterministic
//! [`SmallRng`] instances derived from that seed. When no seed is set, each call
//! returns a randomly-seeded `SmallRng` (equivalent to the previous `rand::rng()`
//! behaviour).
//!
//! # Reproducibility
//!
//! Two single-threaded runs with the same seed will produce identical results. In
//! multi-threaded (rayon) contexts, reproducibility additionally requires
//! deterministic work partitioning — typically achieved by fixing the rayon thread
//! pool size.
//!
//! # Usage
//!
//! ```ignore
//! use genetic_algorithms::rng;
//!
//! rng::set_seed(Some(42));      // enable deterministic mode
//! let mut r = rng::make_rng();  // SmallRng seeded from 42 + counter
//!
//! rng::set_seed(None);          // revert to random seeding
//! ```

use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

/// Global seed value. Negative means "no seed" (entropy-based).
/// Non-negative values are interpreted as `u64` seeds.
static SEED: AtomicI64 = AtomicI64::new(-1);

/// Global monotonic counter used to derive unique but deterministic seeds
/// from the base seed. Each call to [`make_rng`] increments this counter.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Sets the RNG seed for the current thread.
///
/// - `Some(seed)`: subsequent [`make_rng`] calls on this thread return
///   deterministic `SmallRng` instances derived from `seed`.
/// - `None`: revert to entropy-based seeding (non-deterministic).
///
/// Also resets the global counter to zero so that repeated runs with the
/// same seed produce the same sequence.
pub fn set_seed(seed: Option<u64>) {
    match seed {
        Some(s) => {
            SEED.store(s as i64, Ordering::SeqCst);
            COUNTER.store(0, Ordering::SeqCst);
        }
        None => {
            SEED.store(-1, Ordering::SeqCst);
        }
    }
}

/// Creates a new [`SmallRng`].
///
/// When a seed has been set via [`set_seed`], the returned RNG is seeded
/// deterministically from `seed ⊕ counter` (where `counter` is a global
/// monotonically increasing value). This guarantees that every call site gets
/// a unique but reproducible stream.
///
/// When no seed is set, the RNG is seeded from operating-system entropy
/// (equivalent to the default `rand::rng()` behaviour).
pub fn make_rng() -> SmallRng {
    let raw = SEED.load(Ordering::SeqCst);
    if raw >= 0 {
        let base_seed = raw as u64;
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        // Combine base seed with counter via a simple mixing function
        // to avoid correlated streams.
        let combined = base_seed
            .wrapping_add(n)
            .wrapping_mul(6_364_136_223_846_793_005);
        SmallRng::seed_from_u64(combined)
    } else {
        SmallRng::from_os_rng()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    /// Verifies that `SmallRng::seed_from_u64` with the same seed always
    /// produces the same first random value. This tests our deterministic
    /// RNG construction logic in isolation, without touching global state.
    #[test]
    fn seed_from_u64_is_deterministic() {
        let seed = 42u64;
        let combined = seed.wrapping_add(0).wrapping_mul(6_364_136_223_846_793_005);

        let mut r1 = SmallRng::seed_from_u64(combined);
        let mut r2 = SmallRng::seed_from_u64(combined);
        let v1: f64 = r1.random();
        let v2: f64 = r2.random();

        assert_eq!(v1, v2, "Same combined seed should produce identical values");
    }

    /// Verifies that different counter values produce different RNG streams.
    #[test]
    fn different_counters_produce_different_streams() {
        let seed = 42u64;
        let combined_0 = seed.wrapping_add(0).wrapping_mul(6_364_136_223_846_793_005);
        let combined_1 = seed.wrapping_add(1).wrapping_mul(6_364_136_223_846_793_005);

        let mut r1 = SmallRng::seed_from_u64(combined_0);
        let mut r2 = SmallRng::seed_from_u64(combined_1);
        let v1: u64 = r1.random();
        let v2: u64 = r2.random();

        assert_ne!(
            v1, v2,
            "Different counters should produce different streams"
        );
    }

    /// Verifies that different base seeds produce different RNG streams.
    #[test]
    fn different_seeds_produce_different_values() {
        let combined_a = 42u64
            .wrapping_add(0)
            .wrapping_mul(6_364_136_223_846_793_005);
        let combined_b = 99u64
            .wrapping_add(0)
            .wrapping_mul(6_364_136_223_846_793_005);

        let mut r1 = SmallRng::seed_from_u64(combined_a);
        let mut r2 = SmallRng::seed_from_u64(combined_b);
        let v1: u64 = r1.random();
        let v2: u64 = r2.random();

        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    /// Verifies that `set_seed(None)` causes `make_rng` to return an
    /// entropy-seeded RNG (non-deterministic path). We can only test that it
    /// doesn't panic; entropy values are inherently unpredictable.
    #[test]
    fn make_rng_without_seed_does_not_panic() {
        // Just verify the entropy path works — save/restore the seed so
        // we don't disturb any other test's deterministic run.
        let prev_seed = SEED.load(Ordering::SeqCst);
        let prev_counter = COUNTER.load(Ordering::SeqCst);

        SEED.store(-1, Ordering::SeqCst);
        let mut r = make_rng();
        let _v: u64 = r.random();

        // Restore previous state
        SEED.store(prev_seed, Ordering::SeqCst);
        COUNTER.store(prev_counter, Ordering::SeqCst);
    }

    /// Full integration test of `set_seed` + `make_rng` determinism.
    /// Requires `--test-threads=1` because it relies on global state not
    /// being modified by concurrent tests.
    #[test]
    #[ignore]
    fn make_rng_with_seed_is_deterministic() {
        set_seed(Some(12345));
        let mut r1 = make_rng();
        let mut r2 = make_rng();
        let v1: f64 = r1.random();
        let v2: f64 = r2.random();

        // Reset and repeat — should get the same values
        set_seed(Some(12345));
        let mut r1b = make_rng();
        let mut r2b = make_rng();
        let v1b: f64 = r1b.random();
        let v2b: f64 = r2b.random();

        assert_eq!(v1, v1b, "First RNG should be deterministic");
        assert_eq!(v2, v2b, "Second RNG should be deterministic");

        // Clean up
        set_seed(None);
    }
}
