use rand::Rng;
use crate::genotypes::Binary as BinaryGenotype;

/// Initializes a vector of `Binary` genes with random values.
///
/// # Arguments
///
/// * `genes_per_chromosome` - The number of genes per chromosome.
/// * `_alleles` - An optional slice of `Binary` to use as a source of alleles (not used in this function).
/// * `_needs_unique_ids` - An optional boolean indicating if unique IDs are needed (not used in this function).
///
/// # Returns
///
/// A vector of `Binary` genes with random values.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::initializers::binary_random_initialization;
///
/// let genes = binary_random_initialization(100, None, None);
/// assert_eq!(genes.len(), 100);
/// ```
pub fn binary_random_initialization(genes_per_chromosome: i32, _alleles: Option<&[BinaryGenotype]>, _needs_unique_ids: Option<bool>) -> Vec<BinaryGenotype> {
    let mut genes = Vec::new();
    let mut rng = rand::rng();
    for i in 0..genes_per_chromosome {
        let gene = BinaryGenotype{id:i, value: rng.random_bool(0.5)};
        genes.push(gene);
    }
    genes
}