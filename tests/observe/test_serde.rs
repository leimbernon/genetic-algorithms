//! Integration tests for serde serialization / deserialization round-trips.
//!
//! These tests are gated behind `#[cfg(feature = "serde")]` and exercise
//! the JSON round-trip for core GA types.
#![cfg(feature = "serde")]

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::error::GaError;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::island::configuration::{IslandConfiguration, MigrationPolicy};
use genetic_algorithms::island::topology::MigrationTopology;
use genetic_algorithms::niching::configuration::NichingConfiguration;
use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use genetic_algorithms::operations::{
    AlignmentStrategy, CauchyParams, CreepParams, Crossover, DifferentialParams, GaussianParams,
    LevyFlightParams, Mutation, NonUniformParams, PolynomialParams, Selection,
    SelfAdaptiveGaussianParams, Survivor,
};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::ChromosomeLength;

/// Helper: serialize to JSON and deserialize back, returning the round-tripped value.
fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
    let json = serde_json::to_string(value).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

// ---- Operation enums ----

#[test]
fn serde_selection_enum() {
    let variants = [
        Selection::Random,
        Selection::RouletteWheel,
        Selection::Tournament,
        Selection::Boltzmann,
        Selection::Truncation,
        Selection::Rank,
        Selection::StochasticUniversalSampling,
        Selection::Clearing,
    ];
    for v in &variants {
        assert_eq!(&round_trip(v), v);
    }
}

#[test]
fn serde_crossover_enum() {
    let variants = [
        Crossover::Cycle,
        Crossover::MultiPoint,
        Crossover::Uniform,
        Crossover::SinglePoint,
        Crossover::Order,
        Crossover::Pmx,
        Crossover::Sbx,
        Crossover::BlendAlpha,
        Crossover::Arithmetic,
        Crossover::Clone,
        Crossover::Rejuvenate,
        Crossover::EdgeRecombination,
        Crossover::VariableLength(AlignmentStrategy::Trim),
        Crossover::VariableLength(AlignmentStrategy::Pad),
    ];
    for v in &variants {
        assert_eq!(&round_trip(v), v);
    }
}

#[test]
fn serde_alignment_strategy_enum() {
    assert_eq!(
        &round_trip(&AlignmentStrategy::Trim),
        &AlignmentStrategy::Trim
    );
    assert_eq!(
        &round_trip(&AlignmentStrategy::Pad),
        &AlignmentStrategy::Pad
    );
}

#[test]
fn serde_mutation_enum() {
    let variants = vec![
        Mutation::Swap,
        Mutation::Inversion,
        Mutation::Scramble,
        Mutation::Value,
        Mutation::BitFlip,
        Mutation::Creep(CreepParams { step: None }),
        Mutation::Creep(CreepParams { step: Some(0.05) }),
        Mutation::Gaussian(GaussianParams { sigma: None }),
        Mutation::Gaussian(GaussianParams { sigma: Some(0.2) }),
        Mutation::Polynomial(PolynomialParams { eta: None }),
        Mutation::Polynomial(PolynomialParams { eta: Some(20.0) }),
        Mutation::NonUniform(NonUniformParams { b: None }),
        Mutation::NonUniform(NonUniformParams { b: Some(2.0) }),
        Mutation::PermutationInsert,
        Mutation::Insertion,
        Mutation::Deletion,
        Mutation::Differential(DifferentialParams { f: None }),
        Mutation::Differential(DifferentialParams { f: Some(0.5) }),
        Mutation::Cauchy(CauchyParams { scale: None }),
        Mutation::Cauchy(CauchyParams { scale: Some(1.0) }),
        Mutation::LevyFlight(LevyFlightParams { alpha: None }),
        Mutation::LevyFlight(LevyFlightParams { alpha: Some(1.5) }),
        Mutation::Uniform,
        Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
            tau: None,
            tau_prime: None,
            sigma_min: None,
            sigma_max: None,
        }),
        Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
            tau: Some(0.3),
            tau_prime: Some(0.2),
            sigma_min: Some(1e-5),
            sigma_max: Some(1.0),
        }),
    ];
    for v in &variants {
        assert_eq!(&round_trip(v), v, "Round-trip failed for {:?}", v);
    }
}

#[test]
fn serde_survivor_enum() {
    let variants = [
        Survivor::Fitness,
        Survivor::Age,
        Survivor::MuPlusLambda,
        Survivor::MuCommaLambda,
        Survivor::DeterministicCrowding,
    ];
    for v in &variants {
        assert_eq!(&round_trip(v), v);
    }
}

// ---- Configuration types ----

