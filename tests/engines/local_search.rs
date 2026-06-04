use genetic_algorithms::configuration::{
    LocalSearchConfiguration, ProblemSolving,
};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{
    Crossover, LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode, Mutation, Selection,
    Survivor,
};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LocalSearchConfig, MutationConfig,
    SelectionConfig, StoppingConfig,
};

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
        .with_mutation_method(Mutation::Gaussian { sigma: None })
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
            application_strategy: LocalSearchApplicationStrategy::EveryNGenerations {
                interval: 2,
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

    let population = ga.run().expect("GA run should succeed without local search");
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness must be finite without local search: {}",
        best.fitness()
    );
}
