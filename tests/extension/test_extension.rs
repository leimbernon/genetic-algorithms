use crate::structures::{Chromosome, Gene};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::extension;
use genetic_algorithms::operations::Extension;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

fn make_chromosome(ids: &[i32], fitness: f64) -> Chromosome {
    let dna: Vec<Gene> = ids.iter().map(|&id| Gene { id }).collect();
    Chromosome {
        dna,
        fitness,
        age: 0,
        ..Default::default()
    }
}

fn make_population(specs: &[(&[i32], f64)]) -> Vec<Chromosome> {
    specs
        .iter()
        .map(|(ids, fitness)| make_chromosome(ids, *fitness))
        .collect()
}

// ============================================================================
// Noop
// ============================================================================

#[test]
fn noop_does_not_modify_population() {
    let mut pop = make_population(&[(&[1, 2, 3], 10.0), (&[4, 5, 6], 5.0), (&[7, 8, 9], 1.0)]);
    let original = pop.clone();
    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::Noop,
        &mut pop,
        3,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), original.len());
    for (a, b) in pop.iter().zip(original.iter()) {
        assert_eq!(a.dna(), b.dna());
        assert_eq!(a.fitness(), b.fitness());
    }
}

// ============================================================================
// MassExtinction
// ============================================================================

#[test]
fn mass_extinction_reduces_population() {
    let mut pop = make_population(&[
        (&[1, 2, 3], 100.0),
        (&[4, 5, 6], 90.0),
        (&[7, 8, 9], 80.0),
        (&[10, 11, 12], 70.0),
        (&[13, 14, 15], 60.0),
        (&[16, 17, 18], 50.0),
        (&[19, 20, 21], 40.0),
        (&[22, 23, 24], 30.0),
        (&[25, 26, 27], 20.0),
        (&[28, 29, 30], 10.0),
    ]);

    let config = ExtensionConfiguration::new()
        .with_method(Extension::MassExtinction)
        .with_survival_rate(0.3)
        .with_elite_count(2);

    extension::factory(
        Extension::MassExtinction,
        &mut pop,
        10,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    // survival_rate=0.3 * 10 = 3 survivors (at least elite_count=2)
    assert!(pop.len() <= 4); // 3 target survivors, but at least elite_count
    assert!(pop.len() >= 2); // at least elite preserved

    // The 2 best (fitness 100 and 90) must be present
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&100.0));
    assert!(fitnesses.contains(&90.0));
}

#[test]
fn mass_extinction_preserves_elite_minimization() {
    let mut pop = make_population(&[
        (&[1, 2], 1.0),
        (&[3, 4], 2.0),
        (&[5, 6], 50.0),
        (&[7, 8], 100.0),
    ]);

    let config = ExtensionConfiguration::new()
        .with_survival_rate(0.5)
        .with_elite_count(1);

    extension::factory(
        Extension::MassExtinction,
        &mut pop,
        4,
        ProblemSolving::Minimization,
        &config,
    )
    .unwrap();

    // Best in minimization is fitness=1.0
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&1.0));
    assert!(pop.len() <= 3);
}

#[test]
fn mass_extinction_empty_population() {
    let mut pop: Vec<Chromosome> = vec![];
    let config = ExtensionConfiguration::new().with_survival_rate(0.3);

    extension::factory(
        Extension::MassExtinction,
        &mut pop,
        10,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert!(pop.is_empty());
}

// ============================================================================
// MassGenesis
// ============================================================================

#[test]
fn mass_genesis_keeps_two_best() {
    let mut pop = make_population(&[
        (&[1, 2, 3], 50.0),
        (&[4, 5, 6], 100.0),
        (&[7, 8, 9], 30.0),
        (&[10, 11, 12], 80.0),
        (&[13, 14, 15], 10.0),
    ]);

    let config = ExtensionConfiguration::new().with_method(Extension::MassGenesis);

    extension::factory(
        Extension::MassGenesis,
        &mut pop,
        5,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 2);
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&100.0));
    assert!(fitnesses.contains(&80.0));
}

