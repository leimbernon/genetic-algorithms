use genetic_algorithms::gp::{GpConfiguration, GpCrossover, GpMutation};
use genetic_algorithms::operations::Survivor;

#[test]
fn gp_configuration_defaults() {
    let cfg = GpConfiguration::new();
    assert_eq!(cfg.population_size(), 100);
    assert_eq!(cfg.max_generations(), 50);
    assert_eq!(cfg.init_max_depth(), 4);
    assert_eq!(cfg.max_depth(), 8);
    assert_eq!(cfg.max_node_count(), 200);
    assert!(!cfg.is_maximization());
}

#[test]
fn gp_configuration_build_valid() {
    assert!(GpConfiguration::new().build().is_ok());
}

#[test]
fn gp_configuration_builder_chain() {
    let cfg = GpConfiguration::new()
        .with_population_size(50)
        .with_max_generations(20)
        .with_init_max_depth(3)
        .with_max_depth(6)
        .with_max_node_count(100)
        .with_is_maximization(true);
    assert_eq!(cfg.population_size(), 50);
    assert_eq!(cfg.max_generations(), 20);
    assert_eq!(cfg.init_max_depth(), 3);
    assert_eq!(cfg.max_depth(), 6);
    assert_eq!(cfg.max_node_count(), 100);
    assert!(cfg.is_maximization());
}

#[test]
fn gp_configuration_with_stagnation_and_target() {
    let cfg = GpConfiguration::new()
        .with_max_stagnation(Some(10))
        .with_fitness_target(Some(0.01));
    assert!(cfg.build().is_ok());
}

#[test]
fn gp_configuration_with_crossover() {
    let cfg = GpConfiguration::new().with_crossover(GpCrossover::SubtreeCrossover);
    assert!(cfg.build().is_ok());
}

#[test]
fn gp_configuration_with_mutations() {
    let cfg = GpConfiguration::new().with_mutations(vec![
        (
            GpMutation::SubtreeMutation {
                mutation_max_depth: 3,
            },
            0.2,
        ),
        (GpMutation::PointMutation { p_per_node: 0.05 }, 0.05),
    ]);
    assert!(cfg.build().is_ok());
}

#[test]
fn gp_configuration_with_survivor() {
    let cfg = GpConfiguration::new().with_survivor_config(Survivor::Fitness);
    assert!(cfg.build().is_ok());
}

#[test]
fn gp_configuration_error_max_depth_zero() {
    let result = GpConfiguration::new().with_max_depth(0).build();
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("max_depth"), "Error: {}", msg);
}

#[test]
fn gp_configuration_error_max_depth_too_large() {
    let result = GpConfiguration::new().with_max_depth(1_001).build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_init_depth_exceeds_max() {
    let result = GpConfiguration::new()
        .with_init_max_depth(10)
        .with_max_depth(8)
        .build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_init_depth_zero() {
    let result = GpConfiguration::new().with_init_max_depth(0).build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_node_count_less_than_depth() {
    let result = GpConfiguration::new()
        .with_max_depth(10)
        .with_max_node_count(5)
        .build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_node_count_too_large() {
    let result = GpConfiguration::new()
        .with_max_depth(10)
        .with_max_node_count(100_001)
        .build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_population_size_zero() {
    let result = GpConfiguration::new().with_population_size(0).build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_max_generations_zero() {
    let result = GpConfiguration::new().with_max_generations(0).build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_empty_mutations() {
    let result = GpConfiguration::new().with_mutations(vec![]).build();
    assert!(result.is_err());
}

#[test]
fn gp_configuration_error_invalid_mutation_probability() {
    let result = GpConfiguration::new()
        .with_mutations(vec![(GpMutation::PointMutation { p_per_node: 0.05 }, 1.5)])
        .build();
    assert!(result.is_err());
}
