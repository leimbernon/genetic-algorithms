use crate::configuration::ProblemSolving;
use crate::island::configuration::{IslandConfiguration, MigrationPolicy};
use crate::island::topology::neighbors;
use crate::nsga2::pareto::ParetoIndividual;
use crate::population::Population;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;

/// Performs migration between islands.
///
/// Selects migrants from each island and distributes them to neighbor islands
/// according to the configured topology and migration policy.
///
/// # Arguments
///
/// * `islands` - Mutable slice of populations representing the islands.
/// * `config` - Island configuration with topology and migration parameters.
/// * `problem_solving` - Whether we are minimizing or maximizing (determines "best").
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError)` if migration fails.
///
/// # Errors
///
/// Returns `GaError::MigrationError` if an island is empty or migration count exceeds
/// population size.
pub fn migrate<U>(
    islands: &mut [Population<U>],
    config: &IslandConfiguration,
    problem_solving: ProblemSolving,
) -> Result<(), crate::error::GaError>
where
    U: ChromosomeT,
{
    let num_islands = islands.len();
    if num_islands <= 1 {
        return Ok(());
    }

    for island in islands.iter() {
        if island.size() == 0 {
            return Err(crate::error::GaError::MigrationError(
                "Cannot migrate from an empty island".to_string(),
            ));
        }
        if config.migration_count > island.size() {
            return Err(crate::error::GaError::MigrationError(format!(
                "Migration count ({}) exceeds island population size ({})",
                config.migration_count,
                island.size()
            )));
        }
    }

    // Collect migrants from each island based on policy
    let mut rng = rand::rng();
    let mut all_migrants: Vec<Vec<U>> = Vec::with_capacity(num_islands);
    for island in islands.iter() {
        let migrants = match config.migration_policy {
            MigrationPolicy::BestReplaceWorst => {
                select_best(island, config.migration_count, problem_solving)
            }
            MigrationPolicy::RandomReplaceWorst | MigrationPolicy::RandomReplaceRandom => {
                select_random(island, config.migration_count, &mut rng)
            }
            MigrationPolicy::TournamentMigrant => {
                select_tournament(island, config.migration_count, problem_solving, &mut rng)
            }
        };
        all_migrants.push(migrants);
    }

    // Distribute migrants to neighbors
    for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
        let neighbors = neighbors(source_idx, num_islands, &config.topology);
        for &dest_idx in &neighbors {
            let migrants = source_migrants.clone();
            match config.migration_policy {
                MigrationPolicy::BestReplaceWorst
                | MigrationPolicy::RandomReplaceWorst
                | MigrationPolicy::TournamentMigrant => {
                    replace_worst(&mut islands[dest_idx], &migrants, problem_solving);
                }
                MigrationPolicy::RandomReplaceRandom => {
                    replace_random(&mut islands[dest_idx], &migrants, &mut rng);
                }
            }
            debug!(
                target: "island_events",
                "Migrated {} individuals from island {} to island {} (policy={:?})",
                migrants.len(),
                source_idx,
                dest_idx,
                config.migration_policy
            );
        }
    }

    Ok(())
}

/// Selects the best `count` individuals from a population.
fn select_best<U>(
    population: &Population<U>,
    count: usize,
    problem_solving: ProblemSolving,
) -> Vec<U>
where
    U: ChromosomeT,
{
    let mut indices: Vec<usize> = (0..population.size()).collect();
    indices.sort_by(|&a, &b| {
        let fa = population.chromosomes[a].fitness();
        let fb = population.chromosomes[b].fitness();
        match problem_solving {
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            }
            ProblemSolving::Maximization => {
                fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });
    indices
        .into_iter()
        .take(count)
        .map(|i| population.chromosomes[i].clone())
        .collect()
}

/// Selects `count` random individuals from a population.
fn select_random<U>(population: &Population<U>, count: usize, rng: &mut impl Rng) -> Vec<U>
where
    U: ChromosomeT,
{
    let n = population.size();
    (0..count)
        .map(|_| {
            let idx = rng.random_range(0..n);
            population.chromosomes[idx].clone()
        })
        .collect()
}

/// Selects `count` individuals from a population via binary tournament.
fn select_tournament<U>(
    population: &Population<U>,
    count: usize,
    problem_solving: ProblemSolving,
    rng: &mut impl Rng,
) -> Vec<U>
where
    U: ChromosomeT,
{
    let n = population.size();
    (0..count)
        .map(|_| {
            let i = rng.random_range(0..n);
            let j = rng.random_range(0..n);
            let fi = population.chromosomes[i].fitness();
            let fj = population.chromosomes[j].fitness();
            let winner = match problem_solving {
                ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                    if fi <= fj {
                        i
                    } else {
                        j
                    }
                }
                ProblemSolving::Maximization => {
                    if fi >= fj {
                        i
                    } else {
                        j
                    }
                }
            };
            population.chromosomes[winner].clone()
        })
        .collect()
}

/// Replaces the worst `migrants.len()` individuals in the population with the migrants.
fn replace_worst<U>(population: &mut Population<U>, migrants: &[U], problem_solving: ProblemSolving)
where
    U: ChromosomeT,
{
    if migrants.is_empty() || population.size() == 0 {
        return;
    }

    let mut indices: Vec<usize> = (0..population.size()).collect();
    // Sort by worst first
    indices.sort_by(|&a, &b| {
        let fa = population.chromosomes[a].fitness();
        let fb = population.chromosomes[b].fitness();
        match problem_solving {
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
            }
            ProblemSolving::Maximization => {
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });

    let replace_count = migrants.len().min(population.size());
    for (m_idx, &worst_idx) in indices.iter().take(replace_count).enumerate() {
        population.chromosomes[worst_idx] = migrants[m_idx].clone();
    }
}

/// Replaces `migrants.len()` random individuals in the population with the migrants.
fn replace_random<U>(population: &mut Population<U>, migrants: &[U], rng: &mut impl Rng)
where
    U: ChromosomeT,
{
    if migrants.is_empty() || population.size() == 0 {
        return;
    }

    let n = population.size();
    let replace_count = migrants.len().min(n);
    for (m_idx, _) in migrants.iter().enumerate().take(replace_count) {
        let target = rng.random_range(0..n);
        population.chromosomes[target] = migrants[m_idx].clone();
    }
}

/// Performs Pareto-aware migration between islands of `ParetoIndividual`s.
///
/// Selects the best `migration_count` individuals from each island (lowest rank,
/// breaking ties by highest crowding distance) and copies them to neighbor islands
/// according to the configured topology. Migrants replace the worst individuals
/// (highest rank, breaking ties by lowest crowding distance) in the destination.
///
/// # Arguments
///
/// * `islands` - Mutable slice of island populations (each a `Vec<ParetoIndividual<U>>`).
/// * `config` - Island configuration with topology and migration parameters.
///
/// # Errors
///
/// Returns `GaError::MigrationError` if an island is empty or migration count exceeds
/// the island population size.
pub fn migrate_pareto<U>(
    islands: &mut [Vec<ParetoIndividual<U>>],
    config: &IslandConfiguration,
) -> Result<(), crate::error::GaError>
where
    U: ChromosomeT,
{
    let num_islands = islands.len();
    if num_islands <= 1 {
        return Ok(());
    }

    for island in islands.iter() {
        if island.is_empty() {
            return Err(crate::error::GaError::MigrationError(
                "Cannot migrate from an empty island".to_string(),
            ));
        }
        if config.migration_count > island.len() {
            return Err(crate::error::GaError::MigrationError(format!(
                "Migration count ({}) exceeds island population size ({})",
                config.migration_count,
                island.len()
            )));
        }
    }

    // Collect migrants from each island (best M by Pareto rank / crowding distance)
    let mut all_migrants: Vec<Vec<ParetoIndividual<U>>> = Vec::with_capacity(num_islands);
    for island in islands.iter() {
        let migrants = select_best_pareto(island, config.migration_count);
        all_migrants.push(migrants);
    }

    // Distribute migrants to neighbors
    for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
        let dest_indices = neighbors(source_idx, num_islands, &config.topology);
        for &dest_idx in &dest_indices {
            let migrants = source_migrants.clone();
            replace_worst_pareto(&mut islands[dest_idx], &migrants);
            debug!(
                target: "island_events",
                "Pareto migration: {} individuals from island {} to island {}",
                migrants.len(),
                source_idx,
                dest_idx
            );
        }
    }

    Ok(())
}

/// Selects the best `count` Pareto individuals from an island.
///
/// "Best" means lowest rank first, then highest crowding distance to break ties.
fn select_best_pareto<U>(island: &[ParetoIndividual<U>], count: usize) -> Vec<ParetoIndividual<U>>
where
    U: ChromosomeT,
{
    let mut indices: Vec<usize> = (0..island.len()).collect();
    indices.sort_by(|&a, &b| {
        island[a].rank.cmp(&island[b].rank).then_with(|| {
            island[b]
                .crowding_distance
                .partial_cmp(&island[a].crowding_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    indices
        .into_iter()
        .take(count)
        .map(|i| island[i].clone())
        .collect()
}

/// Replaces the worst `migrants.len()` Pareto individuals in an island with the migrants.
///
/// "Worst" means highest rank first, then lowest crowding distance to break ties.
fn replace_worst_pareto<U>(island: &mut [ParetoIndividual<U>], migrants: &[ParetoIndividual<U>])
where
    U: ChromosomeT,
{
    if migrants.is_empty() || island.is_empty() {
        return;
    }

    let mut indices: Vec<usize> = (0..island.len()).collect();
    // Sort by worst first: highest rank, then lowest crowding distance
    indices.sort_by(|&a, &b| {
        island[b].rank.cmp(&island[a].rank).then_with(|| {
            island[a]
                .crowding_distance
                .partial_cmp(&island[b].crowding_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let replace_count = migrants.len().min(island.len());
    for (m_idx, &worst_idx) in indices.iter().take(replace_count).enumerate() {
        island[worst_idx] = migrants[m_idx].clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::island::configuration::IslandConfiguration;
    use crate::island::topology::MigrationTopology;
    use crate::nsga2::pareto::ParetoIndividual;
    use crate::population::Population;
    use crate::traits::ChromosomeT;
    use std::borrow::Cow;

    /// Simple test gene satisfying `GeneT`.
    #[derive(Debug, Copy, Clone, Default, PartialEq)]
    struct TestGene {
        id: i32,
    }

    impl crate::traits::GeneT for TestGene {
        fn id(&self) -> i32 {
            self.id
        }
        fn set_id(&mut self, id: i32) -> &mut Self {
            self.id = id;
            self
        }
    }

    /// Simple test chromosome for migration tests.
    #[derive(Debug, Clone, Default, PartialEq)]
    struct MigrationTestChromosome {
        dna: Vec<TestGene>,
        fitness: f64,
        age: usize,
    }

    impl ChromosomeT for MigrationTestChromosome {
        type Gene = TestGene;

        fn dna(&self) -> &[Self::Gene] {
            &self.dna
        }

        fn dna_mut(&mut self) -> &mut [Self::Gene] {
            &mut self.dna
        }

        fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
            self.dna = dna.into_owned();
            self
        }

        fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
        where
            F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
        {
            self
        }

        fn calculate_fitness(&mut self) {}

        fn fitness(&self) -> f64 {
            self.fitness
        }

        fn set_fitness(&mut self, fitness: f64) -> &mut Self {
            self.fitness = fitness;
            self
        }

        fn set_age(&mut self, age: usize) -> &mut Self {
            self.age = age;
            self
        }

        fn age(&self) -> usize {
            self.age
        }
    }

    fn make_population(fitnesses: &[f64]) -> Population<MigrationTestChromosome> {
        let chromosomes: Vec<MigrationTestChromosome> = fitnesses
            .iter()
            .map(|&f| MigrationTestChromosome {
                dna: vec![],
                fitness: f,
                age: 0,
            })
            .collect();
        Population::new(chromosomes)
    }

    #[test]
    fn test_migrate_single_island_is_noop() {
        let mut islands = vec![make_population(&[1.0, 2.0, 3.0])];
        let config = IslandConfiguration::new()
            .with_num_islands(1)
            .with_migration_count(1);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_empty_island_returns_error() {
        let mut islands = vec![make_population(&[1.0, 2.0]), Population::new_empty()];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_err());
    }

    #[test]
    fn test_migrate_count_exceeds_size_returns_error() {
        let mut islands = vec![make_population(&[1.0]), make_population(&[2.0])];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(5);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_err());
    }

    #[test]
    fn test_migrate_ring_topology_transfers_best() {
        // Island 0: fitnesses [10.0, 20.0, 30.0] (best for min = 10.0)
        // Island 1: fitnesses [100.0, 200.0, 300.0] (worst for min = 300.0)
        // With ring: island 0 -> island 1, island 1 -> island 0
        let mut islands = vec![
            make_population(&[10.0, 20.0, 30.0]),
            make_population(&[100.0, 200.0, 300.0]),
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_ok());

        // After migration, island 1 should have received the best from island 0 (fitness 10.0)
        // It should replace the worst in island 1 (fitness 300.0)
        let island1_fitnesses: Vec<f64> =
            islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
        assert!(
            island1_fitnesses.contains(&10.0),
            "Island 1 should contain migrated individual with fitness 10.0"
        );
    }

    // ---- Pareto migration tests ----

    fn make_pareto_individual(
        rank: usize,
        crowding: f64,
    ) -> ParetoIndividual<MigrationTestChromosome> {
        let mut ind = ParetoIndividual::new(
            MigrationTestChromosome {
                dna: vec![],
                fitness: 0.0,
                age: 0,
            },
            vec![],
        );
        ind.rank = rank;
        ind.crowding_distance = crowding;
        ind
    }

    #[test]
    fn test_migrate_pareto_single_island_noop() {
        let mut islands = vec![vec![make_pareto_individual(0, 1.0)]];
        let config = IslandConfiguration::new()
            .with_num_islands(1)
            .with_migration_count(1);

        let result = migrate_pareto(&mut islands, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_pareto_empty_island_error() {
        let mut islands: Vec<Vec<ParetoIndividual<MigrationTestChromosome>>> =
            vec![vec![make_pareto_individual(0, 1.0)], vec![]];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1);

        let result = migrate_pareto(&mut islands, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_migrate_pareto_count_exceeds_size_error() {
        let mut islands = vec![
            vec![make_pareto_individual(0, 1.0)],
            vec![make_pareto_individual(1, 0.5)],
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(5);

        let result = migrate_pareto(&mut islands, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_migrate_pareto_ring_transfers_best() {
        // Island 0: rank 0 (best) individuals
        // Island 1: rank 2 (worst) individuals
        let mut islands = vec![
            vec![
                make_pareto_individual(0, 2.0), // best: rank 0, high crowding
                make_pareto_individual(1, 1.0),
                make_pareto_individual(2, 0.5),
            ],
            vec![
                make_pareto_individual(2, 0.1), // worst
                make_pareto_individual(2, 0.2),
                make_pareto_individual(2, 0.3),
            ],
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring);

        let result = migrate_pareto(&mut islands, &config);
        assert!(result.is_ok());

        // After migration, island 1 should now contain an individual with rank 0
        let has_rank_0 = islands[1].iter().any(|ind| ind.rank == 0);
        assert!(
            has_rank_0,
            "Island 1 should contain a migrated rank-0 individual"
        );
    }

    #[test]
    fn test_migrate_pareto_replaces_worst() {
        // Island 0: one rank-0 individual with high crowding
        // Island 1: three individuals with varying ranks
        let mut islands = vec![
            vec![
                make_pareto_individual(0, 10.0),
                make_pareto_individual(0, 5.0),
                make_pareto_individual(1, 1.0),
            ],
            vec![
                make_pareto_individual(0, 3.0),  // should survive
                make_pareto_individual(1, 2.0),  // should survive
                make_pareto_individual(3, 0.01), // worst — should be replaced
            ],
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring);

        let result = migrate_pareto(&mut islands, &config);
        assert!(result.is_ok());

        // The worst individual (rank 3) in island 1 should have been replaced
        let has_rank_3 = islands[1].iter().any(|ind| ind.rank == 3);
        assert!(
            !has_rank_3,
            "Island 1 should no longer contain the rank-3 individual"
        );
    }

    // ---- Migration policy tests ----

    #[test]
    fn test_migrate_random_replace_worst_policy() {
        let mut islands = vec![
            make_population(&[10.0, 20.0, 30.0]),
            make_population(&[100.0, 200.0, 300.0]),
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring)
            .with_migration_policy(MigrationPolicy::RandomReplaceWorst);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_ok());

        // Island 1 should no longer have 300.0 (worst) — it should be replaced
        // by a random migrant from island 0
        let island1_fitnesses: Vec<f64> =
            islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
        let has_original_worst = island1_fitnesses.contains(&300.0);
        let has_migrant = island1_fitnesses
            .iter()
            .any(|f| [10.0, 20.0, 30.0].contains(f));
        // The worst should be replaced (since there's only 1 migrant, 300.0 is replaced)
        assert!(
            !has_original_worst || has_migrant,
            "Worst individual should be replaced or a migrant should be present"
        );
    }

    #[test]
    fn test_migrate_random_replace_random_policy() {
        let mut islands = vec![
            make_population(&[10.0, 20.0, 30.0]),
            make_population(&[100.0, 200.0, 300.0]),
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring)
            .with_migration_policy(MigrationPolicy::RandomReplaceRandom);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_ok());

        // After migration, island 1 should contain at least one individual
        // from island 0 (the migrant replaces a random individual)
        let island1_fitnesses: Vec<f64> =
            islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
        let has_migrant = island1_fitnesses
            .iter()
            .any(|f| [10.0, 20.0, 30.0].contains(f));
        assert!(
            has_migrant,
            "Island 1 should contain a migrant from island 0"
        );
    }

    #[test]
    fn test_migrate_tournament_policy() {
        let mut islands = vec![
            make_population(&[10.0, 20.0, 30.0]),
            make_population(&[100.0, 200.0, 300.0]),
        ];
        let config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(1)
            .with_topology(MigrationTopology::Ring)
            .with_migration_policy(MigrationPolicy::TournamentMigrant);

        let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
        assert!(result.is_ok());

        // After migration, island 1 should have a migrant from island 0
        let island1_fitnesses: Vec<f64> =
            islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
        let has_migrant = island1_fitnesses
            .iter()
            .any(|f| [10.0, 20.0, 30.0].contains(f));
        assert!(
            has_migrant,
            "Island 1 should contain a tournament-selected migrant"
        );
    }
}
