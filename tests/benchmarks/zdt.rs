#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::zdt::{
    ZDT1, ZDT2, ZDT3, ZDT4, ZDT5, ZDT6,
};
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::BenchmarkFn;

#[cfg(feature = "benchmarks")]
const EPSILON: f64 = 1e-12;

// ── ZDT1 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt1_optimum() {
    let zdt1 = ZDT1::default();
    let result = zdt1.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "ZDT1 f1 expected 0, got {}",
        result[0]
    );
    assert!(
        (result[1] - 1.0).abs() < EPSILON,
        "ZDT1 f2 expected 1, got {}",
        result[1]
    );
}

// ── ZDT2 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt2_optimum() {
    let zdt2 = ZDT2::default();
    let result = zdt2.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "ZDT2 f1 expected 0, got {}",
        result[0]
    );
    assert!(
        (result[1] - 1.0).abs() < EPSILON,
        "ZDT2 f2 expected 1, got {}",
        result[1]
    );
}

// ── ZDT3 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt3_optimum() {
    let zdt3 = ZDT3::default();
    let result = zdt3.evaluate(&[0.0; 30]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "ZDT3 f1 expected 0, got {}",
        result[0]
    );
    assert!(
        (result[1] - 1.0).abs() < EPSILON,
        "ZDT3 f2 expected 1, got {}",
        result[1]
    );
}

// ── ZDT4 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt4_optimum() {
    let zdt4 = ZDT4::default();
    let result = zdt4.evaluate(&[0.0; 10]);
    assert!(
        (result[0] - 0.0).abs() < EPSILON,
        "ZDT4 f1 expected 0, got {}",
        result[0]
    );
    assert!(
        (result[1] - 1.0).abs() < EPSILON,
        "ZDT4 f2 expected 1, got {}",
        result[1]
    );
}

// ── ZDT5 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt5_optimum() {
    let zdt5 = ZDT5::default();
    let result = zdt5.evaluate(&[0.0; 11]);
    assert!(
        (result[0] - 2.0).abs() < EPSILON,
        "ZDT5 f1 expected 2, got {}",
        result[0]
    );
    assert!(
        (result[1] - 5.5).abs() < EPSILON,
        "ZDT5 f2 expected 5.5, got {}",
        result[1]
    );
}

// ── ZDT6 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt6_optimum() {
    let zdt6 = ZDT6::default();
    let result = zdt6.evaluate(&[0.0; 10]);
    assert!(
        (result[0] - 1.0).abs() < EPSILON,
        "ZDT6 f1 expected 1, got {}",
        result[0]
    );
    assert!(
        (result[1] - 0.0).abs() < EPSILON,
        "ZDT6 f2 expected 0, got {}",
        result[1]
    );
}

// ── Dimension mismatch ──────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
#[should_panic(expected = "ZDT1::evaluate called with 2 variables, expected 3")]
fn test_zdt_dimension_mismatch() {
    let zdt1 = ZDT1::new(3);
    zdt1.evaluate(&[0.0, 0.0]);
}

// ── Bounds ──────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt4_bounds_length() {
    let zdt4 = ZDT4::new(10);
    assert_eq!(zdt4.bounds().len(), 10);
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt4_bounds_mixed() {
    let zdt4 = ZDT4::new(10);
    assert_eq!(zdt4.bounds()[0], (0.0, 1.0));
    assert_eq!(zdt4.bounds()[1], (-5.0, 5.0));
    assert_eq!(zdt4.bounds()[9], (-5.0, 5.0));
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_zdt_zdt5_default_dimension() {
    let zdt5 = ZDT5::default();
    assert_eq!(zdt5.bounds().len(), 11);
}

// ── Serde tests (benchmarks + serde) ─────────────────────────────

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_zdt1_serde_roundtrip() {
    let z = ZDT1::new(30);
    let json = serde_json::to_string(&z).unwrap();
    let z2: ZDT1 = serde_json::from_str(&json).unwrap();
    assert_eq!(z.bounds().len(), z2.bounds().len());
    let x = vec![0.0; 30];
    assert_eq!(z.evaluate(&x), z2.evaluate(&x));
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_zdt4_serde_roundtrip() {
    let z = ZDT4::new(10);
    let json = serde_json::to_string(&z).unwrap();
    let z2: ZDT4 = serde_json::from_str(&json).unwrap();
    assert_eq!(z.bounds()[0], (0.0, 1.0));
    assert_eq!(z2.bounds()[0], z.bounds()[0]);
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_zdt6_serde_roundtrip() {
    let z = ZDT6::new(10);
    let json = serde_json::to_string(&z).unwrap();
    let z2: ZDT6 = serde_json::from_str(&json).unwrap();
    assert_eq!(z.bounds().len(), z2.bounds().len());
    let x = vec![0.0; 10];
    assert_eq!(z.evaluate(&x), z2.evaluate(&x));
}
