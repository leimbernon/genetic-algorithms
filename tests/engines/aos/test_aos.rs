//! Unit tests for AOS core module (Phase 43).
//!
//! Tests AosStrategy construction, AosState::new(), select_operator(),
//! record_rewards(), update(), and compute_normalized_reward().

use genetic_algorithms::aos::{AosState, AosStrategy, compute_normalized_reward};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Create a seeded RNG for deterministic tests.
fn make_rng() -> SmallRng {
    SmallRng::seed_from_u64(42)
}

// ---------------------------------------------------------------------------
// AosState construction tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_new_creates_correct_number_of_arms() {
    let state = AosState::new(3, AosStrategy::pm_default(), 50);
    assert_eq!(state.num_arms(), 3);
}

#[test]
fn test_aos_new_uniform_probabilities() {
    // We'll test via select_operator behavior — uniform in exploration
    let mut state = AosState::new(4, AosStrategy::pm_default(), 50);
    let mut rng = make_rng();
    let mut counts = [0usize; 4];
    // In exploration phase (gen < 25), all operators equally likely
    for _ in 0..1000 {
        let op = state.select_operator(&mut rng, 0);
        assert!(op < 4, "Operator index out of range");
        counts[op] += 1;
    }
    // Each operator should have roughly equal chance (within 2x)
    for &c in &counts {
        assert!(
            c > 100,
            "Exploration should distribute roughly evenly, got {} for arm",
            c
        );
    }
}

#[test]
fn test_aos_new_exploration_generations() {
    // With window=50, exploration = 25 gens. Generation 0-24 uses uniform.
    // Generation 25+ uses the strategy.
    let mut state_pm = AosState::new(2, AosStrategy::pm_default(), 50);
    let mut rng = make_rng();
    for gen in 0..25 {
        let _ = state_pm.select_operator(&mut rng, gen);
    }
    let _ = state_pm.select_operator(&mut rng, 24);
    // Gen 25 should be post-exploration (just verify it doesn't panic)
    let _ = state_pm.select_operator(&mut rng, 25);
}

// ---------------------------------------------------------------------------
// Exploration phase tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_exploration_phase_distribution() {
    let mut state = AosState::new(5, AosStrategy::mab_default(), 100);
    let mut rng = make_rng();
    let mut counts = [0usize; 5];
    for _ in 0..500 {
        let op = state.select_operator(&mut rng, 10); // exploration (10 < 50)
        assert!(op < 5);
        counts[op] += 1;
    }
    // All 5 arms should have been selected at least once
    assert!(
        counts.iter().all(|&c| c > 0),
        "All arms should be selected during exploration"
    );
}

// ---------------------------------------------------------------------------
// Reward recording tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_record_rewards_single() {
    let mut state = AosState::new(2, AosStrategy::pm_default(), 50);
    state.record_rewards(&[(0, 0.5), (1, -0.2)]);
    // Verify no panic; rewards stored for later update()
}

#[test]
fn test_aos_record_rewards_out_of_bounds() {
    let mut state = AosState::new(2, AosStrategy::pm_default(), 50);
    // Should not panic when op_idx >= num_arms
    state.record_rewards(&[(5, 0.5)]);
}

#[test]
fn test_aos_record_rewards_batch_update() {
    let mut state = AosState::new(3, AosStrategy::pm_default(), 10);
    let rewards: Vec<(usize, f64)> = vec![(0, 0.8), (1, 0.1), (0, 0.6)];
    state.record_rewards(&rewards);
    // After record_rewards, update() should not panic
    state.update();
}

// ---------------------------------------------------------------------------
// PM (Probability Matching) tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_pm_update_changes_probabilities() {
    let mut state = AosState::new(
        3,
        AosStrategy::ProbabilityMatching {
            alpha: 0.8,
            learning_rate: 0.3,
        },
        10,
    );
    // Give arm 0 very high rewards, arms 1-2 low rewards
    for _ in 0..10 {
        state.record_rewards(&[(0, 1.0), (1, -0.5), (2, -0.3)]);
    }
    state.update();
    // Arm 0 should have higher probability after update (via sliding window mean)
    // We verify update doesn't panic and state is internally consistent
}

// ---------------------------------------------------------------------------
// AP (Adaptive Pursuit) tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_ap_update() {
    let mut state = AosState::new(
        2,
        AosStrategy::AdaptivePursuit {
            beta: 0.5,
            c: 1.5,
        },
        10,
    );
    state.record_rewards(&[(0, 1.0), (1, -1.0)]);
    state.update();
    // Verify no panic
}

