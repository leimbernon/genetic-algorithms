use genetic_algorithms::benchmarks::BenchmarkFn;
use genetic_algorithms::benchmarks::Sphere;
use std::borrow::Cow;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::de::{DeConfiguration, DeEngine, DeMutationStrategy};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    let x: Vec<f64> = dna.iter().map(|g| g.value()).collect();
    Sphere::new(dna.len()).evaluate(&x)[0]
}

fn make_pop(n: usize, dim: usize) -> Vec<RangeChromosome<f64>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0))
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

fn bench_mutation_strategies(c: &mut Criterion) {
    let strategies = [
        ("rand1", DeMutationStrategy::Rand1),
        ("best1", DeMutationStrategy::Best1),
        ("current_to_best1", DeMutationStrategy::CurrentToBest1),
        ("rand2", DeMutationStrategy::Rand2),
        ("best2", DeMutationStrategy::Best2),
    ];

    let mut group = c.benchmark_group("de_mutation_strategies");
    group.sample_size(10);

    for (name, strategy) in strategies {
        group.bench_with_input(BenchmarkId::new("sphere_5d", name), &strategy, |b, strat| {
            b.iter(|| {
                let config = DeConfiguration::default()
                    .with_population_size(30)
                    .with_max_generations(100)
                    .with_mutation_strategy(strat.clone())
                    .with_problem_solving(ProblemSolving::Minimization);
                let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
                engine.run()
            });
        });
    }
    group.finish();
}

fn bench_de_vs_ga(c: &mut Criterion) {
    let mut group = c.benchmark_group("de_vs_ga");
    group.sample_size(10);

    // DE on sphere(5D) — per D-09: same problem and max_generations as GA
    group.bench_function("de_sphere_5d", |b| {
        b.iter(|| {
            let config = DeConfiguration::default()
                .with_population_size(30)
                .with_max_generations(100)
                .with_mutation_strategy(DeMutationStrategy::Rand1)
                .with_problem_solving(ProblemSolving::Minimization);
            let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    });

    // GA on sphere(5D) — per D-10: standard Ga<RangeChromosome<f64>> with default operators
    group.bench_function("ga_sphere_5d", |b| {
        b.iter(|| {
            let chromosomes = make_pop(30, 5);
            let population = Population::new(chromosomes);
            let mut ga = Ga::new()
                .with_population_size(30)
                .with_genes_per_chromosome(5)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(100)
                .with_fitness_fn(sphere)
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Gaussian)
                .with_survivor_method(Survivor::Fitness)
                .with_population(population)
                .build()
                .expect("valid GA config for sphere benchmark");
            let _ = ga.run().expect("GA run should succeed");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mutation_strategies, bench_de_vs_ga);
criterion_main!(benches);
