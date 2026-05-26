use genetic_algorithms::chromosomes::{Binary as BinaryChromosome, Range as RangeChromosome};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::{LinearChromosome, MutationConfig};
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
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(1.0), None).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Cauchy mutation never changed a value across 200 iterations");
}

#[test]
fn cauchy_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(50.0), None).unwrap();
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
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(1.0), None).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before.iter().zip(after.iter()).filter(|(b, a)| b != a).count();
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
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(10.0), None).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn cauchy_mutation_default_scale_when_step_none() {
    let mut c = build_f64_chromosome(4);
    for _ in 0..50 {
        // step = None must default to 1.0 inside the Cauchy match arm
        mutation::factory_with_params(Mutation::Cauchy, &mut c, None, None).unwrap();
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
    let result = mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(1.0), None);
    assert!(
        result.is_err(),
        "Cauchy mutation must error on Binary chromosomes (got {:?})",
        result
    );
}

#[test]
fn cauchy_scale_builder_sets_field() {
    let ga = Ga::<RangeChromosome<f64>>::default().with_cauchy_scale(2.5);
    assert_eq!(ga.configuration().mutation().cauchy_scale, Some(2.5));
    let cfg = GaConfiguration::default().with_cauchy_scale(3.5);
    assert_eq!(cfg.mutation().cauchy_scale, Some(3.5));
}

#[test]
fn levy_alpha_builder_sets_field() {
    let ga = Ga::<RangeChromosome<f64>>::default().with_levy_alpha(1.7);
    assert_eq!(ga.configuration().mutation().levy_alpha, Some(1.7));
    let cfg = GaConfiguration::default().with_levy_alpha(1.2);
    assert_eq!(cfg.mutation().levy_alpha, Some(1.2));
}

// ---- LevyFlight active tests ----

#[test]
fn levy_flight_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        // alpha = 1.5 routed via the `sigma` slot (engine routing convention)
        mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, Some(1.5)).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(changed, "LevyFlight mutation never changed a value across 200 iterations");
}

#[test]
fn levy_flight_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, Some(1.5)).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "LevyFlight: value {} out of range [{}, {}]",
                gene.value, lo, hi
            );
        }
    }
}

#[test]
fn levy_flight_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, Some(1.5)).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before.iter().zip(after.iter()).filter(|(b, a)| b != a).count();
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
        mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, Some(1.5)).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn levy_flight_mutation_errors_on_binary_chromosome() {
    let mut c = BinaryChromosome::new();
    let result = mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, Some(1.5));
    assert!(
        result.is_err(),
        "LevyFlight mutation must error on Binary chromosomes (got {:?})",
        result
    );
}

#[test]
fn levy_flight_default_alpha_when_sigma_none() {
    let mut c = build_f64_chromosome(4);
    for _ in 0..50 {
        // sigma = None must default to alpha = 1.5 inside the LevyFlight match arm
        mutation::factory_with_params(Mutation::LevyFlight, &mut c, None, None).unwrap();
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
        // Uniform takes no parameter — pass None for both step and sigma.
        mutation::factory_with_params(Mutation::Uniform, &mut c, None, None).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        if before.iter().zip(after.iter()).any(|(b, a)| b != a) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Uniform mutation never changed a value across 200 iterations");
}

#[test]
fn uniform_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::Uniform, &mut c, None, None).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Uniform: value {} out of range [{}, {}]",
                gene.value, lo, hi
            );
        }
    }
}

#[test]
fn uniform_mutation_changes_at_most_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory_with_params(Mutation::Uniform, &mut c, None, None).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        let changed_count = before.iter().zip(after.iter()).filter(|(b, a)| b != a).count();
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
        mutation::factory_with_params(Mutation::Uniform, &mut c, None, None).unwrap();
        for gene in c.dna().iter() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn uniform_mutation_errors_on_binary_chromosome() {
    let mut c = BinaryChromosome::new();
    let result = mutation::factory_with_params(Mutation::Uniform, &mut c, None, None);
    assert!(
        result.is_err(),
        "Uniform mutation must error on Binary chromosomes (got {:?})",
        result
    );
}
