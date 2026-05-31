use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::traits::VectorFitness;

// --- Custom test chromosome that populates fitness_values in calculate_fitness() ---

use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
struct MoTestChromosome {
    dna: Vec<genetic_algorithms::genotypes::Binary>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl genetic_algorithms::traits::ChromosomeT for MoTestChromosome {
    type Gene = genetic_algorithms::genotypes::Binary;

    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, v: f64) -> &mut Self {
        self.fitness = v;
        self
    }
    fn set_age(&mut self, _: usize) -> &mut Self {
        self
    }
    fn age(&self) -> usize {
        0
    }
    /// Produces 3 objectives from the binary DNA (sum, -sum, sum-of-squares).
    fn calculate_fitness(&mut self) {
        let sum: f64 = self.dna.iter().map(|g| if g.value { 1.0 } else { 0.0 }).sum();
        let sq: f64 = self.dna.iter().map(|g| if g.value { 1.0 } else { 0.0 }).sum::<f64>().powi(2);
        self.fitness_values = vec![sum, -sum, sq];
        self.fitness = sum;
    }
}

impl genetic_algorithms::traits::LinearChromosome for MoTestChromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned();
        self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

impl VectorFitness for MoTestChromosome {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }
    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl genetic_algorithms::operations::mutation::ValueMutable for MoTestChromosome {}
impl genetic_algorithms::traits::OperatorCompat for MoTestChromosome {}

// --- A chromosome that always produces 2 fitness values (for mismatch test) ---

#[derive(Debug, Clone, Default)]
struct TwoObjectiveChromosome {
    dna: Vec<genetic_algorithms::genotypes::Binary>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl genetic_algorithms::traits::ChromosomeT for TwoObjectiveChromosome {
    type Gene = genetic_algorithms::genotypes::Binary;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        let sum: f64 = self.dna.iter().map(|g| if g.value { 1.0 } else { 0.0 }).sum();
        // Always produces only 2 values, even when num_objectives=3
        self.fitness_values = vec![sum, -sum];
        self.fitness = sum;
    }
}

impl genetic_algorithms::traits::LinearChromosome for TwoObjectiveChromosome {
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
impl genetic_algorithms::traits::OperatorCompat for TwoObjectiveChromosome {}

// --- validate() error-path tests ---

#[test]
fn test_nsga2_validate_no_init_fn() {
    let config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config);

    let result = nsga2.validate();
    assert!(result.is_err());
}

#[test]
fn test_nsga2_validate_zero_objectives() {
    let config = Nsga2Configuration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config).with_initialization_fn(|_, _| vec![]);

    let result = nsga2.validate();
    assert!(result.is_err());
}

#[test]
fn test_nsga2_validate_population_too_small() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(1)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);

    let result = nsga2.validate();
    assert!(result.is_err());
}

/// Runtime objective-count mismatch: engine expects 3 objectives but chromosome
/// produces only 2 values in fitness_values(). run() must return an error.
#[test]
fn test_nsga2_run_rejects_mismatched_objective_count() {
    use genetic_algorithms::initializers::binary_random_initialization;
    use genetic_algorithms::ChromosomeLength;
    use genetic_algorithms::traits::ConfigurationT;

    let config = Nsga2Configuration::new()
        .with_num_objectives(3)  // expects 3
        .with_population_size(8)
        .with_max_generations(1);
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(4));

    let mut nsga2 = Nsga2Ga::<TwoObjectiveChromosome>::new(config, ga_config)
        .with_initialization_fn(|n, _| binary_random_initialization(n, None));

    // run() should fail because fitness_values().len() == 2 != num_objectives == 3
    let result = nsga2.run();
    assert!(
        matches!(result, Err(GaError::InvalidNsga2Configuration(_))),
        "Expected InvalidNsga2Configuration for mismatched objective count, got: {:?}",
        result
    );
}
