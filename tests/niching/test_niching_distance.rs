use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::genotypes::Range;
use genetic_algorithms::niching::distance::{
    euclidean_distance, hamming_distance, DistanceMetric, EuclideanDistance, HammingDistance,
};

#[test]
fn test_hamming_distance_identical() {
    let a = vec![
        Binary { id: 0, value: true },
        Binary {
            id: 1,
            value: false,
        },
        Binary { id: 2, value: true },
    ];
    let b = a.clone();
    assert!((hamming_distance(&a, &b) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_hamming_distance_all_different() {
    let a = vec![
        Binary { id: 0, value: true },
        Binary {
            id: 1,
            value: false,
        },
        Binary { id: 2, value: true },
    ];
    let b = vec![
        Binary {
            id: 0,
            value: false,
        },
        Binary { id: 1, value: true },
        Binary {
            id: 2,
            value: false,
        },
    ];
    assert!((hamming_distance(&a, &b) - 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_hamming_distance_different_lengths() {
    let a = vec![
        Binary { id: 0, value: true },
        Binary {
            id: 1,
            value: false,
        },
    ];
    let b = vec![
        Binary { id: 0, value: true },
        Binary {
            id: 1,
            value: false,
        },
        Binary { id: 2, value: true },
    ];
    // Third position: a has None, b has true -> different
    assert!((hamming_distance(&a, &b) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_euclidean_distance_basic() {
    let a = vec![
        Range::new(0, vec![(0, 10)], 3),
        Range::new(1, vec![(0, 10)], 4),
    ];
    let b = vec![
        Range::new(0, vec![(0, 10)], 0),
        Range::new(1, vec![(0, 10)], 0),
    ];
    // sqrt(9 + 16) = 5
    assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-10);
}

#[test]
fn test_euclidean_distance_identical() {
    let a = vec![
        Range::new(0, vec![(0, 10)], 5),
        Range::new(1, vec![(0, 10)], 5),
    ];
    let b = a.clone();
    assert!((euclidean_distance(&a, &b) - 0.0).abs() < 1e-10);
}

#[test]
fn test_distance_metric_trait_hamming() {
    let a = vec![
        Binary { id: 0, value: true },
        Binary {
            id: 1,
            value: false,
        },
    ];
    let b = vec![
        Binary {
            id: 0,
            value: false,
        },
        Binary {
            id: 1,
            value: false,
        },
    ];
    let d = HammingDistance::distance(&a, &b);
    assert!((d - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_distance_metric_trait_euclidean() {
    let a = vec![Range::new(0, vec![(0, 10)], 1i32)];
    let b = vec![Range::new(0, vec![(0, 10)], 4i32)];
    let d = EuclideanDistance::distance(&a, &b);
    assert!((d - 3.0).abs() < 1e-10);
}
