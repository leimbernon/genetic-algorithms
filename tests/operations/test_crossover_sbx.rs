use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::sbx::sbx;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;

fn build_parents() -> (RangeChromosome<f64>, RangeChromosome<f64>) {
    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    let dna1 = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 20.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0),
        RangeGenotype::new(2, vec![(0.0, 100.0)], 50.0),
    ];
    let dna2 = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 30.0),
        RangeGenotype::new(2, vec![(0.0, 100.0)], 50.0),
    ];
    p1.set_dna(Cow::Owned(dna1));
    p2.set_dna(Cow::Owned(dna2));
    (p1, p2)
}

#[test]
fn sbx_produces_two_children_same_length() {
    let (p1, p2) = build_parents();
    let children = sbx(&p1, &p2, 2.0).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna().len(), 3);
    assert_eq!(children[1].dna().len(), 3);
}

#[test]
fn sbx_children_stay_within_range() {
    let (p1, p2) = build_parents();
    for _ in 0..100 {
        let children = sbx(&p1, &p2, 2.0).unwrap();
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
fn sbx_error_on_different_lengths() {
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
    let result = sbx(&p1, &p2, 2.0);
    assert!(result.is_err());
}

#[test]
fn sbx_identical_parents_produce_same_children() {
    let mut p1 = RangeChromosome::<f64>::new();
    let dna = vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 50.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 50.0),
    ];
    p1.set_dna(Cow::Owned(dna.clone()));
    let p2 = p1.clone();
    let children = sbx(&p1, &p2, 10.0).unwrap();
    for child in &children {
        for (i, gene) in child.dna().iter().enumerate() {
            assert!(
                (gene.value - 50.0).abs() < 1e-10,
                "Gene {} should be 50.0, got {}",
                i,
                gene.value
            );
        }
    }
}

#[test]
fn sbx_with_i32() {
    let mut p1 = RangeChromosome::<i32>::new();
    let mut p2 = RangeChromosome::<i32>::new();
    p1.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0, 100)], 20),
        RangeGenotype::new(1, vec![(0, 100)], 80),
    ]));
    p2.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0, 100)], 60),
        RangeGenotype::new(1, vec![(0, 100)], 30),
    ]));
    let children = sbx(&p1, &p2, 2.0).unwrap();
    assert_eq!(children.len(), 2);
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

#[test]
fn sbx_high_eta_produces_children_close_to_parents() {
    let (p1, p2) = build_parents();
    // With very high eta, children should be very close to parents
    let mut close_count = 0;
    for _ in 0..100 {
        let children = sbx(&p1, &p2, 100.0).unwrap();
        let c1_val = children[0].dna()[0].value;
        let p1_val = p1.dna()[0].value;
        let p2_val = p2.dna()[0].value;
        let midpoint = (p1_val + p2_val) / 2.0;
        let range = (p1_val - p2_val).abs();
        // Children should be within parent range
        if (c1_val - midpoint).abs() <= range {
            close_count += 1;
        }
    }
    assert!(
        close_count > 90,
        "High eta should keep children close to parents"
    );
}
