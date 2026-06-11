use std::borrow::Cow;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use genetic_algorithms::cellular::{CellularConfiguration, CellularEngine, Neighborhood, UpdateMode};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
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

fn bench_cellular_neighborhoods(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellular_ga");
    group.sample_size(10);

    for (label, neighborhood) in [
        ("von_neumann", Neighborhood::VonNeumann),
        ("moore", Neighborhood::Moore),
        ("compact_r2", Neighborhood::CompactR2),
        ("linear", Neighborhood::Linear),
    ] {
        group.bench_with_input(
            BenchmarkId::new("async_sphere_5d", label),
            &(),
            |b, _| {
                b.iter(|| {
                    let config = CellularConfiguration::default()
                        .with_grid(8, 8)
                        .with_neighborhood(neighborhood.clone())
                        .with_update_mode(UpdateMode::Asynchronous)
                        .with_max_generations(100)
                        .with_mutation(genetic_algorithms::operations::Mutation::Gaussian { sigma: Some(0.3) });
                    let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
                    engine.run()
                });
            },
        );
    }

    group.finish();
}

fn bench_cellular_sync_vs_async(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellular_sync_vs_async");
    group.sample_size(10);

    for (label, update_mode) in [
        ("synchronous", UpdateMode::Synchronous),
        ("asynchronous", UpdateMode::Asynchronous),
    ] {
        group.bench_with_input(
            BenchmarkId::new("moore_sphere_5d", label),
            &(),
            |b, _| {
                b.iter(|| {
                    let config = CellularConfiguration::default()
                        .with_grid(8, 8)
                        .with_neighborhood(Neighborhood::Moore)
                        .with_update_mode(update_mode.clone())
                        .with_max_generations(100)
                        .with_mutation(genetic_algorithms::operations::Mutation::Gaussian { sigma: Some(0.3) });
                    let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
                    engine.run()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_cellular_neighborhoods, bench_cellular_sync_vs_async);
criterion_main!(benches);
