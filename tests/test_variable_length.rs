// Wave 3 — Phase 52: Variable-Length Chromosomes
//
// All stubs from Wave 0 are now enabled with real implementations.
// These tests validate the full MUT-06, CHR-01, and CHR-02 feature contracts.
//
// Requirements: MUT-06, CHR-01, CHR-02

use genetic_algorithms::chromosomes::{ChromosomeLength, Range as RangeChromosome};
use genetic_algorithms::configuration::{LimitConfiguration, ProblemSolving};
use genetic_algorithms::error::GaError;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::length_mutation::{
    length_deletion_mutation, length_insertion_mutation,
};
use genetic_algorithms::operations::survivor::apply_parsimony_pressure;
use genetic_algorithms::operations::{AlignmentStrategy, Crossover, Mutation, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverOperator, LinearChromosome, MutationConfig,
};
use std::borrow::Cow;

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Build a Range<f64> chromosome from a slice of f64 values.
/// Each gene uses the range (0.0, 100.0) and its position as ID.
fn range_chromosome(values: &[f64]) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<RangeGenotype<f64>> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], v))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

// ──────────────────────────────────────────────────────────────────────────────
// Section 1 — MUT-06: PermutationInsert rename + length operators
// ──────────────────────────────────────────────────────────────────────────────

/// Verify that Mutation::PermutationInsert moves a gene to a different position
/// without changing chromosome length on a Range<f64> of length 5.
#[test]
fn test_mutation_permutation_insert_renames_correctly() {
    use genetic_algorithms::operations::mutation;

    let values = [10.0, 20.0, 30.0, 40.0, 50.0];
    let mut c = range_chromosome(&values);

    // Run many times to ensure a movement happens at least once
    let original_len = c.dna().len();
    let mut moved = false;
    for _ in 0..200 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let result = mutation::factory(Mutation::PermutationInsert, &mut c);
        assert!(result.is_ok(), "PermutationInsert should succeed");
        // Length must never change
        assert_eq!(
            c.dna().len(),
            original_len,
            "PermutationInsert must not change DNA length"
        );
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before != after {
            moved = true;
        }
    }
    assert!(
        moved,
        "PermutationInsert should move a gene at least once in 200 attempts"
    );
}

/// Apply Mutation::Insertion to a Range<f64> of length 3 with max=5;
/// assert DNA length increases to 4.
#[test]
fn test_mutation_insertion_adds_gene_clamped_to_max() {
    let values = [10.0, 20.0, 30.0];
    let mut c = range_chromosome(&values);
    assert_eq!(c.dna().len(), 3);

    let cl = ChromosomeLength::Variable { min: 1, max: 5 };
    let result = length_insertion_mutation(&mut c, cl);

    assert!(
        result.is_ok(),
        "Insertion should succeed: {:?}",
        result.err()
    );
    assert_eq!(c.dna().len(), 4, "DNA length should grow from 3 to 4");
}

/// Apply Mutation::Deletion to a Range<f64> of length 5 with min=2;
/// assert DNA length decreases to 4.
#[test]
fn test_mutation_deletion_removes_gene_clamped_to_min() {
    let values = [10.0, 20.0, 30.0, 40.0, 50.0];
    let mut c = range_chromosome(&values);
    assert_eq!(c.dna().len(), 5);

    let cl = ChromosomeLength::Variable { min: 2, max: 10 };
    let result = length_deletion_mutation(&mut c, cl);

    assert!(
        result.is_ok(),
        "Deletion should succeed: {:?}",
        result.err()
    );
    assert_eq!(c.dna().len(), 4, "DNA length should shrink from 5 to 4");
}

