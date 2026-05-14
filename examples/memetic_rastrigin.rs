/*!
# Memetic Algorithm: Rastrigin with Local Search

This example demonstrates how to use the `genetic_algorithms` library to minimize the Rastrigin
function using a memetic algorithm — a GA augmented with HillClimbing local search refinement.

Features demonstrated:
- Range<f64> chromosomes (continuous representation)
- HillClimbing local search operator (memetic algorithm)
- AllOffspring application strategy (every offspring refined)
- Lamarckian mode (DNA and fitness updated)
- Comparison with standard GA (no local search)

Run with:
```sh
cargo run --example memetic_rastrigin
```
*/

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::LocalSearchConfiguration;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::local_search::{LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, ElitismConfig, LocalSearchConfig, MutationConfig,
    SelectionConfig, StoppingConfig,
};

/// Rastrigin function: f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))  where A = 10
fn rastrigin_fitness(dna: &[RangeGenotype<f64>]) -> f64 {
    let a = 10.0;
    let n = dna.len() as f64;
    a * n
        + dna.iter()
            .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
            .sum::<f64>()
}

fn run_ga(name: &str, use_local_search: bool) -> f64 {
    const DIMENSIONS: usize = 5;
    const POP_SIZE: usize = 100;
    const MAX_GENERATIONS: usize = 200;

    // Allele range: each dimension in [-5.12, 5.12]
    let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // Build the GA with common configuration
    let mut builder = Ga::<RangeChromosome<f64>>::new()
        .with_genes_per_chromosome(DIMENSIONS)
        .with_population_size(POP_SIZE)
        .with_initialization_fn(move |genes, _, _| {
            range_random_initialization(genes, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(rastrigin_fitness)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Gaussian)
        .with_mutation_step(0.5)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(MAX_GENERATIONS)
        .with_fitness_target(0.0)
        .with_number_of_couples(POP_SIZE)
        .with_elitism(2);

    // Conditionally add local search (memetic algorithm refinement)
    if use_local_search {
        builder = builder
            .with_local_search(LocalSearch::HillClimbing)
            .with_local_search_configuration(LocalSearchConfiguration {
                application_strategy: LocalSearchApplicationStrategy::AllOffspring,
                mode: LocalSearchMode::Lamarckian,
                ..Default::default()
            });
    }

    let mut ga = builder
        .with_logs(genetic_algorithms::configuration::LogLevel::Warn)
        .build()
        .expect("Invalid GA configuration");

    let population = ga
        .run()
        .expect("GA run failed");

    let best_fitness = population.best_chromosome.fitness();

    println!(
        "{} - Best fitness: {:.6} (terminated: {:?})",
        name, best_fitness, ga.termination_cause,
    );

    best_fitness
}

fn main() {
    println!("=== Memetic Algorithm: Rastrigin Minimization ===\n");

    let memetic_fitness = run_ga("Memetic GA (HillClimbing)", true);
    let standard_fitness = run_ga("Standard GA", false);

    println!();
    println!("Comparison:");
    println!("  Memetic GA best:   {:.6}", memetic_fitness);
    println!("  Standard GA best:  {:.6}", standard_fitness);
    if memetic_fitness < standard_fitness {
        println!("  => HillClimbing local search improved convergence!");
    } else {
        println!("  => Standard GA performed comparably (local search params may need tuning)");
    }
}
