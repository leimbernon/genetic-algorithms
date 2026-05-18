//! SMS-EMOA engine tests.
//!
//! Wave 0 scope: validate() error paths. Plan 38-03 appends run() integration
//! tests once the run() loop is implemented.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration;
use genetic_algorithms::sms_emoa::SmsEmoaGa;

#[test]
fn test_sms_emoa_validate_no_init_fn() {
    let config = SmsEmoaConfiguration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config);
    let result = sms.validate();
    assert!(matches!(result, Err(GaError::InvalidSmsEmoaConfiguration(_))));
}

#[test]
fn test_sms_emoa_validate_zero_objectives() {
    let config = SmsEmoaConfiguration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = sms.validate();
    assert!(matches!(result, Err(GaError::InvalidSmsEmoaConfiguration(_))));
}

#[test]
fn test_sms_emoa_validate_one_objective() {
    // SMS-EMOA requires at least 2 objectives (hypervolume needs 2D)
    let config = SmsEmoaConfiguration::new().with_num_objectives(1);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = sms.validate();
    assert!(matches!(result, Err(GaError::InvalidSmsEmoaConfiguration(_))));
}

#[test]
fn test_sms_emoa_validate_population_too_small() {
    let config = SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = sms.validate();
    assert!(matches!(result, Err(GaError::InvalidSmsEmoaConfiguration(_))));
}

#[test]
fn test_sms_emoa_validate_mismatched_objective_fns() {
    let config = SmsEmoaConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = sms.validate();
    assert!(matches!(result, Err(GaError::InvalidSmsEmoaConfiguration(_))));
}

#[test]
fn test_sms_emoa_validate_mismatched_objective_directions() {
    let config = SmsEmoaConfiguration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![genetic_algorithms::sms_emoa::configuration::ObjectiveDirection::Minimize]);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        sms.validate(),
        Err(GaError::InvalidSmsEmoaConfiguration(ref msg)) if msg.contains("objective_directions")
    ));
}

#[test]
fn test_sms_emoa_validate_passes_with_complete_config() {
    let config = SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(20);
    let ga_config = GaConfiguration::default();
    let sms = SmsEmoaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(sms.validate().is_ok());
}

// --- Run integration tests ---

#[test]
fn test_sms_emoa_run_produces_pareto_front() {
    // Minimal 2-objective configuration (ZDT1-like with simple objectives)
    let config = genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(8)
        .with_max_generations(5);
    let mut ga_config = genetic_algorithms::configuration::GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 3;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut sms = genetic_algorithms::sms_emoa::SmsEmoaGa::<
        genetic_algorithms::chromosomes::Range<f64>,
    >::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![
            Box::new(|dna: &[genetic_algorithms::genotypes::Range<f64>]| dna[0].value),
            Box::new(|dna: &[genetic_algorithms::genotypes::Range<f64>]| {
                1.0 - dna[0].value.sqrt()
            }),
        ])
        .build()
        .expect("Failed to build SmsEmoaGa");

    let front = sms.run().expect("SMS-EMOA run failed");
    assert!(!front.is_empty(), "Pareto front must not be empty");
    // At least one rank-0 individual
    assert!(front.individuals.iter().any(|ind| ind.rank == 0));
}

#[test]
fn test_sms_emoa_run_small_population() {
    let config = genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(4)
        .with_max_generations(3);
    let mut ga_config = genetic_algorithms::configuration::GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 2;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut sms = genetic_algorithms::sms_emoa::SmsEmoaGa::<
        genetic_algorithms::chromosomes::Range<f64>,
    >::new(config, genetic_algorithms::configuration::GaConfiguration::default())
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.5),
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.5),
        ])
        .build()
        .expect("Failed to build SmsEmoaGa");

    let result = sms.run();
    assert!(result.is_ok(), "SMS-EMOA run with small pop failed: {:?}", result.err());
}

#[test]
fn test_sms_emoa_run_invokes_observer_hooks() {
    use std::sync::Arc;
    use genetic_algorithms::observer::SmsEmoaObserver;
    use genetic_algorithms::LogObserver;

    let config = genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(6)
        .with_max_generations(4);
    let mut ga_config = genetic_algorithms::configuration::GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 2;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut sms = genetic_algorithms::sms_emoa::SmsEmoaGa::<
        genetic_algorithms::chromosomes::Range<f64>,
    >::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.5),
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.3),
        ])
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn SmsEmoaObserver<genetic_algorithms::chromosomes::Range<f64>> + Send + Sync>
        )
        .build()
        .expect("Failed to build SmsEmoaGa");

    let result = sms.run();
    assert!(result.is_ok(), "SMS-EMOA observer test run failed: {:?}", result.err());
}
