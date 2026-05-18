use genetic_algorithms::moead::configuration::{
    MoeaDConfiguration, ObjectiveDirection, ScalarizationFn,
};

#[test]
fn test_moead_configuration_default() {
    let config = MoeaDConfiguration::default();
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 200);
    assert!(config.objective_directions.is_empty());
    assert!(matches!(config.scalarization, ScalarizationFn::Tchebycheff));
    assert_eq!(config.neighborhood_size, 20);
    assert_eq!(config.max_neighbor_replacements, 2);
    assert!(config.effective_weight_vectors().is_none());
}

#[test]
fn test_moead_configuration_builder() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(4)
        .with_population_size(50)
        .with_max_generations(1000)
        .with_neighborhood_size(15)
        .with_max_neighbor_replacements(3);
    assert_eq!(config.num_objectives, 4);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.max_generations, 1000);
    assert_eq!(config.neighborhood_size, 15);
    assert_eq!(config.max_neighbor_replacements, 3);
}

#[test]
fn test_moead_with_weight_vectors_auto_generates_correct_count() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let wvs = config
        .effective_weight_vectors()
        .expect("auto weight vectors should be Some");
    // C(4 + 3 - 1, 3 - 1) = C(6, 2) = 15
    assert_eq!(wvs.len(), 15);
    for wv in &wvs {
        let sum: f64 = wv.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_moead_with_weight_vectors_custom() {
    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors(custom.clone());
    let wvs = config
        .effective_weight_vectors()
        .expect("custom weight vectors should be Some");
    assert_eq!(wvs, custom);
}

#[test]
fn test_moead_last_call_wins_auto_then_custom() {
    // D-07: auto then custom -> custom wins
    let custom = vec![vec![0.5, 0.3, 0.2]];
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4)
        .with_weight_vectors(custom.clone());
    let wvs = config.effective_weight_vectors().unwrap();
    assert_eq!(wvs, custom);
}

#[test]
fn test_moead_last_call_wins_custom_then_auto() {
    // D-07: custom then auto -> auto wins
    let custom = vec![vec![0.5, 0.3, 0.2]];
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors(custom)
        .with_weight_vectors_auto(2);
    let wvs = config.effective_weight_vectors().unwrap();
    // C(2 + 3 - 1, 3 - 1) = 6
    assert_eq!(wvs.len(), 6);
}

#[test]
fn test_moead_no_weight_vectors_returns_none() {
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    assert!(config.effective_weight_vectors().is_none());
}

#[test]
fn test_moead_effective_directions_default_minimize() {
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_moead_effective_directions_explicit() {
    let config = MoeaDConfiguration::new()
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

#[test]
fn test_scalarization_default() {
    // D-03: default is Tchebycheff
    assert!(matches!(ScalarizationFn::default(), ScalarizationFn::Tchebycheff));
    let config = MoeaDConfiguration::default();
    assert!(matches!(config.scalarization, ScalarizationFn::Tchebycheff));
}

#[test]
fn test_scalarization_pbi_holds_theta() {
    let config = MoeaDConfiguration::new().with_scalarization(ScalarizationFn::Pbi { theta: 5.0 });
    assert!(matches!(config.scalarization, ScalarizationFn::Pbi { theta } if (theta - 5.0).abs() < 1e-9));
}
