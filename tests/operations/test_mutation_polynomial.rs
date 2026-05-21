use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::polynomial::polynomial_mutation;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;

fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

#[test]
fn polynomial_mutation_stays_within_range() {
    let mut c = build_f64_chromosome(5);
    for _ in 0..200 {
        polynomial_mutation(&mut c, 20.0).unwrap();
        for gene in c.dna() {
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
fn polynomial_mutation_empty_dna_does_nothing() {
    let mut c = RangeChromosome::<f64>::new();
    let result = polynomial_mutation(&mut c, 20.0);
    assert!(result.is_ok());
    assert_eq!(c.dna().len(), 0);
}

#[test]
fn polynomial_mutation_can_change_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        polynomial_mutation(&mut c, 2.0).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Polynomial mutation did not change any value after 200 attempts"
    );
}

#[test]
fn polynomial_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(8);
    let before = c.dna().to_vec();
    polynomial_mutation(&mut c, 10.0).unwrap();
    let diff_count = before
        .iter()
        .zip(c.dna())
        .filter(|(b, a)| b.value != a.value)
        .count();
    assert!(
        diff_count <= 1,
        "More than one gene changed: {}",
        diff_count
    );
}

#[test]
fn polynomial_mutation_negative_eta_returns_error() {
    let mut c = build_f64_chromosome(3);
    let result = polynomial_mutation(&mut c, -1.0);
    assert!(result.is_err());
}

#[test]
fn polynomial_mutation_with_i32() {
    let mut c = RangeChromosome::<i32>::new();
    let dna = vec![
        RangeGenotype::new(0, vec![(0, 100)], 50),
        RangeGenotype::new(1, vec![(0, 100)], 50),
    ];
    c.set_dna(Cow::Owned(dna));

    for _ in 0..100 {
        polynomial_mutation(&mut c, 5.0).unwrap();
        for gene in c.dna() {
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
fn polynomial_mutation_high_eta_small_perturbation() {
    let mut c = RangeChromosome::<f64>::new();
    let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
    c.set_dna(Cow::Owned(dna));

    // With very high eta, perturbations should be small
    for _ in 0..100 {
        let before_val = c.dna()[0].value;
        polynomial_mutation(&mut c, 200.0).unwrap();
        let after_val = c.dna()[0].value;
        assert!(
            (after_val - before_val).abs() < 100.0,
            "Perturbation {} too large for eta_m=200",
            (after_val - before_val).abs()
        );
    }
}
