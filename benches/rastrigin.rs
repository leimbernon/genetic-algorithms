use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use rand::Rng;

// ---------------------------------------------------------------------------
// Rastrigin fitness function (inline — not exported from the library)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn build_rastrigin_ga(
    population_size: usize,
    dims: usize,
    max_generations: usize,
) -> Ga<RangeChromosome<f64>> {
    let mut rng = rand::rng();
    let chromosomes: Vec<RangeChromosome<f64>> = (0..population_size)
        .map(|_| {
            let mut c = RangeChromosome::<f64>::new();
            c.dna = (0..dims)
                .map(|j| {
                    RangeGenotype::new(
                        j as i32,
                        vec![(-5.12_f64, 5.12_f64)],
                        rng.random_range(-5.12_f64..5.12_f64),
                    )
                })
                .collect();
            c
        })
        .collect();

    let population = Population::new(chromosomes);

    Ga::new()
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(max_generations)
        .with_fitness_fn(rastrigin)
        .with_population(population)
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

mod rastrigin {
    use super::*;

    /// args = dims; pop_size=500, max_generations=50 fixed
    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 20, 50])]
    fn ga_run(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| build_rastrigin_ga(500, dim, 50))
            .bench_values(|mut ga| {
                let _ = ga.run();
            });
    }
}

fn main() {
    divan::main();
}
