use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::nsga2::configuration::ObjectiveDirection;
use genetic_algorithms::nsga2::pareto::{
    constrained_dominates, dominates, dominates_with_directions, ParetoFront, ParetoIndividual,
};

#[test]
fn test_dominates_clear() {
    assert!(dominates(&[1.0, 1.0], &[2.0, 2.0]));
}

#[test]
fn test_dominates_equal() {
    assert!(!dominates(&[1.0, 1.0], &[1.0, 1.0]));
}

#[test]
fn test_dominates_partial() {
    // a is better on first, equal on second
    assert!(dominates(&[1.0, 2.0], &[2.0, 2.0]));
}

#[test]
fn test_dominates_incomparable() {
    // Neither dominates: a better on first, worse on second
    assert!(!dominates(&[1.0, 3.0], &[2.0, 2.0]));
}

#[test]
fn test_dominates_reversed() {
    assert!(!dominates(&[2.0, 2.0], &[1.0, 1.0]));
}

#[test]
fn test_pareto_front_len() {
    let front: ParetoFront<Binary> = ParetoFront::new(vec![]);
    assert_eq!(front.len(), 0);
    assert!(front.is_empty());
}

// --- Tests for dominates_with_directions ---

#[test]
fn test_dominates_with_directions_all_minimize() {
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize];
    assert!(dominates_with_directions(&[1.0, 1.0], &[2.0, 2.0], &dirs));
    assert!(!dominates_with_directions(&[2.0, 2.0], &[1.0, 1.0], &dirs));
}

#[test]
fn test_dominates_with_directions_all_maximize() {
    let dirs = vec![ObjectiveDirection::Maximize, ObjectiveDirection::Maximize];
    // Higher is better for maximize
    assert!(dominates_with_directions(&[3.0, 3.0], &[1.0, 1.0], &dirs));
    assert!(!dominates_with_directions(&[1.0, 1.0], &[3.0, 3.0], &dirs));
}

#[test]
fn test_dominates_with_directions_mixed() {
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
    // a = [1.0, 3.0]: obj0 minimize → 1 < 2 (better), obj1 maximize → 3 > 1 (better)
    assert!(dominates_with_directions(&[1.0, 3.0], &[2.0, 1.0], &dirs));
    // a = [2.0, 1.0]: obj0 minimize → 2 > 1 (worse)
    assert!(!dominates_with_directions(&[2.0, 1.0], &[1.0, 3.0], &dirs));
}

#[test]
fn test_dominates_with_directions_incomparable_mixed() {
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
    // a = [1.0, 1.0], b = [2.0, 3.0]
    // obj0: 1 < 2 (a better), obj1: 1 < 3 (b better since maximize)
    assert!(!dominates_with_directions(&[1.0, 1.0], &[2.0, 3.0], &dirs));
    assert!(!dominates_with_directions(&[2.0, 3.0], &[1.0, 1.0], &dirs));
}

#[test]
fn test_dominates_with_empty_directions_defaults_to_minimize() {
    // Empty directions should behave like all-minimize
    assert!(dominates_with_directions(&[1.0, 1.0], &[2.0, 2.0], &[]));
    assert!(!dominates_with_directions(&[2.0, 2.0], &[1.0, 1.0], &[]));
}

// --- Tests for constrained_dominates ---

#[test]
fn test_constrained_dominates_both_feasible() {
    let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize];
    // Both feasible → standard dominance
    assert!(constrained_dominates(
        &[1.0, 1.0],
        &[2.0, 2.0],
        0.0,
        0.0,
        &dirs
    ));
    assert!(!constrained_dominates(
        &[2.0, 2.0],
        &[1.0, 1.0],
        0.0,
        0.0,
        &dirs
    ));
}

#[test]
fn test_constrained_dominates_feasible_beats_infeasible() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // a feasible, b infeasible → a dominates regardless of objectives
    assert!(constrained_dominates(&[100.0], &[1.0], 0.0, 5.0, &dirs));
}

#[test]
fn test_constrained_dominates_infeasible_does_not_beat_feasible() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // a infeasible, b feasible → a does NOT dominate
    assert!(!constrained_dominates(&[1.0], &[100.0], 5.0, 0.0, &dirs));
}

#[test]
fn test_constrained_dominates_both_infeasible_less_violation_wins() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // Both infeasible: a has violation 2.0 < b's 5.0 → a dominates
    assert!(constrained_dominates(&[100.0], &[1.0], 2.0, 5.0, &dirs));
    assert!(!constrained_dominates(&[1.0], &[100.0], 5.0, 2.0, &dirs));
}

#[test]
fn test_constrained_dominates_both_infeasible_equal_violation() {
    let dirs = vec![ObjectiveDirection::Minimize];
    // Both infeasible with same violation → neither dominates
    assert!(!constrained_dominates(&[1.0], &[2.0], 3.0, 3.0, &dirs));
}

#[test]
fn test_pareto_individual_is_feasible() {
    let mut ind = ParetoIndividual::new(Binary::new(), vec![1.0]);
    assert!(ind.is_feasible());
    ind.constraint_violation = 0.5;
    assert!(!ind.is_feasible());
}
