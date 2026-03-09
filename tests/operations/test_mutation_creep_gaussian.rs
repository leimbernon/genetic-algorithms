use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;

fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

fn build_i32_chromosome(n: usize) -> RangeChromosome<i32> {
    let mut c = RangeChromosome::<i32>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0, 100)], 50))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

// --- Creep mutation tests ---

#[test]
fn creep_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(10.0), None).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Creep mutation via factory did not change any value"
    );
}

#[test]
fn creep_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..100 {
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Creep: value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn creep_mutation_i32_via_factory() {
    let mut c = build_i32_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Creep mutation i32 via factory did not change any value"
    );
}

// --- Gaussian mutation tests ---

#[test]
fn gaussian_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(10.0)).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Gaussian mutation via factory did not change any value"
    );
}

#[test]
fn gaussian_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(20.0)).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Gaussian: value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn gaussian_mutation_i32_via_factory() {
    let mut c = build_i32_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(5.0)).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Gaussian mutation i32 via factory did not change any value"
    );
}

// ==================== Phase 5 edge-case tests ====================

// --- Creep mutation: step == 0 ---

#[test]
fn creep_mutation_step_zero_no_change() {
    let mut c = build_f64_chromosome(5);
    let before = c.dna().to_vec();
    // step = 0 means perturbation range [current, current], so value should not change
    mutation::factory_with_params(Mutation::Creep, &mut c, Some(0.0), None).unwrap();
    for (b, a) in before.iter().zip(c.dna()) {
        assert_eq!(b.value, a.value, "Step 0 should not change values");
    }
}

// --- Creep mutation: very large step ---

#[test]
fn creep_mutation_large_step_stays_in_range() {
    let mut c = build_f64_chromosome(3);
    for _ in 0..100 {
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(1e10), None).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

// --- Creep mutation: single gene ---

#[test]
fn creep_mutation_single_gene() {
    let mut c = RangeChromosome::<f64>::new();
    let dna = vec![RangeGenotype::new(0, vec![(0.0, 100.0)], 50.0)];
    c.set_dna(Cow::Owned(dna));
    mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
    assert_eq!(c.dna().len(), 1);
    let (lo, hi) = c.dna()[0].ranges[0];
    assert!(c.dna()[0].value >= lo && c.dna()[0].value <= hi);
}

// --- Gaussian mutation: sigma == 0 ---

#[test]
fn gaussian_mutation_sigma_zero_no_change() {
    let mut c = build_f64_chromosome(5);
    let before = c.dna().to_vec();
    // sigma = 0 means noise = 0, so value should not change
    mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(0.0)).unwrap();
    for (b, a) in before.iter().zip(c.dna()) {
        assert_eq!(b.value, a.value, "Sigma 0 should not change values");
    }
}

// --- Gaussian mutation: very large sigma stays in range ---

#[test]
fn gaussian_mutation_large_sigma_stays_in_range() {
    let mut c = build_f64_chromosome(3);
    for _ in 0..100 {
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(1e10)).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

// --- Gaussian mutation: single gene ---

#[test]
fn gaussian_mutation_single_gene() {
    let mut c = RangeChromosome::<f64>::new();
    let dna = vec![RangeGenotype::new(0, vec![(0.0, 100.0)], 50.0)];
    c.set_dna(Cow::Owned(dna));
    mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(5.0)).unwrap();
    assert_eq!(c.dna().len(), 1);
    let (lo, hi) = c.dna()[0].ranges[0];
    assert!(c.dna()[0].value >= lo && c.dna()[0].value <= hi);
}

// --- Empty chromosome through factory ---

#[test]
fn creep_mutation_empty_via_factory() {
    let mut c = RangeChromosome::<f64>::new();
    mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
    assert_eq!(c.dna().len(), 0);
}

#[test]
fn gaussian_mutation_empty_via_factory() {
    let mut c = RangeChromosome::<f64>::new();
    mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(5.0)).unwrap();
    assert_eq!(c.dna().len(), 0);
}

// --- Default step/sigma (None) ---

#[test]
fn creep_mutation_default_step() {
    let mut c = build_f64_chromosome(3);
    // step=None should default to 1.0
    mutation::factory_with_params(Mutation::Creep, &mut c, None, None).unwrap();
    for gene in c.dna() {
        let (lo, hi) = gene.ranges[0];
        assert!(gene.value >= lo && gene.value <= hi);
    }
}

#[test]
fn gaussian_mutation_default_sigma() {
    let mut c = build_f64_chromosome(3);
    // sigma=None should default to 1.0
    mutation::factory_with_params(Mutation::Gaussian, &mut c, None, None).unwrap();
    for gene in c.dna() {
        let (lo, hi) = gene.ranges[0];
        assert!(gene.value >= lo && gene.value <= hi);
    }
}
