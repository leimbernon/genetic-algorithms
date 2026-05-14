use genetic_algorithms::sms_emoa::configuration::{SmsEmoaConfiguration, ObjectiveDirection};

#[test]
fn test_sms_emoa_configuration_default() {
    let config = SmsEmoaConfiguration::default();
    assert_eq!(config.num_objectives, 2);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 250);
    assert!(config.objective_directions.is_empty());
    assert!(config.hypervolume_reference_point.is_none());
}

#[test]
fn test_sms_emoa_configuration_builder() {
    let config = SmsEmoaConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(50)
        .with_max_generations(500);
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 50);
    assert_eq!(config.max_generations, 500);
}

#[test]
fn test_sms_emoa_effective_directions_default_minimize() {
    let config = SmsEmoaConfiguration::new().with_num_objectives(3);
    let dirs = config.effective_directions();
    assert_eq!(dirs.len(), 3);
    assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
}

#[test]
fn test_sms_emoa_effective_directions_explicit() {
    let config = SmsEmoaConfiguration::new()
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
fn test_sms_emoa_with_hypervolume_reference_point() {
    let config = SmsEmoaConfiguration::new()
        .with_hypervolume_reference_point(vec![2.0, 2.0]);
    assert_eq!(config.hypervolume_reference_point, Some(vec![2.0, 2.0]));
}
