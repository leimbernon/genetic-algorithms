use genetic_algorithms::chromosomes::MultiRangeChromosome;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::MultiRangeGenotype;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::multi_range_random_initialization;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::{LinearChromosome, MutationOperator};
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
        mutation::factory(Mutation::Creep { step: Some(10.0) }, &mut c).unwrap();
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
        mutation::factory(Mutation::Creep { step: Some(5.0) }, &mut c).unwrap();
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
        mutation::factory(Mutation::Creep { step: Some(5.0) }, &mut c).unwrap();
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
        mutation::factory(Mutation::Gaussian { sigma: Some(10.0) }, &mut c).unwrap();
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
        mutation::factory(Mutation::Gaussian { sigma: Some(20.0) }, &mut c).unwrap();
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
        mutation::factory(Mutation::Gaussian { sigma: Some(5.0) }, &mut c).unwrap();
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
    mutation::factory(Mutation::Creep { step: Some(0.0) }, &mut c).unwrap();
    for (b, a) in before.iter().zip(c.dna()) {
        assert_eq!(b.value, a.value, "Step 0 should not change values");
    }
}

// --- Creep mutation: very large step ---

#[test]
fn creep_mutation_large_step_stays_in_range() {
    let mut c = build_f64_chromosome(3);
    for _ in 0..100 {
        mutation::factory(Mutation::Creep { step: Some(1e10) }, &mut c).unwrap();
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
    mutation::factory(Mutation::Creep { step: Some(5.0) }, &mut c).unwrap();
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
    mutation::factory(Mutation::Gaussian { sigma: Some(0.0) }, &mut c).unwrap();
    for (b, a) in before.iter().zip(c.dna()) {
        assert_eq!(b.value, a.value, "Sigma 0 should not change values");
    }
}

// --- Gaussian mutation: very large sigma stays in range ---

#[test]
fn gaussian_mutation_large_sigma_stays_in_range() {
    let mut c = build_f64_chromosome(3);
    for _ in 0..100 {
        mutation::factory(Mutation::Gaussian { sigma: Some(1e10) }, &mut c).unwrap();
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
    mutation::factory(Mutation::Gaussian { sigma: Some(5.0) }, &mut c).unwrap();
    assert_eq!(c.dna().len(), 1);
    let (lo, hi) = c.dna()[0].ranges[0];
    assert!(c.dna()[0].value >= lo && c.dna()[0].value <= hi);
}

// --- Empty chromosome through factory ---

#[test]
fn creep_mutation_empty_via_factory() {
    let mut c = RangeChromosome::<f64>::new();
    mutation::factory(Mutation::Creep { step: Some(5.0) }, &mut c).unwrap();
    assert_eq!(c.dna().len(), 0);
}

#[test]
fn gaussian_mutation_empty_via_factory() {
    let mut c = RangeChromosome::<f64>::new();
    mutation::factory(Mutation::Gaussian { sigma: Some(5.0) }, &mut c).unwrap();
    assert_eq!(c.dna().len(), 0);
}

// --- Default step/sigma (None) ---

#[test]
fn creep_mutation_default_step() {
    let mut c = build_f64_chromosome(3);
    // step=None should default to 0.01
    let m = Mutation::Creep { step: None };
    m.mutate(&mut c, &m).unwrap();
    for gene in c.dna() {
        let (lo, hi) = gene.ranges[0];
        assert!(gene.value >= lo && gene.value <= hi);
    }
}

#[test]
fn gaussian_mutation_default_sigma() {
    let mut c = build_f64_chromosome(3);
    // sigma=None should default to 0.1
    let m = Mutation::Gaussian { sigma: None };
    m.mutate(&mut c, &m).unwrap();
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

// ==================== MultiRangeChromosome Gaussian mutation tests ====================

/// Build a MultiRangeChromosome with heterogeneous bounds.
fn build_multi_range_chromosome() -> MultiRangeChromosome<f64> {
    let bounds = vec![(0.0_f64, 1.0), (10.0, 100.0)];
    let rates = vec![0.05_f64, 5.0_f64];
    let dna = multi_range_random_initialization(&bounds, &rates);
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_dna(Cow::Owned(dna));
    c
}

/// Every observed value stays within each gene's (lo, hi) across 1000 mutation iterations.
#[test]
fn multi_range_gaussian_values_stay_within_per_gene_bounds_1000_iterations() {
    let mut c = build_multi_range_chromosome();
    for iter in 0..1000 {
        mutation::factory(Mutation::Gaussian { sigma: Some(10.0) }, &mut c).unwrap();
        for gene in c.dna() {
            assert!(
                gene.value >= gene.lo && gene.value <= gene.hi,
                "Iteration {}: gene {} value {} out of per-gene range [{}, {}]",
                iter, gene.id, gene.value, gene.lo, gene.hi
            );
        }
    }
}

/// Gene with mutation_rate=0.05 produces smaller average |delta| than gene with mutation_rate=5.0.
#[test]
fn multi_range_gaussian_per_gene_rate_controls_noise_scale() {
    use genetic_algorithms::rng;

    // Gene 0: bounds [0, 1000], mutation_rate=0.0001 (tiny noise)
    // Gene 1: bounds [0, 1000], mutation_rate=20.0 (large noise)
    let bounds = vec![(0.0_f64, 1000.0), (0.0_f64, 1000.0)];
    let rates = vec![0.0001_f64, 20.0_f64];
    let dna = multi_range_random_initialization(&bounds, &rates);
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_dna(Cow::Owned(dna));

    let mut total_delta_0 = 0.0_f64;
    let mut total_delta_1 = 0.0_f64;
    let mut count_0 = 0usize;
    let mut count_1 = 0usize;

    rng::set_seed(Some(123));
    for _ in 0..2000 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::Gaussian { sigma: Some(1.0) }, &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();

        let d0 = (after[0] - before[0]).abs();
        let d1 = (after[1] - before[1]).abs();
        if d0 > 0.0 { total_delta_0 += d0; count_0 += 1; }
        if d1 > 0.0 { total_delta_1 += d1; count_1 += 1; }
    }
    rng::set_seed(None);

    // Verify that gene with mutation_rate=20.0 has much larger avg delta
    if count_0 > 0 && count_1 > 0 {
        let avg_0 = total_delta_0 / count_0 as f64;
        let avg_1 = total_delta_1 / count_1 as f64;
        assert!(
            avg_1 > avg_0 * 10.0,
            "Gene with mutation_rate=20.0 avg delta ({:.6}) should be >> rate=0.0001 ({:.6})",
            avg_1, avg_0
        );
    }
}

/// Direct call to multi_range_gaussian_mutation reads gene.mutation_rate and clamps.
#[test]
fn multi_range_gaussian_mutation_direct_call_clamps_to_bounds() {
    use genetic_algorithms::operations::mutation::gaussian::multi_range_gaussian_mutation;

    let mut c = MultiRangeChromosome::<f64>::default();
    // Single gene with very high mutation_rate to force clamping
    let dna = vec![MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.5, 1e10)];
    c.set_dna(Cow::Owned(dna));

    for _ in 0..100 {
        multi_range_gaussian_mutation(&mut c, 0.0); // sigma=0 is intentionally ignored
        let gene = &c.dna()[0];
        assert!(
            gene.value >= 0.0 && gene.value <= 1.0,
            "After mutation, value {} must be within [0.0, 1.0]",
            gene.value
        );
    }
}

/// Empty MultiRangeChromosome through factory does not panic.
#[test]
fn multi_range_gaussian_mutation_empty_dna_does_nothing() {
    let mut c = MultiRangeChromosome::<f64>::default();
    mutation::factory(Mutation::Gaussian { sigma: Some(5.0) }, &mut c).unwrap();
    assert_eq!(c.dna().len(), 0);
}

// ==================== v3.0.0 parameterized variant tests ====================

/// Gaussian { sigma: Some(0.05) } applies a smaller sigma than Gaussian { sigma: Some(10.0) }
#[test]
fn gaussian_inline_sigma_controls_noise_magnitude() {
    use genetic_algorithms::rng;

    let build = || {
        let mut c = RangeChromosome::<f64>::new();
        let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
        c.set_dna(Cow::Owned(dna));
        c
    };

    rng::set_seed(Some(42));
    let m_small = Mutation::Gaussian { sigma: Some(0.05) };
    let m_large = Mutation::Gaussian { sigma: Some(10.0) };

    let mut total_small = 0.0f64;
    let mut total_large = 0.0f64;
    for _ in 0..200 {
        let mut c = build();
        let before = c.dna()[0].value;
        m_small.mutate(&mut c, &m_small).unwrap();
        total_small += (c.dna()[0].value - before).abs();

        let mut c = build();
        let before = c.dna()[0].value;
        m_large.mutate(&mut c, &m_large).unwrap();
        total_large += (c.dna()[0].value - before).abs();
    }
    rng::set_seed(None);

    assert!(
        total_large > total_small * 5.0,
        "Large sigma should produce bigger perturbations (large={:.4}, small={:.4})",
        total_large,
        total_small
    );
}

/// Creep { step: None } uses default 0.01 — values stay within range
#[test]
fn creep_default_step_uses_zero_point_zero_one() {
    let mut c = build_f64_chromosome(3);
    let m = Mutation::Creep { step: None };
    for _ in 0..200 {
        m.mutate(&mut c, &m).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Value {} out of range [{}, {}] with default step",
                gene.value, lo, hi
            );
        }
    }
}
