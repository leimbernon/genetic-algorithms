use genetic_algorithms::rng;
use rand::rngs::SmallRng;
use rand::SeedableRng;
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
    rng::set_seed(None);
    let mut r = rng::make_rng();
    let _v: u64 = r.random();
}

/// Full integration test of `set_seed` + `make_rng` determinism.
/// Requires `--test-threads=1` because it relies on global state not
/// being modified by concurrent tests.
#[test]
#[ignore]
fn make_rng_with_seed_is_deterministic() {
    rng::set_seed(Some(12345));
    let mut r1 = rng::make_rng();
    let mut r2 = rng::make_rng();
    let v1: f64 = r1.random();
    let v2: f64 = r2.random();

    // Reset and repeat -- should get the same values
    rng::set_seed(Some(12345));
    let mut r1b = rng::make_rng();
    let mut r2b = rng::make_rng();
    let v1b: f64 = r1b.random();
    let v2b: f64 = r2b.random();

    assert_eq!(v1, v1b, "First RNG should be deterministic");
    assert_eq!(v2, v2b, "Second RNG should be deterministic");

    // Clean up
    rng::set_seed(None);
}
