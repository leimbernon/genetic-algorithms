//! Tests for `MultiUniqueChromosome<T>` — GEN-04: group_ranges() correctness,
//! ChromosomeT + LinearChromosome + OperatorCompat impls for the multi-group
//! permutation chromosome type.

use genetic_algorithms::chromosomes::MultiUniqueChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::error::GaError;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::UniqueGenotype;
use genetic_algorithms::initializers::unique_random_initialization;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, OperatorCompat,
    SelectionConfig, StoppingConfig,
};
use std::borrow::Cow;
use std::sync::Arc;

// -- Helper: build a minimal Ga<MultiUniqueChromosome<i32>> with specific operators --

fn make_ga(
    crossover: Crossover,
    mutation: Mutation,
) -> Result<Ga<MultiUniqueChromosome<i32>>, GaError> {
    // One group of 5 elements for simplicity
    let alphabet: Vec<i32> = (0..5).collect();
    Ga::new()
        .with_population_size(10)
        .with_initialization_fn({
            let alphabet = alphabet.clone();
            move |_n, _| unique_random_initialization(&alphabet)
        })
        .with_fitness_fn(|dna: &[UniqueGenotype<i32>]| dna.len() as f64)
        .with_selection_method(Selection::Random)
        .with_crossover_method(crossover)
        .with_mutation_method(mutation)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(1)
        .build()
}

// -- Tests for new() and group alphabets --

/// `MultiUniqueChromosome::new()` with three groups produces correct alphabet contents.
#[test]
fn multi_unique_chromosome_new_produces_correct_alphabets() {
    let c =
        MultiUniqueChromosome::<i32>::new(vec![vec![0, 1, 2], vec![10, 20, 30], vec![100, 200]]);
    assert_eq!(c.groups.len(), 3);
    assert_eq!(&*c.groups[0], &[0, 1, 2]);
    assert_eq!(&*c.groups[1], &[10, 20, 30]);
    assert_eq!(&*c.groups[2], &[100, 200]);
    assert!(c.dna.is_empty(), "DNA initialized empty by new()");
}

// -- Tests for group_ranges() --

/// `group_ranges()` returns `[(0,2), (3,5), (6,7)]` for groups of sizes 3, 3, 2.
#[test]
fn group_ranges_three_groups() {
    let c =
        MultiUniqueChromosome::<i32>::new(vec![vec![0, 1, 2], vec![10, 20, 30], vec![100, 200]]);
    assert_eq!(c.group_ranges(), vec![(0, 2), (3, 5), (6, 7)]);
}

/// `group_ranges()` on an empty groups Vec returns an empty Vec.
#[test]
fn group_ranges_empty_groups_returns_empty() {
    let c = MultiUniqueChromosome::<i32>::default();
    assert_eq!(c.group_ranges(), vec![]);
}

/// `group_ranges()` on a single group of size 1 returns `[(0, 0)]`.
#[test]
fn group_ranges_single_element_group() {
    let c = MultiUniqueChromosome::<i32>::new(vec![vec![42]]);
    assert_eq!(c.group_ranges(), vec![(0, 0)]);
}

