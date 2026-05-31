//! IBEA engine tests.

use std::borrow::Cow;
use genetic_algorithms::traits::{ConfigurationT, ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::ibea::configuration::IbeaConfiguration;
use genetic_algorithms::ibea::IbeaGa;

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
fn test_ibea_validate_no_init_fn() {
    let config = IbeaConfiguration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_zero_objectives() {
    let config = IbeaConfiguration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_one_objective() {
    let config = IbeaConfiguration::new().with_num_objectives(1);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_population_too_small() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = ibea.validate();
    assert!(matches!(result, Err(GaError::InvalidIbeaConfiguration(_))));
}

#[test]
fn test_ibea_validate_mismatched_objective_directions() {
    let config = IbeaConfiguration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![genetic_algorithms::ibea::configuration::ObjectiveDirection::Minimize]);
    let ga_config = GaConfiguration::default();
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
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
    let ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    assert!(ibea.validate().is_ok());
}

// --- Run integration tests ---

fn build_test_ibea(pop_size: usize, max_gens: usize) -> IbeaGa<TwoObjRangeChromosome> {
    let config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(pop_size)
        .with_max_generations(max_gens);
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3));

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .build()
        .expect("Failed to build IbeaGa")
}

#[test]
fn test_ibea_run_produces_pareto_front() {
    let mut ibea = build_test_ibea(8, 5);
    let front = ibea.run().expect("IBEA run failed");
    assert!(!front.is_empty(), "Pareto front must not be empty");
    assert!(front.individuals.iter().any(|ind| ind.rank == 0));
}

#[test]
fn test_ibea_run_small_population() {
    let mut ibea = build_test_ibea(4, 3);
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
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(2));

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            genetic_algorithms::initializers::range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn IbeaObserver<TwoObjRangeChromosome> + Send + Sync>
        )
        .build()
        .expect("Failed to build IbeaGa");

    let result = ibea.run();
    assert!(result.is_ok(), "IBEA observer test run failed: {:?}", result.err());
}

#[test]
fn test_ibea_run_rejects_mismatched_objective_count() {
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::ChromosomeLength;

    let config = IbeaConfiguration::new()
        .with_num_objectives(3) // expects 3, chromosome provides 2
        .with_population_size(8)
        .with_max_generations(1);
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(2));

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<TwoObjRangeChromosome>::new(config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| range_random_initialization(n, Some(&alleles_clone)));

    let result = ibea.run();
    assert!(
        matches!(result, Err(GaError::InvalidIbeaConfiguration(_))),
        "Expected InvalidIbeaConfiguration for mismatched objective count, got: {:?}",
        result
    );
}
