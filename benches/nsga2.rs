use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};

use genetic_algorithms::nsga2::crowding_distance::assign_crowding_distance;
use genetic_algorithms::nsga2::non_dominated_sort::non_dominated_sort;
use rand::Rng;

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

/// Generate `n` individuals, each with `m` random objective values in [0, 1).
#[cfg(not(tarpaulin_include))]
fn random_objectives(n: usize, m: usize) -> Vec<Vec<f64>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| (0..m).map(|_| rng.random_range(0.0..1.0)).collect())
        .collect()
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn benchmark_non_dominated_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("non_dominated_sort");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let population_sizes = vec![10, 50, 100, 500];
    let num_objectives = vec![2, 3, 5];

    for &pop_size in &population_sizes {
        for &n_obj in &num_objectives {
            let objectives = random_objectives(pop_size, n_obj);
            let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();

            group.throughput(Throughput::Elements(pop_size as u64));

            group.bench_with_input(
                BenchmarkId::new(
                    "non_dominated_sort",
                    format!("pop_{}_obj_{}", pop_size, n_obj),
                ),
                &refs,
                |b, refs| {
                    b.iter(|| {
                        let _ = non_dominated_sort(refs);
                    });
                },
            );
        }
    }

    group.finish();
}

#[cfg(not(tarpaulin_include))]
fn benchmark_crowding_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("crowding_distance");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let population_sizes = vec![10, 50, 100, 500];
    let num_objectives = vec![2, 3, 5];

    for &pop_size in &population_sizes {
        for &n_obj in &num_objectives {
            let objectives = random_objectives(pop_size, n_obj);
            let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();

            group.throughput(Throughput::Elements(pop_size as u64));

            group.bench_with_input(
                BenchmarkId::new(
                    "assign_crowding_distance",
                    format!("pop_{}_obj_{}", pop_size, n_obj),
                ),
                &refs,
                |b, refs| {
                    b.iter(|| {
                        let mut crowding = vec![0.0_f64; pop_size];
                        assign_crowding_distance(refs, &mut crowding);
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group! {
    name = nsga2_benchmarks;
    config = Criterion::default();
    targets = benchmark_non_dominated_sort, benchmark_crowding_distance
}

criterion_main!(nsga2_benchmarks);
