//! Tests for `UniqueChromosome<T>` — GEN-01: ChromosomeT + LinearChromosome + OperatorCompat
//! impls for the unique permutation chromosome type.

use genetic_algorithms::chromosomes::UniqueChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::error::GaError;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::UniqueGenotype;
use genetic_algorithms::initializers::unique_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig,
    SelectionConfig, StoppingConfig,
};
use std::borrow::Cow;
use std::sync::Arc;

// -- Helper: build a minimal Ga<UniqueChromosome<i32>> with specific operators -

fn make_ga(crossover: Crossover, mutation: Mutation) -> Result<Ga<UniqueChromosome<i32>>, GaError> {
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

// -- Tests --------------------------------------------------------------------

/// Default returns empty dna, empty alphabet, NaN fitness, age 0.
#[test]
fn unique_chromosome_default() {
    let c = UniqueChromosome::<i32>::default();
    assert!(c.dna.is_empty(), "default dna should be empty");
    assert_eq!(
        c.alphabet.len(),
        0,
        "default alphabet should be empty (Arc::from([]))"
    );
    assert!(c.fitness.is_nan(), "default fitness should be NaN");
    assert_eq!(c.age, 0, "default age should be 0");
}

/// Ga::build() with Crossover::SinglePoint + UniqueChromosome<i32> returns ConfigurationError.
#[test]
fn single_point_crossover_rejected_at_build() {
    let result = make_ga(Crossover::SinglePoint, Mutation::Insertion);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for SinglePoint crossover with UniqueChromosome"
    );
}

/// Ga::build() with Crossover::Pmx + UniqueChromosome<i32> succeeds.
#[test]
fn pmx_crossover_accepted_at_build() {
    let result = make_ga(Crossover::Pmx, Mutation::Insertion);
    assert!(
        result.is_ok(),
        "Expected Ok for Pmx crossover with UniqueChromosome"
    );
}

/// Ga::build() with Mutation::Gaussian + UniqueChromosome<i32> returns ConfigurationError.
#[test]
fn gaussian_mutation_rejected_at_build() {
    let result = make_ga(Crossover::Pmx, Mutation::Gaussian { sigma: None });
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for Gaussian mutation with UniqueChromosome"
    );
}

/// Ga::build() with Crossover::Order + Mutation::Swap succeeds.
#[test]
fn order_crossover_swap_mutation_accepted() {
    let result = make_ga(Crossover::Order, Mutation::Swap);
    assert!(
        result.is_ok(),
        "Expected Ok for Order crossover + Swap mutation with UniqueChromosome"
    );
}

/// ChromosomeT::calculate_fitness invokes the wrapped fitness function.
#[test]
fn calculate_fitness_invokes_fn() {
    let mut c = UniqueChromosome::<i32>::default();
    c.set_fitness_fn(|dna: &[UniqueGenotype<i32>]| dna.len() as f64 * 2.0);
    let genes = vec![
        UniqueGenotype::new(0, 10i32),
        UniqueGenotype::new(1, 20i32),
        UniqueGenotype::new(2, 30i32),
    ];
    c.set_dna(Cow::Owned(genes));
    c.calculate_fitness();
    assert_eq!(c.fitness(), 6.0, "fitness should be len * 2.0 = 6.0");
}

/// LinearChromosome::set_dna with Cow::Owned replaces the dna and returns &mut Self.
#[test]
fn set_dna_cow_owned_replaces_dna() {
    let mut c = UniqueChromosome::<i32>::default();
    let genes = vec![UniqueGenotype::new(0, 1i32), UniqueGenotype::new(1, 2i32)];
    let ret = c.set_dna(Cow::Owned(genes.clone()));
    assert_eq!(
        ret.dna(),
        genes.as_slice(),
        "set_dna should replace the dna"
    );
    assert_eq!(c.dna.len(), 2, "dna len should be 2 after set_dna");
}

/// LinearChromosome::set_dna with Cow::Borrowed also replaces the dna.
#[test]
fn set_dna_cow_borrowed_replaces_dna() {
    let mut c = UniqueChromosome::<i32>::default();
    let genes = vec![UniqueGenotype::new(0, 42i32)];
    c.set_dna(Cow::Borrowed(genes.as_slice()));
    assert_eq!(c.dna.len(), 1);
    assert_eq!(c.dna[0].value, 42);
}

/// alphabet field uses Arc<[T]> — cloning is O(1) atomic refcount.
#[test]
#[allow(clippy::field_reassign_with_default)]
fn alphabet_is_arc_shared() {
    let mut c = UniqueChromosome::<i32>::default();
    c.alphabet = Arc::from(vec![1i32, 2, 3, 4, 5]);
    let c2 = c.clone();
    assert_eq!(c2.alphabet.len(), 5);
    // Shared — both point to same allocation
    assert!(Arc::ptr_eq(&c.alphabet, &c2.alphabet));
}

/// OperatorCompat valid_crossovers returns a restricted Some set.
#[test]
fn valid_crossovers_is_restricted() {
    let valid =
        <UniqueChromosome<i32> as genetic_algorithms::traits::OperatorCompat>::valid_crossovers();
    assert!(
        valid.is_some(),
        "UniqueChromosome should have a restricted crossover set"
    );
    let v = valid.unwrap();
    assert!(v.contains(&Crossover::Pmx));
    assert!(v.contains(&Crossover::Order));
    assert!(v.contains(&Crossover::EdgeRecombination));
    assert!(!v.contains(&Crossover::SinglePoint));
    assert!(!v.contains(&Crossover::Uniform));
}

/// OperatorCompat valid_mutations returns a restricted Some set.
#[test]
fn valid_mutations_is_restricted() {
    let valid =
        <UniqueChromosome<i32> as genetic_algorithms::traits::OperatorCompat>::valid_mutations();
    assert!(
        valid.is_some(),
        "UniqueChromosome should have a restricted mutation set"
    );
    let v = valid.unwrap();
    assert!(v.contains(&Mutation::Insertion));
    assert!(v.contains(&Mutation::Swap));
    assert!(v.contains(&Mutation::Inversion));
    assert!(!v.contains(&Mutation::Gaussian { sigma: None }));
    assert!(!v.contains(&Mutation::Value));
}
