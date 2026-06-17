//! MOEA/D engine tests.

use std::borrow::Cow;
use genetic_algorithms::traits::{ConfigurationT, ChromosomeT, LinearChromosome, VectorFitness, MutationConfig};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ObjectiveDirection, ScalarizationFn};
use genetic_algorithms::moead::MoeaDGa;
#[cfg(feature = "logging")]
use genetic_algorithms::LogObserver;
use genetic_algorithms::MoeaDObserver;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Custom 3-objective chromosome using RangeGenotype<f64> genes.
// Simple 3-objective: f1 = dna[0].value, f2 = dna[1].value, f3 = 1 - f1 - f2
#[derive(Debug, Clone, Default)]
struct ThreeObjRangeChromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for ThreeObjRangeChromosome {
    type Gene = RangeGenotype<f64>;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        let f1 = self.dna.first().map(|g| g.value).unwrap_or(0.0);
        let f2 = self.dna.get(1).map(|g| g.value).unwrap_or(0.0);
        let f3 = (1.0_f64 - f1 - f2).max(0.0);
        self.fitness_values = vec![f1, f2, f3];
        self.fitness = f1 + f2 + f3;
    }
}

impl LinearChromosome for ThreeObjRangeChromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for ThreeObjRangeChromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for ThreeObjRangeChromosome {}
impl genetic_algorithms::traits::OperatorCompat for ThreeObjRangeChromosome {}

// --- Validation tests ---

#[test]
fn test_moead_validate_no_init_fn() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_zero_objectives() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(0)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_missing_weight_vectors() {
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("weight vectors")));
}

#[test]
fn test_moead_validate_custom_weight_vector_wrong_dimension() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors(vec![vec![0.5, 0.5]]); // 2-dim, not 3
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("dimension")));
}

#[test]
fn test_moead_validate_das_dennis_p_zero() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(0);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let moead = MoeaDGa::<ThreeObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    assert!(moead.validate().is_ok());
}

// ===== Run() integration tests =====

fn build_test_moead(
    population_size: usize,
    max_generations: usize,
    scalarization: ScalarizationFn,
) -> MoeaDGa<ThreeObjRangeChromosome> {
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_weight_vectors_auto(4)
        .with_scalarization(scalarization)
        .with_neighborhood_size(5)
        .with_max_neighbor_replacements(2);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(42);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    MoeaDGa::<ThreeObjRangeChromosome>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("MoeaD build should succeed with all required builders called")
}

#[test]
fn test_moead_run_produces_pareto_front() {
    let mut moead = build_test_moead(15, 10, ScalarizationFn::Tchebycheff);
    let result = moead.run();
    assert!(result.is_ok(), "MoeaD run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All ParetoFront members must be rank 0");
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_moead_run_with_pbi() {
    let mut moead = build_test_moead(15, 10, ScalarizationFn::Pbi { theta: 5.0 });
    let result = moead.run();
    assert!(result.is_ok(), "MoeaD run with PBI should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "PBI run should produce a non-empty front");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0);
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_moead_run_with_custom_weight_vectors() {
    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.5, 0.5, 0.0],
        vec![0.5, 0.0, 0.5],
        vec![0.0, 0.5, 0.5],
        vec![0.34, 0.33, 0.33],
    ];
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(7)
        .with_max_generations(8)
        .with_weight_vectors(custom)
        .with_scalarization(ScalarizationFn::Tchebycheff)
        .with_neighborhood_size(3)
        .with_max_neighbor_replacements(2);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(7);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut moead = MoeaDGa::<ThreeObjRangeChromosome>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("build should succeed with custom weight vectors");

    let result = moead.run();
    assert!(result.is_ok(), "run with custom weight vectors: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
}

/// Counter-based observer to verify that lifecycle hooks fire.
#[derive(Default)]
struct CountingObserver {
    sort_count: AtomicUsize,
    pareto_count: AtomicUsize,
}

impl MoeaDObserver<ThreeObjRangeChromosome> for CountingObserver {
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
fn test_moead_run_invokes_observer_hooks() {
    let observer = Arc::new(CountingObserver::default());
    let observer_handle = observer.clone();

    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(15)
        .with_max_generations(5)
        .with_weight_vectors_auto(4)
        .with_neighborhood_size(5)
        .with_max_neighbor_replacements(2);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(123);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut moead = MoeaDGa::<ThreeObjRangeChromosome>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(
            observer as Arc<dyn MoeaDObserver<ThreeObjRangeChromosome> + Send + Sync>,
        )
        .build()
        .expect("build should succeed");

    moead.run().expect("run should succeed");

    assert_eq!(observer_handle.pareto_count.load(Ordering::Relaxed), 5);
    assert_eq!(observer_handle.sort_count.load(Ordering::Relaxed), 5);
}

#[test]
fn test_moead_run_rejects_differential_mutation() {
    use genetic_algorithms::operations::Mutation;
    let mut moead = build_test_moead(15, 3, ScalarizationFn::Tchebycheff);
    moead.ga_config = moead.ga_config.with_mutation_method(Mutation::Differential { f: None }).with_mutation_probability_max(1.0);
    let result = moead.run();
    assert!(matches!(result, Err(GaError::MutationError(ref msg)) if msg.contains("Differential mutation is not supported in MOEA/D")));
}

#[cfg(feature = "logging")]
#[test]
fn test_moead_log_observer() {
    let mut moead = build_test_moead(15, 3, ScalarizationFn::Tchebycheff)
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn MoeaDObserver<ThreeObjRangeChromosome> + Send + Sync>,
        );

    let result = moead.run();
    assert!(
        result.is_ok(),
        "MoeaD run with LogObserver should succeed: {:?}",
        result.err()
    );
    let front = result.unwrap();
    assert!(!front.is_empty(), "front should be non-empty under LogObserver");
}
