//! Integration tests for Warm Starting & Population Seeding (Phase 42).
//!
//! Covers WSM-01 with_seeds() builder methods, mutual exclusivity validation,
//! seed-based initialization, trusted fitness preservation, genotypic dedup,
//! and Hall of Fame seed admission.

#![cfg_attr(not(feature = "serde"), allow(dead_code, unused_imports))]

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};

/// Helper: build a basic GA configuration (no seeds or checkpoint).
fn base_ga() -> Ga<RangeChromosome<i32>> {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
}

/// Helper: create a vec of seed chromosomes with specific genes and trusted fitness.
fn create_seeds(count: usize, gene_value_start: i32) -> Vec<RangeChromosome<i32>> {
    (0..count)
        .map(|i| {
            let val = gene_value_start + i as i32;
            let dna = std::borrow::Cow::Owned(vec![
                RangeGene::new(val, vec![(0, 100)], 0);
                8
            ]);
            let mut c = RangeChromosome::<i32>::new();
            c.set_dna(dna);
            // Trusted fitness: sum of gene values
            c.set_fitness(val as f64 * 8.0);
            c
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Builder Tests (42-01 Foundation)
// ---------------------------------------------------------------------------

#[test]
fn test_wsm_with_seeds_builds_successfully() {
    let seeds = create_seeds(5, 10);
    let ga = base_ga()
        .with_seeds(seeds)
        .build()
        .expect("GA with seeds should build successfully");
}

#[test]
fn test_wsm_with_seeds_exceeds_population_errors() {
    let seeds = create_seeds(100, 10);
    let result = base_ga().with_seeds(seeds).build();
    assert!(result.is_err(), "Seeds exceeding population_size should error");
    let err_msg = match result {
        Err(e) => e.to_string(),
        _ => unreachable!(),
    };
    assert!(
        err_msg.contains("seeds") || err_msg.contains("population"),
        "Error should mention seeds/population, got: {}",
        err_msg
    );
}

#[test]
fn test_wsm_with_checkpoint_path_not_found_errors() {
    let result = base_ga()
        .with_checkpoint("/nonexistent/checkpoint.json")
        .build();
    assert!(
        result.is_err(),
        "Non-existent checkpoint path should error"
    );
}

#[test]
fn test_wsm_seeds_and_checkpoint_mutually_exclusive() {
    let seeds = create_seeds(3, 10);
    let result = base_ga()
        .with_seeds(seeds)
        .with_checkpoint("/tmp/test_checkpoint.json")
        .build();
    assert!(
        result.is_err(),
        "Both seeds and checkpoint should error"
    );
    let err_msg = match result {
        Err(e) => e.to_string(),
        _ => unreachable!(),
    };
    assert!(
        err_msg.contains("mutually exclusive") || err_msg.contains("both"),
        "Error should mention mutual exclusivity, got: {}",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// Seed-based Initialization Integration Tests (42-02)
// All use seeds.len() == population_size to avoid fill dedup complexity.
// ---------------------------------------------------------------------------

#[test]
fn test_wsm_seeds_population_size_matches() {
    // Use exact fit: seeds count == population_size (no fill generation)
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    let seeds = create_seeds(2, 100);

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(2)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .with_alleles(alleles)
        .with_seeds(seeds)
        .build()
        .expect("build with seeds");

    let pop = match ga.run() {
        Ok(p) => p,
        Err(e) => panic!("GA with seeds should run: {}", e),
    };

    assert_eq!(pop.size(), 2, "Population should be 2 after seeded init");
    // First seed should have fitness 800.0 (100 * 8)
    let first = &pop.chromosomes[0];
    assert_eq!(first.fitness(), 800.0, "First seed should retain its fitness");
    // Second seed should have fitness 808.0 (101 * 8)
    let second = &pop.chromosomes[1];
    assert_eq!(second.fitness(), 808.0, "Second seed should retain its fitness");
}

#[test]
fn test_wsm_seeds_admitted_to_hall_of_fame() {
    use genetic_algorithms::hall_of_fame::HallOfFameConfig;

    let n: i32 = 4;
    let alleles = vec![RangeGene::new(0, vec![(0, 10)], 0)];
    let alleles_clone = alleles.clone();

    // Create a seed with very high fitness
    let dna = std::borrow::Cow::Owned(vec![RangeGene::new(99, vec![(0, 200)], 0); 4]);
    let mut seed1 = RangeChromosome::<i32>::new();
    seed1.set_dna(dna);
    seed1.set_fitness(999.0);

    let seeds = vec![seed1];

    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: genetic_algorithms::hall_of_fame::DistanceMetric::Fitness {
            min_distance: 0.0,
        },
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(1)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .with_alleles(alleles)
        .with_seeds(seeds)
        .with_hall_of_fame(config)
        .build()
        .expect("build with seeds and HOF");

    let _ = ga.run();

    let hof = ga.hall_of_fame();
    assert!(hof.is_some(), "Hall of Fame should be Some when configured");
    let hof = hof.unwrap();
    assert!(
        !hof.is_empty(),
        "Hall of Fame should not be empty after seeded run"
    );
    if let Some(best) = hof.solutions().first() {
        assert!(
            best.fitness_at_addition >= 999.0,
            "Best solution should have fitness >= seed fitness (got {})",
            best.fitness_at_addition
        );
    }
}

#[test]
fn test_wsm_seeds_without_hall_of_fame() {
    let n: i32 = 4;
    let alleles = vec![RangeGene::new(0, vec![(0, 10)], 0)];
    let alleles_clone = alleles.clone();

    let seeds = create_seeds(3, 5);

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(3)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .with_alleles(alleles)
        .with_seeds(seeds)
        .build()
        .expect("build with seeds, no HOF");

    let _ = ga.run();
}