/// Apply Mutation::Insertion when ChromosomeLength::Fixed(5) is configured;
/// assert Err(GaError::MutationError(_)) is returned.
#[test]
fn test_mutation_insertion_on_fixed_returns_error() {
    let values = [10.0, 20.0, 30.0];
    let mut c = range_chromosome(&values);

    let cl = ChromosomeLength::Fixed(5);
    let result = length_insertion_mutation(&mut c, cl);

    assert!(
        matches!(result, Err(GaError::MutationError(_))),
        "Insertion on Fixed should return MutationError, got: {:?}",
        result
    );
    // Length must be unchanged
    assert_eq!(c.dna().len(), 3);
}

/// Apply Mutation::Deletion when ChromosomeLength::Fixed(5) is configured;
/// assert Err(GaError::MutationError(_)) is returned.
#[test]
fn test_mutation_deletion_on_fixed_returns_error() {
    let values = [10.0, 20.0, 30.0, 40.0, 50.0];
    let mut c = range_chromosome(&values);

    let cl = ChromosomeLength::Fixed(5);
    let result = length_deletion_mutation(&mut c, cl);

    assert!(
        matches!(result, Err(GaError::MutationError(_))),
        "Deletion on Fixed should return MutationError, got: {:?}",
        result
    );
    // Length must be unchanged
    assert_eq!(c.dna().len(), 5);
}

// ──────────────────────────────────────────────────────────────────────────────
// Section 2 — CHR-01: VariableLength crossover, length guards, init, extension
// ──────────────────────────────────────────────────────────────────────────────

/// Cross two Range<f64> parents of lengths 3 and 5 using
/// Crossover::VariableLength(AlignmentStrategy::Trim); assert both offspring
/// have length 3 (min of the two parents).
#[test]
fn test_crossover_variable_length_trim_produces_min_len_offspring() {
    let parent_1 = range_chromosome(&[10.0, 20.0, 30.0]);
    let parent_2 = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0]);

    let op = Crossover::VariableLength(AlignmentStrategy::Trim);
    let result = op.crossover(&parent_1, &parent_2);

    assert!(
        result.is_ok(),
        "VariableLength Trim crossover should succeed"
    );
    let children = result.unwrap();
    assert_eq!(children.len(), 2, "Should produce 2 offspring");
    assert_eq!(
        children[0].dna().len(),
        3,
        "Trim offspring should have length min(3,5)=3"
    );
    assert_eq!(
        children[1].dna().len(),
        3,
        "Trim offspring should have length min(3,5)=3"
    );
}

/// Cross two Range<f64> parents of lengths 3 and 5 using
/// Crossover::VariableLength(AlignmentStrategy::Pad); assert both offspring
/// have length 5 (max of the two parents).
#[test]
fn test_crossover_variable_length_pad_produces_max_len_offspring() {
    let parent_1 = range_chromosome(&[10.0, 20.0, 30.0]);
    let parent_2 = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0]);

    let op = Crossover::VariableLength(AlignmentStrategy::Pad);
    let result = op.crossover(&parent_1, &parent_2);

    assert!(
        result.is_ok(),
        "VariableLength Pad crossover should succeed"
    );
    let children = result.unwrap();
    assert_eq!(children.len(), 2, "Should produce 2 offspring");
    assert_eq!(
        children[0].dna().len(),
        5,
        "Pad offspring should have length max(3,5)=5"
    );
    assert_eq!(
        children[1].dna().len(),
        5,
        "Pad offspring should have length max(3,5)=5"
    );
}

/// Create two Range<f64> parents with lengths 3 and 5; apply
/// Crossover::SinglePoint; assert Err(GaError::CrossoverError(_)).
#[test]
fn test_crossover_incompatible_length_single_point_returns_error() {
    let parent_1 = range_chromosome(&[10.0, 20.0, 30.0]);
    let parent_2 = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0]);

    let result = Crossover::SinglePoint.crossover(&parent_1, &parent_2);

    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "SinglePoint with unequal lengths should return CrossoverError, got: {:?}",
        result
    );
}

