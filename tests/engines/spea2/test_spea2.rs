//! SPEA2 engine tests.
//!
//! Wave 0 scope: validate() error paths. Plan 37-02 appends run() integration
//! tests once the run() loop is implemented; Plan 37-03 adds the LogObserver
//! smoke test.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
use genetic_algorithms::spea2::Spea2Ga;

#[test]
fn test_spea2_validate_no_init_fn() {
    let config = Spea2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_zero_objectives() {
    let config = Spea2Configuration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_population_too_small() {
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_archive_size_exceeds_population() {
    // D-01: archive_size > population_size is rejected
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(10)
        .with_archive_size(50);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(ref msg)) if msg.contains("archive_size")));
}

#[test]
fn test_spea2_validate_archive_size_zero() {
    // D-01: archive_size == 0 is rejected
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20)
        .with_archive_size(0);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(ref msg)) if msg.contains("archive_size")));
}

#[test]
fn test_spea2_validate_mismatched_objective_fns() {
    let config = Spea2Configuration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_mismatched_objective_directions() {
    let config = Spea2Configuration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize]);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        spea2.validate(),
        Err(GaError::InvalidSpea2Configuration(ref msg)) if msg.contains("objective_directions")
    ));
}

#[test]
fn test_spea2_validate_passes_with_complete_config() {
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20)
        .with_archive_size(15);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(spea2.validate().is_ok());
}
