use genetic_algorithms::error::GaError;

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
fn display_checkpoint_error() {
    let e = GaError::CheckpointError("ckpt fail".into());
    assert_eq!(e.to_string(), "Checkpoint error: ckpt fail");
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

#[test]
fn internal_error_constructs() {
    let e = GaError::InternalError("boom".to_string());
    let _ = e; // must compile
}

#[test]
fn internal_error_display() {
    let e = GaError::InternalError("boom".to_string());
    assert_eq!(e.to_string(), "Internal error: boom");
}

#[test]
fn internal_error_derives_clone_partial_eq() {
    let e1 = GaError::InternalError("x".to_string());
    let e2 = e1.clone();
    assert_eq!(e1, e2);
}

#[test]
fn internal_error_debug_contains_variant_name() {
    let e = GaError::InternalError("test".to_string());
    let debug = format!("{:?}", e);
    assert!(debug.contains("InternalError"), "got: {debug}");
}
