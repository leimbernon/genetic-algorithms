use genetic_algorithms::chromosomes::{Binary as BinaryChromosome, Range as RangeChromosome};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::{CauchyParams, LevyFlightParams, Mutation};
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
        .map(|i| RangeGenotype::new(i as i32, vec![(-50, 50)], 0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

// ---- Cauchy active tests ----

#[test]
fn cauchy_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::Cauchy(CauchyParams { scale: Some(1.0) }), &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Cauchy mutation never changed a value across 200 iterations"
    );
}

#[test]
fn cauchy_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory(Mutation::Cauchy(CauchyParams { scale: Some(50.0) }), &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Cauchy: value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn cauchy_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::Cauchy(CauchyParams { scale: Some(1.0) }), &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before
            .iter()
            .zip(after.iter())
            .filter(|(b, a)| b != a)
            .count();
        assert!(
            changed_count <= 1,
            "Cauchy changed {} genes in one call (expected <= 1)",
            changed_count
        );
    }
}

#[test]
fn cauchy_mutation_works_on_i32() {
    let mut c = build_i32_chromosome(6);
    for _ in 0..200 {
        mutation::factory(Mutation::Cauchy(CauchyParams { scale: Some(10.0) }), &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn cauchy_mutation_default_scale_when_none() {
    let mut c = build_f64_chromosome(4);
    // scale=None must default to 1.0 inside the Cauchy variant
    let m = Mutation::Cauchy(CauchyParams { scale: None });
    for _ in 0..50 {
        m.mutate(&mut c, &m).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn cauchy_mutation_errors_on_binary_chromosome() {
    let mut c = BinaryChromosome::new();
    // BinaryChromosome doesn't take Range genes; call factory and assert error.
    let result = mutation::factory(Mutation::Cauchy(CauchyParams { scale: Some(1.0) }), &mut c);
    assert!(
        result.is_err(),
        "Cauchy mutation must error on Binary chromosomes (got {:?})",
        result
    );
}

/// Cauchy { scale: Some(2.0) } applies scale 2.0 — variant carries its own params.
#[test]
fn cauchy_inline_scale_applies() {
    let mut c = build_f64_chromosome(4);
    let m = Mutation::Cauchy(CauchyParams { scale: Some(2.0) });
    for _ in 0..50 {
        m.mutate(&mut c, &m).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

// ---- LevyFlight active tests ----

#[test]
fn levy_flight_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }), &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "LevyFlight mutation never changed a value across 200 iterations"
    );
}

#[test]
fn levy_flight_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory(Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }), &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "LevyFlight: value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn levy_flight_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }), &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before
            .iter()
            .zip(after.iter())
            .filter(|(b, a)| b != a)
            .count();
        assert!(
            changed_count <= 1,
            "LevyFlight changed {} genes in one call (expected <= 1)",
            changed_count
        );
    }
}

#[test]
fn levy_flight_mutation_works_on_i32() {
    let mut c = build_i32_chromosome(6);
    for _ in 0..200 {
        mutation::factory(Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }), &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn levy_flight_mutation_errors_on_binary_chromosome() {
    let mut c = BinaryChromosome::new();
    let result = mutation::factory(Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }), &mut c);
    assert!(
        result.is_err(),
        "LevyFlight mutation must error on Binary chromosomes (got {:?})",
        result
    );
}

#[test]
fn levy_flight_default_alpha_when_none() {
    let mut c = build_f64_chromosome(4);
    // alpha=None must default to 1.5 inside the LevyFlight variant
    let m = Mutation::LevyFlight(LevyFlightParams { alpha: None });
    for _ in 0..50 {
        m.mutate(&mut c, &m).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

// ---- Uniform active tests ----

#[test]
fn uniform_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        // Uniform takes no parameter.
        mutation::factory(Mutation::Uniform, &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "Uniform mutation never changed a value across 200 iterations"
    );
}

#[test]
fn uniform_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory(Mutation::Uniform, &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Uniform: value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn uniform_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::Uniform, &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before
            .iter()
            .zip(after.iter())
            .filter(|(b, a)| b != a)
            .count();
        assert!(
            changed_count <= 1,
            "Uniform changed {} genes in one call (expected <= 1)",
            changed_count
        );
    }
}

#[test]
fn uniform_mutation_works_on_i32() {
    let mut c = build_i32_chromosome(6);
    for _ in 0..200 {
        mutation::factory(Mutation::Uniform, &mut c).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn uniform_mutation_errors_on_binary_chromosome() {
    let mut c = BinaryChromosome::new();
    let result = mutation::factory(Mutation::Uniform, &mut c);
    assert!(
        result.is_err(),
        "Uniform mutation must error on Binary chromosomes (got {:?})",
        result
    );
}
