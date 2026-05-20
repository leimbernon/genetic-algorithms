#![cfg(feature = "serde")]

use genetic_algorithms::checkpoint::{load_checkpoint, save_checkpoint, Checkpoint};
use genetic_algorithms::chromosomes;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::genotypes;
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;
use std::path::Path;

/// Helper: create a Binary chromosome with the given fitness.
fn make_binary_chromosome(bits: &[bool], fitness: f64) -> chromosomes::Binary {
    let genes: Vec<genotypes::Binary> = bits
        .iter()
        .enumerate()
        .map(|(i, &b)| genotypes::Binary {
            id: i as i32,
            value: b,
        })
        .collect();
    let mut c = <chromosomes::Binary as Default>::default();
    c.set_dna(Cow::Owned(genes));
    c.set_fitness(fitness);
    c
}

#[test]
fn save_and_load_checkpoint_round_trip() {
    let dir = std::env::temp_dir().join("ga_test_checkpoint_rt");
    let _ = std::fs::remove_dir_all(&dir);

    let c1 = make_binary_chromosome(&[true, false], 1.0);
    let c2 = make_binary_chromosome(&[false, true], 2.0);
    let mut pop = Population::new(vec![c1.clone(), c2]);
    // Set the best chromosome so we don't have NaN fitness in the serialized output
    pop.decide_best_chromosome(&c1, ProblemSolving::Maximization);

    let config = GaConfiguration::default();
    let stats = vec![GenerationStats::from_fitness_values(0, &[1.0, 2.0], false)];

    let ckpt = Checkpoint {
        population: pop,
        configuration: config.clone(),
        generation: 5,
        stats: stats.clone(),
    };

    let path = dir.join("checkpoint.json");
    save_checkpoint(&ckpt, &path).expect("save should succeed");
    assert!(path.exists());

    let loaded: Checkpoint<chromosomes::Binary> =
        load_checkpoint(&path).expect("load should succeed");

    assert_eq!(loaded.generation, 5);
    assert_eq!(loaded.population.size(), 2);
    assert_eq!(loaded.configuration, config);
    assert_eq!(loaded.stats.len(), 1);
    assert!((loaded.stats[0].avg_fitness - stats[0].avg_fitness).abs() < 1e-10);

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_checkpoint_missing_file_returns_error() {
    let result: Result<Checkpoint<chromosomes::Binary>, _> =
        load_checkpoint(Path::new("/tmp/ga_test_nonexistent_checkpoint.json"));
    assert!(result.is_err());
    match result.unwrap_err() {
        genetic_algorithms::error::GaError::CheckpointError(msg) => {
            assert!(msg.contains("Failed to read"), "got: {msg}");
        }
        other => panic!("Expected CheckpointError, got: {other:?}"),
    }
}

#[test]
fn save_checkpoint_creates_parent_directories() {
    let dir = std::env::temp_dir().join("ga_test_ckpt_nested/a/b/c");
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("ga_test_ckpt_nested"));

    let ckpt = Checkpoint {
        population: Population::<chromosomes::Binary>::new_empty(),
        configuration: GaConfiguration::default(),
        generation: 0,
        stats: vec![],
    };

    let path = dir.join("ckpt.json");
    save_checkpoint(&ckpt, &path).expect("save should create dirs");
    assert!(path.exists());

    // Cleanup
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("ga_test_ckpt_nested"));
}
