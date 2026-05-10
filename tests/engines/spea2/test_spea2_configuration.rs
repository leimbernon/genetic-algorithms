use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};

#[test]
fn test_spea2_configuration_default() {
    let config = Spea2Configuration::default();
    assert_eq!(config.num_objectives, 2);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.archive_size, 100);
    assert_eq!(config.max_generations, 250);
    assert!(config.objective_directions.is_empty());
}

#[test]
fn test_spea2_configuration_builder() {
    let config = Spea2Configuration::new()
        .with_num_objectives(3)
        .with_population_size(50)
        .with_archive_size(40)
        .with_max_generations(500);
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.archive_size, 40);
    assert_eq!(config.max_generations, 500);
}

#[test]
fn test_spea2_with_archive_size() {
    let config = Spea2Configuration::new().with_archive_size(50);
    assert_eq!(config.archive_size, 50);
    // Other fields should still be at defaults
    assert_eq!(config.population_size, 100);
}

#[test]
fn test_spea2_effective_directions_default_minimize() {
    let config = Spea2Configuration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_spea2_effective_directions_explicit() {
    let config = Spea2Configuration::new()
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
fn test_spea2_archive_size_default_equals_population() {
    // D-01: default archive_size equals population_size (canonical SPEA2)
    let config = Spea2Configuration::default();
    assert_eq!(config.archive_size, config.population_size);
}
