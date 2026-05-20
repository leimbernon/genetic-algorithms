use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::constraints::{
    apply_dynamic_penalty, apply_static_penalty, total_violation, validate_penalty_strategy,
    PenaltyStrategy,
};
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::ga::Ga;
use std::borrow::Cow;

#[test]
fn test_total_violation() {
    assert_eq!(total_violation(&[]), 0.0);
    assert_eq!(total_violation(&[1.0, 2.0, 3.0]), 6.0);
    assert_eq!(total_violation(&[0.0, 0.0, 0.0]), 0.0);
}

#[test]
fn test_static_penalty() {
    let penalized = apply_static_penalty(10.0, 5.0, 2.0);
    assert!((penalized - 20.0).abs() < 1e-10);
}

#[test]
fn test_dynamic_penalty() {
    let penalized = apply_dynamic_penalty(10.0, 5.0, 10, 0.5, 2.0, 1.0);
    // (0.5 * 10)^2 * 5 = 25 * 5 = 125, plus 10 = 135
    assert!((penalized - 135.0).abs() < 1e-10);
}

#[test]
fn test_validate_penalty_strategy() {
    assert!(validate_penalty_strategy(&PenaltyStrategy::None).is_ok());
    assert!(validate_penalty_strategy(&PenaltyStrategy::Static { coefficient: 10.0 }).is_ok());
    assert!(validate_penalty_strategy(&PenaltyStrategy::Static { coefficient: -1.0 }).is_err());
    assert!(validate_penalty_strategy(&PenaltyStrategy::Adaptive { initial_coefficient: 1.0, window_size: 5 }).is_ok());
    assert!(validate_penalty_strategy(&PenaltyStrategy::Adaptive { initial_coefficient: 1.0, window_size: 0 }).is_err());
}

#[test]
fn test_constraint_handling_ga_with_static_penalty() {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    // Constraint: solution sum must be >= 24 (violation = max(0, 24 - sum))
    let constraint = |dna: &[RangeGene<i32>]| {
        let sum: i32 = dna.iter().map(|g| g.value()).sum();
        (24.0 - sum as f64).max(0.0)
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(50)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| {
            let sum: i32 = dna.iter().map(|g| g.value()).sum();
            -(sum as f64) // Minimization: lower absolute sum = better
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(50)
        .with_constraint_fns(vec![constraint])
        .with_penalty_strategy(PenaltyStrategy::Static { coefficient: 100.0 })
        .build()
        .expect("Failed to build GA");

    let _population = ga.run().expect("GA run failed");
}

#[test]
fn test_repair_operator() {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    // Repair operator: clamp all gene values to max of 3
    let repair = move |c: &mut RangeChromosome<i32>| -> Result<(), GaError> {
        let dna = c.dna().to_vec();
        let clamped: Vec<RangeGene<i32>> = dna
            .into_iter()
            .map(|g| {
                let val = g.value();
                let clamped_val = val.min(3);
                RangeGene::new(0, vec![(0, n - 1)], clamped_val)
            })
            .collect();
        c.set_dna(Cow::Owned(clamped));
        Ok(())
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| {
            let sum: i32 = dna.iter().map(|g| g.value()).sum();
            -sum as f64
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_repair_operator(repair)
        .build()
        .expect("Failed to build GA with repair");

    let _population = ga.run().expect("GA run failed");
}

#[test]
fn test_constraint_handling_adaptive_penalty() {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    // Constraint: sum must be >= 20 (violation = max(0, 20 - sum))
    let constraint = |dna: &[RangeGene<i32>]| {
        let sum: i32 = dna.iter().map(|g| g.value()).sum();
        (20.0 - sum as f64).max(0.0)
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(50)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| {
            let sum: i32 = dna.iter().map(|g| g.value()).sum();
            -(sum as f64)
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(50)
        .with_constraint_fns(vec![constraint])
        .with_penalty_strategy(PenaltyStrategy::Adaptive {
            initial_coefficient: 50.0,
            window_size: 10,
        })
        .build()
        .expect("Failed to build GA with adaptive penalty");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with adaptive penalty should succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn test_constraint_handling_feasibility_rules() {
    use genetic_algorithms::constraints::ConstraintHandling;

    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    // Constraint: sum must be >= 24 (violation = max(0, 24 - sum))
    let constraint = |dna: &[RangeGene<i32>]| {
        let sum: i32 = dna.iter().map(|g| g.value()).sum();
        (24.0 - sum as f64).max(0.0)
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(50)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| {
            let sum: i32 = dna.iter().map(|g| g.value()).sum();
            -(sum as f64)
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(50)
        .with_constraint_fns(vec![constraint])
        .with_constraint_handling(ConstraintHandling::FeasibilityRules)
        .build()
        .expect("Failed to build GA with feasibility rules");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with feasibility rules should succeed, got: {:?}",
        result.err()
    );
}