/// `group_ranges()` with two equal-size groups returns non-overlapping ranges.
#[test]
fn group_ranges_two_equal_groups() {
    let c = MultiUniqueChromosome::<i32>::new(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
    assert_eq!(c.group_ranges(), vec![(0, 3), (4, 7)]);
}

// -- Tests for Default --

/// Default returns empty DNA, empty groups, NaN fitness, age 0.
#[test]
fn multi_unique_chromosome_default() {
    let c = MultiUniqueChromosome::<i32>::default();
    assert!(c.dna.is_empty(), "default dna should be empty");
    assert!(c.groups.is_empty(), "default groups should be empty");
    assert!(c.fitness.is_nan(), "default fitness should be NaN");
    assert_eq!(c.age, 0, "default age should be 0");
}

// -- Tests for ChromosomeT --

/// `calculate_fitness` invokes the fitness function.
#[test]
fn multi_unique_chromosome_calculate_fitness_invokes_fn() {
    let mut c = MultiUniqueChromosome::<i32>::new(vec![vec![0, 1, 2]]);
    c.set_dna(Cow::Owned(vec![
        UniqueGenotype::new(0, 0),
        UniqueGenotype::new(1, 1),
        UniqueGenotype::new(2, 2),
    ]));
    c.set_fitness_fn(|dna: &[UniqueGenotype<i32>]| dna.len() as f64 * 10.0);
    c.calculate_fitness();
    assert_eq!(c.fitness(), 30.0);
}

/// `set_fitness` and `set_age` work correctly.
#[test]
fn multi_unique_chromosome_set_fitness_set_age() {
    let mut c = MultiUniqueChromosome::<i32>::default();
    c.set_fitness(7.5);
    assert_eq!(c.fitness(), 7.5);
    c.set_age(3);
    assert_eq!(c.age(), 3);
}

// -- Tests for LinearChromosome --

/// `set_dna(Cow::Owned(_))` replaces the DNA (no extra clone).
#[test]
fn multi_unique_chromosome_set_dna_cow_owned() {
    let mut c = MultiUniqueChromosome::<i32>::default();
    let genes = vec![UniqueGenotype::new(0, 99), UniqueGenotype::new(1, 88)];
    c.set_dna(Cow::Owned(genes.clone()));
    assert_eq!(c.dna(), genes.as_slice());
}

/// `set_dna(Cow::Borrowed(_))` clones into internal storage.
#[test]
fn multi_unique_chromosome_set_dna_cow_borrowed() {
    let mut c = MultiUniqueChromosome::<i32>::default();
    let genes = vec![UniqueGenotype::new(0, 11), UniqueGenotype::new(1, 22)];
    c.set_dna(Cow::Borrowed(genes.as_slice()));
    assert_eq!(c.dna(), genes.as_slice());
}

// -- Tests for OperatorCompat --

/// `valid_crossovers()` contains MultiGroupPmx and MultiGroupOx.
#[test]
fn multi_unique_chromosome_valid_crossovers_contains_multi_group_variants() {
    let valid = <MultiUniqueChromosome<i32> as OperatorCompat>::valid_crossovers();
    let valid_slice = valid.expect("MultiUniqueChromosome should restrict crossovers");
    assert!(valid_slice.contains(&Crossover::MultiGroupPmx));
    assert!(valid_slice.contains(&Crossover::MultiGroupOx));
    assert!(valid_slice.contains(&Crossover::Clone));
    assert!(valid_slice.contains(&Crossover::Rejuvenate));
    // Standard PMX and Order are excluded
    assert!(!valid_slice.contains(&Crossover::Pmx));
    assert!(!valid_slice.contains(&Crossover::Order));
}

/// `valid_mutations()` contains only permutation-safe mutations.
#[test]
fn multi_unique_chromosome_valid_mutations_restricted() {
    let valid = <MultiUniqueChromosome<i32> as OperatorCompat>::valid_mutations();
    let valid_slice = valid.expect("MultiUniqueChromosome should restrict mutations");
    assert!(valid_slice.contains(&Mutation::Insertion));
    assert!(valid_slice.contains(&Mutation::Swap));
    assert!(valid_slice.contains(&Mutation::Inversion));
    // Gaussian is excluded
    assert!(!valid_slice.contains(&Mutation::Gaussian(GaussianParams { sigma: None })));
}

// -- Integration tests with Ga::build() --

/// Ga::build() with Crossover::Pmx + MultiUniqueChromosome returns ConfigurationError.
#[test]
fn pmx_crossover_rejected_at_build() {
    let result = make_ga(Crossover::Pmx, Mutation::Swap);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for Pmx crossover with MultiUniqueChromosome"
    );
}

/// Ga::build() with Crossover::SinglePoint + MultiUniqueChromosome returns ConfigurationError.
#[test]
fn single_point_crossover_rejected_at_build() {
    let result = make_ga(Crossover::SinglePoint, Mutation::Swap);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for SinglePoint crossover with MultiUniqueChromosome"
    );
}

/// Ga::build() with Crossover::Order + MultiUniqueChromosome returns ConfigurationError.
#[test]
fn order_crossover_rejected_at_build() {
    let result = make_ga(Crossover::Order, Mutation::Swap);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for Order crossover with MultiUniqueChromosome"
    );
}

/// Ga::build() with Crossover::MultiGroupPmx + Mutation::Swap succeeds.
#[test]
fn multi_group_pmx_crossover_swap_mutation_accepted() {
    let result = make_ga(Crossover::MultiGroupPmx, Mutation::Swap);
    // MultiGroupPmx is in valid_crossovers for MultiUniqueChromosome
    // With one group of 5 elements, this should succeed at build time
    // (may fail at run time if Ga dispatch is not wired, but build() should succeed)
    match &result {
        Ok(_) => {}
        Err(GaError::ConfigurationError(msg)) => {
            panic!("Expected Ok but got ConfigurationError: {}", msg);
        }
        Err(e) => {
            panic!("Expected Ok but got error: {:?}", e);
        }
    }
}

/// Ga::build() with Crossover::MultiGroupOx + Mutation::Inversion succeeds.
#[test]
fn multi_group_ox_crossover_inversion_mutation_accepted() {
    let result = make_ga(Crossover::MultiGroupOx, Mutation::Inversion);
    match &result {
        Ok(_) => {}
        Err(GaError::ConfigurationError(msg)) => {
            panic!("Expected Ok but got ConfigurationError: {}", msg);
        }
        Err(e) => {
            panic!("Expected Ok but got error: {:?}", e);
        }
    }
}

/// Groups field is Vec<Arc<[T]>> and clone shares the same Arc allocation.
#[test]
fn groups_is_arc_shared() {
    let c = MultiUniqueChromosome::<i32>::new(vec![vec![1, 2, 3]]);
    let c2 = c.clone();
    // Arc::ptr_eq verifies the same underlying allocation is shared
    assert!(
        Arc::ptr_eq(&c.groups[0], &c2.groups[0]),
        "Cloned chromosome should share the same Arc<[T]> allocation for groups"
    );
}
