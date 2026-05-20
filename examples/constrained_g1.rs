//! Constrained optimization example: G1 benchmark (simplified).
//!
//! Uses a GA with static penalty to solve a constrained problem
//! with 13 real-valued variables and 3 inequality constraints.
//!
//! Run with:
//! ```sh
//! cargo run --example constrained_g1
//! ```

use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::constraints::PenaltyStrategy;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig,
    StoppingConfig,
};

const N_VARS: usize = 13;
const POP_SIZE: usize = 200;
const MAX_GEN: usize = 200;

fn main() {
    println!("Constrained G1 benchmark with static penalty");
    println!(
        "Variables: {}, Population: {}, Generations: {}",
        N_VARS, POP_SIZE, MAX_GEN
    );
    println!();
    println!("Constraints:");
    println!("  g1: x[0..5] sum <= 4.0");
    println!("  g2: x[5..10] sum <= 3.0");
    println!("  g3: x[10..13] sum <= 2.0");
    println!();

    // Alleles: each gene is f64 in [0.0, 1.0]
    let alleles = vec![RangeGene::new(0, vec![(0.0_f64, 1.0_f64)], 0.0)];
    let alleles_clone = alleles.clone();

    // Constraint functions
    let constraints = vec![
        // g1: first 5 variables sum <= 4.0
        |dna: &[RangeGene<f64>]| {
            let sum: f64 = dna[0..5].iter().map(|g| g.value).sum();
            (sum - 4.0).max(0.0)
        },
        // g2: next 5 variables sum <= 3.0
        |dna: &[RangeGene<f64>]| {
            let sum: f64 = dna[5..10].iter().map(|g| g.value).sum();
            (sum - 3.0).max(0.0)
        },
        // g3: last 3 variables sum <= 2.0
        |dna: &[RangeGene<f64>]| {
            let sum: f64 = dna[10..13].iter().map(|g| g.value).sum();
            (sum - 2.0).max(0.0)
        },
    ];

    // Fitness: minimize sum(x) (simplified G1 objective)
    let fitness_fn = |dna: &[RangeGene<f64>]| -> f64 { dna.iter().map(|g| g.value).sum() };

    let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
        .with_genes_per_chromosome(N_VARS)
        .with_population_size(POP_SIZE)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::BlendAlpha)
        .with_mutation_method(Mutation::Gaussian)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(MAX_GEN)
        .with_constraint_fns(constraints)
        .with_penalty_strategy(PenaltyStrategy::Static { coefficient: 100.0 })
        .build()
        .expect("Failed to build GA");

    println!("Running GA with static penalty (R = 100.0)...");
    let population = ga.run().expect("GA run failed");

    let best = &population.best_chromosome;
    println!();
    println!("Results:");
    println!("  Best fitness (penalized): {:.6}", best.fitness());
    println!();

    // Evaluate constraint violations on the best solution
    let dna = best.dna();
    let violations: Vec<f64> = vec![
        (dna[0..5].iter().map(|g| g.value).sum::<f64>() - 4.0).max(0.0),
        (dna[5..10].iter().map(|g| g.value).sum::<f64>() - 3.0).max(0.0),
        (dna[10..13].iter().map(|g| g.value).sum::<f64>() - 2.0).max(0.0),
    ];
    let total_violation: f64 = violations.iter().sum();

    println!(
        "  Constraint violations: g1={:.6}, g2={:.6}, g3={:.6}",
        violations[0], violations[1], violations[2]
    );
    println!("  Total violation: {:.6}", total_violation);
    println!("  Feasible: {}", total_violation <= 1e-10);
    println!();
    println!("  Best DNA (first 5 values):");
    for (i, gene) in dna.iter().take(5.min(N_VARS)).enumerate() {
        println!("    x[{}] = {:.6}", i, gene.value);
    }
}
