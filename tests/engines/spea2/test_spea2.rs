//! SPEA2 engine tests.

use std::borrow::Cow;
use genetic_algorithms::traits::{ConfigurationT, ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
use genetic_algorithms::spea2::Spea2Ga;
use genetic_algorithms::LogObserver;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Custom 2-objective chromosome using RangeGenotype<f64> genes.
// f1 = dna[0].value, f2 = 1.0 - sqrt(dna[0].value) (ZDT1-like simple objectives)
#[derive(Debug, Clone, Default)]
struct TwoObjRangeChromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for TwoObjRangeChromosome {
    type Gene = RangeGenotype<f64>;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        let f1 = self.dna.first().map(|g| g.value).unwrap_or(0.0);
        let f2 = 1.0 - f1.sqrt();
        self.fitness_values = vec![f1, f2];
        self.fitness = f1;
    }
}

impl LinearChromosome for TwoObjRangeChromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for TwoObjRangeChromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for TwoObjRangeChromosome {}
impl genetic_algorithms::traits::OperatorCompat for TwoObjRangeChromosome {}

// --- Validation tests ---

#[test]
fn test_spea2_validate_no_init_fn() {
    let config = Spea2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_zero_objectives() {
    let config = Spea2Configuration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_population_too_small() {
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
}

#[test]
fn test_spea2_validate_archive_size_exceeds_population() {
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(10)
        .with_archive_size(50);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(ref msg)) if msg.contains("archive_size")));
}

#[test]
fn test_spea2_validate_archive_size_zero() {
    let config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20)
        .with_archive_size(0);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = spea2.validate();
    assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(ref msg)) if msg.contains("archive_size")));
}

#[test]
fn test_spea2_validate_mismatched_objective_directions() {
    let config = Spea2Configuration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize]);
    let ga_config = GaConfiguration::default();
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    assert!(spea2.validate().is_ok());
}

// ===== Run() integration tests =====

fn build_test_spea2(
    population_size: usize,
    archive_size: usize,
    max_generations: usize,
) -> Spea2Ga<TwoObjRangeChromosome> {
    let spea2_config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(population_size)
        .with_archive_size(archive_size)
        .with_max_generations(max_generations);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3))
        .with_rng_seed(42);

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    Spea2Ga::<TwoObjRangeChromosome>::new(spea2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("Spea2 build should succeed with all required builders called")
}

#[test]
fn test_spea2_run_produces_pareto_front() {
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
    let mut spea2 = build_test_spea2(30, 15, 10);
    let result = spea2.run();
    assert!(result.is_ok(), "Run with archive_size < population_size should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
    assert!(front.individuals.len() <= 15,
        "Front from archive of size 15 should have at most 15 members, got {}",
        front.individuals.len());
}

#[test]
fn test_spea2_run_with_archive_equals_population() {
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

impl genetic_algorithms::Spea2Observer<TwoObjRangeChromosome> for CountingObserver {
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

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3))
        .with_rng_seed(123);

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut spea2 = Spea2Ga::<TwoObjRangeChromosome>::new(spea2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(
            observer as Arc<dyn genetic_algorithms::Spea2Observer<TwoObjRangeChromosome> + Send + Sync>,
        )
        .build()
        .expect("build should succeed");

    spea2.run().expect("run should succeed");

    assert_eq!(observer_handle.fitness_count.load(Ordering::Relaxed), 5,
        "on_fitness_assigned should fire once per generation");
    assert_eq!(observer_handle.archive_count.load(Ordering::Relaxed), 5,
        "on_archive_updated should fire once per generation");
}

#[test]
fn test_spea2_log_observer() {
    let mut spea2 = build_test_spea2(15, 10, 3)
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn genetic_algorithms::Spea2Observer<TwoObjRangeChromosome> + Send + Sync>,
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
