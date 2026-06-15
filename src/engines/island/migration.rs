use crate::configuration::ProblemSolving;
use crate::island::configuration::{IslandConfiguration, MigrationPolicy};
use crate::island::topology::neighbors;
use crate::nsga2::pareto::ParetoIndividual;
use crate::population::Population;
use crate::traits::ChromosomeT;
use rand::Rng;
use std::sync::Arc;

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
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::island::migration::migrate;
/// use genetic_algorithms::island::configuration::IslandConfiguration;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::population::Population;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let config = IslandConfiguration::new().with_migration_count(1);
/// let mut islands: Vec<Population<RangeChromosome<f64>>> = vec![
///     Population::new(vec![RangeChromosome::default()]),
///     Population::new(vec![RangeChromosome::default()]),
/// ];
/// let _ = migrate(&mut islands, &config, ProblemSolving::Minimization);
/// ```
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
    let mut rng = crate::rng::make_rng();
    let mut all_migrants: Vec<Arc<Vec<U>>> = Vec::with_capacity(num_islands);
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
        all_migrants.push(Arc::new(migrants));
    }

    // Distribute migrants to neighbors
    for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
        let neighbor_list = neighbors(source_idx, num_islands, &config.topology);
        for &dest_idx in &neighbor_list {
            match config.migration_policy {
                MigrationPolicy::BestReplaceWorst
                | MigrationPolicy::RandomReplaceWorst
                | MigrationPolicy::TournamentMigrant => {
                    replace_worst(&mut islands[dest_idx], source_migrants, problem_solving);
                }
                MigrationPolicy::RandomReplaceRandom => {
                    replace_random(&mut islands[dest_idx], source_migrants, &mut rng);
                }
            }
            crate::log_debug!(
                target: "island_events",
                "Migrated {} individuals from island {} to island {} (policy={:?})",
                source_migrants.len(),
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
    let k = count.min(population.size());
    if k == 0 {
        return Vec::new();
    }
    indices.select_nth_unstable_by(k - 1, |&a, &b| {
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
    indices.truncate(k);
    indices
        .into_iter()
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

    let replace_count = migrants.len().min(population.size());
    if replace_count == 0 {
        return;
    }
    let mut indices: Vec<usize> = (0..population.size()).collect();
    // Partition by worst first
    indices.select_nth_unstable_by(replace_count - 1, |&a, &b| {
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
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::island::migration::migrate_pareto;
/// use genetic_algorithms::island::configuration::IslandConfiguration;
/// use genetic_algorithms::nsga2::pareto::ParetoIndividual;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let config = IslandConfiguration::new().with_migration_count(1);
/// let ind = ParetoIndividual::new(RangeChromosome::default(), vec![0.0, 1.0]);
/// let mut islands: Vec<Vec<ParetoIndividual<RangeChromosome<f64>>>> = vec![
///     vec![ind.clone()],
///     vec![ind.clone()],
/// ];
/// let _ = migrate_pareto(&mut islands, &config);
/// ```
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
            crate::log_debug!(
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
