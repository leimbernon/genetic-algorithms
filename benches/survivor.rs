use criterion::{criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration};

use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::Rng;
use pprof::criterion::{Output, PProfProfiler};

use genetic_algorithms::traits::{GeneT, ChromosomeT};
use genetic_algorithms::operations::survivor::age::age_based;
use genetic_algorithms::operations::survivor::fitness::fitness_based;


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

fn setup_population(population_size: usize, gene_length: usize) -> Vec<SimpleChromosome> {
    (0..population_size)
        .map(|_| SimpleChromosome {
            fitness: rand::thread_rng().gen_range(0.0..1.0),
            dna: (0..gene_length)
                .map(|_| Gene { id: rand::thread_rng().gen_range(0..255) })
                .collect(),
            age: rand::thread_rng().gen_range(0..100),
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect()
}

// Benchmark the survivor methods
fn benchmark_survivor_methods(c: &mut Criterion) {

    let population_size = 1000;
    let gene_lengths = vec![10, 100, 1000];

    let mut group = c.benchmark_group("survivor_methods");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &gene_length in &gene_lengths {
        let chromosomes = setup_population(population_size, gene_length);

        // Benchmark for age survivor
        group.bench_with_input(BenchmarkId::new("age survivor", format!("genes_{}", gene_length)), &gene_length, |b, _| {
            b.iter(|| {
                let mut chromosomes = chromosomes.clone();
                age_based(&mut chromosomes, population_size);
            });
        });

         // Benchmark for fitness survivor
        group.bench_with_input(BenchmarkId::new("fitness survivor", format!("genes_{}", gene_length)), &gene_length, |b, _| {
            b.iter(|| {
                let mut chromosomes = chromosomes.clone();
                let limit_configuration = genetic_algorithms::configuration::LimitConfiguration::default();
                fitness_based(&mut chromosomes, population_size, limit_configuration);
            });
        });
    } 

    group.finish();
} 

// Configure the benchmark group with Criterion and PProf
criterion_group! {
    name = survivor_benchmarks;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_survivor_methods
}

criterion_main!(survivor_benchmarks);