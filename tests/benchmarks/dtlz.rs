#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::dtlz::{
    DTLZ1, DTLZ2, DTLZ3, DTLZ4, DTLZ5, DTLZ6, DTLZ7,
};
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::BenchmarkFn;

#[cfg(feature = "benchmarks")]
const EPSILON: f64 = 1e-12;

// ── DTLZ1 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz1_form() {
    let dtlz1 = DTLZ1::new(7, 3);
    let result = dtlz1.evaluate(&[0.0; 7]);
    assert_eq!(result.len(), 3);
    assert!(result[2] > 0.0, "DTLZ1 f3 should be > 0");
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz1_uniform_optimum() {
    let dtlz1 = DTLZ1::new(7, 3);
    let result = dtlz1.evaluate(&[0.5; 7]);
    assert!(
        (result[0] - 0.125).abs() < EPSILON,
        "DTLZ1 f1 expected 0.125, got {}",
        result[0]
    );
    assert!(
        (result[1] - 0.125).abs() < EPSILON,
        "DTLZ1 f2 expected 0.125, got {}",
        result[1]
    );
    assert!(
        (result[2] - 0.25).abs() < EPSILON,
        "DTLZ1 f3 expected 0.25, got {}",
        result[2]
    );
}

// ── DTLZ2 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz2_sphere_surface() {
    let dtlz2 = DTLZ2::new(12, 3);
    let result = dtlz2.evaluate(&[0.5; 12]);
    let sum_sq: f64 = result.iter().map(|&fi| fi * fi).sum();
    assert!(
        (sum_sq - 1.0).abs() < 1e-10,
        "DTLZ2 sum of squares expected 1.0, got {}",
        sum_sq
    );
}

// ── DTLZ3 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz3_uniform_optimum() {
    let dtlz3 = DTLZ3::new(12, 3);
    let result = dtlz3.evaluate(&[0.5; 12]);
    let sum_sq: f64 = result.iter().map(|&fi| fi * fi).sum();
    assert!(
        (sum_sq - 1.0).abs() < 1e-10,
        "DTLZ3 sum of squares expected 1.0, got {}",
        sum_sq
    );
}

// ── DTLZ4 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz4_default_alpha() {
    let dtlz4 = DTLZ4::new(12, 3);
    let result = dtlz4.evaluate(&[0.5; 12]);
    assert_eq!(result.len(), 3);
    assert!(
        result[2].abs() < 1e-10,
        "DTLZ4 f3 should be near 0 with alpha=100, got {}",
        result[2]
    );
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz4_alpha_one() {
    let dtlz4 = DTLZ4::with_alpha(12, 3, 1.0);
    let dtlz2 = DTLZ2::new(12, 3);
    let result4 = dtlz4.evaluate(&[0.3; 12]);
    let result2 = dtlz2.evaluate(&[0.3; 12]);
    for i in 0..3 {
        assert!(
            (result4[i] - result2[i]).abs() < EPSILON,
            "DTLZ4(alpha=1) and DTLZ2 differ at f{}: {} vs {}",
            i,
            result4[i],
            result2[i]
        );
    }
}

// ── DTLZ5 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz5_uniform_optimum() {
    let dtlz5 = DTLZ5::new(12, 3);
    let result = dtlz5.evaluate(&[0.5; 12]);
    assert_eq!(result.len(), 3);
    assert!(result[0] > 0.0, "DTLZ5 f1 should be positive");
    assert!(
        result[2] > result[1],
        "DTLZ5 f3 should be > f2 with uniform input"
    );
}

// ── DTLZ6 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz6_zero_g() {
    let dtlz6 = DTLZ6::new(12, 3);
    let result = dtlz6.evaluate(&[0.5; 12]);
    assert_eq!(result.len(), 3);
    assert!(
        result.iter().all(|&v| v >= 0.0),
        "DTLZ6 all values should be non-negative"
    );
}