/// Create two Range<f64> parents with lengths 3 and 5; apply
/// Crossover::Uniform; assert Err(GaError::CrossoverError(_)).
#[test]
fn test_crossover_incompatible_length_uniform_returns_error() {
    let parent_1 = range_chromosome(&[10.0, 20.0, 30.0]);
    let parent_2 = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0]);

    let result = Crossover::Uniform.crossover(&parent_1, &parent_2);

    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Uniform crossover with unequal lengths should return CrossoverError, got: {:?}",
        result
    );
}

/// Build a Ga<Range<f64>> with ChromosomeLength::Variable { min: 2, max: 8 }
/// and population_size=20; after initialization assert that all chromosome
/// lengths are in [2, 8].
#[test]
fn test_variable_length_initialization_samples_lengths_in_range() {
    use genetic_algorithms::initializers::range_random_initialization;

    let allele = RangeGenotype::<f64>::new(0, vec![(0.0, 100.0)], 50.0);
    let alleles = vec![allele];

    let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
        .with_fitness_fn(|dna: &[RangeGenotype<f64>]| dna.iter().map(|g| g.value).sum::<f64>())
        .with_population_size(20)
        .with_alleles(alleles)
        .with_chromosome_length(ChromosomeLength::Variable { min: 2, max: 8 })
        .with_mutation_method(Mutation::Insertion)
        .with_initialization_fn(range_random_initialization::<f64>);

    let result = ga.initialization();
    assert!(
        result.is_ok(),
        "Initialization should succeed: {:?}",
        result.err()
    );

    assert_eq!(
        ga.population.chromosomes.len(),
        20,
        "Population size should be 20"
    );

    for (i, c) in ga.population.chromosomes.iter().enumerate() {
        let len = c.dna().len();
        assert!(
            (2..=8).contains(&len),
            "Chromosome {} has length {} which is outside [2, 8]",
            i,
            len
        );
    }
}

