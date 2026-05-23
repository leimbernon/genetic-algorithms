use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::undx::undx;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;

fn build_parents(n: usize) -> Vec<RangeChromosome<f64>> {
    (0..n)
        .map(|k| {
            let mut p = RangeChromosome::<f64>::new();
            let dna = vec![
                RangeGenotype::new(0, vec![(0.0, 100.0)], 10.0 + 20.0 * k as f64),
                RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0 - 10.0 * k as f64),
            ];
            p.set_dna(Cow::Owned(dna));
            p
        })
        .collect()
}

#[test]
fn undx_produces_one_offspring_within_bounds() {
    let parents = build_parents(3);
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();
    for _ in 0..100 {
        let result = undx(&parent_refs, 3);
        assert!(result.is_ok(), "undx returned Err: {:?}", result.err());
        let children = result.unwrap();
        assert_eq!(children.len(), 1, "undx should produce exactly 1 offspring");
        for child in &children {
            for gene in child.dna() {
                let (lo, hi) = gene.ranges[0];
                assert!(
                    gene.value >= lo && gene.value <= hi,
                    "Gene value {} out of range [{}, {}]",
                    gene.value,
                    lo,
                    hi
                );
            }
        }
    }
}

#[test]
fn undx_rejects_fewer_than_three_parents() {
    let parents = build_parents(2);
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();
    let result = undx(&parent_refs, 2);
    assert!(
        result.is_err(),
        "undx with < 3 parents should return Err, got {:?}",
        result.ok()
    );
}
