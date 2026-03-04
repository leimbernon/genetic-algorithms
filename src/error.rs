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
    /// An island model configuration parameter is invalid.
    InvalidIslandConfiguration(String),
    /// A niching / fitness sharing configuration parameter is invalid.
    InvalidNichingConfiguration(String),
    /// An NSGA-II configuration parameter is invalid.
    InvalidNsga2Configuration(String),
    /// A migration operation between islands failed.
    MigrationError(String),
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
            GaError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
        }
    }
}

impl std::error::Error for GaError {}
