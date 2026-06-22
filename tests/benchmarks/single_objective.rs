#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::single_objective::{Ackley, Rastrigin, Sphere};
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::BenchmarkFn;

#[cfg(feature = "benchmarks")]
const EPSILON: f64 = 1e-12;

// ── Sphere ──────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_sphere_optimum() {
    let sphere = Sphere::default();
    let result = sphere.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "Sphere optimum expected 0.0, got {}",
        result[0]
    );
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_sphere_nonzero() {
    let sphere = Sphere::new(3);
    let result = sphere.evaluate(&[1.0, 2.0, 3.0]);
    assert!(
        (result[0] - 14.0).abs() < EPSILON,
        "Sphere(1,2,3) expected 14.0, got {}",
        result[0]
    );
}

// ── Rastrigin ───────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_rastrigin_optimum() {
    let rastrigin = Rastrigin::default();
    let result = rastrigin.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "Rastrigin optimum expected 0.0, got {}",
        result[0]
    );
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_rastrigin_nonzero() {
    let rastrigin = Rastrigin::new(1);
    let result = rastrigin.evaluate(&[1.0]);
    assert!(
        (result[0] - 1.0).abs() < EPSILON,
        "Rastrigin(1) expected 1.0, got {}",
        result[0]
    );
}

// ── Ackley ──────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_ackley_optimum() {
    let ackley = Ackley::default();
    let result = ackley.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "Ackley optimum expected 0.0, got {}",
        result[0]
    );
}

// ── Dimension validation ────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
#[should_panic(expected = "Ackley::evaluate called with 2 variables, expected 3")]
fn test_dimension_mismatch() {
    let ackley = Ackley::new(3);
    ackley.evaluate(&[0.0, 0.0]);
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_bounds_length_matches_dimension() {
    let sphere = Sphere::new(5);
    assert_eq!(sphere.bounds().len(), 5);
}

// ── Serde tests (benchmarks + serde) ─────────────────────────────

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_sphere_serde_roundtrip() {
    let s = Sphere::new(10);
    let json = serde_json::to_string(&s).unwrap();
    let s2: Sphere = serde_json::from_str(&json).unwrap();
    assert_eq!(s.n, s2.n);
    assert_eq!(s.bounds().len(), s2.bounds().len());
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_rastrigin_serde_roundtrip() {
    let r = Rastrigin::new(20);
    let json = serde_json::to_string(&r).unwrap();
    let r2: Rastrigin = serde_json::from_str(&json).unwrap();
    assert_eq!(r.n, r2.n);
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_ackley_serde_roundtrip() {
    let a = Ackley::new(15);
    let json = serde_json::to_string(&a).unwrap();
    let a2: Ackley = serde_json::from_str(&json).unwrap();
    assert_eq!(a.n, a2.n);
}
