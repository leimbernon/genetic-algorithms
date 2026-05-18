//! IBEA engine tests.
//!
//! Wave 0 scope: validate() error paths. Plan 38-03 appends run() integration
//! tests once the run() loop is implemented.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::ibea::configuration::IbeaConfiguration;
use genetic_algorithms::ibea::IbeaGa;

#[test]
fn test_ibea_validate_no_init_fn() {
    let config = IbeaConfiguration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_zero_objectives() {
    let config = IbeaConfiguration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_one_objective() {
    let config = IbeaConfiguration::new().with_num_objectives(1);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_population_too_small() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_mismatched_objective_fns() {
    let config = IbeaConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_mismatched_objective_directions() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![genetic_algorithms::ibea::configuration::ObjectiveDirection::Minimize]);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        ibea.validate(),
        Err(GaError::InvalidIbeaConfiguration(ref msg)) if msg.contains("objective_directions")
    ));
}

#[test]
fn test_ibea_validate_passes_with_complete_config() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(20);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(ibea.validate().is_ok());
}

// --- Run integration tests ---

#[test]
fn test_ibea_run_produces_pareto_front() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(8)
        .with_max_generations(5);
    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 3;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<
        RangeChromosome<f64>,
    >::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![
            Box::new(|dna: &[genetic_algorithms::genotypes::Range<f64>]| dna[0].value),
            Box::new(|dna: &[genetic_algorithms::genotypes::Range<f64>]| 1.0 - dna[0].value.sqrt()),
        ])
        .build()
        .expect("Failed to build IbeaGa");

    let front = ibea.run().expect("IBEA run failed");
    assert!(!front.is_empty(), "Pareto front must not be empty");
    assert!(front.individuals.iter().any(|ind| ind.rank == 0));
}

#[test]
fn test_ibea_run_small_population() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(4)
        .with_max_generations(3);
    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 2;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<
        RangeChromosome<f64>,
    >::new(config, GaConfiguration::default())
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.5),
            Box::new(|_: &[genetic_algorithms::genotypes::Range<f64>]| 0.5),
        ])
        .build()
        .expect("Failed to build IbeaGa");

    let result = ibea.run();
    assert!(result.is_ok(), "IBEA run with small pop failed: {:?}", result.err());
}

#[test]
fn test_ibea_run_invokes_observer_hooks() {
    use std::sync::Arc;
    use genetic_algorithms::observer::IbeaObserver;
    use genetic_algorithms::LogObserver;

    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(6)
        .with_max_generations(4);
    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 2;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![genetic_algorithms::genotypes::Range::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<
        RangeChromosome<f64>,
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
            Arc::new(LogObserver) as Arc<dyn IbeaObserver<RangeChromosome<f64>> + Send + Sync>
        )
        .build()
        .expect("Failed to build IbeaGa");

    let result = ibea.run();
    assert!(result.is_ok(), "IBEA observer test run failed: {:?}", result.err());
}
