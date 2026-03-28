/*!
# Island Model Parallel Evolution (Rastrigin 20D)

This example demonstrates the island model genetic algorithm to minimize the Rastrigin
function in 20 dimensions — significantly harder than the single-population 5D example.
The island model helps escape local optima by maintaining diverse search strategies
across independent populations that periodically exchange individuals.

## GA Mode

`IslandGa` with 4 islands, Ring topology, and heterogeneous mutation rates.

## Features demonstrated
- Island model with migration topology
- Heterogeneous mutation rates per island
- CompositeObserver with IslandGaObserver-aware LogObserver and optional MetricsObserver

## Why Heterogeneous?

Each island uses a different mutation probability (0.01, 0.05, 0.10, 0.20) to balance
exploration versus exploitation across the search space:

- **Island 1 (0.01):** Low mutation — exploits promising regions found by other islands.
- **Island 2 (0.05):** Moderate mutation — balanced exploration and exploitation.
- **Island 3 (0.10):** Higher mutation — explores broadly, escapes local optima.
- **Island 4 (0.20):** Aggressive mutation — wide exploration, injects diversity.

Migration every 10 generations moves the best individuals between neighboring islands
in the Ring topology, allowing successful strategies to spread across the population.

## Key Configuration

- 4 islands, 50 chromosomes per island (200 total)
- Ring topology for migration
- Migration every 10 generations, 2 migrants per event
- 200 total generations
- Gaussian mutation, Uniform crossover, Tournament selection

## API Limitation

Note: `IslandGa` evolves all islands internally via `run()`. The methods
`evolve_islands_one_generation()` and `global_best()` are private, so per-migration
progress cannot be reported. Only the final global best fitness is printed.

## Run

```sh
cargo run --example island_model
```
*/

use std::sync::Arc;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::topology::MigrationTopology;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::ChromosomeT;
use genetic_algorithms::{CompositeObserver, IslandGaObserver, LogObserver};
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;

fn main() {
    // --- Problem parameters ---
    const DIMENSIONS: usize = 20;
    const POP_SIZE_PER_ISLAND: usize = 50;
    const MAX_GENERATIONS: usize = 200;
    const NUM_ISLANDS: usize = 4;
    const MIGRATION_INTERVAL: usize = 10;
    const MIGRATION_COUNT: usize = 2;

    // --- Fitness function: Rastrigin 20D (minimize toward 0.0 at origin) ---
    // f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))  where A = 10
    let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
        let a = 10.0;
        let n = dna.len() as f64;
        a * n
            + dna.iter()
                .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
                .sum::<f64>()
    };

    // --- Allele range: each dimension in [-5.12, 5.12] ---
    let alleles = vec![RangeGenotype::new(0, vec![(-5.12_f64, 5.12_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Island model configuration ---
    let island_config = IslandConfiguration::new()
        .with_num_islands(NUM_ISLANDS)
        .with_migration_interval(MIGRATION_INTERVAL)
        .with_migration_count(MIGRATION_COUNT)
        .with_topology(MigrationTopology::Ring);

    // --- Heterogeneous GA configs: 4 islands with different mutation probabilities ---
    // Low mutation islands exploit; high mutation islands explore broadly.
    let mutation_probs = [0.01_f64, 0.05, 0.10, 0.20];
    let ga_configs: Vec<GaConfiguration> = mutation_probs
        .iter()
        .map(|&prob| {
            let mut cfg = GaConfiguration::default();
            cfg.limit_configuration.population_size = POP_SIZE_PER_ISLAND;
            cfg.limit_configuration.genes_per_chromosome = DIMENSIONS;
            cfg.limit_configuration.problem_solving = ProblemSolving::Minimization;
            cfg.limit_configuration.max_generations = MAX_GENERATIONS;
            cfg.mutation_configuration.probability_max = Some(prob);
            cfg.mutation_configuration.method = Mutation::Gaussian;
            cfg.crossover_configuration.method = Crossover::Uniform;
            cfg.selection_configuration.method = Selection::Tournament;
            cfg.survivor = Survivor::Fitness;
            cfg
        })
        .collect();

    // --- Build composite observer (LogObserver always active; MetricsObserver when feature flag set) ---
    // CompositeObserver implements IslandGaObserver — forwards all island hooks to inner observers.
    let composite = CompositeObserver::new()
        .add(Arc::new(LogObserver));
    #[cfg(feature = "observer-metrics")]
    let composite = composite.add(Arc::new(MetricsObserver::new("island_model")));

    // --- Print problem summary ---
    println!("== Island Model: Rastrigin {}D Minimization ==", DIMENSIONS);
    println!(
        "Islands: {}, Population per island: {}, Total population: {}",
        NUM_ISLANDS,
        POP_SIZE_PER_ISLAND,
        NUM_ISLANDS * POP_SIZE_PER_ISLAND
    );
    println!(
        "Topology: Ring, Migration: every {} gens, {} migrants",
        MIGRATION_INTERVAL, MIGRATION_COUNT
    );
    println!(
        "Mutation probs per island: {:?}",
        mutation_probs
    );
    println!("Max generations: {}", MAX_GENERATIONS);
    println!("-------------------------------------------------------");

    // Note: IslandGa::run() handles all evolution internally; per-migration progress
    // is not exposed by the current API (evolve_islands_one_generation() and
    // global_best() are private methods). Only the final global best is available.

    // --- Build the island GA ---
    let mut island_ga =
        IslandGa::<RangeChromosome<f64>>::with_heterogeneous_configs(island_config, ga_configs)
            .with_alleles(alleles)
            .with_initialization_fn(move |n, _, _| {
                range_random_initialization(n, Some(&alleles_clone), Some(true))
            })
            .with_fitness_fn(fitness_fn)
            // Observer: CompositeObserver fans out to LogObserver (and MetricsObserver if feature enabled)
            .with_observer(Arc::new(composite) as Arc<dyn IslandGaObserver<RangeChromosome<f64>> + Send + Sync>)
            .build()
            .expect("Failed to build island GA");

    // --- Run and report final result ---
    match island_ga.run() {
        Ok(best) => {
            println!("-------------------------------------------------------");
            println!("Best fitness: {:.6}", best.fitness());
            let first_five: Vec<f64> = best.dna()[0..5].iter().map(|g| g.value).collect();
            println!("Best solution (first 5 dims): {:?}", first_five);
            if best.fitness() < 50.0 {
                println!("Good convergence for 20D Rastrigin!");
            } else {
                println!("Try increasing generations or population size.");
            }
        }
        Err(e) => {
            println!("Island GA failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