#[test]
fn serde_ga_configuration_round_trip() {
    let config = GaConfiguration::default();
    let rt = round_trip(&config);
    assert_eq!(rt, config);
}

#[test]
fn serde_ga_configuration_with_values() {
    use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_initializer::binary_random_initialization;
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    // Build a GA with specific operator settings via the public builder API,
    // then extract and round-trip its configuration.
    let ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(200)
        .with_chromosome_length(ChromosomeLength::Fixed(16))
        .with_selection_method(Selection::Boltzmann)
        .with_crossover_method(Crossover::Sbx)
        .with_mutation_method(Mutation::Polynomial(PolynomialParams { eta: None }))
        .with_survivor_method(Survivor::MuPlusLambda)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(500)
        .with_stagnation_limit(50)
        .with_convergence_threshold(0.001)
        .with_max_duration_secs(60.0)
        .with_initialization_fn(|n, _| binary_random_initialization(n, None))
        .with_fitness_fn(|_: &[_]| 0.0);

    let config = ga.configuration.clone();
    let rt = round_trip(&config);
    assert_eq!(rt, config);
    // Spot-check a few values
    assert_eq!(rt.selection().method, Selection::Boltzmann);
    assert_eq!(rt.limit().population_size, 200);
    assert_eq!(rt.stagnation_generations(), Some(50));
    assert_eq!(rt.convergence_threshold(), Some(0.001));
}

#[test]
fn serde_stopping_criteria_flat() {
    // StoppingCriteria struct was removed in v3.0.0. Test the flat fields on GaConfiguration.
    use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_initializer::binary_random_initialization;
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};

    let ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(10)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_stagnation_limit(100)
        .with_convergence_threshold(0.01)
        .with_max_duration_secs(300.0)
        .with_initialization_fn(|n, _| binary_random_initialization(n, None))
        .with_fitness_fn(|_: &[_]| 0.0);

    let config = ga.configuration.clone();
    let rt = round_trip(&config);
    assert_eq!(rt.stagnation_generations(), Some(100));
    assert!((rt.convergence_threshold().unwrap() - 0.01).abs() < f64::EPSILON);
    assert!((rt.max_duration_secs().unwrap() - 300.0).abs() < f64::EPSILON);
}

// ---- Error type ----

#[test]
fn serde_ga_error_round_trip() {
    let errors = [
        GaError::ConfigurationError("bad config".into()),
        GaError::ValidationError("invalid".into()),
        GaError::CrossoverError("cx fail".into()),
        GaError::MutationError("mut fail".into()),
        GaError::InitializationError("init fail".into()),
        GaError::SelectionError("sel fail".into()),
        GaError::InvalidIslandConfiguration("island bad".into()),
        GaError::InvalidNichingConfiguration("niche bad".into()),
        GaError::InvalidNsga2Configuration("nsga bad".into()),
        GaError::MigrationError("mig fail".into()),
    ];
    for e in &errors {
        assert_eq!(&round_trip(e), e);
    }
}

// ---- TerminationCause ----

#[test]
fn serde_termination_cause() {
    let causes = [
        TerminationCause::GenerationLimitReached,
        TerminationCause::FitnessTargetReached,
        TerminationCause::StagnationReached,
        TerminationCause::ConvergenceReached,
        TerminationCause::TimeLimitReached,
        TerminationCause::CallbackRequested,
        TerminationCause::NotTerminated,
    ];
    for c in &causes {
        assert_eq!(&round_trip(c), c);
    }
}

// ---- GenerationStats ----

#[test]
fn serde_generation_stats() {
    let stats = GenerationStats::from_fitness_values(5, &[1.0, 2.0, 3.0, 4.0, 5.0], true);
    let rt = round_trip(&stats);
    assert_eq!(rt.generation, stats.generation);
    assert_eq!(rt.best_fitness, stats.best_fitness);
    assert_eq!(rt.worst_fitness, stats.worst_fitness);
    assert!((rt.avg_fitness - stats.avg_fitness).abs() < 1e-10);
    assert!((rt.fitness_std_dev - stats.fitness_std_dev).abs() < 1e-10);
    assert_eq!(rt.population_size, stats.population_size);
    assert!((rt.diversity - stats.diversity).abs() < 1e-10);
}

#[test]
fn serde_generation_stats_backward_compat() {
    // JSON without diversity field (simulates old checkpoint)
    let json = r#"{"generation":5,"best_fitness":5.0,"worst_fitness":1.0,"avg_fitness":3.0,"fitness_std_dev":1.4142135623730951,"population_size":5}"#;
    let stats: GenerationStats = serde_json::from_str(json).unwrap();
    assert_eq!(stats.diversity, 0.0);
}

