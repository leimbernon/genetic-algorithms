use genetic_algorithms::niching::configuration::NichingConfiguration;

#[test]
fn test_niching_configuration_default() {
    let config = NichingConfiguration::default();
    assert!(!config.enabled);
    assert!((config.sigma_share - 1.0).abs() < f64::EPSILON);
    assert!((config.alpha - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_niching_configuration_builder() {
    let config = NichingConfiguration::new()
        .with_enabled(true)
        .with_sigma_share(2.5)
        .with_alpha(0.5);

    assert!(config.enabled);
    assert!((config.sigma_share - 2.5).abs() < f64::EPSILON);
    assert!((config.alpha - 0.5).abs() < f64::EPSILON);
}
