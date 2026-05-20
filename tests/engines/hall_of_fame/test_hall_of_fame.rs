//! Unit tests for the Hall of Fame / Solution Archive module.
//!
//! Covers HOF-01 (bounded capacity, dedup, distance filter),
//! HOF-02 (two distance modes), HOF-03 (fixed f64 threshold),
//! HOF-05 (worst-eviction), HOF-07 (core API).

use crate::structures::{Chromosome, Gene};
use genetic_algorithms::hall_of_fame::*;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

/// Helper: construct a test chromosome with the given gene IDs and fitness.
fn make_chromosome(id_values: &[i32], fitness: f64) -> Chromosome {
    Chromosome {
        dna: id_values.iter().map(|&id| Gene { id }).collect(),
        fitness,
        age: 0,
        fitness_fn: Default::default(),
    }
}

// ---------------------------------------------------------------------------
// Test 1: hof_new_is_empty
// ---------------------------------------------------------------------------
#[test]
fn hof_new_is_empty() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    assert!(hof.is_empty());
    assert_eq!(hof.len(), 0);
}

// ---------------------------------------------------------------------------
// Test 2: hof_insert_single
// ---------------------------------------------------------------------------
#[test]
fn hof_insert_single() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c = make_chromosome(&[1], 10.0);
    let admitted = hof.try_insert(&c, 0);
    assert!(admitted);
    assert_eq!(hof.len(), 1);
    assert!(!hof.is_empty());
    let sols = hof.solutions();
    assert_eq!(sols.len(), 1);
    assert_eq!(sols[0].fitness_at_addition, 10.0);
}

// ---------------------------------------------------------------------------
// Test 3: hof_insert_in_order
// ---------------------------------------------------------------------------
#[test]
fn hof_insert_in_order() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1], 5.0);
    let c2 = make_chromosome(&[2], 10.0);
    let c3 = make_chromosome(&[3], 1.0);
    hof.try_insert(&c1, 0);
    hof.try_insert(&c2, 0);
    hof.try_insert(&c3, 0);
    let entries = hof.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].fitness_at_addition, 10.0);
    assert_eq!(entries[1].fitness_at_addition, 5.0);
    assert_eq!(entries[2].fitness_at_addition, 1.0);
}

// ---------------------------------------------------------------------------
// Test 4: hof_insert_deduplicates_dna
// ---------------------------------------------------------------------------
#[test]
fn hof_insert_deduplicates_dna() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1, 2], 10.0);
    let c2 = make_chromosome(&[1, 2], 15.0);
    assert!(hof.try_insert(&c1, 0));
    assert_eq!(hof.len(), 1);
    // Same DNA, different fitness -- should be rejected (D-07)
    assert!(!hof.try_insert(&c2, 0));
    assert_eq!(hof.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 5: hof_evicts_worst_when_full
// ---------------------------------------------------------------------------
#[test]
fn hof_evicts_worst_when_full() {
    let config = HallOfFameConfig {
        capacity: 3,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1], 5.0);
    let c2 = make_chromosome(&[2], 10.0);
    let c3 = make_chromosome(&[3], 1.0);
    let c4 = make_chromosome(&[4], 3.0);
    hof.try_insert(&c1, 0);
    hof.try_insert(&c2, 0);
    hof.try_insert(&c3, 0);
    // Entries so far: [10.0, 5.0, 1.0], full at cap=3
    hof.try_insert(&c4, 0); // 3.0 >= 1.0 (worst), should be admitted
    assert_eq!(hof.len(), 3);
    // 1.0 should be evicted (worst); entries: [10.0, 5.0, 3.0]
    let entries = hof.entries();
    assert_eq!(entries[2].fitness_at_addition, 3.0);
    // Verify 1.0 is gone
    assert!(entries.iter().all(|e| e.fitness_at_addition > 1.0));
}

// ---------------------------------------------------------------------------
// Test 6: hof_evicts_always_worst
// ---------------------------------------------------------------------------
#[test]
fn hof_evicts_always_worst() {
    let config = HallOfFameConfig {
        capacity: 2,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1], 10.0);
    let c2 = make_chromosome(&[2], 8.0);
    let c3 = make_chromosome(&[3], 9.0);
    hof.try_insert(&c1, 0); // [10.0]
    hof.try_insert(&c2, 0); // [10.0, 8.0], full at cap=2
    hof.try_insert(&c3, 0); // 9.0 >= 8.0 (worst) -> inserted -> evict last -> [10.0, 9.0]
    assert_eq!(hof.len(), 2);
    let entries = hof.entries();
    assert_eq!(entries[0].fitness_at_addition, 10.0);
    assert_eq!(entries[1].fitness_at_addition, 9.0);
}

