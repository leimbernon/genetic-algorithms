use genetic_algorithms::ibea::configuration::{IbeaConfiguration, ObjectiveDirection};

#[test]
fn test_ibea_configuration_default() {
    let config = IbeaConfiguration::default();
    assert_eq!(config.num_objectives, 2);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 250);
    assert!(config.objective_directions.is_empty());
}

#[test]
fn test_ibea_configuration_builder() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(50)
        .with_max_generations(500);
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.max_generations, 500);
}

#[test]
fn test_ibea_effective_directions_default_minimize() {
    let config = IbeaConfiguration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_ibea_effective_directions_explicit() {
    let config = IbeaConfiguration::new()
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