// ── DTLZ7 ────────────────────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz7_return_length() {
    let dtlz7 = DTLZ7::new(22, 3);
    let result = dtlz7.evaluate(&[0.5; 22]);
    assert_eq!(result.len(), 3);
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz7_first_two_linear() {
    let dtlz7 = DTLZ7::new(22, 3);
    let x = [
        0.2_f64, 0.7, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
        0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
    ];
    let result = dtlz7.evaluate(&x);
    assert!(
        (result[0] - 0.2).abs() < EPSILON,
        "DTLZ7 f1 expected 0.2, got {}",
        result[0]
    );
    assert!(
        (result[1] - 0.7).abs() < EPSILON,
        "DTLZ7 f2 expected 0.7, got {}",
        result[1]
    );
}

// ── Dimension validation ─────────────────────────────────────────

#[cfg(feature = "benchmarks")]
#[test]
#[should_panic(expected = "DTLZ2::evaluate called with 11 variables, expected 12")]
fn test_dtlz_dimension_mismatch() {
    let dtlz2 = DTLZ2::new(12, 3);
    dtlz2.evaluate(&[0.0; 11]);
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz_bounds_length() {
    let dtlz2 = DTLZ2::new(12, 3);
    assert_eq!(dtlz2.bounds().len(), 12);
}

#[cfg(feature = "benchmarks")]
#[test]
fn test_all_dtlz_bound_01() {
    for name in &[
        "DTLZ1", "DTLZ2", "DTLZ3", "DTLZ4", "DTLZ5", "DTLZ6", "DTLZ7",
    ] {
        let d: Box<dyn BenchmarkFn> = match *name {
            "DTLZ1" => Box::new(DTLZ1::new(10, 3)),
            "DTLZ2" => Box::new(DTLZ2::new(10, 3)),
            "DTLZ3" => Box::new(DTLZ3::new(10, 3)),
            "DTLZ4" => Box::new(DTLZ4::new(10, 3)),
            "DTLZ5" => Box::new(DTLZ5::new(10, 3)),
            "DTLZ6" => Box::new(DTLZ6::new(10, 3)),
            "DTLZ7" => Box::new(DTLZ7::new(10, 3)),
            _ => unreachable!(),
        };
        for (j, &(low, high)) in d.bounds().iter().enumerate() {
            assert!(
                (low - 0.0).abs() < EPSILON,
                "{} bound[{}].0 expected 0, got {}",
                name,
                j,
                low
            );
            assert!(
                (high - 1.0).abs() < EPSILON,
                "{} bound[{}].1 expected 1, got {}",
                name,
                j,
                high
            );
        }
    }
}

// ── Serde tests (benchmarks + serde) ─────────────────────────────

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz1_serde_roundtrip() {
    let d = DTLZ1::new(7, 3);
    let json = serde_json::to_string(&d).unwrap();
    let d2: DTLZ1 = serde_json::from_str(&json).unwrap();
    assert_eq!(d.bounds().len(), d2.bounds().len());
    let x = vec![0.5; 7];
    assert_eq!(d.evaluate(&x), d2.evaluate(&x));
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz2_serde_roundtrip() {
    let d = DTLZ2::new(12, 3);
    let json = serde_json::to_string(&d).unwrap();
    let d2: DTLZ2 = serde_json::from_str(&json).unwrap();
    assert_eq!(d.bounds().len(), d2.bounds().len());
    let x = vec![0.5; 12];
    assert_eq!(d.evaluate(&x), d2.evaluate(&x));
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz4_serde_roundtrip() {
    let d = DTLZ4::new(10, 3);
    let json = serde_json::to_string(&d).unwrap();
    let d2: DTLZ4 = serde_json::from_str(&json).unwrap();
    let x = vec![0.3; 10];
    assert_eq!(d.evaluate(&x), d2.evaluate(&x));
}

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz7_serde_roundtrip() {
    let d = DTLZ7::new(22, 3);
    let json = serde_json::to_string(&d).unwrap();
    let d2: DTLZ7 = serde_json::from_str(&json).unwrap();
    assert_eq!(d.bounds().len(), d2.bounds().len());
    let x = vec![0.5; 22];
    assert_eq!(d.evaluate(&x), d2.evaluate(&x));
}
