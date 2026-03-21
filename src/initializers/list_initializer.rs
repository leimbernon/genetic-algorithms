//! List chromosome initializer.
//!
//! Creates random [`List<T>`](crate::genotypes::List) genes for
//! [`ListChromosome`](crate::chromosomes::ListChromosome) chromosomes by
//! sampling from allele sets defined in template genes.

use crate::genotypes::List;
use rand::Rng;
use std::fmt::Debug;

/// Randomly initializes a chromosome's DNA for a list-encoded chromosome,
/// sampling allele indices **with** replacement.
///
/// Each gene position is filled by:
/// 1. Picking a random template gene from `alleles`.
/// 2. Picking a random allele index from that template's allele set.
/// 3. Constructing a `List<T>` gene with that index and the corresponding value.
///
/// # Arguments
///
/// * `genes_per_chromosome` — Number of genes to produce.
/// * `alleles` — Template genes carrying the allele sets to sample from.
/// * `_needs_unique_ids` — Unused; accepted for API consistency.
///
/// # Returns
///
/// A `Vec<List<T>>` of length `genes_per_chromosome`.
///
/// # Panics
///
/// Panics if `alleles` is `None`.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::List;
/// use genetic_algorithms::initializers::list_random_initialization;
///
/// let templates = vec![List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap()];
/// let genes = list_random_initialization(5, Some(&templates), None);
/// assert_eq!(genes.len(), 5);
/// ```
pub fn list_random_initialization<T>(
    genes_per_chromosome: usize,
    alleles: Option<&[List<T>]>,
    _needs_unique_ids: Option<bool>,
) -> Vec<List<T>>
where
    T: Clone + Sync + Send + Default + Debug,
{
    let alleles = alleles.expect("Alleles must be provided for list_random_initialization");
    let mut rng = crate::rng::make_rng();
    let mut dna = Vec::with_capacity(genes_per_chromosome);

    for _ in 0..genes_per_chromosome {
        let template = &alleles[rng.random_range(0..alleles.len())];
        let index = rng.random_range(0..template.alleles.len());
        let gene = List::new(
            index as i32,
            template.alleles.clone(),
            template.alleles[0].clone(),
        )
        .expect("list_random_initialization: index always valid");
        dna.push(gene);
    }

    dna
}

