//! Error — GaError enum and Result type alias for the crate.
//!
//! All fallible operations in this crate return [`GaError`], a single enum
//! that covers configuration mistakes, operator failures, initialization
//! problems, and I/O issues (checkpoints). It implements [`std::error::Error`]
//! and [`std::fmt::Display`] for seamless integration with the `?` operator
//! and error-reporting crates.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`GaError`] | Unified error enum covering all crate failure modes |
//! | `GaResult<T>` | Alias for `Result<T, GaError>` |
//!
//! # When to use
//! All engine methods and operator calls return `GaResult<T>`. Match on
//! [`GaError`] variants to handle specific failure modes in application
//! code.

use std::fmt;

/// Error type for all genetic algorithm operations.
///
/// This enum covers configuration errors, validation errors,
/// operator errors, and initialization errors.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::error::GaError;
///
/// let err = GaError::ConfigurationError("population_size must be > 0".to_string());
/// assert!(err.to_string().contains("population_size"));
///
/// // GaError implements std::error::Error — use it with the ? operator:
/// fn check(ok: bool) -> Result<(), GaError> {
///     if ok { Ok(()) }
///     else { Err(GaError::ValidationError("check failed".to_string())) }
/// }
/// assert!(check(true).is_ok());
/// assert!(check(false).is_err());
/// ```
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
    /// An SMS-EMOA configuration parameter is invalid.
    InvalidSmsEmoaConfiguration(String),
    /// An IBEA configuration parameter is invalid.
    InvalidIbeaConfiguration(String),
    /// A constraint processing configuration parameter is invalid.
    InvalidConstraintConfiguration(String),
    /// An indicator configuration parameter is invalid.
    InvalidIndicatorConfiguration(String),
    /// A migration operation between islands failed.
    MigrationError(String),
    /// A checkpoint save or load operation failed.
    CheckpointError(String),
    /// A local search operation failed.
    LocalSearchError(String),
    /// A tree exceeded the configured maximum depth limit.
    TreeDepthExceeded(String),
    /// A tree exceeded the configured maximum node count limit.
    TreeSizeExceeded(String),
    /// An internal invariant was violated — for example, a mutex was poisoned by a
    /// panicking thread. This is not a user configuration error; it signals that the
    /// GA runtime itself encountered an unrecoverable internal state. Callers should
    /// treat this as a fatal run-time fault and propagate it rather than swallowing it.
    InternalError(String),
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
            GaError::InvalidSmsEmoaConfiguration(msg) => {
                write!(f, "Invalid SMS-EMOA configuration: {}", msg)
            }
            GaError::InvalidIbeaConfiguration(msg) => {
                write!(f, "Invalid IBEA configuration: {}", msg)
            }
            GaError::InvalidConstraintConfiguration(msg) => {
                write!(f, "Invalid constraint configuration: {}", msg)
            }
            GaError::InvalidIndicatorConfiguration(msg) => {
                write!(f, "Invalid indicator configuration: {}", msg)
            }
            GaError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
            GaError::CheckpointError(msg) => write!(f, "Checkpoint error: {}", msg),
            GaError::LocalSearchError(msg) => write!(f, "Local search error: {}", msg),
            GaError::TreeDepthExceeded(msg) => write!(f, "Tree depth exceeded: {}", msg),
            GaError::TreeSizeExceeded(msg) => write!(f, "Tree size exceeded: {}", msg),
            GaError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for GaError {}
