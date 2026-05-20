/*!
# Niching / Fitness Sharing Example — Multimodal Optimization

This example demonstrates how to use the `genetic_algorithms` library to find multiple peaks
of a multimodal function simultaneously using fitness sharing (niching).

The landscape has three Gaussian peaks at x=2, x=5, and x=8 over the domain [0, 10].
Without niching the GA would converge to a single dominant peak. With fitness sharing
enabled, selection pressure is reduced for crowded regions of the search space, which
encourages the population to spread across and maintain all three peaks.

Features demonstrated:
- Range<f64> chromosomes (1-D continuous search space)
- Niching / Fitness Sharing (`sigma_share`, `alpha`)
- Maximization mode
- Multimodal solution reporting (counting individuals near each peak)
- LogObserver lifecycle hooks

Run with:
```sh
cargo run --example niching
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
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, NichingConfig,
    SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

fn main() {
    // --- Problem parameters ---
    const POP_SIZE: usize = 150;
    const MAX_GENERATIONS: usize = 300;
    const SIGMA_SHARE: f64 = 1.5;
    const ALPHA: f64 = 1.0;

    // --- Fitness function: three Gaussian peaks at x=2, x=5, x=8 ---
    let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
        let x = dna[0].value;
        let peak = |center: f64, height: f64| -> f64 {
            height * (-((x - center).powi(2)) / (2.0 * 0.5_f64.powi(2))).exp()
        };
        peak(2.0, 1.0) + peak(5.0, 0.9) + peak(8.0, 0.8)
    };

    // --- Allele range: x in [0.0, 10.0] ---
    let alleles = vec![RangeGenotype::new(0_i32, vec![(0.0, 10.0)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Build the GA configuration ---
    let mut ga = Ga::new()
        // 1-D problem: one gene (x) per chromosome
        .with_genes_per_chromosome(1)
        .with_population_size(POP_SIZE)
        // Random initialisation within [0.0, 10.0]
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(fitness_fn)
        // Selection: Tournament
        .with_selection_method(Selection::Tournament)
        // Crossover: Uniform
        .with_crossover_method(Crossover::Uniform)
        // Mutation: Gaussian (small perturbations to x)
        .with_mutation_method(Mutation::Gaussian)
        // Survivor selection: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Problem: maximise the fitness landscape
        .with_problem_solving(ProblemSolving::Maximization)
        // Niching: fitness sharing to maintain multiple peaks
        .with_niching_enabled(true)
        .with_niching_sigma_share(SIGMA_SHARE)
        .with_niching_alpha(ALPHA)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Failed to build GA configuration");

    println!("== Niching / Fitness Sharing Example ==");
    println!("Landscape: 3 Gaussian peaks at x=2, x=5, x=8 in [0, 10]");
    println!("Population: {POP_SIZE}, Max generations: {MAX_GENERATIONS}");
    println!("Operators: Selection=Tournament, Crossover=Uniform, Mutation=Gaussian");
    println!("Niching: sigma_share={SIGMA_SHARE}, alpha={ALPHA}");
    println!("-------------------------------------------------------");

    // --- Run the GA with a progress callback every 50 generations ---
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
                    "Generation {:4}: best = {:.4}, avg = {:.4}",
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
                "Finished. Best fitness: {:.4}",
                population.best_chromosome.fitness
            );
            println!("\nTop solutions (showing population spread across peaks):");

            // Collect all chromosome positions and sort
            let mut positions: Vec<f64> = population
                .chromosomes
                .iter()
                .map(|c| c.dna()[0].value)
                .collect();
            positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Count solutions near each peak (within sigma_share)
            let peaks = [
                (2.0, "Peak 1 (x=2)"),
                (5.0, "Peak 2 (x=5)"),
                (8.0, "Peak 3 (x=8)"),
            ];
            for (center, label) in &peaks {
                let count = positions
                    .iter()
                    .filter(|&&x| (x - center).abs() < SIGMA_SHARE)
                    .count();
                println!("  {label}: {count} individuals");
            }

            let peaks_found = peaks
                .iter()
                .filter(|(center, _)| positions.iter().any(|&x| (x - center).abs() < SIGMA_SHARE))
                .count();
            if peaks_found >= 3 {
                println!("SUCCESS: Population covers all {peaks_found} peaks!");
            } else {
                println!(
                    "Found {peaks_found} of 3 peaks. Try increasing population or adjusting sigma_share."
                );
            }
        }
        Err(e) => {
            println!("GA failed: {:?}", e);
        }
    }
}
