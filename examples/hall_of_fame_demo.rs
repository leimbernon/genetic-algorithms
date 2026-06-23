//! Hall of Fame / Solution Archive demo.
//!
//! Demonstrates how to configure and use the Hall of Fame to maintain
//! a bounded archive of the best unique solutions found during a GA run.
//!
//! Run with: cargo run --example hall_of_fame_demo

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::hall_of_fame::{DistanceMetric, HallOfFameConfig};
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig,
    SelectionConfig, StoppingConfig,
};

fn main() {
    let _ = env_logger::try_init();
    println!("=== Hall of Fame / Solution Archive Demo ===");
    println!();

    // Problem: maximize sum of gene values
    // 8 genes, each in range [0, 99]
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 99_i32)], 0)];
    let alleles_clone = alleles.clone();

    // Hall of Fame configuration:
    // - Capacity 15 solutions
    // - No diversity filtering (Fitness with min_distance 0.0)
    let hof_config = HallOfFameConfig {
        capacity: 15,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(
            n.try_into().unwrap(),
        ))
        .with_population_size(50)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum::<f64>())
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(100)
        .with_hall_of_fame(hof_config)
        .build()
        .expect("Failed to build GA");

    println!("Running GA (100 generations, population 50)...");
    let population = ga.run().expect("GA run failed");
    println!("Run complete.");
    println!();

    // Show best chromosome (for comparison with existing tracking)
    println!("=== Best Chromosome (existing tracking) ===");
    let best_dna: Vec<i32> = population
        .best_chromosome
        .dna()
        .iter()
        .map(|g| g.value())
        .collect();
    println!(
        "  Fitness: {:.2} | DNA: {:?}",
        population.best_chromosome.fitness(),
        best_dna,
    );
    println!();

    // Access the Hall of Fame (population reference is no longer used,
    // so the borrow is released and ga is available)
    let hof = ga.hall_of_fame().expect("Hall of Fame should be populated");

    println!("=== Hall of Fame Summary ===");
    println!("  Total archived solutions: {}", hof.len());
    println!("  Archive capacity: {}", hof.capacity());
    println!();

    // Show top 5 entries
    let top_n = 5.min(hof.len());
    println!("=== Top {} Solutions (best first) ===", top_n);
    for (rank, entry) in hof.top(top_n).iter().enumerate() {
        let dna_values: Vec<i32> = entry.chromosome.dna().iter().map(|g| g.value()).collect();
        println!(
            "  #{:<3} | Fitness: {:>8.2} | Gen: {:<5} | DNA: {:?}",
            rank + 1,
            entry.fitness_at_addition,
            entry.generation_added,
            dna_values,
        );
    }
    println!();

    // Iterate over the first 10 entries
    println!("=== Full Archive (first 10) ===");
    let max_to_show = 10usize.min(hof.len());
    for (i, entry) in hof.iter().enumerate().take(max_to_show) {
        let dna_values: Vec<i32> = entry.chromosome.dna().iter().map(|g| g.value()).collect();
        println!(
            "  [{:>2}] Fitness: {:>8.2} (gen {}) DNA: {:?}",
            i + 1,
            entry.fitness_at_addition,
            entry.generation_added,
            dna_values,
        );
    }
    if hof.len() > max_to_show {
        println!("  ... ({} more entries)", hof.len() - max_to_show);
    }
    println!();

    // Demonstrate API methods
    println!("=== API Method Demo ===");
    println!("  hof.len()         = {}", hof.len());
    println!("  hof.capacity()    = {}", hof.capacity());
    println!("  hof.is_empty()    = {}", hof.is_empty());
    println!("  hof.solutions().len() = {}", hof.solutions().len());
    println!("  hof.top(3).len()  = {}", hof.top(3).len());
    println!();
    println!("Demo complete.");
}
