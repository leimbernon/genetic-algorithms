use genetic_algorithms::island::configuration::{IslandConfiguration, MigrationPolicy};
use genetic_algorithms::island::topology::MigrationTopology;

#[test]
fn test_island_configuration_default() {
    let config = IslandConfiguration::default();
    assert_eq!(config.num_islands, 4);
    assert_eq!(config.migration_interval, 10);
    assert_eq!(config.migration_count, 1);
    assert_eq!(config.topology, MigrationTopology::Ring);
    assert_eq!(config.migration_policy, MigrationPolicy::BestReplaceWorst);
}

#[test]
fn test_island_configuration_builder() {
    let config = IslandConfiguration::new()
        .with_num_islands(8)
        .with_migration_interval(20)
        .with_migration_count(3)
        .with_topology(MigrationTopology::FullyConnected)
        .with_migration_policy(MigrationPolicy::RandomReplaceWorst);

    assert_eq!(config.num_islands, 8);
    assert_eq!(config.migration_interval, 20);
    assert_eq!(config.migration_count, 3);
    assert_eq!(config.topology, MigrationTopology::FullyConnected);
    assert_eq!(config.migration_policy, MigrationPolicy::RandomReplaceWorst);
}

#[test]
fn test_island_configuration_migration_policy_variants() {
    let config =
        IslandConfiguration::new().with_migration_policy(MigrationPolicy::TournamentMigrant);
    assert_eq!(config.migration_policy, MigrationPolicy::TournamentMigrant);

    let config =
        IslandConfiguration::new().with_migration_policy(MigrationPolicy::RandomReplaceRandom);
    assert_eq!(
        config.migration_policy,
        MigrationPolicy::RandomReplaceRandom
    );
}
