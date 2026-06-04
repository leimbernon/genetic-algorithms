//! NSGA-III engine tests.
//!
//! Wave 0 scope: validate() error paths only. The full engine integration
//! test (run() produces a non-empty Pareto front on a 3-objective problem)
//! is added in Plan 35-03 once the run() loop is implemented.

use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, LinearChromosome, OperatorCompat, VectorFitness};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
use genetic_algorithms::nsga3::Nsga3Ga;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Cow;

// ===== Custom test chromosome: DTLZ2 3-objective benchmark =====
//
// calculate_fitness() evaluates the DTLZ2 sphere function (M=3, n=4 variables)
// and stores the 3 objective values into fitness_values.

#[derive(Debug, Clone, Default)]
struct Dtlz2Chromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for Dtlz2Chromosome {
    type Gene = RangeGenotype<f64>;

    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }

    fn calculate_fitness(&mut self) {
        use std::f64::consts::FRAC_PI_2;
        let dna = &self.dna;
        if dna.len() < 4 {
            self.fitness_values = vec![0.0, 0.0, 0.0];
            self.fitness = 0.0;
            return;
        }
        let g: f64 = dna[2..].iter().map(|gene| (gene.value - 0.5).powi(2)).sum();
        let x1 = dna[0].value;
        let x2 = dna[1].value;
        let f1 = (x1 * FRAC_PI_2).cos() * (x2 * FRAC_PI_2).cos() * (1.0 + g);
        let f2 = (x1 * FRAC_PI_2).cos() * (x2 * FRAC_PI_2).sin() * (1.0 + g);
        let f3 = (x1 * FRAC_PI_2).sin() * (1.0 + g);
        self.fitness_values = vec![f1, f2, f3];
        self.fitness = f1 + f2 + f3;
    }
}

impl LinearChromosome for Dtlz2Chromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for Dtlz2Chromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for Dtlz2Chromosome {}
impl OperatorCompat for Dtlz2Chromosome {}

// ===== A chromosome that always produces 2 fitness values (for mismatch test) =====

#[derive(Debug, Clone, Default)]
struct TwoObjectiveChromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for TwoObjectiveChromosome {
    type Gene = RangeGenotype<f64>;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        // Always 2 values even when engine expects 3
        let sum: f64 = self.dna.iter().map(|g| g.value).sum();
        self.fitness_values = vec![sum, -sum];
        self.fitness = sum;
    }
}

impl LinearChromosome for TwoObjectiveChromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for TwoObjectiveChromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for TwoObjectiveChromosome {}
impl OperatorCompat for TwoObjectiveChromosome {}

// ===== validate() error-path tests =====

#[test]
fn test_nsga3_validate_no_init_fn() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_zero_objectives() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(0)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_missing_reference_points() {
    // Neither with_reference_points_auto nor with_reference_points was called.
    let config = Nsga3Configuration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(msg)) if msg.contains("dimension")));
}

#[test]
fn test_nsga3_validate_passes_with_complete_config() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    assert!(nsga3.validate().is_ok());
}

#[test]
fn test_nsga3_validate_mismatched_objective_directions() {
    // objective_directions has 1 entry but num_objectives is 3 — should fail.
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize]) // 1 != 3
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    assert!(matches!(
        nsga3.validate(),
        Err(GaError::InvalidNsga3Configuration(ref msg)) if msg.contains("objective_directions")
    ));
}

/// Runtime objective-count mismatch: engine expects 3 objectives but chromosome
/// produces only 2 values in fitness_values(). run() must return an error.
#[test]
fn test_nsga3_run_rejects_mismatched_objective_count() {
    use genetic_algorithms::ChromosomeLength;

    let config = Nsga3Configuration::new()
        .with_num_objectives(3)  // expects 3
        .with_population_size(15)
        .with_max_generations(1)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(4));

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<TwoObjectiveChromosome>::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("build should succeed");

    let result = nsga3.run();
    assert!(
        matches!(result, Err(GaError::InvalidNsga3Configuration(_))),
        "Expected InvalidNsga3Configuration for mismatched objective count, got: {:?}",
        result
    );
}

// ===== Run() integration tests — added by Plan 35-03 =====

fn build_test_nsga3(
    population_size: usize,
    max_generations: usize,
) -> Nsga3Ga<Dtlz2Chromosome> {
    use genetic_algorithms::ChromosomeLength;

    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_reference_points_auto(4); // 15 reference points for M=3, p=4

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_rng_seed(42);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    Nsga3Ga::<Dtlz2Chromosome>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("Nsga3 build should succeed with all required builders called")
}

#[test]
fn test_nsga3_run_produces_pareto_front() {
    let mut nsga3 = build_test_nsga3(40, 10);
    let result = nsga3.run();
    assert!(result.is_ok(), "Nsga3 run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    // Every front individual should have rank == 0.
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All ParetoFront members must be rank 0");
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_nsga3_run_with_custom_reference_points() {
    use genetic_algorithms::ChromosomeLength;

    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.5, 0.5, 0.0],
        vec![0.5, 0.0, 0.5],
        vec![0.0, 0.5, 0.5],
        vec![0.33, 0.33, 0.34],
    ];
    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(30)
        .with_max_generations(8)
        .with_reference_points(custom);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_rng_seed(7);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("build should succeed");

    let result = nsga3.run();
    assert!(result.is_ok(), "run with custom ref points: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
}

/// Counter-based observer to verify that lifecycle hooks fire.
#[derive(Default)]
struct CountingObserver {
    sort_count: AtomicUsize,
    pareto_count: AtomicUsize,
}

impl genetic_algorithms::Nsga3Observer<Dtlz2Chromosome> for CountingObserver {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
        self.pareto_count.fetch_add(1, Ordering::Relaxed);
    }

    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {
        self.sort_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn test_nsga3_run_invokes_observer_hooks() {
    use genetic_algorithms::ChromosomeLength;

    let observer = Arc::new(CountingObserver::default());
    let observer_handle = observer.clone();

    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(30)
        .with_max_generations(5)
        .with_reference_points_auto(2);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_rng_seed(123);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(observer as Arc<dyn genetic_algorithms::Nsga3Observer<Dtlz2Chromosome> + Send + Sync>)
        .build()
        .expect("build should succeed");

    nsga3.run().expect("run should succeed");

    // on_non_dominated_sort_complete fires only when observer is Some and Instant is available.
    // On non-WASM: fires exactly max_generations times (5).
    // The pareto hook fires unconditionally (not gated by Instant).
    assert_eq!(observer_handle.pareto_count.load(Ordering::Relaxed), 5);
    // sort_count fires from inside the Instant block — always 5 on non-WASM host test.
    assert_eq!(observer_handle.sort_count.load(Ordering::Relaxed), 5);
}
