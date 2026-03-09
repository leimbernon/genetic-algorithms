//! Shared type aliases and helper functions used across GA orchestrators.
//!
//! This module centralizes the common `InitializationFn` and `FitnessFn` type
//! aliases as well as the `initialize_chromosomes` helper that all three
//! orchestrators (`Ga`, `IslandGa`, `Nsga2Ga`) use during population setup.

use crate::traits::{ChromosomeT, GeneT};
use rayon::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;

/// Type alias for the initialization function signature.
///
/// The function receives:
/// - `genes_per_chromosome`: number of genes each chromosome should contain.
/// - `alleles`: optional slice of allele templates.
/// - `needs_unique_ids`: optional flag indicating whether gene IDs must be unique.
///
/// Returns a `Vec<G>` representing the DNA for one chromosome.
pub type InitializationFn<G> = dyn Fn(usize, Option<&[G]>, Option<bool>) -> Vec<G> + Send + Sync;

/// Type alias for the fitness function signature.
///
/// Receives a chromosome's DNA slice and returns a scalar fitness value.
pub type FitnessFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;

/// Creates a single chromosome by invoking the initialization function and
/// optionally attaching a fitness function.
///
/// This is the core building block used by both [`initialize_chromosomes`] and
/// [`initialize_chromosomes_par`].
fn build_one_chromosome<U>(
    genes_per_chromosome: usize,
    alleles: Option<&[U::Gene]>,
    alleles_flag: Option<bool>,
    init_fn: &InitializationFn<U::Gene>,
    fitness_fn: Option<&Arc<FitnessFn<U::Gene>>>,
    age: usize,
) -> U
where
    U: ChromosomeT,
    U::Gene: GeneT,
{
    let dna = init_fn(genes_per_chromosome, alleles, alleles_flag);
    let mut chromosome = U::new();
    chromosome.set_dna(Cow::Owned(dna));

    if let Some(ff) = fitness_fn {
        let ff_clone = Arc::clone(ff);
        chromosome.set_fitness_fn(move |genes| ff_clone(genes));
        chromosome.calculate_fitness();
    }

    chromosome.set_age(age);
    chromosome
}

/// Creates a batch of chromosomes sequentially.
///
/// This helper encapsulates the boilerplate shared by `IslandGa::initialize`
/// and `Nsga2Ga::initialize_population`:
///
/// 1. Call `init_fn` to generate DNA.
/// 2. Create a new chromosome and set its DNA via `Cow::Owned`.
/// 3. If a `fitness_fn` is provided, attach a clone and compute fitness.
/// 4. Set the chromosome's age.
///
/// # Arguments
///
/// * `count` - Number of chromosomes to create.
/// * `genes_per_chromosome` - Passed to `init_fn`.
/// * `alleles` - Passed to `init_fn`.
/// * `alleles_flag` - Passed as the third argument to `init_fn` (e.g. `needs_unique_ids` or `alleles_can_be_repeated`).
/// * `init_fn` - The initialization function that generates DNA.
/// * `fitness_fn` - Optional fitness function. When `Some`, each chromosome gets a clone and its fitness is calculated.
/// * `age` - The age to assign to each chromosome.
///
/// # Returns
///
/// A `Vec<U>` of fully initialized chromosomes.
pub fn initialize_chromosomes<U>(
    count: usize,
    genes_per_chromosome: usize,
    alleles: Option<&[U::Gene]>,
    alleles_flag: Option<bool>,
    init_fn: &Arc<InitializationFn<U::Gene>>,
    fitness_fn: Option<&Arc<FitnessFn<U::Gene>>>,
    age: usize,
) -> Vec<U>
where
    U: ChromosomeT,
    U::Gene: GeneT,
{
    (0..count)
        .map(|_| {
            build_one_chromosome(
                genes_per_chromosome,
                alleles,
                alleles_flag,
                init_fn.as_ref(),
                fitness_fn,
                age,
            )
        })
        .collect()
}

/// Creates a batch of chromosomes in parallel using rayon.
///
/// Same semantics as [`initialize_chromosomes`] but distributes work across
/// the rayon thread pool.
pub fn initialize_chromosomes_par<U>(
    count: usize,
    genes_per_chromosome: usize,
    alleles: Option<&[U::Gene]>,
    alleles_flag: Option<bool>,
    init_fn: &Arc<InitializationFn<U::Gene>>,
    fitness_fn: Option<&Arc<FitnessFn<U::Gene>>>,
    age: usize,
) -> Vec<U>
where
    U: ChromosomeT,
    U::Gene: GeneT,
{
    (0..count)
        .into_par_iter()
        .map(|_| {
            build_one_chromosome(
                genes_per_chromosome,
                alleles,
                alleles_flag,
                init_fn.as_ref(),
                fitness_fn,
                age,
            )
        })
        .collect()
}
