use std::fmt;

/// Error type for all genetic algorithm operations.
///
/// This enum covers configuration errors, validation errors,
/// operator errors, and initialization errors.
#[derive(Debug)]
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
        }
    }
}

impl std::error::Error for GaError {}
