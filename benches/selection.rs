use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::operations::selection::fitness_proportionate::roulette_wheel_selection;
use genetic_algorithms::operations::selection::fitness_proportionate::stochastic_universal_sampling;
use genetic_algorithms::operations::selection::random::random;
use genetic_algorithms::operations::selection::tournament::tournament;
use genetic_algorithms::traits::{ChromosomeT, GeneT};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Gene {
    pub id: i32,
}
impl GeneT for Gene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SimpleChromosome {
    dna: Vec<Gene>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<Gene>,
}
impl ChromosomeT for SimpleChromosome {
    type Gene = Gene;
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }
    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }
    fn age(&self) -> usize {
        self.age
    }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
    fn calculate_fitness(&mut self) {
        self.fitness = 0.0;
    }
}

// Setup function to create a population with configurable size and gene length
#[cfg(not(tarpaulin_include))]
fn setup_population(population_size: usize, gene_length: usize) -> Vec<SimpleChromosome> {
    let mut rng = rand::rng();
    (0..population_size)
        .map(|_| SimpleChromosome {
            fitness: rng.random_range(0.0..=1.0),
            dna: (0..gene_length)
                .map(|_| Gene {
                    id: rng.random_range(0..255),
                })
                .collect(),
            age: rng.random_range(0..=100),
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect()
}

// Benchmark function with parameterized population and gene length
#[cfg(not(tarpaulin_include))]
fn benchmark_selection_methods(c: &mut Criterion) {
    let population_sizes = vec![10, 100, 1000];
    let gene_lengths = vec![10, 100, 1000];
    let tournament_threads = vec![1, 2, 4, 8];

    let mut group = c.benchmark_group("selection_methods");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &population_size in &population_sizes {
        for &gene_length in &gene_lengths {
            let chromosomes = setup_population(population_size, gene_length);

            group.throughput(Throughput::Elements(population_size as u64));

            // Benchmark random selection
            group.bench_with_input(
                BenchmarkId::new(
                    "random selection",
                    format!("population_{}_genes_{}", population_size, gene_length),
                ),
                &chromosomes,
                |b, chromosomes| {
                    b.iter(|| {
                        let _ = random(chromosomes);
                    });
                },
            );

            // Benchmark roulette wheel selection
            group.bench_with_input(
                BenchmarkId::new(
                    "roulette wheel selection",
                    format!("population_{}_genes_{}", population_size, gene_length),
                ),
                &chromosomes,
                |b, chromosomes| {
                    b.iter(|| {
                        let _ = roulette_wheel_selection(chromosomes);
                    });
                },
            );

            // Benchmark stochastic universal sampling
            group.bench_with_input(
                BenchmarkId::new(
                    "stochastic universal sampling",
                    format!("population_{}_genes_{}", population_size, gene_length),
                ),
                &chromosomes,
                |b, chromosomes| {
                    b.iter(|| {
                        let _ = stochastic_universal_sampling(chromosomes, 50);
                    });
                },
            );

            // Benchmarks for tournament selection with different threads
            for &threads in &tournament_threads {
                group.bench_with_input(
                    BenchmarkId::new(
                        format!("tournament {} threads", threads),
                        format!("population_{}_genes_{}", population_size, gene_length),
                    ),
                    &chromosomes,
                    |b, chromosomes| {
                        b.iter(|| {
                            let _ = tournament(chromosomes, 5, threads);
                        });
                    },
                );
            }
        }
    }
    group.finish();
}

// Create the benchmark group (profiler removed due to criterion version mismatch with pprof)
criterion_group! {
    name = selection_benchmarks;
    config = Criterion::default();
    targets = benchmark_selection_methods
}

criterion_main!(selection_benchmarks);