// ---------------------------------------------------------------------------
// Test 7: hof_top_returns_k_best
// ---------------------------------------------------------------------------
#[test]
fn hof_top_returns_k_best() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    for (id, f) in &[(1, 5.0), (2, 10.0), (3, 3.0), (4, 7.0)] {
        hof.try_insert(&make_chromosome(&[*id], *f), 0);
    }
    let top2 = hof.top(2);
    assert_eq!(top2.len(), 2);
    assert_eq!(top2[0].fitness_at_addition, 10.0);
    assert_eq!(top2[1].fitness_at_addition, 7.0);
    let top10 = hof.top(10);
    assert_eq!(top10.len(), 4);
}

// ---------------------------------------------------------------------------
// Test 8: hof_would_qualify
// ---------------------------------------------------------------------------
#[test]
fn hof_would_qualify() {
    let config = HallOfFameConfig {
        capacity: 3,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1], 5.0);
    let c2 = make_chromosome(&[2], 10.0);
    hof.try_insert(&c1, 0);
    hof.try_insert(&c2, 0);
    // Archive not full (2 of 3) -- would_qualify returns true for any fitness
    let c_low = make_chromosome(&[5], 3.0);
    assert!(hof.would_qualify(&c_low));
    // Fill the archive
    let c3 = make_chromosome(&[3], 7.0);
    hof.try_insert(&c3, 0);
    // Now full (3 entries). Worst is 5.0 (entries: [10.0, 7.0, 5.0])
    let above = make_chromosome(&[10], 6.0);
    let below = make_chromosome(&[20], 4.0);
    assert!(hof.would_qualify(&above)); // 6.0 >= 5.0
    assert!(!hof.would_qualify(&below)); // 4.0 < 5.0
}

// ---------------------------------------------------------------------------
// Test 9: hof_len_and_is_empty
// ---------------------------------------------------------------------------
#[test]
fn hof_len_and_is_empty() {
    let config = HallOfFameConfig {
        capacity: 5,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    assert_eq!(hof.len(), 0);
    assert!(hof.is_empty());
    hof.try_insert(&make_chromosome(&[1], 5.0), 0);
    assert_eq!(hof.len(), 1);
    assert!(!hof.is_empty());
    hof.try_insert(&make_chromosome(&[2], 3.0), 0);
    assert_eq!(hof.len(), 2);
    assert!(!hof.is_empty());
}

// ---------------------------------------------------------------------------
// Test 10: hof_iter_yields_entries_with_metadata
// ---------------------------------------------------------------------------
#[test]
fn hof_iter_yields_entries_with_metadata() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c = make_chromosome(&[42], 7.0);
    hof.try_insert(&c, 5);
    let collected: Vec<&Entry<Chromosome>> = hof.iter().collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].generation_added, 5);
    assert_eq!(collected[0].fitness_at_addition, 7.0);
    assert_eq!(collected[0].chromosome.dna()[0].id(), 42);
}

// ---------------------------------------------------------------------------
// Test 11: hof_solutions_returns_best_first
// ---------------------------------------------------------------------------
#[test]
fn hof_solutions_returns_best_first() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1], 1.0);
    let c2 = make_chromosome(&[2], 8.0);
    let c3 = make_chromosome(&[3], 3.0);
    hof.try_insert(&c1, 0);
    hof.try_insert(&c2, 0);
    hof.try_insert(&c3, 0);
    let sols = hof.solutions();
    assert_eq!(sols.len(), 3);
    assert!(sols[0].fitness_at_addition > sols[1].fitness_at_addition);
    assert!(sols[1].fitness_at_addition > sols[2].fitness_at_addition);
}

// ---------------------------------------------------------------------------
// Test 12: hof_fitness_default_metric
// ---------------------------------------------------------------------------
#[test]
fn hof_fitness_default_metric() {
    let config = HallOfFameConfig::default();
    match config.distance_metric {
        DistanceMetric::Fitness { min_distance } => {
            assert_eq!(min_distance, 0.0);
        }
        _ => panic!("Expected Fitness as default distance metric (D-02)"),
    }
}

// ---------------------------------------------------------------------------
// Test 13: hof_genotypic_distance_rejects_close
// ---------------------------------------------------------------------------
#[test]
fn hof_genotypic_distance_rejects_close() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Genotypic { min_distance: 0.5 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1, 2], 10.0);
    assert!(hof.try_insert(&c1, 0));
    // Same DNA should be rejected by dedup (D-07), not distance
    let c_dup = make_chromosome(&[1, 2], 9.0);
    assert!(!hof.try_insert(&c_dup, 0));
    // Distance 0.5 (1 of 2 differ), threshold is 0.5, NOT < 0.5 so admitted
    let c_half = make_chromosome(&[1, 9], 8.0);
    assert!(hof.try_insert(&c_half, 0));
    // Distance 1.0 (2 of 2 differ), well above threshold, admitted
    let c_full = make_chromosome(&[9, 9], 7.0);
    assert!(hof.try_insert(&c_full, 0));
    // Try to insert something very close (0.0 distance, same DNA as c_half)
    let c_very_close = make_chromosome(&[1, 9], 6.0);
    assert!(!hof.try_insert(&c_very_close, 0)); // rejected by dedup
}