// ---- Genotypes ----

#[test]
fn serde_binary_gene() {
    let gene = BinaryGene {
        id: 42,
        value: true,
    };
    assert_eq!(round_trip(&gene), gene);
}

#[test]
fn serde_range_gene_i32() {
    let gene = RangeGene::<i32>::new(1, vec![(0, 100)], 50);
    assert_eq!(round_trip(&gene), gene);
}

#[test]
fn serde_range_gene_f64() {
    let gene = RangeGene::<f64>::new(2, vec![(0.0, 1.0)], 0.5);
    let rt = round_trip(&gene);
    assert_eq!(rt.id, gene.id);
    assert_eq!(rt.ranges, gene.ranges);
    assert!((rt.value - gene.value).abs() < 1e-10);
}

// ---- Chromosomes ----

#[test]
fn serde_binary_chromosome_round_trip() {
    let mut chrom = BinaryChromosome::new();
    chrom.dna = vec![
        BinaryGene { id: 0, value: true },
        BinaryGene {
            id: 1,
            value: false,
        },
        BinaryGene { id: 2, value: true },
    ];
    chrom.fitness = 3.0;
    chrom.age = 7;

    let rt = round_trip(&chrom);
    assert_eq!(rt.dna, chrom.dna);
    assert_eq!(rt.fitness, chrom.fitness);
    assert_eq!(rt.age, chrom.age);
    // fitness_fn is skipped; after deserialization the default returns 0.0
    assert_eq!(rt.fitness_fn.call(&[]), 0.0);
}

#[test]
fn serde_range_chromosome_round_trip() {
    let mut chrom = RangeChromosome::<f64>::new();
    chrom.dna = vec![
        RangeGene::new(0, vec![(0.0, 10.0)], 5.0),
        RangeGene::new(1, vec![(0.0, 10.0)], 7.5),
    ];
    chrom.fitness = 12.5;
    chrom.age = 3;

    let rt = round_trip(&chrom);
    assert_eq!(rt.dna, chrom.dna);
    assert_eq!(rt.fitness, chrom.fitness);
    assert_eq!(rt.age, chrom.age);
    // fitness_fn is skipped; after deserialization the default returns 0.0
    assert_eq!(rt.fitness_fn.call(&[]), 0.0);
}

// ---- Population ----

#[test]
fn serde_population_binary_round_trip() {
    let mut chrom1 = BinaryChromosome::new();
    chrom1.dna = vec![BinaryGene { id: 0, value: true }];
    chrom1.fitness = 1.0;

    let mut chrom2 = BinaryChromosome::new();
    chrom2.dna = vec![BinaryGene {
        id: 0,
        value: false,
    }];
    chrom2.fitness = 0.0;

    let mut pop = Population::new(vec![chrom1.clone(), chrom2.clone()]);
    pop.best_chromosome = chrom1.clone();
    pop.f_avg = 0.5;
    pop.f_max = 1.0;

    let rt = round_trip(&pop);
    assert_eq!(rt.chromosomes.len(), 2);
    assert_eq!(rt.chromosomes[0].dna, chrom1.dna);
    assert_eq!(rt.chromosomes[1].dna, chrom2.dna);
    assert_eq!(rt.best_chromosome.dna, chrom1.dna);
    assert!((rt.f_avg - 0.5).abs() < f64::EPSILON);
    assert!((rt.f_max - 1.0).abs() < f64::EPSILON);
}

// ---- NSGA-II configuration ----

#[test]
fn serde_nsga2_configuration() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(3)
        .with_population_size(50)
        .with_max_generations(1000)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Maximize,
            ObjectiveDirection::Minimize,
        ]);

    let rt = round_trip(&config);
    assert_eq!(rt.num_objectives, 3);
    assert_eq!(rt.population_size, 50);
    assert_eq!(rt.max_generations, 1000);
    assert_eq!(rt.objective_directions.len(), 3);
    assert_eq!(rt.objective_directions[1], ObjectiveDirection::Maximize);
}

// ---- Island model configuration ----

#[test]
fn serde_island_configuration() {
    let config = IslandConfiguration::new()
        .with_num_islands(8)
        .with_migration_interval(20)
        .with_migration_count(3)
        .with_topology(MigrationTopology::Grid(2, 4))
        .with_migration_policy(MigrationPolicy::TournamentMigrant);

    let rt = round_trip(&config);
    assert_eq!(rt.num_islands, 8);
    assert_eq!(rt.migration_interval, 20);
    assert_eq!(rt.migration_count, 3);
    assert_eq!(rt.topology, MigrationTopology::Grid(2, 4));
    assert_eq!(rt.migration_policy, MigrationPolicy::TournamentMigrant);
}