#[test]
fn mass_genesis_keeps_two_best_minimization() {
    let mut pop = make_population(&[
        (&[1, 2], 50.0),
        (&[3, 4], 10.0),
        (&[5, 6], 30.0),
        (&[7, 8], 5.0),
    ]);

    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassGenesis,
        &mut pop,
        4,
        ProblemSolving::Minimization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 2);
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&5.0));
    assert!(fitnesses.contains(&10.0));
}

#[test]
fn mass_genesis_no_change_if_two_or_fewer() {
    let mut pop = make_population(&[(&[1, 2], 10.0), (&[3, 4], 20.0)]);
    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassGenesis,
        &mut pop,
        2,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 2);
}

// ============================================================================
// MassDegeneration
// ============================================================================

#[test]
fn mass_degeneration_preserves_elite() {
    let mut pop = make_population(&[
        (&[1, 2, 3, 4, 5], 100.0),
        (&[6, 7, 8, 9, 10], 50.0),
        (&[11, 12, 13, 14, 15], 30.0),
        (&[16, 17, 18, 19, 20], 10.0),
    ]);

    let elite_dna: Vec<i32> = pop[0].dna().iter().map(|g| g.id()).collect();

    let config = ExtensionConfiguration::new()
        .with_mutation_rounds(5)
        .with_elite_count(1);

    extension::factory(
        Extension::MassDegeneration,
        &mut pop,
        4,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 4);

    // Elite (best fitness=100) should be untouched
    // After sorting by fitness desc, first is the elite
    let best = pop
        .iter()
        .find(|c| c.fitness() == 100.0)
        .expect("Elite should still have fitness 100.0");
    let best_ids: Vec<i32> = best.dna().iter().map(|g| g.id()).collect();
    assert_eq!(best_ids, elite_dna);

    // Non-elite should have NaN fitness (marked for re-evaluation)
    for c in pop.iter().filter(|c| c.fitness() != 100.0) {
        assert!(c.fitness().is_nan(), "Non-elite should have NaN fitness");
    }
}

#[test]
fn mass_degeneration_empty_population() {
    let mut pop: Vec<Chromosome> = vec![];
    let config = ExtensionConfiguration::new().with_mutation_rounds(3);

    extension::factory(
        Extension::MassDegeneration,
        &mut pop,
        10,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert!(pop.is_empty());
}

// ============================================================================
// MassDeduplication
// ============================================================================

#[test]
fn mass_deduplication_removes_duplicates() {
    let mut pop = make_population(&[
        (&[1, 2, 3], 100.0),
        (&[1, 2, 3], 90.0), // duplicate
        (&[4, 5, 6], 80.0),
        (&[4, 5, 6], 70.0), // duplicate
        (&[7, 8, 9], 60.0),
    ]);

    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassDeduplication,
        &mut pop,
        5,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 3);

    // Should keep the best fitness for each unique gene set
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&100.0)); // best of [1,2,3]
    assert!(fitnesses.contains(&80.0)); // best of [4,5,6]
    assert!(fitnesses.contains(&60.0)); // unique [7,8,9]
}

#[test]
fn mass_deduplication_keeps_best_minimization() {
    let mut pop = make_population(&[
        (&[1, 2], 5.0),
        (&[1, 2], 10.0), // duplicate, worse in minimization
        (&[3, 4], 20.0),
    ]);

    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassDeduplication,
        &mut pop,
        3,
        ProblemSolving::Minimization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 2);
    let fitnesses: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    assert!(fitnesses.contains(&5.0)); // best of [1,2] in minimization
    assert!(fitnesses.contains(&20.0));
}

#[test]
fn mass_deduplication_all_unique() {
    let mut pop = make_population(&[(&[1, 2, 3], 10.0), (&[4, 5, 6], 20.0), (&[7, 8, 9], 30.0)]);

    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassDeduplication,
        &mut pop,
        3,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert_eq!(pop.len(), 3);
}

