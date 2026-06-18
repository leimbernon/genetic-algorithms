//! Tests for `MultiRangeChromosome<T>` — ChromosomeT + LinearChromosome impls,
//! OperatorCompat (no restriction), per-gene Gaussian mutation dispatch, Default.
//!
//! Covers GEN-03: MultiRangeChromosome with per-gene independent bounds and mutation rates.

use genetic_algorithms::chromosomes::MultiRangeChromosome;
use genetic_algorithms::genotypes::MultiRangeGenotype;
use genetic_algorithms::initializers::multi_range_random_initialization;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;

// ─── Default ─────────────────────────────────────────────────────────────────

#[test]
fn multi_range_chromosome_default_empty_dna_nan_fitness() {
    let c = MultiRangeChromosome::<f64>::default();
    assert!(c.dna.is_empty(), "default dna should be empty");
    assert!(c.fitness().is_nan(), "default fitness should be NaN");
    assert_eq!(c.age(), 0, "default age should be 0");
}

// ─── ChromosomeT impl ────────────────────────────────────────────────────────

#[test]
fn multi_range_chromosome_calculate_fitness_invokes_fn() {
    let mut c = MultiRangeChromosome::<f64>::new();
    let dna: Vec<_> = vec![
        MultiRangeGenotype::new(0, 0.0_f64, 10.0, 3.0, 0.1),
        MultiRangeGenotype::new(1, 0.0_f64, 10.0, 7.0, 0.1),
    ];
    c.set_dna(Cow::Owned(dna));
    c.set_fitness_fn(|genes| genes.iter().map(|g| g.value()).sum::<f64>());
    c.calculate_fitness();
    assert!(
        (c.fitness() - 10.0).abs() < 1e-10,
        "fitness should be 3+7=10"
    );
}

#[test]
fn multi_range_chromosome_set_fitness_returns_self() {
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_fitness(42.0);
    assert_eq!(c.fitness(), 42.0);
}

#[test]
fn multi_range_chromosome_set_age_returns_self() {
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_age(5);
    assert_eq!(c.age(), 5);
}

// ─── LinearChromosome impl ───────────────────────────────────────────────────

#[test]
fn multi_range_chromosome_set_dna_cow_owned_replaces_dna() {
    let mut c = MultiRangeChromosome::<f64>::default();
    let dna = vec![
        MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.5, 0.1),
        MultiRangeGenotype::new(1, 1.0_f64, 2.0, 1.5, 0.2),
    ];
    c.set_dna(Cow::Owned(dna));
    assert_eq!(c.dna().len(), 2);
    assert_eq!(c.dna()[0].value(), 0.5);
    assert_eq!(c.dna()[1].value(), 1.5);
}

#[test]
fn multi_range_chromosome_set_dna_cow_borrowed_replaces_dna() {
    let mut c = MultiRangeChromosome::<f64>::default();
    let dna = vec![MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.3, 0.1)];
    c.set_dna(Cow::Borrowed(dna.as_slice()));
    assert_eq!(c.dna().len(), 1);
    assert_eq!(c.dna()[0].value(), 0.3);
}

// ─── OperatorCompat (no restriction) ─────────────────────────────────────────

#[test]
fn multi_range_chromosome_operator_compat_no_restriction_crossovers() {
    use genetic_algorithms::traits::OperatorCompat;
    // MultiRangeChromosome should return None (no restriction)
    assert!(
        MultiRangeChromosome::<f64>::valid_crossovers().is_none(),
        "MultiRangeChromosome should have no crossover restriction"
    );
}

#[test]
fn multi_range_chromosome_operator_compat_no_restriction_mutations() {
    use genetic_algorithms::traits::OperatorCompat;
    assert!(
        MultiRangeChromosome::<f64>::valid_mutations().is_none(),
        "MultiRangeChromosome should have no mutation restriction"
    );
}

// ─── Build with SinglePoint crossover succeeds (no OperatorCompat restriction) ─

