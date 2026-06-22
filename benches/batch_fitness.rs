//! Divan benchmark: Batch fitness evaluator vs per-call fitness function.
//!
//! Compares `Ga` with a `BatchFitnessEvaluator` (batch) vs `Ga` with a
//! per-chromosome fitness closure (per-call) on the 10-dimensional sphere function.

use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig,
    StoppingConfig,
};
use genetic_algorithms::{BatchFitnessEvaluator, ChromosomeLength};

const DIMS: usize = 10;
const POPULATION_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 30;

// ---------------------------------------------------------------------------
// Fitness functions
// ---------------------------------------------------------------------------

/// Sphere function: f(x) = sum(x_i^2)
fn sphere(dna: &[RangeGenotype<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

// ---------------------------------------------------------------------------
// Batch evaluator
// ---------------------------------------------------------------------------

/// Batch evaluator that computes sphere fitness for all chromosomes at once.
struct SphereBatch;

impl BatchFitnessEvaluator<RangeChromosome<f64>> for SphereBatch {
    fn evaluate_batch(&self, chromosomes: &[RangeChromosome<f64>]) -> Vec<f64> {
        chromosomes.iter().map(|c| sphere(c.dna())).collect()
    }
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

mod batch_fitness {
    use super::*;

    #[divan::bench]
    fn batch_evaluator(bencher: divan::Bencher) {
        bencher.bench(|| {
            let alleles = vec![RangeGenotype::new(0, vec![(-5.0_f64, 5.0)], 0.0_f64)];
            let alleles_clone = alleles.clone();

            let mut ga = Ga::<RangeChromosome<f64>>::new()
                .with_chromosome_length(ChromosomeLength::Fixed(DIMS))
                .with_population_size(POPULATION_SIZE)
                .with_initialization_fn(move |genes_per_chromosome, _| {
                    range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
                })
                .with_batch_evaluator(Arc::new(SphereBatch)
                    as Arc<dyn BatchFitnessEvaluator<RangeChromosome<f64>> + Send + Sync>)
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Swap)
                .with_survivor_method(Survivor::Fitness)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(MAX_GENERATIONS)
                .build()
                .expect("Failed to build GA configuration");

            let _ = ga.run();
        });
    }

    #[divan::bench]
    fn per_call_fitness(bencher: divan::Bencher) {
        bencher.bench(|| {
            let alleles = vec![RangeGenotype::new(0, vec![(-5.0_f64, 5.0)], 0.0_f64)];
            let alleles_clone = alleles.clone();

            let mut ga = Ga::<RangeChromosome<f64>>::new()
                .with_chromosome_length(ChromosomeLength::Fixed(DIMS))
                .with_population_size(POPULATION_SIZE)
                .with_initialization_fn(move |genes_per_chromosome, _| {
                    range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
                })
                .with_fitness_fn(sphere)
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Swap)
                .with_survivor_method(Survivor::Fitness)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(MAX_GENERATIONS)
                .build()
                .expect("Failed to build GA configuration");

            let _ = ga.run();
        });
    }
}

fn main() {
    divan::main();
}
