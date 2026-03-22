//! List-value mutation operator for list-encoded chromosomes.
//!
//! Replaces a single gene's value with a different allele drawn from that
//! gene's own allele set. The "different" constraint is enforced by allele
//! index (gene.id), avoiding any `PartialEq` requirement on T.

use crate::chromosomes::ListChromosome;
use crate::traits::ChromosomeT;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::ListChromosome;
    use crate::genotypes::List;
    use crate::operations::mutation::factory;
    use crate::operations::mutation::factory_non_value;
    use crate::operations::Mutation;
    use crate::error::GaError;

    fn make_gene(id: i32, alleles: Vec<char>) -> List<char> {
        List::new(id, alleles, 'a').unwrap()
    }

    fn make_chromosome_5genes() -> ListChromosome<char> {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b', 'c', 'd']));
        c.dna.push(make_gene(1, vec!['a', 'b', 'c', 'd']));
        c.dna.push(make_gene(2, vec!['a', 'b', 'c', 'd']));
        c.dna.push(make_gene(3, vec!['a', 'b', 'c', 'd']));
        c.dna.push(make_gene(0, vec!['a', 'b', 'c', 'd']));
        c
    }

    // Test: list_value_mutation on a 5-gene chromosome changes exactly 1 gene
    #[test]
    fn list_value_mutation_changes_exactly_one_gene() {
        let mut c = make_chromosome_5genes();
        let original_ids: Vec<i32> = c.dna.iter().map(|g| g.id).collect();
        let original_values: Vec<char> = c.dna.iter().map(|g| g.value).collect();

        list_value_mutation(&mut c);

        let changed = c
            .dna
            .iter()
            .enumerate()
            .filter(|(i, g)| g.id != original_ids[*i] || g.value != original_values[*i])
            .count();
        assert_eq!(changed, 1, "Exactly 1 gene should change; got {}", changed);
    }

    // Test: mutation never picks the same allele index
    #[test]
    fn list_value_mutation_picks_different_allele_index() {
        // Run many times to verify the constraint holds
        for seed in 0..50u64 {
            crate::rng::set_seed(Some(seed));
            let mut c = ListChromosome::<char>::new();
            c.dna.push(make_gene(0, vec!['a', 'b', 'c'])); // current index = 0
            let before_id = c.dna[0].id;

            list_value_mutation(&mut c);

            assert_ne!(
                c.dna[0].id, before_id,
                "mutation must pick a different allele index (seed {})",
                seed
            );
        }
        crate::rng::set_seed(None);
    }

    // Test: single-allele chromosome is a no-op
    #[test]
    fn list_value_mutation_single_allele_is_noop() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(List::new(0, vec!['x'], 'x').unwrap());
        c.dna.push(make_gene(1, vec!['a', 'b', 'c', 'd']));
        // Force the mutation to select the single-allele gene (index 0)
        // We can't force it, so run many times and check no panic/inconsistency
        for seed in 0..20u64 {
            crate::rng::set_seed(Some(seed));
            let mut c2 = c.clone();
            list_value_mutation(&mut c2);
            // single-allele gene must remain unchanged
            assert_eq!(c2.dna[0].id, 0);
            assert_eq!(c2.dna[0].value, 'x');
        }
        crate::rng::set_seed(None);
    }

    // Test: empty chromosome is a no-op
    #[test]
    fn list_value_mutation_empty_chromosome_is_noop() {
        let mut c = ListChromosome::<char>::new();
        list_value_mutation(&mut c);
        assert!(c.dna().is_empty());
    }

    // Test: all genes have single alleles — function returns without change
    #[test]
    fn list_value_mutation_all_single_allele_is_noop() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(List::new(0, vec!['a'], 'a').unwrap());
        c.dna.push(List::new(0, vec!['b'], 'b').unwrap());
        c.dna.push(List::new(0, vec!['c'], 'c').unwrap());
        let dna_before: Vec<_> = c.dna.iter().map(|g| (g.id, g.value)).collect();

        list_value_mutation(&mut c);

        let dna_after: Vec<_> = c.dna.iter().map(|g| (g.id, g.value)).collect();
        assert_eq!(dna_before, dna_after);
    }

    // Test: Mutation::ListValue dispatches through MutationOperator::mutate correctly
    #[test]
    fn list_value_mutation_via_factory_returns_ok() {
        let mut c = make_chromosome_5genes();
        let result = factory(Mutation::ListValue, &mut c);
        assert!(result.is_ok(), "factory should return Ok(()) for ListValue");
    }

    // Test: factory_non_value returns Err for ListValue
    #[test]
    fn list_value_mutation_factory_non_value_returns_err() {
        let mut c = make_chromosome_5genes();
        let result = factory_non_value(Mutation::ListValue, &mut c);
        assert!(
            matches!(result, Err(GaError::MutationError(_))),
            "factory_non_value should return Err for ListValue"
        );
    }

    // Test: value consistency invariant — mutated gene's value == alleles[id]
    #[test]
    fn list_value_mutation_value_consistency_invariant() {
        for seed in 0..30u64 {
            crate::rng::set_seed(Some(seed));
            let mut c = make_chromosome_5genes();
            list_value_mutation(&mut c);

            for gene in c.dna.iter() {
                assert_eq!(
                    gene.value, gene.alleles[gene.id as usize],
                    "value must equal alleles[id] after mutation (seed {})",
                    seed
                );
            }
        }
        crate::rng::set_seed(None);
    }
}
