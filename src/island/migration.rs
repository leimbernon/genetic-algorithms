use crate::configuration::ProblemSolving;
use crate::island::configuration::IslandConfiguration;
use crate::island::topology::neighbors;
use crate::population::Population;
use crate::traits::ChromosomeT;
use log::debug;

/// Performs migration between islands.
///
/// Selects the best `migration_count` individuals from each island and copies them
/// to the neighbor islands according to the configured topology. Migrants replace
/// the worst individuals in the destination island.
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

    // Collect migrants from each island (best M individuals)
    let mut all_migrants: Vec<Vec<U>> = Vec::with_capacity(num_islands);
    for island in islands.iter() {
        let migrants = select_best(island, config.migration_count, problem_solving);
        all_migrants.push(migrants);
    }

    // Distribute migrants to neighbors
    for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
        let neighbors = neighbors(source_idx, num_islands, &config.topology);
        for &dest_idx in &neighbors {
            let migrants = source_migrants.clone();
            replace_worst(&mut islands[dest_idx], &migrants, problem_solving);
            debug!(
                target: "island_events",
                "Migrated {} individuals from island {} to island {}",
                migrants.len(),
                source_idx,
                dest_idx
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::island::configuration::IslandConfiguration;
    use crate::island::topology::MigrationTopology;
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
        let island1_fitnesses: Vec<f64> = islands[1]
            .chromosomes
            .iter()
            .map(|c| c.fitness())
            .collect();
        assert!(
            island1_fitnesses.contains(&10.0),
            "Island 1 should contain migrated individual with fitness 10.0"
        );
    }
}
