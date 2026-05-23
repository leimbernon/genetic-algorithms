use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome, SelfAdaptive};
use std::borrow::Cow;

#[test]
fn range_chromosome_lazy_init_strategy_params_to_ones() {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..5)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 1.0)], 0.5))
        .collect();
    c.set_dna(Cow::Owned(dna));
    let params = c.strategy_params();
    assert_eq!(
        params,
        &[1.0; 5],
        "strategy_params should be initialized to ones after set_dna with 5 genes, got {:?}",
        params
    );
}

#[test]
fn range_chromosome_set_strategy_params_round_trip() {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..3)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 1.0)], 0.5))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c.set_strategy_params(vec![0.3, 0.4, 0.5]);
    assert_eq!(
        c.strategy_params(),
        &[0.3, 0.4, 0.5],
        "strategy_params should round-trip via set_strategy_params"
    );
}

#[test]
fn range_chromosome_adapt_strategy_params_changes_values() {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..4)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 1.0)], 0.5))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c.set_strategy_params(vec![1.0; 4]);

    // After 10 attempts, at least one sigma should differ from 1.0 by > 1e-9
    let mut changed = false;
    for _ in 0..10 {
        c.set_strategy_params(vec![1.0; 4]);
        c.adapt_strategy_params(0.5, 0.5, 1e-5);
        let params = c.strategy_params();
        if params.iter().any(|&s: &f64| (s - 1.0).abs() > 1e-9) {
            changed = true;
            break;
        }
    }
    assert!(
        changed,
        "adapt_strategy_params never changed sigma across 10 attempts"
    );

    // All sigmas must remain >= sigma_min
    let sigma_min = 1e-5;
    for _ in 0..50 {
        c.adapt_strategy_params(0.5, 0.5, sigma_min);
        for &s in c.strategy_params() {
            assert!(
                s >= sigma_min,
                "adapt_strategy_params produced sigma {} below sigma_min {}",
                s,
                sigma_min
            );
        }
    }
}

#[test]
fn range_chromosome_empty_dna_returns_empty_strategy_params() {
    let c = RangeChromosome::<f64>::new();
    // Before set_dna, strategy_params should be empty
    assert_eq!(
        c.strategy_params().len(),
        0,
        "strategy_params before set_dna should be empty"
    );
    // adapt_strategy_params on empty params must not panic
    let mut c2 = RangeChromosome::<f64>::new();
    c2.adapt_strategy_params(0.5, 0.5, 1e-5); // must not panic
    assert_eq!(c2.strategy_params().len(), 0);
}
