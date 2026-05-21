use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::traits::ConfigurationT;

#[test]
fn test_island_ga_validate_no_init_fn() {
    let config = IslandConfiguration::new().with_num_islands(2);
    let ga_config = GaConfiguration::default();
    let island_ga: IslandGa<Binary> = IslandGa::new(config, ga_config);

    let result = island_ga.validate();
    assert!(result.is_err());
}

#[test]
fn test_island_ga_validate_zero_islands() {
    let config = IslandConfiguration::new().with_num_islands(0);
    let ga_config = GaConfiguration::default();
    let island_ga = IslandGa::<Binary>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_fitness_fn(|_| 0.0);

    let result = island_ga.validate();
    assert!(result.is_err());
}

#[test]
fn test_island_ga_validate_zero_migration_interval() {
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(0);
    let ga_config = GaConfiguration::default();
    let island_ga = IslandGa::<Binary>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_fitness_fn(|_| 0.0);

    let result = island_ga.validate();
    assert!(result.is_err());
}

#[test]
fn test_island_ga_heterogeneous_configs() {
    let config1 = GaConfiguration::default().with_population_size(20);
    let config2 = GaConfiguration::default().with_population_size(30);

    let island_config = IslandConfiguration::new().with_num_islands(2);
    let island_ga =
        IslandGa::<Binary>::with_heterogeneous_configs(island_config, vec![config1, config2])
            .with_initialization_fn(|_, _| vec![])
            .with_fitness_fn(|_| 0.0);

    assert_eq!(island_ga.ga_configs.len(), 2);
    assert_eq!(
        island_ga
            .config_for_island(0)
            .limit().population_size,
        20
    );
    assert_eq!(
        island_ga
            .config_for_island(1)
            .limit().population_size,
        30
    );
}

#[test]
fn test_island_ga_config_for_island_cycles_last() {
    let config = GaConfiguration::default().with_population_size(50);

    let island_config = IslandConfiguration::new().with_num_islands(4);
    let island_ga = IslandGa::<Binary>::new(island_config, config);

    // Only one config — all islands should get the same one
    for i in 0..4 {
        assert_eq!(
            island_ga
                .config_for_island(i)
                .limit().population_size,
            50
        );
    }
}

#[test]
fn test_island_ga_validate_empty_configs() {
    let island_config = IslandConfiguration::new().with_num_islands(2);
    let island_ga = IslandGa::<Binary>::with_heterogeneous_configs(island_config, vec![]);

    let result = island_ga.validate();
    assert!(result.is_err());
}
