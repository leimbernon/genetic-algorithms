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
use std::sync::atomic::{AtomicUsize, Ordering};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

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

// ===== Run() integration tests — added by Plan 37-02 =====

/// ZDT1 objective functions (2-objective, 30 variables).
///
/// f1(x) = x_1
/// f2(x) = g(x) * (1 - sqrt(x_1 / g(x)))
/// g(x) = 1 + (9 / (n-1)) * sum(x_2..x_n)
fn zdt1_objectives(
) -> Vec<Box<genetic_algorithms::multi_objective::ObjectiveFn<genetic_algorithms::genotypes::Range<f64>>>> {
    let g_fn = |dna: &[genetic_algorithms::genotypes::Range<f64>]| -> f64 {
        let n = dna.len();
        let sum: f64 = dna[1..].iter().map(|g| g.value).sum();
        1.0 + (9.0 / (n - 1) as f64) * sum
    };
    let f1 = move |dna: &[genetic_algorithms::genotypes::Range<f64>]| -> f64 { dna[0].value };
    let f2 = move |dna: &[genetic_algorithms::genotypes::Range<f64>]| -> f64 {
        let g = g_fn(dna);
        g * (1.0 - (dna[0].value / g).sqrt())
    };
    vec![Box::new(f1), Box::new(f2)]
}

fn build_test_spea2(
    population_size: usize,
    archive_size: usize,
    max_generations: usize,
) -> Spea2Ga<RangeChromosome<f64>> {
    let spea2_config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(population_size)
        .with_archive_size(archive_size)
        .with_max_generations(max_generations);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 30;  // ZDT1: 30 variables
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(42);

    let alleles = vec![genetic_algorithms::genotypes::Range::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    Spea2Ga::<RangeChromosome<f64>>::new(spea2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(zdt1_objectives())
        .build()
        .expect("Spea2 build should succeed with all required builders called")
}

#[test]
fn test_spea2_run_produces_pareto_front() {
    // 20 population, 10 archive, 10 generations — small enough for fast tests.
    let mut spea2 = build_test_spea2(20, 10, 10);
    let result = spea2.run();
    assert!(result.is_ok(), "Spea2 run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All ParetoFront members must be rank 0");
        assert_eq!(ind.objectives.len(), 2, "Each individual should have 2 objectives");
    }
}

#[test]
fn test_spea2_run_with_archive_smaller_than_population() {
    // D-01: archive_size < population_size is valid as long as archive > 0
    let mut spea2 = build_test_spea2(30, 15, 10);
    let result = spea2.run();
    assert!(result.is_ok(), "Run with archive_size < population_size should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
    // Archive was limited to 15; the front extracted from archive should have <= 15 entries
    assert!(front.individuals.len() <= 15,
        "Front from archive of size 15 should have at most 15 members, got {}",
        front.individuals.len());
}

#[test]
fn test_spea2_run_with_archive_equals_population() {
    // Canonical SPEA2: archive_size == population_size
    let mut spea2 = build_test_spea2(15, 15, 10);
    let result = spea2.run();
    assert!(result.is_ok(), "Canonical SPEA2 run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
}

/// Counter-based observer to verify that lifecycle hooks fire per generation.
#[derive(Default)]
struct CountingObserver {
    fitness_count: AtomicUsize,
    archive_count: AtomicUsize,
}

impl genetic_algorithms::Spea2Observer<RangeChromosome<f64>> for CountingObserver {
    fn on_fitness_assigned(
        &self,
        _generation: usize,
        _duration_ms: f64,
        _pop_size: usize,
        _archive_size: usize,
    ) {
        self.fitness_count.fetch_add(1, Ordering::Relaxed);
    }

    fn on_archive_updated(
        &self,
        _generation: usize,
        _archive_size: usize,
        _non_dominated_count: usize,
    ) {
        self.archive_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn test_spea2_run_invokes_observer_hooks() {
    let observer = Arc::new(CountingObserver::default());
    let observer_handle = observer.clone();

    let spea2_config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(15)
        .with_archive_size(10)
        .with_max_generations(5);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 30;
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(123);

    let alleles = vec![genetic_algorithms::genotypes::Range::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut spea2 = Spea2Ga::<RangeChromosome<f64>>::new(spea2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(zdt1_objectives())
        .with_observer(
            observer as Arc<dyn genetic_algorithms::Spea2Observer<RangeChromosome<f64>> + Send + Sync>,
        )
        .build()
        .expect("build should succeed");

    spea2.run().expect("run should succeed");

    // Both hooks fire once per generation (5 total) on non-WASM host tests.
    assert_eq!(observer_handle.fitness_count.load(Ordering::Relaxed), 5,
        "on_fitness_assigned should fire once per generation");
    assert_eq!(observer_handle.archive_count.load(Ordering::Relaxed), 5,
        "on_archive_updated should fire once per generation");
}

#[test]
fn test_spea2_log_observer() {
    // D-06 smoke test: confirm `impl<U> Spea2Observer<U> for LogObserver` compiles
    // and runs without panic.
    let mut spea2 = build_test_spea2(15, 10, 3)
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn genetic_algorithms::Spea2Observer<RangeChromosome<f64>> + Send + Sync>,
        );

    let result = spea2.run();
    assert!(
        result.is_ok(),
        "Spea2 run with LogObserver should succeed: {:?}",
        result.err()
    );
    let front = result.unwrap();
    assert!(!front.is_empty(), "front should be non-empty under LogObserver");
}
