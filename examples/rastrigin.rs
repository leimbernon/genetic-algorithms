/*!
# Rastrigin Continuous Optimization Example

This example demonstrates how to use the `genetic_algorithms` library to minimize the Rastrigin
function — a classic continuous optimization benchmark with many local minima and a global
minimum of 0.0 at the origin.

Features demonstrated:
- Range<f64> chromosomes (continuous representation)
- Gaussian mutation
- Tournament selection
- Uniform crossover
- Minimization mode
- Progress callback
- CompositeObserver combining LogObserver and optional MetricsObserver

Run with:
```sh
cargo run --example rastrigin
```
*/

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;
use genetic_algorithms::{CompositeObserver, LogObserver};
use std::sync::Arc;

fn main() {
    // --- Problem parameters ---
    const DIMENSIONS: usize = 5;
    const POP_SIZE: usize = 100;
    const MAX_GENERATIONS: usize = 500;

    // --- Fitness function: Rastrigin function (minimize toward 0.0 at origin) ---
    // f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))  where A = 10
    let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
        let a = 10.0;
        let n = dna.len() as f64;
        a * n
            + dna
                .iter()
                .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
                .sum::<f64>()
    };

    // --- Allele range: each dimension in [-5.12, 5.12] ---
    let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Build composite observer (LogObserver always active; MetricsObserver when feature flag set) ---
    let composite = CompositeObserver::new().add(Arc::new(LogObserver));
    #[cfg(feature = "observer-metrics")]
    let composite = composite.add(Arc::new(MetricsObserver::new("rastrigin")));

    // --- Build the GA configuration ---
    let mut ga = Ga::new()
        // Chromosome: DIMENSIONS genes, each a continuous value in [-5.12, 5.12]
        .with_genes_per_chromosome(DIMENSIONS)
        .with_population_size(POP_SIZE)
        // Random initialization within allele bounds
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(fitness_fn)
        // Selection: Tournament (pressure-based, good for continuous landscapes)
        .with_selection_method(Selection::Tournament)
        // Crossover: Uniform blend
        .with_crossover_method(Crossover::Uniform)
        // Mutation: Gaussian perturbation (appropriate for continuous optimization)
        .with_mutation_method(Mutation::Gaussian)
        // Survivor selection: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Problem: minimize fitness toward 0.0
        .with_problem_solving(ProblemSolving::Minimization)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: CompositeObserver fans out to LogObserver (and MetricsObserver if feature enabled)
        .with_observer(Arc::new(composite))
        .build()
        .expect("Failed to build GA configuration");

    println!("== Rastrigin Continuous Optimization ==");
    println!(
        "Chromosome: {} dimensions, Population: {}, Max generations: {}",
        DIMENSIONS, POP_SIZE, MAX_GENERATIONS
    );
    println!("Operators: Selection=Tournament, Crossover=Uniform, Mutation=Gaussian");
    println!("-------------------------------------------------------");

    // --- Run the GA with a callback to report progress ---
    let report_interval = 50;
    let result = ga.run_with_callback(
        Some(
            |gen: &usize,
             pop: &Population<RangeChromosome<f64>>,
             _stats: &GenerationStats,
             _cause: &TerminationCause|
             -> std::ops::ControlFlow<()> {
                let avg_fitness =
                    pop.chromosomes.iter().map(|c| c.fitness()).sum::<f64>() / pop.size() as f64;
                println!(
                    "Generation {:4}: best = {:8.4}, avg = {:8.4}",
                    gen, pop.best_chromosome.fitness, avg_fitness
                );
                std::ops::ControlFlow::Continue(())
            },
        ),
        report_interval,
    );

    // --- Show the final result ---
    match result {
        Ok(population) => {
            println!("-------------------------------------------------------");
            println!(
                "Finished. Best fitness: {:.6}",
                population.best_chromosome.fitness
            );
            if population.best_chromosome.fitness < 1.0 {
                println!("Near-optimal solution found!");
            } else {
                println!("Did not reach optimum. Try increasing generations or population size.");
            }
        }
        Err(e) => {
            println!("GA failed: {:?}", e);
        }
    }
}
