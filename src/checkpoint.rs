//! Checkpoint save / load support for resuming GA runs.
//!
//! This module is only available when the `serde` feature is enabled.
//! It provides [`Checkpoint`], a serializable snapshot of the GA state
//! at a given generation, along with [`save_checkpoint`] and
//! [`load_checkpoint`] helpers that write/read JSON to disk.
//!
//! The GA run loop automatically saves checkpoints at the interval
//! configured in [`SaveProgressConfiguration`](crate::configuration::SaveProgressConfiguration).

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::population::Population;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A serializable snapshot of the GA state at a given generation.
///
/// Contains the population, configuration, generation index, and
/// accumulated per-generation statistics. The fitness function and
/// initialization function are **not** included because they are not
/// serializable — the caller must re-attach them after loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "U: Serialize", deserialize = "U: Deserialize<'de>"))]
pub struct Checkpoint<U>
where
    U: ChromosomeT,
{
    /// The population at the time of the checkpoint.
    pub population: Population<U>,
    /// The GA configuration at the time of the checkpoint.
    pub configuration: GaConfiguration,
    /// The generation index (0-based) when this checkpoint was created.
    pub generation: usize,
    /// Per-generation statistics accumulated up to this generation.
    pub stats: Vec<GenerationStats>,
}

/// Saves a [`Checkpoint`] to disk as pretty-printed JSON.
///
/// Creates any missing parent directories before writing.
///
/// # Errors
///
/// Returns [`GaError::CheckpointError`] if directory creation, file writing,
/// or JSON serialization fails.
pub fn save_checkpoint<U>(checkpoint: &Checkpoint<U>, path: &Path) -> Result<(), GaError>
where
    U: ChromosomeT + Serialize,
{
    // Ensure parent directories exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            GaError::CheckpointError(format!(
                "Failed to create checkpoint directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    let json = serde_json::to_string_pretty(checkpoint)
        .map_err(|e| GaError::CheckpointError(format!("Failed to serialize checkpoint: {}", e)))?;

    std::fs::write(path, json).map_err(|e| {
        GaError::CheckpointError(format!(
            "Failed to write checkpoint to '{}': {}",
            path.display(),
            e
        ))
    })?;

    Ok(())
}

/// Loads a [`Checkpoint`] from a JSON file on disk.
///
/// # Errors
///
/// Returns [`GaError::CheckpointError`] if the file cannot be read or
/// the JSON cannot be deserialized into a `Checkpoint<U>`.
pub fn load_checkpoint<U>(path: &Path) -> Result<Checkpoint<U>, GaError>
where
    U: ChromosomeT + for<'de> Deserialize<'de>,
{
    let json = std::fs::read_to_string(path).map_err(|e| {
        GaError::CheckpointError(format!(
            "Failed to read checkpoint from '{}': {}",
            path.display(),
            e
        ))
    })?;

    let checkpoint: Checkpoint<U> = serde_json::from_str(&json).map_err(|e| {
        GaError::CheckpointError(format!("Failed to deserialize checkpoint: {}", e))
    })?;

    Ok(checkpoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes;
    use crate::genotypes;
    use std::borrow::Cow;

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
        pop.best_chromosome = c1;
        pop.best_chromosome_is_set = true;

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
            GaError::CheckpointError(msg) => {
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
}
