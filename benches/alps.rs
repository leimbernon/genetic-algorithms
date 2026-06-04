use std::borrow::Cow;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::de::{DeConfiguration, DeEngine, DeMutationStrategy};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::traits::LinearChromosome;
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

fn bench_alps_vs_de(c: &mut Criterion) {
    let mut group = c.benchmark_group("alps_vs_de");
    group.sample_size(10);

    // ALPS with 5 layers × 20 individuals each = 100 total
    group.bench_with_input(
        BenchmarkId::new("sphere_5d", "alps_fibonacci"),
        &(),
        |b, _| {
            b.iter(|| {
                let config = AlpsConfiguration::default()
                    .with_n_layers(5)
                    .with_layer_size(20)
                    .with_age_scheme(AlpsAgeScheme::Fibonacci)
                    .with_age_gap(5)
                    .with_injection_interval(10)
                    .with_max_generations(100)
                    .with_mutation_sigma(0.3);
                let mut engine = AlpsEngine::new(config, |n| make_pop(n, 5), sphere);
                engine.run()
            });
        },
    );

    // DE with equivalent population size of 100
    group.bench_with_input(
        BenchmarkId::new("sphere_5d", "de_rand1"),
        &(),
        |b, _| {
            b.iter(|| {
                let config = DeConfiguration::default()
                    .with_population_size(100)
                    .with_max_generations(100)
                    .with_mutation_strategy(DeMutationStrategy::Rand1)
                    .with_mutation_factor(0.8)
                    .with_crossover_rate(0.9);
                let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
                engine.run()
            });
        },
    );

    group.finish();
}

fn bench_alps_age_schemes(c: &mut Criterion) {
    let mut group = c.benchmark_group("alps_age_schemes");
    group.sample_size(10);

    for (label, scheme) in [
        ("linear", AlpsAgeScheme::Linear),
        ("fibonacci", AlpsAgeScheme::Fibonacci),
        ("polynomial", AlpsAgeScheme::Polynomial),
    ] {
        group.bench_with_input(
            BenchmarkId::new("sphere_5d", label),
            &(),
            |b, _| {
                b.iter(|| {
                    let config = AlpsConfiguration::default()
                        .with_n_layers(4)
                        .with_layer_size(15)
                        .with_age_scheme(scheme.clone())
                        .with_age_gap(5)
                        .with_max_generations(100)
                        .with_mutation_sigma(0.3);
                    let mut engine = AlpsEngine::new(config, |n| make_pop(n, 5), sphere);
                    engine.run()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_alps_vs_de, bench_alps_age_schemes);
criterion_main!(benches);
