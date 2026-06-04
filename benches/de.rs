use std::borrow::Cow;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::de::{DeConfiguration, DeEngine, DeMutationStrategy};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::LinearChromosome;
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
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

criterion_group!(benches, bench_mutation_strategies);
criterion_main!(benches);
