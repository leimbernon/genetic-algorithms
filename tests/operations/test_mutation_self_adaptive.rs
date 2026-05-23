use genetic_algorithms::chromosomes::{Binary as BinaryChromosome, Range as RangeChromosome};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::{LinearChromosome, SelfAdaptive};
use std::borrow::Cow;

fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(-10.0, 10.0)], 0.0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

#[test]
fn self_adaptive_sigma_min_enforced() {
    let mut c = build_f64_chromosome(4);
    // Set strategy_params to nearly-zero values (below sigma_min)
    c.set_strategy_params(vec![1e-8; 4]);
    let sigma_min = 1e-5;
    for _ in 0..100 {
        self_adaptive_gaussian_mutation(&mut c, 0.0, 0.0, sigma_min);
        for &sigma in c.strategy_params() {
            assert!(
                sigma >= sigma_min,
                "Sigma {} dropped below sigma_min {}",
                sigma,
                sigma_min
            );
        }
    }
}

#[test]
fn self_adaptive_sigma_spread_evolves() {
    // pop_a: start with low sigmas, pop_b: start with high sigmas
    // After 200 mutations each, the union of sigmas should cover intermediate range (0.2, 0.8)
    let mut pop_a = build_f64_chromosome(4);
    pop_a.set_strategy_params(vec![0.1; 4]);

    let mut pop_b = build_f64_chromosome(4);
    pop_b.set_strategy_params(vec![0.9; 4]);

    let tau = 0.5;
    let tau_prime = 0.5;
    let sigma_min = 1e-5;

    for _ in 0..200 {
        self_adaptive_gaussian_mutation(&mut pop_a, tau, tau_prime, sigma_min);
        self_adaptive_gaussian_mutation(&mut pop_b, tau, tau_prime, sigma_min);
    }

    // Collect all sigmas from both populations
    let mut all_sigmas: Vec<f64> = Vec::new();
    all_sigmas.extend_from_slice(pop_a.strategy_params());
    all_sigmas.extend_from_slice(pop_b.strategy_params());

    // The union should cover values in (0.2, 0.8) — i.e., there exists at least
    // one sigma in each population that covers the intermediate range
    let has_intermediate = all_sigmas.iter().any(|&s| s > 0.2 && s < 0.8);
    assert!(
        has_intermediate,
        "Expected sigma spread to cover (0.2, 0.8) after 200 iterations. Sigmas: {:?}",
        all_sigmas
    );
}

#[test]
fn self_adaptive_gaussian_returns_error_for_non_self_adaptive() {
    let mut binary_chrom = BinaryChromosome::new();
    // SelfAdaptiveGaussian must return Err for chromosomes not implementing SelfAdaptive
    let result = mutation::factory_with_params(Mutation::SelfAdaptiveGaussian, &mut binary_chrom, None, None);
    assert!(
        result.is_err(),
        "SelfAdaptiveGaussian must return Err for BinaryChromosome (not a SelfAdaptive chromosome), got Ok"
    );
}
