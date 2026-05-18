use genetic_algorithms::nsga3::das_dennis::generate_das_dennis;

#[test]
fn test_das_dennis_m3_p2() {
    // C(2 + 3 - 1, 3 - 1) = C(4, 2) = 6 points
    let pts = generate_das_dennis(3, 2);
    assert_eq!(pts.len(), 6);
    for pt in &pts {
        assert_eq!(pt.len(), 3);
        let sum: f64 = pt.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "point sum was {}, expected 1.0", sum);
    }
}

#[test]
fn test_das_dennis_m3_p4() {
    // C(4 + 3 - 1, 3 - 1) = C(6, 2) = 15 points
    let pts = generate_das_dennis(3, 4);
    assert_eq!(pts.len(), 15);
    for pt in &pts {
        assert_eq!(pt.len(), 3);
        let sum: f64 = pt.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_das_dennis_m5_p6() {
    // C(6 + 5 - 1, 5 - 1) = C(10, 4) = 210 points
    let pts = generate_das_dennis(5, 6);
    assert_eq!(pts.len(), 210);
    for pt in &pts {
        assert_eq!(pt.len(), 5);
    }
}

#[test]
fn test_das_dennis_components_non_negative() {
    let pts = generate_das_dennis(4, 5);
    for pt in &pts {
        for &v in pt {
            assert!(v >= 0.0);
            assert!(v <= 1.0);
        }
    }
}

#[test]
fn test_das_dennis_m1_returns_single_point() {
    let pts = generate_das_dennis(1, 5);
    assert_eq!(pts.len(), 1);
    assert_eq!(pts[0], vec![1.0]);
}

#[test]
fn test_das_dennis_m0_returns_empty() {
    let pts = generate_das_dennis(0, 4);
    assert!(pts.is_empty());
}
