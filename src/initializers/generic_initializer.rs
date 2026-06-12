//! Generic initializer for allele-based chromosomes.
//!
//! Provides two initialization functions that create chromosome DNA by
//! randomly sampling from a user-supplied alleles list — with or without
//! repetition.

use crate::traits::{ChromosomeT, GeneT};
use rand::Rng;

/// Randomly initializes a chromosome's DNA by sampling alleles **with** replacement.
///
/// Each gene position is filled by picking a random allele from the provided
/// list. The same allele may appear more than once in the resulting DNA.
/// Gene IDs are set to sequential integers starting from 0.
///
/// # Panics
///
/// Panics if `alleles` is `None`.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
/// use genetic_algorithms::initializers::generic_initializer::generic_random_initialization;
///
/// let alleles = vec![BinaryGene::default(), BinaryGene::default()];
/// let dna = generic_random_initialization::<Binary>(4, Some(&alleles));
/// assert_eq!(dna.len(), 4);
/// ```
pub fn generic_random_initialization<U>(
    genes_per_chromosome: usize,
    alleles: Option<&[U::Gene]>,
) -> Vec<U::Gene>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    let alleles = alleles.expect("Alleles must be provided for generic_random_initialization");

    let mut rng = crate::rng::make_rng();
    let mut dna = Vec::new();

    //Selects the genes randomly from the vector without repeating them
    for j in 0..genes_per_chromosome {
        let index = rng.random_range(0..alleles.len());
        let mut gene = alleles.get(index).cloned().unwrap();
        gene.set_id(j as i32);
        dna.push(gene);
    }

    dna
}

/// Randomly initializes a chromosome's DNA by sampling alleles **without** replacement.
///
/// Each selected allele is removed from the candidate pool, guaranteeing that
/// no allele appears more than once. Requires `alleles.len() >= genes_per_chromosome`.
/// Gene IDs are set to sequential integers starting from 0.
///
/// # Panics
///
/// Panics if `alleles` is `None`.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
/// use genetic_algorithms::initializers::generic_initializer::generic_random_initialization_without_repetitions;
///
/// let alleles = vec![BinaryGene::default(), BinaryGene::default(), BinaryGene::default(), BinaryGene::default()];
/// let dna = generic_random_initialization_without_repetitions::<Binary>(3, Some(&alleles));
/// assert_eq!(dna.len(), 3);
/// ```
pub fn generic_random_initialization_without_repetitions<U>(
    genes_per_chromosome: usize,
    alleles: Option<&[U::Gene]>,
) -> Vec<U::Gene>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    let alleles = alleles
        .expect("Alleles must be provided for generic_random_initialization_without_repetitions");

    let mut rng = crate::rng::make_rng();
    let mut dna = Vec::new();

    let mut tmp_alleles = alleles.to_vec().clone();

    //Selects the genes randomly from the vector without repeating them
    for j in 0..genes_per_chromosome {
        let index = rng.random_range(0..tmp_alleles.len());
        let mut gene = tmp_alleles.get(index).cloned().unwrap();
        gene.set_id(j as i32);
        tmp_alleles.remove(index);
        dna.push(gene);
    }

    dna
}