/// Randomly initializes a chromosome's DNA for a list-encoded chromosome,
/// sampling allele indices **without** replacement (permutation semantics).
///
/// Uses the first template gene's allele set as the permutation source.
/// Applies Fisher-Yates shuffle to pick `genes_per_chromosome` unique indices.
///
/// # Arguments
///
/// * `genes_per_chromosome` — Number of genes to produce (must be ≤ `alleles.len()`).
/// * `alleles` — Template genes; the first template's allele set is used.
/// * `_needs_unique_ids` — Unused; accepted for API consistency.
///
/// # Returns
///
/// A `Vec<List<T>>` of length `genes_per_chromosome` with no duplicate allele indices.
///
/// # Panics
///
/// - Panics if `alleles` is `None`.
/// - Panics if `genes_per_chromosome` exceeds the template's allele set size.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::List;
/// use genetic_algorithms::initializers::list_random_initialization_without_repetitions;
///
/// let templates = vec![List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap()];
/// let genes = list_random_initialization_without_repetitions(3, Some(&templates), None);
/// assert_eq!(genes.len(), 3);
/// ```
pub fn list_random_initialization_without_repetitions<T>(
    genes_per_chromosome: usize,
    alleles: Option<&[List<T>]>,
    _needs_unique_ids: Option<bool>,
) -> Vec<List<T>>
where
    T: Clone + Sync + Send + Default + Debug,
{
    let alleles = alleles
        .expect("Alleles must be provided for list_random_initialization_without_repetitions");
    let template = &alleles[0];

    assert!(
        genes_per_chromosome <= template.alleles.len(),
        "genes_per_chromosome exceeds allele set size for without-repetitions initialization"
    );

    let mut rng = crate::rng::make_rng();

    // Build a shuffled index list via Fisher-Yates
    let mut indices: Vec<usize> = (0..template.alleles.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    let mut dna = Vec::with_capacity(genes_per_chromosome);
    for &idx in indices.iter().take(genes_per_chromosome) {
        let gene = List::new(
            idx as i32,
            template.alleles.clone(),
            template.alleles[0].clone(),
        )
        .expect("list_random_initialization_without_repetitions: index always valid");
        dna.push(gene);
    }

    dna
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genotypes::List;
    use std::collections::HashSet;

    fn make_templates() -> Vec<List<char>> {
        vec![List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap()]
    }

    // Test: list_random_initialization returns correct length
    #[test]
    fn list_initializer_returns_correct_length() {
        let templates = make_templates();
        let genes = list_random_initialization(5, Some(&templates), None);
        assert_eq!(genes.len(), 5);
    }

    // Test: each gene's id is within valid range
    #[test]
    fn list_initializer_ids_in_valid_range() {
        let templates = make_templates();
        let genes = list_random_initialization(20, Some(&templates), None);
        for gene in &genes {
            assert!(
                (gene.id as usize) < gene.alleles.len(),
                "gene.id {} out of range (alleles.len() = {})",
                gene.id,
                gene.alleles.len()
            );
            assert!(gene.id >= 0, "gene.id must be non-negative");
        }
    }

    // Test: value == alleles[id] consistency
    #[test]
    fn list_initializer_value_consistency() {
        let templates = make_templates();
        let genes = list_random_initialization(20, Some(&templates), None);
        for gene in &genes {
            assert_eq!(
                gene.value, gene.alleles[gene.id as usize],
                "value must equal alleles[id]"
            );
        }
    }

    // Test: allele sets are preserved from templates
    #[test]
    fn list_initializer_alleles_preserved() {
        let templates = make_templates();
        let genes = list_random_initialization(10, Some(&templates), None);
        for gene in &genes {
            assert_eq!(gene.alleles, vec!['a', 'b', 'c', 'd']);
        }
    }

    // Test: panics on None alleles
    #[test]
    #[should_panic(expected = "Alleles must be provided for list_random_initialization")]
    fn list_initializer_panics_on_none_alleles() {
        list_random_initialization::<char>(5, None, None);
    }

    // Test: without_repetitions returns correct length
    #[test]
    fn list_initializer_without_repetitions_correct_length() {
        let templates = make_templates();
        let genes = list_random_initialization_without_repetitions(3, Some(&templates), None);
        assert_eq!(genes.len(), 3);
    }

    // Test: without_repetitions has no duplicate allele indices
    #[test]
    fn list_initializer_without_repetitions_no_duplicate_ids() {
        let templates = make_templates();
        for seed in 0..20u64 {
            crate::rng::set_seed(Some(seed));
            let genes =
                list_random_initialization_without_repetitions(4, Some(&templates), None);
            let ids: HashSet<i32> = genes.iter().map(|g| g.id).collect();
            assert_eq!(
                ids.len(),
                genes.len(),
                "duplicate allele indices found (seed {})",
                seed
            );
        }
        crate::rng::set_seed(None);
    }

    // Test: without_repetitions panics when genes_per_chromosome > alleles.len()
    #[test]
    #[should_panic(
        expected = "genes_per_chromosome exceeds allele set size for without-repetitions initialization"
    )]
    fn list_initializer_without_repetitions_panics_on_overflow() {
        let templates = make_templates(); // alleles.len() == 4
        list_random_initialization_without_repetitions(10, Some(&templates), None);
    }

    // Test: without_repetitions value consistency
    #[test]
    fn list_initializer_without_repetitions_value_consistency() {
        let templates = make_templates();
        let genes = list_random_initialization_without_repetitions(4, Some(&templates), None);
        for gene in &genes {
            assert_eq!(
                gene.value, gene.alleles[gene.id as usize],
                "value must equal alleles[id]"
            );
        }
    }
}
