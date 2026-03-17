use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::arithmetic::arithmetic;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;

fn build_parents() -> (RangeChromosome<f64>, RangeChromosome<f64>) {
    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    let dna1 = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 20.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0),
    ];
    let dna2 = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 40.0),
    ];
    p1.set_dna(Cow::Owned(dna1));
    p2.set_dna(Cow::Owned(dna2));
    (p1, p2)
}

#[test]
fn arithmetic_produces_two_children_same_length() {
    let (p1, p2) = build_parents();
    let children = arithmetic(&p1, &p2, 0.5).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna().len(), 2);
    assert_eq!(children[1].dna().len(), 2);
}

#[test]
fn arithmetic_children_stay_within_range() {
    let (p1, p2) = build_parents();
    // Try several alpha values including edge cases
    for &alpha in &[0.0, 0.25, 0.5, 0.75, 1.0] {
        let children = arithmetic(&p1, &p2, alpha).unwrap();
        for child in &children {
            for gene in child.dna() {
                let (lo, hi) = gene.ranges[0];
                assert!(
                    gene.value >= lo && gene.value <= hi,
                    "Gene value {} out of range [{}, {}] with alpha={}",
                    gene.value,
                    lo,
                    hi,
                    alpha,
                );
            }
        }
    }
}

#[test]
fn arithmetic_error_on_different_lengths() {
    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    p1.set_dna(Cow::Owned(vec![RangeGenotype::new(
        0,
        vec![(0.0, 10.0)],
        5.0,
    )]));
    p2.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 10.0)], 5.0),
        RangeGenotype::new(1, vec![(0.0, 10.0)], 5.0),
    ]));
    let result = arithmetic(&p1, &p2, 0.5);
    assert!(result.is_err());
}

#[test]
fn arithmetic_half_alpha_produces_midpoint() {
    let (p1, p2) = build_parents();
    let children = arithmetic(&p1, &p2, 0.5).unwrap();

    // With alpha=0.5, both children should be the midpoint of each gene pair.
    // Gene 0: midpoint of 20 and 60 = 40
    // Gene 1: midpoint of 80 and 40 = 60
    let c1 = &children[0];
    let c2 = &children[1];

    assert!(
        (c1.dna()[0].value - 40.0).abs() < 1e-10,
        "Expected 40.0, got {}",
        c1.dna()[0].value
    );
    assert!(
        (c1.dna()[1].value - 60.0).abs() < 1e-10,
        "Expected 60.0, got {}",
        c1.dna()[1].value
    );
    // Child 2 should be identical to child 1 when alpha = 0.5
    assert!(
        (c2.dna()[0].value - 40.0).abs() < 1e-10,
        "Expected 40.0, got {}",
        c2.dna()[0].value
    );
    assert!(
        (c2.dna()[1].value - 60.0).abs() < 1e-10,
        "Expected 60.0, got {}",
        c2.dna()[1].value
    );
}