#[test]
fn multi_range_chromosome_single_point_crossover_accepted_at_build() {
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let bounds = vec![(0.0_f64, 1.0); 5];
    let rates = vec![0.1_f64; 5];
    let bounds_clone = bounds.clone();
    let rates_clone = rates.clone();

    let result: Result<Ga<MultiRangeChromosome<f64>>, _> = Ga::new()
        .with_population_size(10)
        .with_initialization_fn(move |_, _| {
            multi_range_random_initialization(&bounds_clone, &rates_clone)
        })
        .with_fitness_fn(|dna: &[MultiRangeGenotype<f64>]| dna.iter().map(|g| g.value()).sum())
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::SinglePoint)
        .with_mutation_method(Mutation::Gaussian { sigma: None })
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(1)
        .build();

    assert!(
        result.is_ok(),
        "MultiRangeChromosome should accept SinglePoint crossover (no operator restriction): {:?}",
        result.err()
    );
}

// ─── Gaussian mutation dispatch ───────────────────────────────────────────────

/// Build a chromosome with heterogeneous bounds for testing Gaussian mutation.
fn build_multi_range_chromosome() -> MultiRangeChromosome<f64> {
    let bounds = vec![(0.0_f64, 1.0), (10.0, 100.0)];
    let rates = vec![0.05_f64, 5.0_f64];
    let dna = multi_range_random_initialization(&bounds, &rates);
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_dna(Cow::Owned(dna));
    c
}

#[test]
fn multi_range_gaussian_mutation_stays_within_per_gene_bounds() {
    use genetic_algorithms::operations::mutation;
    use genetic_algorithms::operations::Mutation;

    let mut c = build_multi_range_chromosome();
    // Run 1000 iterations and verify every value stays within its gene's (lo, hi)
    for _ in 0..1000 {
        mutation::factory(Mutation::Gaussian { sigma: Some(10.0) }, &mut c).unwrap();
        for gene in c.dna() {
            assert!(
                gene.value >= gene.lo && gene.value <= gene.hi,
                "Gene {} value {} out of per-gene range [{}, {}]",
                gene.id,
                gene.value,
                gene.lo,
                gene.hi
            );
        }
    }
}

#[test]
fn multi_range_gaussian_mutation_per_gene_rate_controls_noise_scale() {
    use genetic_algorithms::operations::mutation;
    use genetic_algorithms::operations::Mutation;
    use genetic_algorithms::rng;

    // Gene 0: mutation_rate=0.0001 (tiny perturbations)
    // Gene 1: mutation_rate=20.0 (large perturbations within bounds)
    let bounds = vec![(0.0_f64, 1000.0), (0.0_f64, 1000.0)];
    let rates = vec![0.0001_f64, 20.0_f64];
    let dna = multi_range_random_initialization(&bounds, &rates);
    let mut c = MultiRangeChromosome::<f64>::default();
    c.set_dna(Cow::Owned(dna));

    // Track absolute delta per gene over many mutations
    let mut total_delta_0 = 0.0_f64;
    let mut total_delta_1 = 0.0_f64;
    let mut count_0 = 0usize;
    let mut count_1 = 0usize;

    rng::set_seed(Some(42));
    for _ in 0..2000 {
        let before: Vec<f64> = c.dna().iter().map(|g| g.value).collect();
        mutation::factory(Mutation::Gaussian { sigma: Some(1.0) }, &mut c).unwrap();
        let after: Vec<f64> = c.dna().iter().map(|g| g.value).collect();

        let delta_0 = (after[0] - before[0]).abs();
        let delta_1 = (after[1] - before[1]).abs();

        if delta_0 > 0.0 {
            total_delta_0 += delta_0;
            count_0 += 1;
        }
        if delta_1 > 0.0 {
            total_delta_1 += delta_1;
            count_1 += 1;
        }
    }
    rng::set_seed(None);

    // Gene with mutation_rate=20.0 should produce much larger average delta than rate=0.0001
    if count_0 > 0 && count_1 > 0 {
        let avg_0 = total_delta_0 / count_0 as f64;
        let avg_1 = total_delta_1 / count_1 as f64;
        assert!(
            avg_1 > avg_0 * 10.0,
            "Gene with mutation_rate=20.0 should have much larger avg delta ({:.6}) \
             than mutation_rate=0.0001 ({:.6})",
            avg_1,
            avg_0
        );
    }
}
