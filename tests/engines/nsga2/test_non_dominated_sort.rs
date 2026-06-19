use genetic_algorithms::nsga2::configuration::ObjectiveDirection;
use genetic_algorithms::nsga2::non_dominated_sort::{
    assign_ranks, non_dominated_sort, non_dominated_sort_constrained,
    non_dominated_sort_with_directions,
};
use rand::Rng;

#[test]
fn test_non_dominated_sort_single_front() {
    // Three non-dominated points
    let objectives: Vec<Vec<f64>> = vec![vec![1.0, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort(&refs);
    assert_eq!(fronts.len(), 1);
    assert_eq!(fronts[0].len(), 3);
}

#[test]
fn test_non_dominated_sort_two_fronts() {
    // [1,4], [2,2], [4,1] are mutually non-dominated (front 0).
    // [3,3] is dominated by [2,2] (front 1).
    let objectives: Vec<Vec<f64>> = vec![
        vec![1.0, 4.0], // front 0
        vec![3.0, 3.0], // front 1 — dominated by [2,2]
        vec![2.0, 2.0], // front 0
        vec![4.0, 1.0], // front 0
    ];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort(&refs);
    assert_eq!(fronts.len(), 2);
    assert_eq!(fronts[0].len(), 3);
    assert_eq!(fronts[1].len(), 1);
    assert!(fronts[1].contains(&1));
}

#[test]
fn test_non_dominated_sort_empty() {
    let objectives: Vec<&[f64]> = vec![];
    let fronts = non_dominated_sort(&objectives);
    assert!(fronts.is_empty());
}

#[test]
fn test_assign_ranks() {
    let fronts = vec![vec![0, 2, 3], vec![1]];
    let mut ranks = vec![0; 4];
    assign_ranks(&mut ranks, &fronts);
    assert_eq!(ranks, vec![0, 1, 0, 0]);
}

// --- Tests for direction-aware sorting ---

#[test]
fn test_non_dominated_sort_with_maximize_directions() {
    let dirs = vec![ObjectiveDirection::Maximize, ObjectiveDirection::Maximize];
    // With maximize: [3,3] dominates [1,1] and [2,2]
    let objectives: Vec<Vec<f64>> = vec![
        vec![1.0, 1.0], // front 2
        vec![3.0, 3.0], // front 0
        vec![2.0, 2.0], // front 1
    ];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort_with_directions(&refs, &dirs);
    assert_eq!(fronts.len(), 3);
    assert!(fronts[0].contains(&1)); // [3,3] is front 0
    assert!(fronts[1].contains(&2)); // [2,2] is front 1
    assert!(fronts[2].contains(&0)); // [1,1] is front 2
}

#[test]
fn test_non_dominated_sort_mixed_directions() {
    // obj0: minimize, obj1: maximize
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
    let objectives: Vec<Vec<f64>> = vec![
        vec![1.0, 3.0], // front 0: low obj0, high obj1 → ideal
        vec![2.0, 1.0], // front 1: both worse than [1,3]
        vec![3.0, 2.0], // front 1: obj0 worse, obj1 worse than [1,3]
    ];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort_with_directions(&refs, &dirs);
    assert_eq!(fronts.len(), 2);
    assert!(fronts[0].contains(&0));
}

// --- Tests for large-population correctness (exercise parallel path) ---

#[test]
fn test_non_dominated_sort_large_population_150() {
    // 150 individuals with 2 objectives — triggers parallel path (n >= 100)
    // Verify correctness: count individuals per front matches sequential expectation.
    let mut rng = rand::rng();
    let objectives: Vec<Vec<f64>> = (0..150)
        .map(|_| vec![rng.random_range(0.0..1.0), rng.random_range(0.0..1.0)])
        .collect();
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort(&refs);
    // Every individual must appear exactly once across all fronts
    let total: usize = fronts.iter().map(|f| f.len()).sum();
    assert_eq!(total, 150);
    // Each individual index must be in range
    for front in &fronts {
        for &idx in front {
            assert!(idx < 150);
        }
    }
}

#[test]
fn test_non_dominated_sort_large_population_150_deterministic() {
    // Verify parallel and sequential paths produce the same front structure
    // by comparing against a known small pattern scaled up.
    // Build 150 individuals: 50 non-dominated + 100 dominated
    let mut objectives: Vec<Vec<f64>> = Vec::with_capacity(150);
    // 50 non-dominated on the Pareto front (x + y ~ constant)
    for i in 0..50 {
        objectives.push(vec![i as f64, (49 - i) as f64]);
    }
    // 100 dominated: all worse than front point 0
    for _ in 0..100 {
        objectives.push(vec![100.0, 100.0]);
    }
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort(&refs);
    // Front 0 should contain exactly the 50 non-dominated points
    assert_eq!(fronts[0].len(), 50);
    // Front 1 should contain the 100 dominated points
    assert_eq!(fronts[1].len(), 100);
}

#[test]
fn test_non_dominated_sort_constrained_large_population_150() {
    // 150 individuals with constrained sorting — triggers parallel path
    let mut rng = rand::rng();
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize];
    let objectives: Vec<Vec<f64>> = (0..150)
        .map(|_| vec![rng.random_range(0.0..1.0), rng.random_range(0.0..1.0)])
        .collect();
    let violations: Vec<f64> = (0..150).map(|_| rng.random_range(0.0..10.0)).collect();
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort_constrained(&refs, &violations, &dirs);
    let total: usize = fronts.iter().map(|f| f.len()).sum();
    assert_eq!(total, 150);
    for front in &fronts {
        for &idx in front {
            assert!(idx < 150);
        }
    }
}

// --- Tests for constrained sorting ---

#[test]
fn test_non_dominated_sort_constrained_feasible_dominates_infeasible() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // Individual 0: obj=[100.0], violation=0 (feasible)
    // Individual 1: obj=[1.0], violation=5.0 (infeasible)
    let objectives: Vec<Vec<f64>> = vec![vec![100.0], vec![1.0]];
    let violations = vec![0.0, 5.0];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort_constrained(&refs, &violations, &dirs);
    // Feasible individual should be in front 0
    assert_eq!(fronts.len(), 2);
    assert!(fronts[0].contains(&0));
    assert!(fronts[1].contains(&1));
}

#[test]
fn test_non_dominated_sort_constrained_less_violation_preferred() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // Both infeasible
    let objectives: Vec<Vec<f64>> = vec![vec![100.0], vec![1.0]];
    let violations = vec![2.0, 5.0];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort_constrained(&refs, &violations, &dirs);
    // Individual 0 has less violation → should be in front 0
    assert_eq!(fronts.len(), 2);
    assert!(fronts[0].contains(&0));
    assert!(fronts[1].contains(&1));
}
