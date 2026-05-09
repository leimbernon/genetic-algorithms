//! NSGA-III engine tests.
//!
//! Wave 0 scope: validate() error paths only. The full engine integration
//! test (run() produces a non-empty Pareto front on a 3-objective problem)
//! is added in Plan 35-03 once the run() loop is implemented.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::nsga3::configuration::Nsga3Configuration;
use genetic_algorithms::nsga3::Nsga3Ga;

#[test]
fn test_nsga3_validate_no_init_fn() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_zero_objectives() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(0)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_population_too_small() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(1)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_mismatched_objective_fns() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]); // only 1, expected 3
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_missing_reference_points() {
    // Neither with_reference_points_auto nor with_reference_points was called.
    let config = Nsga3Configuration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(msg)) if msg.contains("reference points")));
}

#[test]
fn test_nsga3_validate_custom_reference_point_wrong_dimension() {
    // num_objectives=3 but custom point has length 2 -> should fail validate.
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points(vec![vec![0.5, 0.5]]); // 2-dim, not 3
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(msg)) if msg.contains("dimension")));
}

#[test]
fn test_nsga3_validate_passes_with_complete_config() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(nsga3.validate().is_ok());
}
