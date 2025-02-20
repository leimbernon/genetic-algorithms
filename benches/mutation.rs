use criterion::{criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration};

use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::Rng;
use pprof::criterion::{Output, PProfProfiler};

use genetic_algorithms::traits::{GeneT, ChromosomeT};
use genetic_algorithms::operations::mutation::swap::swap;
use genetic_algorithms::operations::mutation::inversion::inversion;
use genetic_algorithms::operations::mutation::scramble::scramble;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Gene {
    pub id: i32,
}
impl GeneT for Gene {
    fn get_id(&self) -> i32 {
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
    pub age: i32,
    pub fitness_fn: FitnessFnWrapper<Gene>,
}
impl ChromosomeT for SimpleChromosome {
    type Gene = Gene;

    fn get_dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn get_fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }
    fn set_age(&mut self, age: i32) -> &mut Self {
        self.age = age;
        self
    }
    fn get_age(&self) -> i32 {
        self.age
    }
    fn set_dna(&mut self, dna: &[Self::Gene]) -> &mut Self {
        self.dna = dna.to_vec();
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

fn setup_chromosome(gene_length: usize) -> SimpleChromosome {
    SimpleChromosome {
        fitness: rand::thread_rng().gen_range(0.0..1.0),
        dna: (0..gene_length)
            .map(|_| Gene { id: rand::thread_rng().gen_range(0..255) })
            .collect(),
        age: rand::thread_rng().gen_range(0..100),
        fitness_fn: FitnessFnWrapper::default(),
    }
}

fn benchmark_mutation_methods(c: &mut Criterion) {
    let gene_lengths = vec![10, 100, 1000];

    let mut group = c.benchmark_group("mutation_methods");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &gene_length in &gene_lengths {
        let chromosome = setup_chromosome(gene_length);

        // Benchmark for swap mutation
        group.bench_with_input(
            BenchmarkId::new("swap mutation", format!("genes_{}", gene_length)),
            &chromosome,
            |b, chromosome| {
                b.iter(|| {
                    let _ = swap(&mut chromosome.clone());
                });
            },
        );

        // Benchmark for inversion mutation
        group.bench_with_input(
            BenchmarkId::new("inversion mutation", format!("genes_{}", gene_length)),
            &chromosome,
            |b, chromosome| {
                b.iter(|| {
                    let _ = inversion(&mut chromosome.clone());
                });
            },
        );

         // Benchmark for scramble mutation
         group.bench_with_input(
             BenchmarkId::new("scramble mutation", format!("genes_{}", gene_length)),
             &chromosome,
             |b, chromosome| {
                b.iter(|| {
                    let _ = scramble(&mut chromosome.clone());
                });
            },
        );
    }
     group.finish();
} 

// Configure the benchmark group with Criterion and PProf
criterion_group! {
    name = mutation_benchmarks;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_mutation_methods
}

criterion_main!(mutation_benchmarks);