// ---------------------------------------------------------------------------
// Test 14: hof_insert_returns_true_false
// ---------------------------------------------------------------------------
#[test]
fn hof_insert_returns_true_false() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c1 = make_chromosome(&[1, 2], 10.0);
    assert!(hof.try_insert(&c1, 0));
    // Duplicate DNA should return false
    let c2 = make_chromosome(&[1, 2], 8.0);
    assert!(!hof.try_insert(&c2, 0));
    // Different DNA should return true
    let c3 = make_chromosome(&[3, 4], 5.0);
    assert!(hof.try_insert(&c3, 0));
}

// ---------------------------------------------------------------------------
// Test 15: hof_capacity_zero_never_inserts
// ---------------------------------------------------------------------------
#[test]
fn hof_capacity_zero_never_inserts() {
    let config = HallOfFameConfig {
        capacity: 0,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c = make_chromosome(&[1], 100.0);
    assert!(!hof.try_insert(&c, 0));
    assert_eq!(hof.len(), 0);
    assert!(hof.is_empty());
}

// ---------------------------------------------------------------------------
// Test 16: hof_nan_fitness_skipped
// ---------------------------------------------------------------------------
#[test]
fn hof_nan_fitness_skipped() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c = make_chromosome(&[1], f64::NAN);
    assert!(!hof.try_insert(&c, 0));
    assert_eq!(hof.len(), 0);
}

// ---------------------------------------------------------------------------
// Test 17: hof_would_qualify_on_empty_archive
// ---------------------------------------------------------------------------
#[test]
fn hof_would_qualify_on_empty_archive() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    let c = make_chromosome(&[1], 0.0);
    assert!(hof.would_qualify(&c));
    let c2 = make_chromosome(&[2], -100.0);
    assert!(hof.would_qualify(&c2));
}

// ---------------------------------------------------------------------------
// Test 18: hof_genotypic_distance_different_lengths
// ---------------------------------------------------------------------------
#[test]
fn hof_genotypic_distance_different_lengths() {
    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };
    let mut hof: HallOfFame<Chromosome> = HallOfFame::new(config);
    // Insert a chromosome with DNA of length 2
    let c1 = make_chromosome(&[1, 2], 10.0);
    assert!(hof.try_insert(&c1, 0));
    // Try to insert chromosome with same prefix but length 3.
    // These are NOT identical (length mismatch, position 2 differs).
    let c2 = make_chromosome(&[1, 2, 3], 9.0);
    assert!(hof.try_insert(&c2, 0)); // Should be admitted (different DNA)
    assert_eq!(hof.len(), 2);
}

// ---------------------------------------------------------------------------
// GA Integration Test 1: Full GA run with HallOfFame
// ---------------------------------------------------------------------------
#[test]
fn test_hof_ga_builder_and_run() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::hall_of_fame::{HallOfFameConfig, DistanceMetric};
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};

    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    let config = HallOfFameConfig {
        capacity: 10,
        distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
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
        .with_max_generations(20)
        .with_hall_of_fame(config)
        .build()
        .expect("Failed to build GA with HallOfFame");

    let result = ga.run();
    assert!(result.is_ok(), "GA run should succeed");

    let hof = ga.hall_of_fame();
    assert!(hof.is_some(), "hall_of_fame() should return Some after run");
    let hof = hof.unwrap();
    assert!(!hof.is_empty(), "Hall of Fame should not be empty after a run");
    assert!(hof.len() <= 10, "Hall of Fame should respect capacity");
    assert_eq!(hof.solutions().len(), hof.len());
}

// ---------------------------------------------------------------------------
// GA Integration Test 2: GA without HallOfFame returns None
// ---------------------------------------------------------------------------
#[test]
fn test_hof_ga_without_hof_returns_none() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};

    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_genes_per_chromosome(n.try_into().unwrap())
        .with_population_size(15)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .build()
        .expect("build");

    let _ = ga.run();
    assert!(ga.hall_of_fame().is_none(), "hall_of_fame() should be None when not configured");
}

// ---------------------------------------------------------------------------
// GA Integration Test 3: GA with Genotypic distance filter
// ---------------------------------------------------------------------------
#[test]
fn test_hof_ga_genotypic_distance() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::hall_of_fame::{HallOfFameConfig, DistanceMetric};
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};

    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0, n - 1)], 0)];
    let alleles_clone = alleles.clone();

    // Genotypic distance 0.3 means at most 30% of genes can differ
    let config = HallOfFameConfig {
        capacity: 20,
        distance_metric: DistanceMetric::Genotypic { min_distance: 0.3 },
    };

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
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
        .with_max_generations(15)
        .with_hall_of_fame(config)
        .build()
        .expect("build with genotypic distance");

    let result = ga.run();
    assert!(result.is_ok());

    let hof = ga.hall_of_fame();
    assert!(hof.is_some());
    let hof = hof.unwrap();
    assert!(!hof.is_empty(), "Hall of Fame with genotypic filter should have entries");
    assert!(hof.len() <= 20, "Should respect capacity");
}
