use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
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

// ==================== Extracted from src/operations/mutation/creep.rs ====================

#[test]
fn creep_mutation_stays_within_range() {
    use genetic_algorithms::operations::mutation::creep::creep_mutation;
    let mut c = build_f64_chromosome(5);
    for _ in 0..100 {
        creep_mutation(&mut c, 5.0);
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
fn creep_mutation_can_change_value() {
    use genetic_algorithms::operations::mutation::creep::creep_mutation;
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        creep_mutation(&mut c, 10.0);
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Creep mutation did not change any value after 200 attempts"
    );
}

#[test]
fn creep_mutation_changes_at_most_one_gene() {
    use genetic_algorithms::operations::mutation::creep::creep_mutation;
    let mut c = build_f64_chromosome(8);
    let before = c.dna().to_vec();
    creep_mutation(&mut c, 5.0);
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
fn creep_mutation_respects_step_size() {
    use genetic_algorithms::operations::mutation::creep::creep_mutation;
    let mut c = RangeChromosome::<f64>::new();
    let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
    c.set_dna(Cow::Owned(dna));

    for _ in 0..100 {
        let before_val = c.dna()[0].value;
        creep_mutation(&mut c, 1.0);
        let after_val = c.dna()[0].value;
        assert!(
            (after_val - before_val).abs() <= 1.0 + f64::EPSILON,
            "Perturbation {} exceeded step 1.0",
            (after_val - before_val).abs()
        );
    }
}

#[test]
fn creep_mutation_empty_dna_does_nothing() {
    use genetic_algorithms::operations::mutation::creep::creep_mutation;
    let mut c = RangeChromosome::<f64>::new();
    creep_mutation(&mut c, 5.0);
    assert_eq!(c.dna().len(), 0);
}

// ==================== Extracted from src/operations/mutation/gaussian.rs ====================

#[test]
fn gaussian_mutation_stays_within_range() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = build_f64_chromosome(5);
    for _ in 0..200 {
        gaussian_mutation(&mut c, 10.0);
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
fn gaussian_mutation_can_change_value() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        gaussian_mutation(&mut c, 10.0);
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Gaussian mutation did not change any value after 200 attempts"
    );
}

#[test]
fn gaussian_mutation_changes_at_most_one_gene() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = build_f64_chromosome(8);
    let before = c.dna().to_vec();
    gaussian_mutation(&mut c, 5.0);
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
fn gaussian_mutation_empty_dna_does_nothing() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = RangeChromosome::<f64>::new();
    gaussian_mutation(&mut c, 5.0);
    assert_eq!(c.dna().len(), 0);
}

#[test]
fn gaussian_mutation_with_i32() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = RangeChromosome::<i32>::new();
    let dna = vec![
        RangeGenotype::new(0, vec![(0, 100)], 50),
        RangeGenotype::new(1, vec![(0, 100)], 50),
    ];
    c.set_dna(Cow::Owned(dna));

    for _ in 0..100 {
        gaussian_mutation(&mut c, 5.0);
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
fn gaussian_mutation_small_sigma_small_perturbation() {
    use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
    let mut c = RangeChromosome::<f64>::new();
    let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
    c.set_dna(Cow::Owned(dna));

    // With sigma=0.001, perturbations should be very small
    for _ in 0..100 {
        let before_val = c.dna()[0].value;
        gaussian_mutation(&mut c, 0.001);
        let after_val = c.dna()[0].value;
        // 6-sigma bound: very unlikely to exceed 0.006
        assert!(
            (after_val - before_val).abs() < 1.0,
            "Perturbation {} too large for sigma=0.001",
            (after_val - before_val).abs()
        );
    }
}
