use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::ChromosomeT;

/// Helper to build a Range chromosome with N genes in [0, n-1], initial value = 0.
fn build_range_chromosome(n: i32) -> RangeChromosome<i32> {
    let mut c = RangeChromosome::<i32>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i, vec![(0, n - 1)], 0))
        .collect();
    use std::borrow::Cow;
    c.set_dna(Cow::Borrowed(&dna));
    c
}

#[test]
fn value_mutation_keeps_value_within_range_and_can_change() {
    let n = 8;
    let mut c = build_range_chromosome(n);

    // Try multiple times to increase the chance of value change due to randomness
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory(Mutation::Value, &mut c).unwrap();
        let after = c.dna();

        // Check all genes stay within declared ranges
        for (gene_idx, gene) in after.iter().enumerate() {
            let (lo, hi) = after[gene_idx].ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Gene {} value out of range: {}",
                gene_idx,
                gene.value
            );
        }

        // Detect if any gene changed its value
        if before
            .iter()
            .zip(after.iter())
            .any(|(b, a)| b.value != a.value)
        {
            changed = true;
            break;
        }
    }

    assert!(
        changed,
        "Value mutation did not change any gene value across attempts; randomness too unlucky?"
    );
}

#[test]
fn value_mutation_changes_at_most_one_gene() {
    let n = 8;
    let mut c = build_range_chromosome(n);

    let before = c.dna().to_vec();
    mutation::factory(Mutation::Value, &mut c).unwrap();
    let after = c.dna();

    // Count genes that differ by value
    let diff_count = before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| b.value != a.value)
        .count();
    assert!(
        diff_count <= 1,
        "More than one gene changed value: {}",
        diff_count
    );
}
