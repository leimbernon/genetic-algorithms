use genetic_algorithms::nsga2::crowding_distance::assign_crowding_distance;

#[test]
fn test_crowding_distance_two_individuals() {
    let objectives: Vec<Vec<f64>> = vec![vec![1.0, 2.0], vec![3.0, 1.0]];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let mut crowding = vec![0.0; 2];
    assign_crowding_distance(&refs, &mut crowding);
    assert!(crowding[0].is_infinite());
    assert!(crowding[1].is_infinite());
}

#[test]
fn test_crowding_distance_three_individuals() {
    let objectives: Vec<Vec<f64>> = vec![vec![1.0, 4.0], vec![2.0, 2.0], vec![4.0, 1.0]];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let mut crowding = vec![0.0; 3];
    assign_crowding_distance(&refs, &mut crowding);
    // Boundary individuals get infinity
    assert!(crowding[0].is_infinite());
    assert!(crowding[2].is_infinite());
    // Middle individual gets finite value
    assert!(crowding[1].is_finite());
    assert!(crowding[1] > 0.0);
}

#[test]
fn test_crowding_distance_empty() {
    let objectives: Vec<&[f64]> = vec![];
    let mut crowding: Vec<f64> = vec![];
    assign_crowding_distance(&objectives, &mut crowding);
    assert!(crowding.is_empty());
}

#[test]
fn test_crowding_distance_same_values() {
    // All individuals have the same objectives
    let objectives: Vec<Vec<f64>> = vec![vec![1.0, 1.0], vec![1.0, 1.0], vec![1.0, 1.0]];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let mut crowding = vec![0.0; 3];
    assign_crowding_distance(&refs, &mut crowding);
    // Boundary individuals still get infinity
    assert!(crowding[0].is_infinite());
    assert!(crowding[2].is_infinite());
    // Middle one stays at 0 since range is 0
}