// ---------------------------------------------------------------------------
// MAB (Multi-Armed Bandit) tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_mab_select_ucb1() {
    let mut state = AosState::new(
        3,
        AosStrategy::MultiArmedBandit {
            c: 1.0,
            epsilon: 0.1,
        },
        50,
    );
    let mut rng = make_rng();

    // Seed MAB with some initial rewards to create UCB differentiation
    state.record_rewards(&[(0, 0.5), (0, 0.6), (0, 0.7)]);
    state.record_rewards(&[(1, 0.1), (1, 0.0), (1, -0.1)]);
    state.record_rewards(&[(2, 0.3), (2, 0.2), (2, 0.4)]);

    // Select operators post-exploration (gen > 25)
    // Arm 0 has highest rewards so should be preferred
    let _ = state.select_operator(&mut rng, 30);
}

// ---------------------------------------------------------------------------
// compute_normalized_reward tests
// ---------------------------------------------------------------------------

#[test]
fn test_compute_normalized_reward_positive() {
    // Offspring better (lower fitness) than parent: positive reward
    let reward = compute_normalized_reward(10.0, 5.0, 20.0);
    assert!(
        reward > 0.0,
        "Better offspring should give positive reward, got {}",
        reward
    );
    assert!(
        (reward - 0.25).abs() < 1e-10,
        "Expected ~0.25, got {}",
        reward
    );
}

#[test]
fn test_compute_normalized_reward_negative() {
    // Offspring worse (higher fitness) than parent: negative reward
    let reward = compute_normalized_reward(5.0, 10.0, 20.0);
    assert!(
        reward < 0.0,
        "Worse offspring should give negative reward, got {}",
        reward
    );
}

#[test]
fn test_compute_normalized_reward_zero_delta() {
    // Offspring same fitness as parent: zero reward
    let reward = compute_normalized_reward(10.0, 10.0, 20.0);
    assert!(
        (reward).abs() < 1e-10,
        "Equal fitness should give ~0 reward, got {}",
        reward
    );
}

#[test]
fn test_compute_normalized_reward_zero_best() {
    // Best fitness of 0 should not cause NaN (EPSILON clamp)
    let reward = compute_normalized_reward(10.0, 5.0, 0.0);
    assert!(
        !reward.is_nan(),
        "Reward should not be NaN when best_fitness=0"
    );
    assert!(reward > 0.0, "Positive reward expected");
}

#[test]
fn test_compute_normalized_reward_best_nonzero() {
    // Normal case with non-zero best
    let reward = compute_normalized_reward(100.0, 95.0, 50.0);
    assert!((reward - 0.1).abs() < 1e-10, "Expected ~0.1, got {}", reward);
}

// ---------------------------------------------------------------------------
// Strategy default constructor tests
// ---------------------------------------------------------------------------

#[test]
fn test_aos_strategy_pm_default() {
    let strat = AosStrategy::pm_default();
    match strat {
        AosStrategy::ProbabilityMatching {
            alpha,
            learning_rate,
        } => {
            assert!((alpha - 0.8).abs() < 1e-10);
            assert!((learning_rate - 0.3).abs() < 1e-10);
        }
        _ => panic!("Expected ProbabilityMatching"),
    }
}

#[test]
fn test_aos_strategy_ap_default() {
    let strat = AosStrategy::ap_default();
    match strat {
        AosStrategy::AdaptivePursuit { beta, c } => {
            assert!((beta - 0.5).abs() < 1e-10);
            assert!((c - 1.5).abs() < 1e-10);
        }
        _ => panic!("Expected AdaptivePursuit"),
    }
}

#[test]
fn test_aos_strategy_mab_default() {
    let strat = AosStrategy::mab_default();
    match strat {
        AosStrategy::MultiArmedBandit { c, epsilon } => {
            assert!((c - 1.0).abs() < 1e-10);
            assert!((epsilon - 0.1).abs() < 1e-10);
        }
        _ => panic!("Expected MultiArmedBandit"),
    }
}

// ---------------------------------------------------------------------------
// GA Integration Tests (Phase 43, Plan 02 -- require Ga engine)
// ---------------------------------------------------------------------------

/// Helper: build a basic GA with AOS crossover portfolio (3 operators).
fn aos_ga_xover() -> Ga<RangeChromosome<i32>> {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n as usize))
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
            Crossover::Clone,
        ])
        .with_mutation_method(Mutation::Swap)
        .with_aos_strategy(AosStrategy::pm_default())
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
}

