use genetic_algorithms::chromosomes::ListChromosome;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::List;
use genetic_algorithms::operations::mutation::factory;
use genetic_algorithms::operations::mutation::factory_non_value;
use genetic_algorithms::operations::mutation::list_value::list_value_mutation;
use genetic_algorithms::operations::Mutation;

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
        genetic_algorithms::rng::set_seed(Some(seed));
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
    genetic_algorithms::rng::set_seed(None);
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
        genetic_algorithms::rng::set_seed(Some(seed));
        let mut c2 = c.clone();
        list_value_mutation(&mut c2);
        // single-allele gene must remain unchanged
        assert_eq!(c2.dna[0].id, 0);
        assert_eq!(c2.dna[0].value, 'x');
    }
    genetic_algorithms::rng::set_seed(None);
}

// Test: empty chromosome is a no-op
#[test]
fn list_value_mutation_empty_chromosome_is_noop() {
    let mut c = ListChromosome::<char>::new();
    use genetic_algorithms::traits::ChromosomeT;
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
        genetic_algorithms::rng::set_seed(Some(seed));
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
    genetic_algorithms::rng::set_seed(None);
}
