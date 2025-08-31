use std::fmt::Debug;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use crate::genotypes::Range as RangeGenotype;

/// Initializes a vector of `RangeGenotype` with random values.
///
/// # Arguments
///
/// * `genes_per_chromosome` - The number of genes per chromosome.
/// * `alleles` - An optional slice of `RangeGenotype` to use as a source of alleles.
/// * `_needs_unique_ids` - An optional boolean indicating if unique IDs are needed (not used in this function).
///
/// # Returns
///
/// A vector of `RangeGenotype` with random values.
///
/// # Panics
///
/// This function will panic if no alleles are provided.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::Range;
/// use genetic_algorithms::initializers::range_random_initialization;
///
/// let alleles = vec![Range::new(0, vec![(0.0, 1.0)], 0.0)];
/// let genes = range_random_initialization(10, Some(&alleles), Some(false));
/// assert_eq!(genes.len(), 10);
/// ```
pub fn range_random_initialization<T>(genes_per_chromosome: i32, alleles: Option<&[RangeGenotype<T>]>, _needs_unique_ids: Option<bool>) -> Vec<RangeGenotype<T>>
where
    T: Sync + Send + Clone + Default + Debug + 'static + PartialOrd + SampleUniform + Copy,
{
    let mut genes = Vec::new();
    let mut rng = rand::rng();
    for i in 0..genes_per_chromosome {
        if let Some(alleles) = alleles {
            // Pick a random allele from the provided list
            let allele = alleles[rng.random_range(0..alleles.len())].clone();
            // Pick a random range from the allele
            let range = allele.ranges[rng.random_range(0..allele.ranges.len())].clone();
            // Generate a new gene with a random value from the selected range
            let gene = RangeGenotype::new(i, vec![range], rng.random_range(range.0..range.1));
            genes.push(gene);
        } else {
            panic!("At least 1 allele must be provided for range genotype");
        }
    }
    genes
}