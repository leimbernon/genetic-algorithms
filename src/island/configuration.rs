use crate::island::topology::MigrationTopology;

/// Policy for selecting migrants and replacing individuals in the destination island.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MigrationPolicy {
    /// The best individuals from the source replace the worst in the destination.
    #[default]
    BestReplaceWorst,
    /// Random individuals from the source replace the worst in the destination.
    RandomReplaceWorst,
    /// Migrants are selected via tournament from the source; they replace the worst
    /// in the destination.
    TournamentMigrant,
    /// Random individuals from the source replace random individuals in the destination.
    RandomReplaceRandom,
}

/// Configuration for the island model genetic algorithm.
///
/// Controls how multiple populations (islands) evolve independently and
/// exchange individuals through periodic migration.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::island::configuration::IslandConfiguration;
/// use genetic_algorithms::island::topology::MigrationTopology;
///
/// let config = IslandConfiguration::new()
///     .with_num_islands(4)
///     .with_migration_interval(10)
///     .with_migration_count(2)
///     .with_topology(MigrationTopology::Ring);
///
/// assert_eq!(config.num_islands, 4);
/// assert_eq!(config.migration_interval, 10);
/// assert_eq!(config.migration_count, 2);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IslandConfiguration {
    /// Number of islands (sub-populations).
    pub num_islands: usize,
    /// Migration occurs every `migration_interval` generations.
    pub migration_interval: usize,
    /// Number of individuals to migrate from each island per migration event.
    pub migration_count: usize,
    /// Topology governing which islands exchange individuals.
    pub topology: MigrationTopology,
    /// Policy for selecting migrants and placing them in destinations.
    pub migration_policy: MigrationPolicy,
}

impl Default for IslandConfiguration {
    fn default() -> Self {
        IslandConfiguration {
            num_islands: 4,
            migration_interval: 10,
            migration_count: 1,
            topology: MigrationTopology::Ring,
            migration_policy: MigrationPolicy::BestReplaceWorst,
        }
    }
}

impl IslandConfiguration {
    /// Creates a new `IslandConfiguration` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of islands.
    ///
    /// # Arguments
    ///
    /// * `num_islands` - Number of sub-populations. Must be > 0.
    pub fn with_num_islands(mut self, num_islands: usize) -> Self {
        self.num_islands = num_islands;
        self
    }

    /// Sets the migration interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - Number of generations between migration events. Must be > 0.
    pub fn with_migration_interval(mut self, interval: usize) -> Self {
        self.migration_interval = interval;
        self
    }

    /// Sets the number of migrants per migration event.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of best individuals to migrate from each island.
    pub fn with_migration_count(mut self, count: usize) -> Self {
        self.migration_count = count;
        self
    }

    /// Sets the migration topology.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology for inter-island migration.
    pub fn with_topology(mut self, topology: MigrationTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Sets the migration policy.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy for selecting migrants and replacing individuals.
    pub fn with_migration_policy(mut self, policy: MigrationPolicy) -> Self {
        self.migration_policy = policy;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