#[test]
fn serde_migration_topology_variants() {
    let topologies = vec![
        MigrationTopology::Ring,
        MigrationTopology::FullyConnected,
        MigrationTopology::Grid(3, 3),
        MigrationTopology::Hypercube,
        MigrationTopology::Custom(vec![vec![1, 2], vec![0], vec![0, 1]]),
    ];
    for t in &topologies {
        assert_eq!(&round_trip(t), t);
    }
}

// ---- Niching configuration ----

#[test]
fn serde_niching_configuration() {
    let config = NichingConfiguration::new()
        .with_enabled(true)
        .with_sigma_share(2.5)
        .with_alpha(0.5);

    let rt = round_trip(&config);
    assert!(rt.enabled);
    assert!((rt.sigma_share - 2.5).abs() < f64::EPSILON);
    assert!((rt.alpha - 0.5).abs() < f64::EPSILON);
}

// ---- Checkpoint integration tests ----

use genetic_algorithms::checkpoint::{load_checkpoint, save_checkpoint, Checkpoint};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::binary_initializer::binary_random_initialization;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig,
    SelectionConfig, StoppingConfig,
};
use std::path::Path;

#[test]
fn checkpoint_round_trip_via_api() {
    // Create a checkpoint with a small Binary population
    let genes: Vec<BinaryGene> = (0..4)
        .map(|i| BinaryGene {
            id: i,
            value: i % 2 == 0,
        })
        .collect();

    let mut c1 = <BinaryChromosome as Default>::default();
    c1.set_dna(std::borrow::Cow::Owned(genes.clone()));
    c1.set_fitness(10.0);

    let mut c2 = <BinaryChromosome as Default>::default();
    c2.set_dna(std::borrow::Cow::Owned(genes));
    c2.set_fitness(20.0);

    let mut pop = Population::new(vec![c1.clone(), c2]);
    pop.best_chromosome = c1;

    let ckpt = Checkpoint {
        population: pop,
        configuration: GaConfiguration::default(),
        generation: 3,
        stats: vec![GenerationStats::from_fitness_values(
            0,
            &[10.0, 20.0],
            false,
        )],
    };

    let dir = std::env::temp_dir().join("ga_integ_ckpt_rt");
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("ckpt.json");

    save_checkpoint(&ckpt, &path).expect("save");
    let loaded: Checkpoint<BinaryChromosome> = load_checkpoint(&path).expect("load");

    assert_eq!(loaded.generation, 3);
    assert_eq!(loaded.population.size(), 2);
    assert_eq!(loaded.stats.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ga_run_with_save_progress_creates_checkpoint_files() {
    let ckpt_dir = std::env::temp_dir().join("ga_integ_save_progress");
    let _ = std::fs::remove_dir_all(&ckpt_dir);

    let alleles = vec![BinaryGene {
        id: 0,
        value: false,
    }];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(10)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_save_progress(true)
        .with_save_progress_interval(5)
        .with_save_progress_path(ckpt_dir.to_str().unwrap().to_string())
        .with_initialization_fn(move |genes_per_chromosome, _| {
            binary_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .build()
        .expect("build should succeed");

    let _ = ga.run().expect("run should succeed");

    // Checkpoint files should exist for generation 5 and 10
    let ckpt5 = ckpt_dir.join("checkpoint_gen_5.json");
    let ckpt10 = ckpt_dir.join("checkpoint_gen_10.json");
    assert!(ckpt5.exists(), "checkpoint at generation 5 should exist");
    assert!(ckpt10.exists(), "checkpoint at generation 10 should exist");

    // Load one and verify it's valid
    let loaded: Checkpoint<BinaryChromosome> =
        load_checkpoint(&ckpt5).expect("should load gen 5 checkpoint");
    assert_eq!(loaded.generation, 4); // 0-based index, gen 5 = index 4
    assert_eq!(loaded.population.size(), 10);
    assert_eq!(loaded.stats.len(), 5);

    let _ = std::fs::remove_dir_all(&ckpt_dir);
}

#[test]
fn load_checkpoint_nonexistent_returns_error() {
    let result: Result<Checkpoint<BinaryChromosome>, _> =
        load_checkpoint(Path::new("/tmp/ga_integ_nonexistent_ckpt.json"));
    assert!(result.is_err());
    match result.unwrap_err() {
        GaError::CheckpointError(msg) => assert!(msg.contains("Failed to read"), "{msg}"),
        other => panic!("Expected CheckpointError, got: {other:?}"),
    }
}
