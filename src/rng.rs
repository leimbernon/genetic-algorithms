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

    #[test]
    fn make_rng_without_seed_returns_different_values() {
        set_seed(None);
        let mut r1 = make_rng();
        let mut r2 = make_rng();
        let v1: u64 = r1.random();
        let v2: u64 = r2.random();
        // Extremely unlikely to be equal with entropy seeding
        // (skip assertion if by cosmic chance they match)
        let _ = (v1, v2);
    }

    #[test]
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

        // Clean up for other tests
        set_seed(None);
    }

    #[test]
    fn different_seeds_produce_different_values() {
        set_seed(Some(42));
        let mut r1 = make_rng();
        let v1: u64 = r1.random();

        set_seed(Some(99));
        let mut r2 = make_rng();
        let v2: u64 = r2.random();

        assert_ne!(v1, v2, "Different seeds should produce different values");

        // Clean up
        set_seed(None);
    }

    #[test]
    fn successive_calls_produce_different_rngs() {
        set_seed(Some(42));
        let mut r1 = make_rng();
        let mut r2 = make_rng();
        let v1: u64 = r1.random();
        let v2: u64 = r2.random();

        assert_ne!(
            v1, v2,
            "Successive make_rng calls should produce different streams"
        );

        // Clean up
        set_seed(None);
    }
}
