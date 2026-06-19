//! Divan benchmark: Adaptive Operator Selection (AOS) on/off comparison.
//!
//! Compares `Ga` with crossover portfolio + `AosStrategy` (ON) vs plain `Ga`
//! with a single crossover method (OFF) on the 10-dimensional Rastrigin function.

use genetic_algorithms::aos::AosStrategy;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::ChromosomeLength;

const DIMS: usize = 10;
const POPULATION_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 30;

/// Rastrigin function: f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))
///
/// A = 10.0, bounds [-5.12, 5.12] per dimension.
/// Global minimum is 0.0 at x_i = 0 for all i.
fn rastrigin(genes: &[RangeGenotype<f64>]) -> f64 {
    let a = 10.0_f64;
    let n = genes.len() as f64;
    a * n
        + genes
            .iter()
            .map(|g| {
                let x = g.value();
                x * x - a * (2.0 * std::f64::consts::PI * x).cos()
            })
            .sum::<f64>()
}

mod aos {
    use super::*;

    #[divan::bench]
    fn aos_on(bencher: divan::Bencher) {
        bencher.bench(|| {
            let alleles = vec![RangeGenotype::new(0, vec![(-5.12_f64, 5.12)], 0.0_f64)];
            let alleles_clone = alleles.clone();

            let mut ga = Ga::<RangeChromosome<f64>>::new()
                .with_chromosome_length(ChromosomeLength::Fixed(DIMS))
                .with_population_size(POPULATION_SIZE)
                .with_initialization_fn(move |genes_per_chromosome, _| {
                    range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
                })
                .with_fitness_fn(rastrigin)
                .with_selection_method(Selection::Tournament)
                .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
                .with_survivor_method(Survivor::Fitness)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(MAX_GENERATIONS)
                .with_crossover_portfolio(vec![
                    Crossover::SinglePoint,
                    Crossover::MultiPoint,
                    Crossover::Uniform,
                ])
                .with_aos_strategy(AosStrategy::pm_default())
                .with_reward_window(5)
                .build()
                .expect("Failed to build GA configuration");

            let _ = ga.run();
        });
    }

    #[divan::bench]
    fn aos_off(bencher: divan::Bencher) {
        bencher.bench(|| {
            let alleles = vec![RangeGenotype::new(0, vec![(-5.12_f64, 5.12)], 0.0_f64)];
            let alleles_clone = alleles.clone();

            let mut ga = Ga::<RangeChromosome<f64>>::new()
                .with_chromosome_length(ChromosomeLength::Fixed(DIMS))
                .with_population_size(POPULATION_SIZE)
                .with_initialization_fn(move |genes_per_chromosome, _| {
                    range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
                })
                .with_fitness_fn(rastrigin)
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
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
