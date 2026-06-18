use genetic_algorithms::chromosomes::{Binary as BinaryChromosome, Range as RangeChromosome};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::{LinearChromosome, MutationOperator, SelfAdaptive};
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
        let _ = self_adaptive_gaussian_mutation(&mut c, 0.0, 0.0, sigma_min, None);
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

// Stochastic acceptance probe — the assertion is probabilistic, not deterministic.
// `crate::rng::make_rng()` reads a process-global seed, so setting `rng::set_seed`
// inside the test would leak into other tests running concurrently. Run manually
// with `cargo test -- --ignored self_adaptive_sigma_spread_evolves` when validating
// the self-adaptive Gaussian sigma random walk.
#[test]
#[ignore]
fn self_adaptive_sigma_spread_evolves() {
    // All sigmas start equal. After many mutations the independent per-dimension local
    // noise (tau term) causes dimensions to diverge: some collapse toward sigma_min,
    // others grow large. The log-normal random walk has a constant median but growing
    // variance, so spread (max/min ratio >> 1) is virtually guaranteed after 200 steps
    // with tau = 0.5 (the ES default for n=8 is 1/sqrt(2n) ≈ 0.25, so 0.5 is aggressive).
    let mut c = build_f64_chromosome(8);
    c.set_strategy_params(vec![0.5; 8]);

    let tau = 0.5;
    let tau_prime = 0.5;
    let sigma_min = 1e-5;

    for _ in 0..200 {
        let _ = self_adaptive_gaussian_mutation(&mut c, tau, tau_prime, sigma_min, None);
    }

    let sigmas = c.strategy_params();
    let final_max = sigmas.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let final_min = sigmas.iter().cloned().fold(f64::INFINITY, f64::min);

    // With 8 independent dimensions and 200 steps of log-normal evolution, the
    // spread between the highest and lowest sigma is virtually certain to exceed 10x.
    assert!(
        final_max > final_min * 10.0,
        "Sigma spread should have occurred across dimensions. Max={}, Min={}, All={:?}",
        final_max,
        final_min,
        sigmas
    );
    // sigma_min must always be enforced
    assert!(
        final_min >= sigma_min,
        "No sigma should drop below sigma_min={}. Got {}",
        sigma_min,
        final_min
    );
}

#[test]
fn self_adaptive_gaussian_returns_error_for_non_self_adaptive() {
    let mut binary_chrom = BinaryChromosome::new();
    // SelfAdaptiveGaussian must return Err for chromosomes not implementing SelfAdaptive
    let m = Mutation::SelfAdaptiveGaussian {
        tau: None,
        tau_prime: None,
        sigma_min: None,
        sigma_max: None,
    };
    let result = m.mutate(&mut binary_chrom, &m);
    assert!(
        result.is_err(),
        "SelfAdaptiveGaussian must return Err for BinaryChromosome (not a SelfAdaptive chromosome), got Ok"
    );
}

/// SelfAdaptiveGaussian with explicit params via variant works correctly
#[test]
fn self_adaptive_gaussian_inline_params_work() {
    let mut c = build_f64_chromosome(4);
    c.set_strategy_params(vec![0.1; 4]);

    let m = Mutation::SelfAdaptiveGaussian {
        tau: Some(0.5),
        tau_prime: Some(0.5),
        sigma_min: Some(1e-5),
        sigma_max: None,
    };

    for _ in 0..50 {
        m.mutate(&mut c, &m).unwrap();
        for &sigma in c.strategy_params() {
            assert!(sigma >= 1e-5, "Sigma {} dropped below sigma_min", sigma);
        }
    }
}

/// SelfAdaptiveGaussian { tau: None, ... } uses ES defaults and stays in range
#[test]
fn self_adaptive_gaussian_default_params_stay_in_range() {
    let mut c = build_f64_chromosome(4);
    c.set_strategy_params(vec![0.5; 4]);

    let m = Mutation::SelfAdaptiveGaussian {
        tau: None,
        tau_prime: None,
        sigma_min: None,
        sigma_max: None,
    };

    for _ in 0..100 {
        m.mutate(&mut c, &m).unwrap();
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

/// factory_self_adaptive still works with explicit params
#[test]
fn factory_self_adaptive_with_explicit_params() {
    let mut c = build_f64_chromosome(4);
    c.set_strategy_params(vec![0.1; 4]);
    let result = mutation::factory_self_adaptive(&mut c, Some(0.5), Some(0.5), Some(1e-5), None);
    assert!(
        result.is_ok(),
        "factory_self_adaptive should succeed: {:?}",
        result
    );
}
