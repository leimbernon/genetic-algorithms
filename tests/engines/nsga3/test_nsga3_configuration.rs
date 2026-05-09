use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};

#[test]
fn test_nsga3_configuration_default() {
    let config = Nsga3Configuration::default();
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 200);
    assert!(config.objective_directions.is_empty());
    assert!(config.effective_reference_points().is_none());
}

#[test]
fn test_nsga3_configuration_builder() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(4)
        .with_population_size(50)
        .with_max_generations(1000);
    assert_eq!(config.num_objectives, 4);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.max_generations, 1000);
}

#[test]
fn test_nsga3_with_reference_points_auto_generates_correct_count() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let pts = config
        .effective_reference_points()
        .expect("auto reference points should be Some");
    // C(4 + 3 - 1, 3 - 1) = C(6, 2) = 15
    assert_eq!(pts.len(), 15);
    for pt in &pts {
        let sum: f64 = pt.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_nsga3_with_reference_points_custom() {
    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points(custom.clone());
    let pts = config.effective_reference_points().expect("custom reference points should be Some");
    assert_eq!(pts, custom);
}

#[test]
fn test_nsga3_last_call_wins_auto_then_custom() {
    // D-07: auto then custom -> custom wins
    let custom = vec![vec![0.5, 0.3, 0.2]];
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4)
        .with_reference_points(custom.clone());
    let pts = config.effective_reference_points().unwrap();
    assert_eq!(pts, custom);
}

#[test]
fn test_nsga3_last_call_wins_custom_then_auto() {
    // D-07: custom then auto -> auto wins
    let custom = vec![vec![0.5, 0.3, 0.2]];
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points(custom)
        .with_reference_points_auto(2);
    let pts = config.effective_reference_points().unwrap();
    // C(2 + 3 - 1, 3 - 1) = 6
    assert_eq!(pts.len(), 6);
}

#[test]
fn test_nsga3_no_reference_points_returns_none() {
    let config = Nsga3Configuration::new().with_num_objectives(3);
    assert!(config.effective_reference_points().is_none());
}

#[test]
fn test_nsga3_effective_directions_default_minimize() {
    let config = Nsga3Configuration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_nsga3_effective_directions_explicit() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![
            ObjectiveDirection::Maximize,
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ]);
    let dirs = config.effective_directions();
    assert_eq!(dirs[0], ObjectiveDirection::Maximize);
    assert_eq!(dirs[1], ObjectiveDirection::Minimize);
    assert_eq!(dirs[2], ObjectiveDirection::Minimize);
}
