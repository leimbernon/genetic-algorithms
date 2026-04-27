use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::Extension;

#[test]
fn test_default() {
    let config = ExtensionConfiguration::default();
    assert_eq!(config.method, Extension::Noop);
    assert!((config.diversity_threshold - 0.01).abs() < f64::EPSILON);
    assert!((config.survival_rate - 0.1).abs() < f64::EPSILON);
    assert_eq!(config.mutation_rounds, 3);
    assert_eq!(config.elite_count, 1);
}

#[test]
fn test_builder() {
    let config = ExtensionConfiguration::new()
        .with_method(Extension::MassExtinction)
        .with_diversity_threshold(0.05)
        .with_survival_rate(0.3)
        .with_mutation_rounds(5)
        .with_elite_count(2);

    assert_eq!(config.method, Extension::MassExtinction);
    assert!((config.diversity_threshold - 0.05).abs() < f64::EPSILON);
    assert!((config.survival_rate - 0.3).abs() < f64::EPSILON);
    assert_eq!(config.mutation_rounds, 5);
    assert_eq!(config.elite_count, 2);
}
