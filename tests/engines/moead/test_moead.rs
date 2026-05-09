//! MOEA/D engine tests.
//!
//! Wave 0 scope: validate() error paths. Plan 36-02 appends run() integration
//! tests once the run() loop is implemented; Plan 36-03 adds the LogObserver
//! integration test.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ObjectiveDirection};
use genetic_algorithms::moead::MoeaDGa;

#[test]
fn test_moead_validate_no_init_fn() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_zero_objectives() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(0)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_population_too_small() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(1)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_mismatched_objective_fns() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_missing_weight_vectors() {
    // D-06: neither auto nor custom called
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("weight vectors")));
}

#[test]
fn test_moead_validate_custom_weight_vector_wrong_dimension() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors(vec![vec![0.5, 0.5]]); // 2-dim, not 3
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("dimension")));
}

#[test]
fn test_moead_validate_das_dennis_p_zero() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(0);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("Das-Dennis")));
}

#[test]
fn test_moead_validate_mismatched_objective_directions() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize])
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        moead.validate(),
        Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("objective_directions")
    ));
}

#[test]
fn test_moead_validate_passes_with_complete_config() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(moead.validate().is_ok());
}
