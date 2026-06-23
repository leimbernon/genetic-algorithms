use genetic_algorithms::configuration::{LocalSearchConfiguration, ProblemSolving};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::operations::{
    factory, factory_with_config, Crossover, GaussianParams, HillClimbingConfig, LocalSearch,
    LocalSearchApplicationStrategy, LocalSearchMode, Mutation, Selection, Survivor,
};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, LocalSearchConfig,
    LocalSearchOperator, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::error::GaError;
use std::borrow::Cow;

/// Sphere function: sum of squares — minimum at 0.0.
fn sphere_fitness(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

/// Helper to build a minimal GA with local search configured.
fn make_ga() -> Ga<genetic_algorithms::chromosomes::Range<f64>> {
    let alleles = vec![RangeGene::new(0, vec![(-5.12_f64, 5.12_f64)], 0.0)];
    Ga::new()
        .with_population_size(30)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(10))
        .with_max_generations(10)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
        .with_survivor_method(Survivor::Fitness)
        .with_alleles(alleles)
        .with_initialization_fn(
            genetic_algorithms::initializers::range_random_initialization::<f64>,
        )
        .with_fitness_fn(sphere_fitness)
}

#[test]
fn test_local_search_all_offspring() {
    let mut ga = make_ga()
        .with_local_search(LocalSearch::HillClimbing)
        .with_local_search_configuration(LocalSearchConfiguration {
            application_strategy: LocalSearchApplicationStrategy::AllOffspring,
            mode: LocalSearchMode::Lamarckian,
            ..Default::default()
        })
        .build()
        .expect("GA build should succeed");

    let population = ga.run().expect("GA run should succeed");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite: {}",
        best.fitness()
    );
    assert!(
        best.fitness() >= 0.0,
        "Sphere fitness should be non-negative: {}",
        best.fitness()
    );
}

#[test]
fn test_local_search_best_n() {
    let mut ga = make_ga()
        .with_local_search(LocalSearch::HillClimbing)
        .with_local_search_configuration(LocalSearchConfiguration {
            application_strategy: LocalSearchApplicationStrategy::BestN { n: 3 },
            mode: LocalSearchMode::Lamarckian,
            ..Default::default()
        })
        .build()
        .expect("GA build should succeed");

    let population = ga.run().expect("GA run should succeed");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite: {}",
        best.fitness()
    );
}

#[test]
fn test_local_search_probabilistic() {
    let mut ga = make_ga()
        .with_local_search(LocalSearch::HillClimbing)
        .with_local_search_configuration(LocalSearchConfiguration {
            application_strategy: LocalSearchApplicationStrategy::Probabilistic {
                probability: 1.0,
            },
            mode: LocalSearchMode::Lamarckian,
            ..Default::default()
        })
        .build()
        .expect("GA build should succeed");

    let population = ga.run().expect("GA run should succeed");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite: {}",
        best.fitness()
    );
}

#[test]
fn test_local_search_every_n_generations() {
    let mut ga = make_ga()
        .with_local_search(LocalSearch::HillClimbing)
        .with_local_search_configuration(LocalSearchConfiguration {
            application_strategy: LocalSearchApplicationStrategy::EveryNGenerations { interval: 2 },
            mode: LocalSearchMode::Lamarckian,
            ..Default::default()
        })
        .build()
        .expect("GA build should succeed");

    let population = ga.run().expect("GA run should succeed");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite: {}",
        best.fitness()
    );
}

#[test]
fn test_local_search_baldwinian() {
    let mut ga = make_ga()
        .with_local_search(LocalSearch::HillClimbing)
        .with_local_search_configuration(LocalSearchConfiguration {
            application_strategy: LocalSearchApplicationStrategy::AllOffspring,
            mode: LocalSearchMode::Baldwinian,
            ..Default::default()
        })
        .build()
        .expect("GA build should succeed");

    let population = ga.run().expect("GA run should succeed");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite: {}",
        best.fitness()
    );
}

#[test]
fn test_local_search_not_configured() {
    // No with_local_search or with_local_search_configuration — zero overhead path
    let mut ga = make_ga()
        .build()
        .expect("GA build should succeed without local search");

    let population = ga
        .run()
        .expect("GA run should succeed without local search");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite without local search: {}",
        best.fitness()
    );
}

// ---------------------------------------------------------------------------
// Migrated from src/operations/local_search.rs (inline #[cfg(test)] block)
// ---------------------------------------------------------------------------

/// Simple quadratic fitness: sum of squares.
fn quadratic(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value * g.value).sum::<f64>()
}

#[test]
fn test_hill_climbing_returns_improvements_count() {
    let genes: Vec<RangeGene<f64>> = (0..5)
        .map(|i| RangeGene::new(i, vec![(-10.0, 10.0)], 8.0))
        .collect();
    let mut chromo = RangeChromosome::<f64>::new();
    chromo.set_dna(Cow::Owned(genes));
    chromo.set_fitness(quadratic(chromo.dna()));

    let config = HillClimbingConfig {
        step_size: 1.0,
        max_iterations: 50,
    };
    let result = config.improve(&mut chromo, &quadratic);
    assert!(result.is_ok());
    let improvements = result.unwrap();
    assert!(improvements > 0, "expected at least one improvement");
    // Fitness should be lower (minimization) after hill climbing
    assert!(
        chromo.fitness() < 400.0,
        "fitness should improve: {}",
        chromo.fitness()
    );
}

#[test]
fn test_hill_climbing_unsupported_type() {
    use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
    use genetic_algorithms::genotypes::Binary as BinaryGene;

    let mut chromo = BinaryChromosome::new();
    let result = HillClimbingConfig::default().improve(&mut chromo, &|_: &[BinaryGene]| 0.0);
    assert!(result.is_err());
    match result {
        Err(GaError::LocalSearchError(msg)) => {
            assert!(msg.contains("HillClimbing"));
        }
        _ => panic!("expected LocalSearchError"),
    }
}

#[test]
fn test_hill_climbing_empty_dna() {
    let mut chromo = RangeChromosome::<f64>::new();
    let result = HillClimbingConfig::default().improve(&mut chromo, &quadratic);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_local_search_application_strategy_default() {
    let strategy = LocalSearchApplicationStrategy::default();
    assert!(matches!(
        strategy,
        LocalSearchApplicationStrategy::AllOffspring
    ));
}

#[test]
fn test_local_search_mode_default() {
    let mode = LocalSearchMode::default();
    assert!(matches!(mode, LocalSearchMode::Lamarckian));
}

#[test]
fn test_factory_returns_enum() {
    let op = factory(LocalSearch::HillClimbing);
    assert_eq!(op, LocalSearch::HillClimbing);
}

#[test]
fn test_factory_with_config() {
    let config = HillClimbingConfig {
        step_size: 0.01,
        max_iterations: 10,
    };
    let result = factory_with_config(LocalSearch::HillClimbing, config);
    // factory_with_config returns HillClimbingConfig which implements LocalSearchOperator
    let mut chromo = RangeChromosome::<f64>::new();
    let improve_result = result.improve(&mut chromo, &|_: &[RangeGene<f64>]| 0.0);
    assert!(improve_result.is_ok());
}
