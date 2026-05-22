//! Unique chromosome initializer.
//!
//! Creates a full permutation of [`UniqueGenotype<T>`](crate::genotypes::UniqueGenotype)
//! genes for [`UniqueChromosome`](crate::chromosomes::UniqueChromosome) chromosomes.
//! Uses Fisher-Yates shuffling so the result is a uniformly random permutation of the
//! alphabet — no duplicate genes, all alphabet elements present.

use crate::genotypes::UniqueGenotype;
use rand::Rng;
use std::fmt::Debug;

/// Randomly initializes a chromosome's DNA as a full permutation of the given alphabet.
///
/// Uses Fisher-Yates shuffle to produce a uniformly random permutation. The returned
/// `Vec` has the same length as `alphabet` — every element appears exactly once.
///
/// This is the canonical initializer for [`UniqueChromosome<T>`](crate::chromosomes::UniqueChromosome).
/// Pass it via `with_initialization_fn`:
///
/// ```rust,ignore
/// let alphabet: Vec<i32> = (0..15).collect();
/// let ga = Ga::new()
///     .with_initialization_fn({
///         let alphabet = alphabet.clone();
///         move |_n, _| unique_random_initialization(&alphabet)
///     })
///     // ...
///     .build()?;
/// ```
///
/// # Arguments
///
/// * `alphabet` - The set of values to permute. An empty slice returns an empty `Vec`.
///
/// # Returns
///
/// A `Vec<UniqueGenotype<T>>` of length `alphabet.len()` representing a random permutation.
/// Each gene's `id` is set to the original position of the value in the shuffled index array.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::initializers::unique_random_initialization;
///
/// let alphabet = vec![10, 20, 30, 40, 50];
/// let dna = unique_random_initialization(&alphabet);
/// assert_eq!(dna.len(), 5);
///
/// // All alphabet elements are present exactly once.
/// let mut values: Vec<i32> = dna.iter().map(|g| g.value).collect();
/// values.sort();
/// assert_eq!(values, vec![10, 20, 30, 40, 50]);
/// ```
pub fn unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>
where
    T: Clone + Sync + Send + Default + Debug,
{
    if alphabet.is_empty() {
        return Vec::new();
    }

    let mut rng = crate::rng::make_rng();

    // Fisher-Yates shuffle of indices
    let mut indices: Vec<usize> = (0..alphabet.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    // Map shuffled indices to UniqueGenotype genes
    let mut dna = Vec::with_capacity(alphabet.len());
    for &idx in &indices {
        dna.push(UniqueGenotype {
            id: idx as i32,
            value: alphabet[idx].clone(),
        });
    }
    dna
}