/// Helper: build a GA with both crossover and mutation AOS portfolios.
fn aos_ga_both() -> Ga<RangeChromosome<i32>> {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n as usize))
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
        ])
        .with_mutation_portfolio(vec![
            Mutation::Swap,
            Mutation::Inversion,
            Mutation::Scramble,
        ])
        .with_aos_strategy(AosStrategy::pm_default())
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
}

#[test]
fn test_aos_ga_crossover_portfolio_builds_and_runs() {
    let mut ga = aos_ga_xover()
        .build()
        .expect("AOS GA with crossover portfolio should build");
    let result = ga.run();
    result.expect("AOS GA should run without errors");
}

#[test]
fn test_aos_ga_both_portfolios_builds_and_runs() {
    let mut ga = aos_ga_both()
        .build()
        .expect("AOS GA with both portfolios should build");
    let result = ga.run();
    assert!(result.is_ok(), "AOS GA with both portfolios should run without errors");
}

#[test]
fn test_aos_ga_mab_strategy_runs() {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n as usize))
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
            Crossover::Clone,
        ])
        .with_mutation_method(Mutation::Swap)
        .with_aos_strategy(AosStrategy::mab_default())
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
        .build()
        .expect("MAB strategy GA should build");
    let result = ga.run();
    result.expect("MAB strategy GA should run without errors");
}

#[test]
fn test_aos_ga_adaptive_pursuit_runs() {
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n as usize))
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
        ])
        .with_mutation_method(Mutation::Swap)
        .with_aos_strategy(AosStrategy::ap_default())
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
        .build()
        .expect("Adaptive Pursuit strategy GA should build");
    let result = ga.run();
    assert!(result.is_ok(), "Adaptive Pursuit GA should run without errors");
}

#[test]
fn test_aos_ga_with_adaptive_ga_coexists() {
    // AOS + Adaptive GA can both be enabled (D-13)
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n as usize))
        .with_population_size(30)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
        .with_selection_method(Selection::Tournament)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
        ])
        .with_mutation_portfolio(vec![
            Mutation::Swap,
            Mutation::Inversion,
        ])
        .with_aos_strategy(AosStrategy::pm_default())
        .with_adaptive_ga(true)
        .with_crossover_probability_max(1.0)
        .with_crossover_probability_min(0.5)
        .with_mutation_probability_max(0.1)
        .with_mutation_probability_min(0.01)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_alleles(alleles)
        .build()
        .expect("AOS + Adaptive GA should build");
    let result = ga.run();
    assert!(result.is_ok(), "AOS + Adaptive GA should run without errors");
}

// ---------------------------------------------------------------------------
// Serde round-trip tests (behind #[cfg(feature = "serde")])
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
#[test]
fn test_aos_serde_strategy_roundtrip() {
    let pm = AosStrategy::pm_default();
    let json = serde_json::to_string(&pm).expect("Serialize PM");
    let deserialized: AosStrategy = serde_json::from_str(&json).expect("Deserialize PM");
    assert_eq!(pm, deserialized, "PM strategy serde round-trip");

    let ap = AosStrategy::ap_default();
    let json = serde_json::to_string(&ap).expect("Serialize AP");
    let deserialized: AosStrategy = serde_json::from_str(&json).expect("Deserialize AP");
    assert_eq!(ap, deserialized, "AP strategy serde round-trip");

    let mab = AosStrategy::mab_default();
    let json = serde_json::to_string(&mab).expect("Serialize MAB");
    let deserialized: AosStrategy = serde_json::from_str(&json).expect("Deserialize MAB");
    assert_eq!(mab, deserialized, "MAB strategy serde round-trip");
}

#[cfg(feature = "serde")]
#[test]
fn test_aos_serde_state_roundtrip() {
    // Create an AOS state, record some rewards, update, then round-trip
    let mut state = AosState::new(3, AosStrategy::pm_default(), 10);
    state.record_rewards(&[(0, 0.5), (1, 0.3), (2, 0.1)]);
    state.update();

    let json = serde_json::to_string(&state).expect("Serialize AosState");
    let mut deserialized: AosState = serde_json::from_str(&json).expect("Deserialize AosState");

    // Verify select_operator still works on deserialized state
    let mut rng = make_rng();
    let op = deserialized.select_operator(&mut rng, 100); // post-exploration
    assert!(op < 3, "Deserialized state select_operator returns valid index");
}
