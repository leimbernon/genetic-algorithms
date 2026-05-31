use std::borrow::Cow;
use genetic_algorithms::traits::{ConfigurationT, ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::traits::{CrossoverConfig, MutationConfig, SelectionConfig};

// 2-objective chromosome using RangeGenotype<i32>: f1=sum, f2=sum of (10-val)
#[derive(Debug, Clone, Default)]
struct ConstraintTestChromosome {
    dna: Vec<RangeGene<i32>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for ConstraintTestChromosome {
    type Gene = RangeGene<i32>;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        let f1: f64 = self.dna.iter().map(|g| g.value() as f64).sum();
        let f2: f64 = self.dna.iter().map(|g| (10 - g.value()) as f64).sum();
        self.fitness_values = vec![f1, f2];
        self.fitness = f1;
    }
}

impl LinearChromosome for ConstraintTestChromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for ConstraintTestChromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for ConstraintTestChromosome {}
impl genetic_algorithms::traits::OperatorCompat for ConstraintTestChromosome {}

#[test]
fn test_nsga2_with_constraints() {
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 10_i32)], 0)];
    let alleles_clone = alleles.clone();

    let constraint = |dna: &[RangeGene<i32>]| {
        let val = dna[0].value();
        (5.0 - val as f64).max(0.0)
    };

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(50)
        .with_max_generations(30);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3))
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut nsga2 = Nsga2Ga::<ConstraintTestChromosome>::new(nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_constraint_fns(vec![
            Box::new(constraint) as Box<dyn Fn(&[RangeGene<i32>]) -> f64 + Send + Sync>,
        ]);

    let result = nsga2.run();
    assert!(
        result.is_ok(),
        "NSGA-II with constraints should succeed, got: {:?}",
        result.err()
    );

    let front = result.unwrap();
    assert!(
        !front.individuals.is_empty(),
        "Pareto front should have individuals"
    );
}
