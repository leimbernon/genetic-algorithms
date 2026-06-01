use criterion::{
    criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration,
};

use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::operations::mutation::ValueMutable;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, GeneT, LinearChromosome, MutationConfig,
    OperatorCompat, SelectionConfig, StoppingConfig,
};
use rand::Rng;
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Chromosome / Gene types (must implement ValueMutable for Ga)
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SimpleChromosome {
    dna: Vec<Gene>,
    fitness: f64,
    age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    fitness_fn: FitnessFnWrapper<Gene>,
}
impl ChromosomeT for SimpleChromosome {
    type Gene = Gene;
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
    fn calculate_fitness(&mut self) {
        self.fitness = 0.0;
        for (i, gene) in self.dna.iter().enumerate() {
            self.fitness += f64::from(gene.id() * i as i32);
        }
    }
}
impl LinearChromosome for SimpleChromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
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
}
impl ValueMutable for SimpleChromosome {}
impl OperatorCompat for SimpleChromosome {}

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn setup_population(population_size: usize, gene_length: usize) -> Vec<SimpleChromosome> {
    let mut rng = rand::rng();
    (0..population_size)
        .map(|_| {
            let mut c = SimpleChromosome {
                fitness: 0.0,
                dna: (0..gene_length)
                    .map(|_| Gene {
                        id: rng.random_range(0..255),
                    })
                    .collect(),
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
            };
            c.calculate_fitness();
            c
        })
        .collect()
}

/// Build a ready-to-run `Ga` instance.  Uses `with_population` so we skip
/// the initialization function and go straight to the evolutionary loop.
#[cfg(not(tarpaulin_include))]
fn build_ga(
    population_size: usize,
    gene_length: usize,
    max_generations: usize,
) -> Ga<SimpleChromosome> {
    let chromosomes = setup_population(population_size, gene_length);
    let population = Population::new(chromosomes);

    Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(max_generations)
        .with_population(population)
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn benchmark_ga_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("ga_run");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    // Vary population size while keeping generations and gene length modest
    let configs: Vec<(usize, usize, usize)> = vec![
        // (population_size, gene_length, max_generations)
        (20, 10, 10),
        (50, 10, 10),
        (100, 10, 10),
        (50, 50, 10),
        (50, 10, 50),
    ];

    for &(pop_size, gene_len, max_gen) in &configs {
        group.bench_with_input(
            BenchmarkId::new(
                "Ga::run",
                format!("pop_{}_genes_{}_gen_{}", pop_size, gene_len, max_gen),
            ),
            &(pop_size, gene_len, max_gen),
            |b, &(ps, gl, mg)| {
                b.iter_batched(
                    || build_ga(ps, gl, mg),
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
    name = ga_benchmarks;
    config = Criterion::default();
    targets = benchmark_ga_run
}

criterion_main!(ga_benchmarks);