#[test]
fn mass_deduplication_empty_population() {
    let mut pop: Vec<Chromosome> = vec![];
    let config = ExtensionConfiguration::new();

    extension::factory(
        Extension::MassDeduplication,
        &mut pop,
        10,
        ProblemSolving::Maximization,
        &config,
    )
    .unwrap();

    assert!(pop.is_empty());
}

// ============================================================================
// Factory dispatch
// ============================================================================

#[test]
fn factory_dispatches_all_variants() {
    let variants = [
        Extension::Noop,
        Extension::MassExtinction,
        Extension::MassGenesis,
        Extension::MassDegeneration,
        Extension::MassDeduplication,
    ];

    for variant in &variants {
        let mut pop = make_population(&[
            (&[1, 2, 3, 4, 5], 100.0),
            (&[6, 7, 8, 9, 10], 50.0),
            (&[11, 12, 13, 14, 15], 25.0),
        ]);

        let config = ExtensionConfiguration::new()
            .with_method(*variant)
            .with_survival_rate(0.5)
            .with_mutation_rounds(1)
            .with_elite_count(1)
            .with_diversity_threshold(0.01);

        let result =
            extension::factory(*variant, &mut pop, 3, ProblemSolving::Maximization, &config);

        assert!(result.is_ok(), "Factory failed for {:?}", variant);
    }
}

// ============================================================================
// Configuration builder
// ============================================================================

#[test]
fn extension_config_builder() {
    let config = ExtensionConfiguration::new()
        .with_method(Extension::MassExtinction)
        .with_diversity_threshold(0.05)
        .with_survival_rate(0.2)
        .with_mutation_rounds(5)
        .with_elite_count(3);

    assert_eq!(config.method, Extension::MassExtinction);
    assert!((config.diversity_threshold - 0.05).abs() < f64::EPSILON);
    assert!((config.survival_rate - 0.2).abs() < f64::EPSILON);
    assert_eq!(config.mutation_rounds, 5);
    assert_eq!(config.elite_count, 3);
}

// ============================================================================
// GA integration via builder
// ============================================================================

#[test]
fn ga_extension_triggers_on_diversity() {
    // Create a population where all chromosomes have identical fitness (diversity = 0.0)
    // and configure extension with diversity_threshold = 1.0 so it triggers (0.0 < 1.0).
    use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, ExtensionConfig, MutationConfig, SelectionConfig,
        StoppingConfig,
    };

    // All chromosomes return the same fitness so diversity (std-dev) = 0.0, guaranteeing
    // the extension trigger fires on every generation (0.0 < 1.0 threshold).
    fn uniform_fitness(_dna: &[genetic_algorithms::genotypes::Binary]) -> f64 {
        1.0
    }

    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(uniform_fitness)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .with_extension_method(Extension::MassDeduplication)
        .with_extension_diversity_threshold(1.0)
        .build()
        .expect("Configuration should be valid");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with extension should complete successfully"
    );

    // Verify all stats entries have diversity >= 0.0 (extension uses diversity from stats)
    let stats = ga.stats();
    assert!(!stats.is_empty(), "Stats must be collected");
    for s in stats {
        assert!(s.diversity >= 0.0, "Diversity must be non-negative");
    }
}

#[test]
fn ga_builder_with_extension_config() {
    use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, ExtensionConfig, MutationConfig, SelectionConfig,
        StoppingConfig,
    };

    fn fitness_fn(dna: &[genetic_algorithms::genotypes::Binary]) -> f64 {
        dna.iter().filter(|g| g.value).count() as f64
    }

    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(10))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(50)
        .with_extension_method(Extension::MassDeduplication)
        .with_extension_diversity_threshold(0.5)
        .with_extension_elite_count(2)
        .build()
        .expect("Configuration should be valid");

    let result = ga.run();
    assert!(result.is_ok());

    // Verify extension config was set
    let ext = ga.configuration().extension().unwrap();
    assert_eq!(ext.method, Extension::MassDeduplication);
    assert!((ext.diversity_threshold - 0.5).abs() < f64::EPSILON);
    assert_eq!(ext.elite_count, 2);
}
