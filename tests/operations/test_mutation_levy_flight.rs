use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::levy_flight::levy_flight_mutation;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;

#[test]
fn levy_flight_mutation_produces_finite_value() {
    // Covers the old mantegna_sigma_u_finite_positive_at_default_alpha invariant:
    // if sigma_u or gamma_approx were broken the step would be NaN/Inf.
    for _ in 0..50 {
        let gene = RangeGenotype::new(0, vec![(-10.0_f64, 10.0_f64)], 0.0);
        let mut chromosome = RangeChromosome::<f64>::new();
        chromosome.set_dna(Cow::Owned(vec![gene]));

        levy_flight_mutation(&mut chromosome, 1.5);

        let val = chromosome.dna()[0].value;
        assert!(
            val.is_finite(),
            "levy flight must produce finite value, got {}",
            val
        );
    }
}

#[test]
fn levy_flight_mutation_stays_in_bounds() {
    // Covers the clamp guarantee and that gamma_approx numerics do not blow
    // the value out of range.
    let lo = -10.0_f64;
    let hi = 10.0_f64;
    for _ in 0..50 {
        let gene = RangeGenotype::new(0, vec![(lo, hi)], 0.0);
        let mut chromosome = RangeChromosome::<f64>::new();
        chromosome.set_dna(Cow::Owned(vec![gene]));

        levy_flight_mutation(&mut chromosome, 1.5);

        let val = chromosome.dna()[0].value;
        assert!(
            val >= lo && val <= hi,
            "levy flight value {} out of bounds [{}, {}]",
            val,
            lo,
            hi
        );
    }
}

#[test]
fn levy_flight_mutation_empty_dna_is_noop() {
    // Covers the len == 0 early-return guard.
    let mut chromosome = RangeChromosome::<f64>::new();
    chromosome.set_dna(Cow::Owned(vec![]));

    levy_flight_mutation(&mut chromosome, 1.5);

    assert_eq!(
        chromosome.dna().len(),
        0,
        "empty DNA should remain empty after mutation"
    );
}
