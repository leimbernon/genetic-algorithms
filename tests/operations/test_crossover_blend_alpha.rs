use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::blend_alpha::blend_alpha;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;

fn build_parents() -> (RangeChromosome<f64>, RangeChromosome<f64>) {
    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    let dna1 = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 30.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 70.0),
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
fn blend_alpha_produces_two_children_same_length() {
    let (p1, p2) = build_parents();
    let children = blend_alpha(&p1, &p2, 0.5).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna().len(), 2);
    assert_eq!(children[1].dna().len(), 2);
}

#[test]
fn blend_alpha_children_stay_within_range() {
    let (p1, p2) = build_parents();
    for _ in 0..200 {
        let children = blend_alpha(&p1, &p2, 0.5).unwrap();
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
fn blend_alpha_error_on_different_lengths() {
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
    let result = blend_alpha(&p1, &p2, 0.5);
    assert!(result.is_err());
}

#[test]
fn blend_alpha_zero_keeps_children_between_parents() {
    let (p1, p2) = build_parents();
    // alpha=0 means children strictly between parent values
    for _ in 0..100 {
        let children = blend_alpha(&p1, &p2, 0.0).unwrap();
        for child in &children {
            let val = child.dna()[0].value;
            assert!(
                (30.0..=60.0).contains(&val),
                "With alpha=0, value {} should be between 30 and 60",
                val
            );
        }
    }
}

#[test]
fn blend_alpha_with_i32() {
    let mut p1 = RangeChromosome::<i32>::new();
    let mut p2 = RangeChromosome::<i32>::new();
    p1.set_dna(Cow::Owned(vec![RangeGenotype::new(0, vec![(0, 100)], 30)]));
    p2.set_dna(Cow::Owned(vec![RangeGenotype::new(0, vec![(0, 100)], 70)]));
    let children = blend_alpha(&p1, &p2, 0.5).unwrap();
    assert_eq!(children.len(), 2);
    for child in &children {
        let (lo, hi) = child.dna()[0].ranges[0];
        assert!(child.dna()[0].value >= lo && child.dna()[0].value <= hi);
    }
}
