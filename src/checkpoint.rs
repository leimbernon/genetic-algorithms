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