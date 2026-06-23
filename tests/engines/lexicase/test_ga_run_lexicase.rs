//! Integration tests proving that `Ga::<MultiCaseChromosome>::run_lexicase()` works
//! end-to-end for both `Selection::Lexicase` and `Selection::EpsilonLexicase`, that
//! the standard `run()` path rejects lexicase variants with a clear error naming
//! `run_lexicase`, and that the scalar-fitness mean-sync invariant (D-04 / TRAITS-01)
//! holds after a lexicase run.
//!
//! Closes: SEL-02, SEL-03, TRAITS-01

use crate::structures::{Gene, MultiCaseChromosome};
use genetic_algorithms::{
    error::GaError,
    ga::Ga,
    operations::Selection,
    traits::{ChromosomeT, ConfigurationT, SelectionConfig, StoppingConfig, VectorFitness},
    ChromosomeLength,
};

/// Build a small `Ga<MultiCaseChromosome>` configured with the given selection method.
/// Population = 20, chromosome length = 4, alleles = Gene ids 1..=8.
/// `MultiCaseChromosome::calculate_fitness()` populates `fitness_values` from gene ids
/// and sets scalar `fitness` to their mean.
///
/// A trivial `with_fitness_fn` is provided so that the initialization path calls
/// `calculate_fitness()` on each chromosome — the fixture's `calculate_fitness()`
/// ignores the fn and derives case scores directly from gene IDs.
fn build_ga(selection: Selection) -> Ga<MultiCaseChromosome> {
    let alleles = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    Ga::<MultiCaseChromosome>::new()
        .with_rng_seed(42)
        .with_selection_method(selection)
        .with_number_of_couples(10)
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_alleles(alleles)
        .with_max_generations(20)
        .with_fitness_fn(|_dna: &[Gene]| 0.0) // triggers calculate_fitness() during init
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<MultiCaseChromosome>,
        )
}

/// SEL-02: `Ga::<MultiCaseChromosome>` configured with `Selection::Lexicase` runs to
/// completion via `run_lexicase()` and returns a non-empty population.
#[test]
fn test_ga_run_lexicase_completes() {
    let mut ga = build_ga(Selection::Lexicase);
    let result = ga.run_lexicase();
    assert!(result.is_ok(), "run_lexicase() returned Err: {:?}", result.err());
    let population = result.unwrap();
    assert!(
        !population.chromosomes.is_empty(),
        "run_lexicase() produced an empty population"
    );
}

/// SEL-03: `Ga::<MultiCaseChromosome>` configured with `Selection::EpsilonLexicase`
/// runs to completion via `run_lexicase()`.
#[test]
fn test_ga_run_epsilon_lexicase_completes() {
    let alleles = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let mut ga = Ga::<MultiCaseChromosome>::new()
        .with_rng_seed(42)
        .with_selection_method(Selection::EpsilonLexicase)
        .with_epsilon_lexicase(0.5)
        .with_number_of_couples(10)
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_alleles(alleles)
        .with_max_generations(20)
        .with_fitness_fn(|_dna: &[Gene]| 0.0) // triggers calculate_fitness() during init
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<MultiCaseChromosome>,
        );
    let result = ga.run_lexicase();
    assert!(result.is_ok(), "run_lexicase() with EpsilonLexicase returned Err: {:?}", result.err());
    let population = result.unwrap();
    assert!(
        !population.chromosomes.is_empty(),
        "run_lexicase() with EpsilonLexicase produced an empty population"
    );
}

/// Proves that calling the STANDARD `run()` path with `Selection::Lexicase` returns
/// `Err(GaError::ConfigurationError)` with a message containing `run_lexicase`.
/// This is the T01 guard from Plan 01: misuse of the standard entry point is
/// rejected with a helpful error regardless of chromosome type.
#[test]
fn test_run_lexicase_on_non_vector_fitness_returns_error() {
    let mut ga = build_ga(Selection::Lexicase);
    let result = ga.run();
    match result {
        Err(GaError::ConfigurationError(msg)) => {
            assert!(
                msg.contains("run_lexicase"),
                "ConfigurationError message does not mention 'run_lexicase': {msg}"
            );
        }
        Err(other) => panic!("Expected ConfigurationError, got: {other:?}"),
        Ok(_) => panic!("Expected Err, but run() returned Ok for Selection::Lexicase"),
    }
}

/// TRAITS-01 / D-04: after a lexicase run every chromosome's scalar `fitness()` equals
/// the mean of its `fitness_values()` (the sync performed inside `factory_lexicase`).
#[test]
fn test_lexicase_mean_sync_in_run() {
    // Use 1 generation so the run is fast and the invariant is easy to check.
    let alleles = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let mut ga_1gen = Ga::<MultiCaseChromosome>::new()
        .with_rng_seed(7)
        .with_selection_method(Selection::Lexicase)
        .with_number_of_couples(10)
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_alleles(alleles)
        .with_max_generations(1)
        .with_fitness_fn(|_dna: &[Gene]| 0.0) // triggers calculate_fitness() during init
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<MultiCaseChromosome>,
        );
    let population = ga_1gen.run_lexicase().expect("run_lexicase failed");
    for (i, c) in population.chromosomes.iter().enumerate() {
        let fv = VectorFitness::fitness_values(c);
        if fv.is_empty() {
            continue;
        }
        let mean = fv.iter().sum::<f64>() / fv.len() as f64;
        let diff = (c.fitness() - mean).abs();
        assert!(
            diff < 1e-9,
            "Chromosome {i}: fitness()={} but mean of fitness_values={mean} (diff={diff})",
            c.fitness()
        );
    }
}

/// Proves that a lexicase GA run preserves diversity: after ~20 generations the final
/// population contains at least 2 distinct `fitness_values()` profiles (specialists
/// are not collapsed to a single identical vector).
#[test]
fn test_run_lexicase_diversity() {
    let mut ga = build_ga(Selection::Lexicase);
    let population = ga.run_lexicase().expect("run_lexicase failed");

    // Collect unique fitness_values profiles.
    let mut unique_profiles: Vec<Vec<i64>> = Vec::new();
    for c in &population.chromosomes {
        let fv = c.fitness_values();
        if fv.is_empty() {
            continue;
        }
        // Convert to integer-bits for hashing (values come from gene ids which are integers).
        let profile: Vec<i64> = fv.iter().map(|&v| v.to_bits() as i64).collect();
        if !unique_profiles.contains(&profile) {
            unique_profiles.push(profile);
        }
    }

    // Best chromosome must have non-empty fitness_values (engine is wired correctly).
    assert!(
        !population.best_chromosome.fitness_values().is_empty(),
        "best_chromosome has empty fitness_values after lexicase run"
    );

    // Diversity assertion: at least 2 distinct profiles.
    // With Gene ids 1..=8 and chromosome length 4, the search space is large enough
    // that 20 generations on a population of 20 preserves multiple specialists.
    // Fall back to a weaker assertion (run completes) only if it proves empirically flaky.
    assert!(
        unique_profiles.len() >= 2,
        "Expected >= 2 distinct fitness_values profiles in final population, got {}",
        unique_profiles.len()
    );
}
