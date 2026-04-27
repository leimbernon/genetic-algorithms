use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;

#[test]
fn test_nsga2_validate_no_init_fn() {
    let config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config);

    let result = nsga2.validate();
    assert!(result.is_err());
}

#[test]
fn test_nsga2_validate_zero_objectives() {
    let config = Nsga2Configuration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config).with_initialization_fn(|_, _, _| vec![]);

    let result = nsga2.validate();
    assert!(result.is_err());
}

#[test]
fn test_nsga2_validate_mismatched_objective_fns() {
    let config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);

    let result = nsga2.validate();
    assert!(result.is_err());
}

#[test]
fn test_nsga2_validate_population_too_small() {
    let config = Nsga2Configuration::new()
        .with_num_objectives(1)
        .with_population_size(1);
    let ga_config = GaConfiguration::default();
    let nsga2 = Nsga2Ga::<Binary>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);

    let result = nsga2.validate();
    assert!(result.is_err());
}
