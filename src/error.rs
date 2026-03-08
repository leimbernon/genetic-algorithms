use std::fmt;

/// Error type for all genetic algorithm operations.
///
/// This enum covers configuration errors, validation errors,
/// operator errors, and initialization errors.
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_configuration_error() {
        let e = GaError::ConfigurationError("bad config".into());
        assert_eq!(e.to_string(), "Configuration error: bad config");
    }

    #[test]
    fn display_validation_error() {
        let e = GaError::ValidationError("invalid".into());
        assert_eq!(e.to_string(), "Validation error: invalid");
    }

    #[test]
    fn display_crossover_error() {
        let e = GaError::CrossoverError("cx fail".into());
        assert_eq!(e.to_string(), "Crossover error: cx fail");
    }

    #[test]
    fn display_mutation_error() {
        let e = GaError::MutationError("mut fail".into());
        assert_eq!(e.to_string(), "Mutation error: mut fail");
    }

    #[test]
    fn display_initialization_error() {
        let e = GaError::InitializationError("init fail".into());
        assert_eq!(e.to_string(), "Initialization error: init fail");
    }

    #[test]
    fn display_selection_error() {
        let e = GaError::SelectionError("sel fail".into());
        assert_eq!(e.to_string(), "Selection error: sel fail");
    }

    #[test]
    fn display_invalid_island_configuration() {
        let e = GaError::InvalidIslandConfiguration("island bad".into());
        assert_eq!(e.to_string(), "Invalid island configuration: island bad");
    }

    #[test]
    fn display_invalid_niching_configuration() {
        let e = GaError::InvalidNichingConfiguration("niche bad".into());
        assert_eq!(e.to_string(), "Invalid niching configuration: niche bad");
    }

    #[test]
    fn display_invalid_nsga2_configuration() {
        let e = GaError::InvalidNsga2Configuration("nsga bad".into());
        assert_eq!(e.to_string(), "Invalid NSGA-II configuration: nsga bad");
    }

    #[test]
    fn display_migration_error() {
        let e = GaError::MigrationError("mig fail".into());
        assert_eq!(e.to_string(), "Migration error: mig fail");
    }

    #[test]
    fn debug_format_contains_variant_name() {
        let e = GaError::ConfigurationError("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("ConfigurationError"), "got: {debug}");
    }

    #[test]
    fn implements_std_error_trait() {
        let e: Box<dyn std::error::Error> = Box::new(GaError::ValidationError("test".into()));
        // source() should be None (no chained error)
        assert!(e.source().is_none());
        // Display should still work via the trait object
        assert!(e.to_string().contains("Validation error"));
    }
}
