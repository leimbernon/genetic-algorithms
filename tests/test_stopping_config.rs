use genetic_algorithms::ga::Ga;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, LinearChromosome, SelectionConfig, StoppingConfig,
};

mod structures;

/// Tests that `.with_stagnation_limit(n)` sets `stagnation_generations` to `Some(n)`.
#[test]
fn test_stopping_config_with_stagnation_limit() {
    use structures::{Chromosome, Gene};
    let ga = Ga::<Chromosome>::new()
        .with_population_size(10)
        .with_number_of_couples(4)
        .with_max_generations(10)
        .with_stagnation_limit(50);

    assert_eq!(
        ga.configuration().stagnation_generations(),
        Some(50),
        "stagnation_generations should be Some(50) after with_stagnation_limit(50)"
    );
}

/// Tests that `.with_convergence_threshold(t)` sets `convergence_threshold` to `Some(t)`.
#[test]
fn test_stopping_config_with_convergence_threshold() {
    use structures::{Chromosome, Gene};
    let ga = Ga::<Chromosome>::new()
        .with_population_size(10)
        .with_number_of_couples(4)
        .with_max_generations(10)
        .with_convergence_threshold(0.001);

    assert_eq!(
        ga.configuration().convergence_threshold(),
        Some(0.001),
        "convergence_threshold should be Some(0.001) after with_convergence_threshold(0.001)"
    );
}

/// Tests that `.with_max_duration_secs(s)` sets `max_duration_secs` to `Some(s)`.
///
/// The field exists on all targets; only the usage site in ga.rs is wasm-gated.
#[test]
fn test_stopping_config_with_max_duration_secs() {
    use structures::{Chromosome, Gene};
    let ga = Ga::<Chromosome>::new()
        .with_population_size(10)
        .with_number_of_couples(4)
        .with_max_generations(10)
        .with_max_duration_secs(10.0);

    assert_eq!(
        ga.configuration().max_duration_secs(),
        Some(10.0),
        "max_duration_secs should be Some(10.0) after with_max_duration_secs(10.0)"
    );
}

/// Tests that a freshly defaulted GaConfiguration has all three stopping fields as `None`.
#[test]
fn test_stopping_config_default_is_none() {
    use structures::{Chromosome, Gene};
    let ga = Ga::<Chromosome>::new()
        .with_population_size(10)
        .with_number_of_couples(4)
        .with_max_generations(10);

    assert_eq!(
        ga.configuration().stagnation_generations(),
        None,
        "stagnation_generations should be None by default"
    );
    assert_eq!(
        ga.configuration().convergence_threshold(),
        None,
        "convergence_threshold should be None by default"
    );
    assert_eq!(
        ga.configuration().max_duration_secs(),
        None,
        "max_duration_secs should be None by default"
    );
}
