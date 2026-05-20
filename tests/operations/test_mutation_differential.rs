use genetic_algorithms::chromosomes::{Binary as BinaryChromosome, Range as RangeChromosome};
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::differential::differential_mutation;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;

/// Build a population of `size` RangeChromosome<f64> with distinct initial values
/// so that donor vectors x_r2 and x_r3 produce non-zero differences.
fn make_f64_population(size: usize) -> Vec<RangeChromosome<f64>> {
    (0..size)
        .map(|i| {
            let mut c = RangeChromosome::<f64>::new();
            let dna: Vec<_> = (0_i32..3)
                .map(|j| RangeGenotype::new(j, vec![(0.0, 100.0)], (i as f64 * 10.0).min(90.0)))
                .collect();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

/// Build a population of `size` RangeChromosome<i32> with distinct initial values.
fn make_i32_population(size: usize) -> Vec<RangeChromosome<i32>> {
    (0..size)
        .map(|i| {
            let mut c = RangeChromosome::<i32>::new();
            let dna: Vec<_> = (0_i32..3)
                .map(|j| RangeGenotype::new(j, vec![(0, 100)], ((i as i32) * 10).min(90)))
                .collect();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

/// Build a population of `size` BinaryChromosome individuals.
fn make_binary_population(size: usize) -> Vec<BinaryChromosome> {
    (0..size)
        .map(|_| {
            let mut c = BinaryChromosome::new();
            let dna: Vec<_> = (0_i32..3)
                .map(|j| BinaryGenotype { id: j, value: true })
                .collect();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

#[test]
fn differential_mutation_stays_within_range() {
    let mut pop = make_f64_population(10);
    for _ in 0..200 {
        let mut target = pop[0].clone();
        differential_mutation(&mut target, &pop, 0, 0.5).unwrap();
        for gene in target.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Gene value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
        pop[0] = target;
    }
}

#[test]
fn differential_mutation_can_change_value() {
    let pop = make_f64_population(10);
    let original = pop[0].dna().to_vec();
    let mut changed = false;
    for _ in 0..200 {
        let mut target = pop[0].clone();
        differential_mutation(&mut target, &pop, 0, 0.5).unwrap();
        if target.dna().iter().zip(&original).any(|(a, b)| a.value != b.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Differential mutation did not change any value after 200 attempts");
}

#[test]
fn differential_error_small_population() {
    let mut pop = make_f64_population(3); // Only 3 — too few (need >= 4)
    let mut target = pop[0].clone();
    let result = differential_mutation(&mut target, &pop, 0, 0.5);
    assert!(
        matches!(result, Err(GaError::MutationError(_))),
        "Expected MutationError for population of 3, got {:?}",
        result
    );
    // Also verify pop size of exactly 4 works
    pop.push(make_f64_population(1).remove(0));
    let mut target2 = pop[0].clone();
    assert!(differential_mutation(&mut target2, &pop, 0, 0.5).is_ok());
}

#[test]
fn differential_error_non_range() {
    let pop = make_binary_population(4);
    let mut target = pop[0].clone();
    let result = differential_mutation(&mut target, &pop, 0, 0.5);
    assert!(
        matches!(result, Err(GaError::MutationError(_))),
        "Expected MutationError for non-Range chromosome, got {:?}",
        result
    );
}

#[test]
fn differential_f_parameter() {
    let pop = make_f64_population(10);

    // F=0.0 should produce no change (mutant = x_r1 + 0 * (x_r2 - x_r3) = x_r1)
    // but we just verify it runs without error and values stay within range
    let mut target_zero = pop[0].clone();
    let result_zero = differential_mutation(&mut target_zero, &pop, 0, 0.0);
    assert!(result_zero.is_ok(), "F=0.0 should not error");
    for gene in target_zero.dna() {
        let (lo, hi) = gene.ranges[0];
        assert!(gene.value >= lo && gene.value <= hi, "F=0.0 out of range: {}", gene.value);
    }

    // F=2.0 can produce values outside range, but clamping must keep them in bounds
    let mut target_large = pop[0].clone();
    let result_large = differential_mutation(&mut target_large, &pop, 0, 2.0);
    assert!(result_large.is_ok(), "F=2.0 should not error");
    for gene in target_large.dna() {
        let (lo, hi) = gene.ranges[0];
        assert!(
            gene.value >= lo && gene.value <= hi,
            "F=2.0 out of range after clamping: {}",
            gene.value
        );
    }
}

#[test]
fn differential_mutation_with_i32() {
    let pop = make_i32_population(10);
    for _ in 0..50 {
        let mut target = pop[0].clone();
        differential_mutation(&mut target, &pop, 0, 0.5).unwrap();
        for gene in target.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "i32 gene value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}
