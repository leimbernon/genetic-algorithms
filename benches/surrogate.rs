//! Divan benchmark: Surrogate-assisted fitness prescreening on/off comparison.
//!
//! Compares `Ga` with a `SurrogateModel` (ON) vs plain `Ga` (OFF) on the
//! 10-dimensional Rastrigin function. The surrogate is a cheap linear
//! approximation that prescreens offspring before true evaluation.

use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig,
    StoppingConfig,
};
use genetic_algorithms::{ChromosomeLength, SurrogateModel};

const DIMS: usize = 10;
const POPULATION_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 30;
const PRESCREENING_FRACTION: f64 = 0.4;

// ---------------------------------------------------------------------------
// Surrogate model: cheap linear approximation
// ---------------------------------------------------------------------------

/// Cheap linear surrogate — returns the negated weighted sum of gene values.
///
/// For minimization, chromosomes closer to the origin (global minimum) score
/// higher via the negated l1-norm.
struct LinearSurrogate {
    coeffs: Vec<f64>,
}

impl SurrogateModel<RangeChromosome<f64>> for LinearSurrogate {
    fn predict(&self, chromosome: &RangeChromosome<f64>) -> f64 {
        -chromosome
            .dna()
            .iter()
            .zip(self.coeffs.iter())
            .map(|(g, c): (&RangeGenotype<f64>, &f64)| g.value() * c)
            .sum::<f64>()
    }
}

// ---------------------------------------------------------------------------
// Fitness function
// ---------------------------------------------------------------------------

/// Rastrigin function: f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))
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

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

mod surrogate_benchmark {
    use super::*;

    #[divan::bench]
    fn with_surrogate(bencher: divan::Bencher) {
        let model = Arc::new(LinearSurrogate {
            coeffs: vec![1.0; DIMS],
        });

        bencher.bench(|| {
            let alleles = vec![RangeGenotype::new(0, vec![(-5.12_f64, 5.12)], 0.0_f64)];
            let alleles_clone = alleles.clone();
            let model_clone = Arc::clone(&model);

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
                .with_surrogate(model_clone, PRESCREENING_FRACTION)
                .build()
                .expect("Failed to build GA configuration");

            let _ = ga.run();
        });
    }

    #[divan::bench]
    fn without_surrogate(bencher: divan::Bencher) {
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
