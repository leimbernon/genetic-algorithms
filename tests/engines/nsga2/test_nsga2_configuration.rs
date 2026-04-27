use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};

#[test]
fn test_nsga2_configuration_default() {
    let config = Nsga2Configuration::default();
    assert_eq!(config.num_objectives, 2);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 200);
    assert!(config.objective_directions.is_empty());
}

#[test]
fn test_nsga2_configuration_builder() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(3)
        .with_population_size(50)
        .with_max_generations(1000);

    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.max_generations, 1000);
}

#[test]
fn test_nsga2_configuration_directions() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Maximize,
        ]);

    assert_eq!(config.objective_directions.len(), 2);
    assert_eq!(config.objective_directions[0], ObjectiveDirection::Minimize);
    assert_eq!(config.objective_directions[1], ObjectiveDirection::Maximize);
}

#[test]
fn test_nsga2_configuration_effective_directions_default() {
    let config = Nsga2Configuration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_nsga2_configuration_effective_directions_explicit() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_objective_directions(vec![
            ObjectiveDirection::Maximize,
            ObjectiveDirection::Minimize,
        ]);
    let dirs = config.effective_directions();
    assert_eq!(dirs[0], ObjectiveDirection::Maximize);
    assert_eq!(dirs[1], ObjectiveDirection::Minimize);
}
