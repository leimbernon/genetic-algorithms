//! Error types for the genetic algorithm library.
//!
//! All fallible operations in this crate return [`GaError`], a single enum
//! that covers configuration mistakes, operator failures, initialization
//! problems, and I/O issues (checkpoints). It implements [`std::error::Error`]
//! and [`std::fmt::Display`] for seamless integration with the `?` operator
//! and error-reporting crates.

use std::fmt;

/// Error type for all genetic algorithm operations.
///
/// This enum covers configuration errors, validation errors,
/// operator errors, and initialization errors.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GaError {
    /// A configuration parameter is invalid or missing.
    ConfigurationError(String),
    /// A validation check failed (e.g., DNA length mismatch, unique IDs).
    ValidationError(String),
    /// A crossover operation failed.
    CrossoverError(String),
    /// A mutation operation failed.
    MutationError(String),
    /// An initialization operation failed.
    InitializationError(String),
    /// A selection operation failed.
    SelectionError(String),
    /// An island model configuration parameter is invalid.
    InvalidIslandConfiguration(String),
    /// A niching / fitness sharing configuration parameter is invalid.
    InvalidNichingConfiguration(String),
    /// An NSGA-II configuration parameter is invalid.
    InvalidNsga2Configuration(String),
    /// An NSGA-III configuration parameter is invalid.
    InvalidNsga3Configuration(String),
    /// A MOEA/D configuration parameter is invalid.
    InvalidMoeaDConfiguration(String),
    /// A SPEA2 configuration parameter is invalid.
    InvalidSpea2Configuration(String),
    /// A migration operation between islands failed.
    MigrationError(String),
    /// A checkpoint save or load operation failed.
    CheckpointError(String),
}

impl fmt::Display for GaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GaError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            GaError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            GaError::CrossoverError(msg) => write!(f, "Crossover error: {}", msg),
            GaError::MutationError(msg) => write!(f, "Mutation error: {}", msg),
            GaError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            GaError::SelectionError(msg) => write!(f, "Selection error: {}", msg),
            GaError::InvalidIslandConfiguration(msg) => {
                write!(f, "Invalid island configuration: {}", msg)
            }
            GaError::InvalidNichingConfiguration(msg) => {
                write!(f, "Invalid niching configuration: {}", msg)
            }
            GaError::InvalidNsga2Configuration(msg) => {
                write!(f, "Invalid NSGA-II configuration: {}", msg)
            }
            GaError::InvalidNsga3Configuration(msg) => {
                write!(f, "Invalid NSGA-III configuration: {}", msg)
            }
            GaError::InvalidMoeaDConfiguration(msg) => {
                write!(f, "Invalid MOEA/D configuration: {}", msg)
            }
            GaError::InvalidSpea2Configuration(msg) => {
                write!(f, "Invalid SPEA2 configuration: {}", msg)
            }
            GaError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
            GaError::CheckpointError(msg) => write!(f, "Checkpoint error: {}", msg),
        }
    }
}

impl std::error::Error for GaError {}
