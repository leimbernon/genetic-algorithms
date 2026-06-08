use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

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

#[cfg(not(tarpaulin_include))]
fn benchmark_rastrigin(c: &mut Criterion) {
    let mut group = c.benchmark_group("rastrigin");

    let dims = vec![10usize, 20, 50];

    for &dim in &dims {
        group.bench_with_input(
            BenchmarkId::new("Ga::run", format!("pop_500_dim_{}", dim)),
            &dim,
            |b, &d| {
                b.iter_batched(
                    || build_rastrigin_ga(500, d, 50),
                    |mut ga| {
                        let _ = ga.run();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = rastrigin_benchmarks;
    config = Criterion::default();
    targets = benchmark_rastrigin
}

criterion_main!(rastrigin_benchmarks);