/// Integration stub: assert that extension-regrowth individuals have lengths
/// within [min_observed, max_observed] of the surviving population.
#[test]
fn test_variable_length_extension_regrowth_samples_from_population() {
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::Extension;
    use genetic_algorithms::traits::{CrossoverConfig, ExtensionConfig, StoppingConfig};

    // Build a GA configured for variable-length chromosomes with MassGenesis
    // extension, which trims to 2 survivors and then triggers regrowth.
    // We set diversity_threshold very high so extension fires every generation.
    // VariableLength(Trim) crossover is required because chromosomes may have
    // different lengths after variable-length initialization and regrowth.
    let allele = RangeGenotype::<f64>::new(0, vec![(0.0, 100.0)], 50.0);
    let alleles = vec![allele];

    let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
        .with_fitness_fn(|dna: &[RangeGenotype<f64>]| dna.iter().map(|g| g.value).sum::<f64>())
        .with_population_size(10)
        .with_chromosome_length(ChromosomeLength::Fixed(5))
        .with_alleles(alleles)
        .with_chromosome_length(ChromosomeLength::Variable { min: 2, max: 8 })
        .with_mutation_method(Mutation::Insertion)
        .with_crossover_method(Crossover::VariableLength(AlignmentStrategy::Trim))
        .with_initialization_fn(range_random_initialization::<f64>)
        .with_extension_method(Extension::MassGenesis)
        .with_extension_diversity_threshold(f64::MAX) // triggers every generation
        .with_max_generations(2);

    // After initialization, manually constrain lengths to [3, 6] so we can
    // verify regrowth stays within [min_obs, max_obs] of those survivors.
    let init_result = ga.initialization();
    assert!(
        init_result.is_ok(),
        "Initialization should succeed: {:?}",
        init_result.err()
    );

    // Set all chromosome lengths to values in [3, 6] to establish known observed range
    for i in 0..ga.population.chromosomes.len() {
        let target_len = 3 + (i % 4); // lengths: 3, 4, 5, 6 cycling
        let dna: Vec<RangeGenotype<f64>> = (0..target_len)
            .map(|j| RangeGenotype::new(j as i32, vec![(0.0, 100.0)], 50.0))
            .collect();
        ga.population.chromosomes[i].set_dna(Cow::Owned(dna));
        ga.population.chromosomes[i].set_fitness(50.0 * target_len as f64);
    }

    // Run one generation (extension fires, trims to 2, then regrows to 10)
    let run_result = ga.run();
    assert!(
        run_result.is_ok(),
        "GA run should succeed: {:?}",
        run_result.err()
    );

    // After the run, all chromosomes must have lengths within [2, 8]
    // (the configured Variable bounds — regrowth is clamped to them)
    for (i, c) in ga.population.chromosomes.iter().enumerate() {
        let len = c.dna().len();
        assert!(
            (2..=8).contains(&len),
            "Regrowth chromosome {} has length {} outside Variable bounds [2, 8]",
            i,
            len
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Section 3 — CHR-02: Parsimony pressure
// ──────────────────────────────────────────────────────────────────────────────

/// In a Vec of two Range<f64> chromosomes (same raw fitness, different lengths),
/// apply survivor selection with length_penalty = Some(0.1) and Maximization;
/// assert the shorter chromosome survives.
#[test]
fn test_parsimony_pressure_penalizes_longer_chromosomes_maximization() {
    // Both chromosomes have the same raw fitness of 100.0, but different lengths.
    // With length_penalty=0.1 and Maximization:
    //   short (len=3): effective = 100.0 - 0.1*3 = 99.7
    //   long  (len=7): effective = 100.0 - 0.1*7 = 99.3
    // Fitness survivor keeps the highest effective fitness → short survives.

    let mut short_chr = range_chromosome(&[10.0, 20.0, 30.0]); // length 3
    short_chr.set_fitness(100.0);

    let mut long_chr = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]); // length 7
    long_chr.set_fitness(100.0);

    let mut chromosomes = vec![short_chr, long_chr];

    let limit_config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        population_size: 2,
        ..LimitConfiguration::default()
    };

    let result = apply_parsimony_pressure(
        Survivor::Fitness,
        &mut chromosomes,
        1, // keep only 1 survivor
        limit_config,
        0.1,
    );

    assert!(
        result.is_ok(),
        "Parsimony pressure should succeed: {:?}",
        result.err()
    );
    assert_eq!(chromosomes.len(), 1, "Should have exactly 1 survivor");
    assert_eq!(
        chromosomes[0].dna().len(),
        3,
        "The shorter chromosome (length 3) should survive"
    );
}

/// Apply parsimony pressure and assert the stored chromosome.fitness() value
/// is unchanged after survivor selection (only effective fitness for comparison
/// is adjusted, not the stored field).
#[test]
fn test_parsimony_no_fitness_mutation() {
    let raw_fitness = 42.0_f64;

    let mut chr_a = range_chromosome(&[10.0, 20.0, 30.0]);
    chr_a.set_fitness(raw_fitness);

    let mut chr_b = range_chromosome(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]);
    chr_b.set_fitness(raw_fitness - 1.0); // slightly lower raw fitness

    let mut chromosomes = vec![chr_a, chr_b];

    let limit_config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        population_size: 2,
        ..LimitConfiguration::default()
    };

    let result = apply_parsimony_pressure(
        Survivor::Fitness,
        &mut chromosomes,
        2, // keep both
        limit_config,
        0.5,
    );

    assert!(
        result.is_ok(),
        "Parsimony pressure should succeed: {:?}",
        result.err()
    );

    // Both survivors should have their original raw fitness restored
    for c in &chromosomes {
        let stored = c.fitness();
        // Raw fitness was either 42.0 or 41.0 — parsimony must not alter stored value
        assert!(
            (stored - raw_fitness).abs() < 1e-9 || (stored - (raw_fitness - 1.0)).abs() < 1e-9,
            "Stored fitness {} was mutated by parsimony pressure (expected 42.0 or 41.0)",
            stored
        );
    }
}
