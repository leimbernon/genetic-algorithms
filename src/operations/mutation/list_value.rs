//! List-value mutation operator for list-encoded chromosomes.
//!
//! Replaces a single gene's value with a different allele drawn from that
//! gene's own allele set. The "different" constraint is enforced by allele
//! index (gene.id), avoiding any `PartialEq` requirement on T.

use crate::chromosomes::ListChromosome;
use crate::traits::LinearChromosome;
use std::borrow::Cow;
use std::fmt::Debug;

use super::ValueMutable;

/// List-value mutation for `ListChromosome<T>`.
///
/// - Randomly selects one gene from the DNA.
/// - Picks a *different* allele index from that gene's allele set.
/// - Updates the gene's `id` and `value` to reflect the new allele.
/// - Writes back the mutated DNA into the individual.
///
/// If the chromosome has no genes, or the selected gene has fewer than 2
/// alleles, the function returns without modifying the chromosome (no-op).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::list_value_mutation;
/// use genetic_algorithms::chromosomes::ListChromosome;
/// let mut chromosome: ListChromosome<i32> = ListChromosome::new();
/// list_value_mutation(&mut chromosome);
/// ```
pub fn list_value_mutation<T>(individual: &mut ListChromosome<T>)
where
    T: Clone + Sync + Send + Default + Debug + 'static,
{
    let len = individual.dna().len();
    if len == 0 {
        return;
    }

    let mut rng = crate::rng::make_rng();
    use rand::Rng;
    let idx = rng.random_range(0..len);

    let mut dna = individual.dna().to_vec();
    let gene = &mut dna[idx];

    if gene.alleles.len() < 2 {
        return;
    }

    let current_index = gene.id as usize;
    let new_index = loop {
        let n = rng.random_range(0..gene.alleles.len());
        if n != current_index {
            break n;
        }
    };

    gene.id = new_index as i32;
    gene.value = gene.alleles[new_index].clone();

    individual.set_dna(Cow::Owned(dna));
}

/// `ValueMutable` implementation for `ListChromosome<T>`.
///
/// Overrides `value_mutate` to call [`list_value_mutation`], which replaces
/// a single gene's allele index with a different one.
impl<T: Clone + Sync + Send + Default + Debug + 'static> ValueMutable for ListChromosome<T> {
    fn value_mutate(&mut self) {
        list_value_mutation(self);
    }
}
