//! SPEA2 — Strength Pareto Evolutionary Algorithm 2.
//!
//! SPEA2 (Zitzler, Laumanns & Thiele 2001) is a multi-objective evolutionary
//! algorithm that maintains a fixed-size external archive of non-dominated
//! solutions. Fitness is computed from raw strength (domination count) plus
//! density (k-nearest-neighbour distance), and the archive is truncated using
//! iterative nearest-neighbour removal when it exceeds capacity.
//!
//! Reference: Zitzler, Laumanns & Thiele 2001 (TIK-Report 103).

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::pareto::ParetoIndividual;
use crate::multi_objective::ObjectiveFn;
use crate::nsga2::configuration::ObjectiveDirection;
use crate::observer::Spea2Observer;
use crate::spea2::configuration::Spea2Configuration;
use crate::traits::{ChromosomeT, InitializationFn};
use std::sync::Arc;

/// SPEA2 strength-Pareto multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Spea2Ga<U>
where
    U: ChromosomeT,
{
    /// SPEA2-specific configuration.
    pub spea2_config: Spea2Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for SPEA2-specific events.
    pub observer: Option<Arc<dyn Spea2Observer<U> + Send + Sync>>,
}

#[allow(dead_code)]
impl<U> Spea2Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `Spea2Ga` with the given configurations.
    pub fn new(spea2_config: Spea2Configuration, ga_config: GaConfiguration) -> Self {
        Spea2Ga {
            spea2_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives SPEA2-specific hooks (D-05).
    pub fn with_observer(mut self, obs: Arc<dyn Spea2Observer<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    pub(crate) fn notify<F: FnOnce(&dyn Spea2Observer<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Sets the alleles template.
    pub fn with_alleles(mut self, alleles: Vec<U::Gene>) -> Self {
        self.alleles = alleles;
        self
    }

    /// Sets the initialization function.
    pub fn with_initialization_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(f));
        self
    }

    /// Sets the objective functions.
    pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.objective_fns = fns.into_iter().map(Arc::from).collect();
        self
    }

    /// Validates configuration and returns a ready-to-run instance.
    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the SPEA2 configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidSpea2Configuration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.spea2_config.num_objectives == 0 {
            return Err(GaError::InvalidSpea2Configuration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.spea2_config.population_size < 2 {
            return Err(GaError::InvalidSpea2Configuration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidSpea2Configuration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.spea2_config.num_objectives {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "Expected {} objective functions, got {}",
                self.spea2_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.spea2_config.objective_directions.is_empty()
            && self.spea2_config.objective_directions.len() != self.spea2_config.num_objectives
        {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.spea2_config.objective_directions.len(),
                self.spea2_config.num_objectives
            )));
        }
        // D-01: archive_size must be > 0 and <= population_size
        if self.spea2_config.archive_size == 0 {
            return Err(GaError::InvalidSpea2Configuration(
                "archive_size must be > 0".to_string(),
            ));
        }
        if self.spea2_config.archive_size > self.spea2_config.population_size {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "archive_size ({}) must not exceed population_size ({})",
                self.spea2_config.archive_size,
                self.spea2_config.population_size
            )));
        }
        Ok(())
    }

    /// Euclidean distance between two objective vectors.
    fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
    }

    /// Computes SPEA2 fitness (strength + density) for the combined population + archive set.
    ///
    /// Implements Zitzler, Laumanns & Thiele 2001 Algorithm 1, Step 2.
    /// Returns a vector of fitness values (lower = better) in the same order as
    /// `population` followed by `archive`.
    fn assign_spea2_fitness(
        population: &[ParetoIndividual<U>],
        archive: &[ParetoIndividual<U>],
        directions: &[ObjectiveDirection],
    ) -> Vec<f64> {
        let union: Vec<&ParetoIndividual<U>> =
            population.iter().chain(archive.iter()).collect();
        let n = union.len();
        // D-02: k = floor(sqrt(N_pop + N_archive))
        let k = (n as f64).sqrt().floor() as usize;

        // Step 1: Compute strength S(i) = count of individuals that i dominates
        let mut strength = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                if i != j && crate::multi_objective::pareto::dominates_with_directions(
                    &union[i].objectives,
                    &union[j].objectives,
                    directions,
                ) {
                    strength[i] += 1.0;
                }
            }
        }

        // Step 2: Compute raw fitness R(i) = sum of strengths of individuals dominating i
        let mut raw_fitness = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                if i != j && crate::multi_objective::pareto::dominates_with_directions(
                    &union[j].objectives,
                    &union[i].objectives,
                    directions,
                ) {
                    raw_fitness[i] += strength[j];
                }
            }
        }

        // Step 3: Compute density D(i) = 1 / (sigma_k + 2)
        let mut density = vec![0.0f64; n];
        let effective_k = k.max(1); // k must be at least 1 even for tiny unions
        for i in 0..n {
            let mut distances: Vec<f64> = (0..n)
                .filter(|&j| j != i)
                .map(|j| Self::euclidean_distance(&union[i].objectives, &union[j].objectives))
                .collect();
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let sigma_k = distances
                .get(effective_k.saturating_sub(1))
                .copied()
                .unwrap_or(f64::MAX);
            density[i] = 1.0 / (sigma_k + 2.0);
        }

        // Step 4: Final fitness F(i) = R(i) + D(i)  [lower is better]
        (0..n).map(|i| raw_fitness[i] + density[i]).collect()
    }

    /// Truncates the archive to the target size using iterative nearest-neighbour Euclidean removal
    /// with lexicographic tie-breaking (D-03).
    ///
    /// Implements Zitzler, Laumanns & Thiele 2001 Algorithm 1, Step 3 (truncation).
    /// Repeatedly removes the individual with the smallest nearest-neighbour distance,
    /// recomputing distances after each removal.
    fn truncate_archive(
        archive: &mut Vec<ParetoIndividual<U>>,
        target_size: usize,
    ) {
        while archive.len() > target_size {
            let n = archive.len();
            let mut remove_idx = 0usize;
            let mut remove_dist_list: Vec<f64> = Vec::new();

            // For each individual, compute sorted distances to all others.
            // Find the one with the lexicographically smallest sorted distance list.
            for i in 0..n {
                let mut dists: Vec<f64> = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| Self::euclidean_distance(&archive[i].objectives, &archive[j].objectives))
                    .collect();
                dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                if i == 0 {
                    remove_dist_list = dists;
                    remove_idx = i;
                } else {
                    // Lexicographic comparison: find the individual with smaller distances
                    let mut found_smaller = false;
                    for (a, b) in dists.iter().zip(remove_dist_list.iter()) {
                        if a < b {
                            found_smaller = true;
                            break;
                        } else if a > b {
                            break;
                        }
                        // equal -> continue to next distance
                    }
                    if found_smaller {
                        remove_dist_list = dists;
                        remove_idx = i;
                    }
                }
            }

            archive.remove(remove_idx);
        }
    }

    /// Performs environmental selection: builds the next archive from the combined
    /// population + archive set.
    ///
    /// 1. Copies all non-dominated individuals (fitness < 1.0) to the new archive.
    /// 2. If the new archive is under capacity, fills with best-dominated individuals
    ///    sorted by fitness (lower = better).
    /// 3. If the new archive exceeds capacity, truncates using Euclidean crowding (D-03).
    fn environmental_selection(
        population: &[ParetoIndividual<U>],
        archive: &[ParetoIndividual<U>],
        fitness: &[f64],
        target_archive_size: usize,
    ) -> Vec<ParetoIndividual<U>> {
        let union: Vec<&ParetoIndividual<U>> =
            population.iter().chain(archive.iter()).collect();

        // Step 1: Collect all non-dominated individuals (R(i) < 1.0 means non-dominated)
        let mut new_archive: Vec<ParetoIndividual<U>> = union
            .iter()
            .enumerate()
            .filter(|(i, _)| fitness[*i] < 1.0)
            .map(|(_, ind)| (*ind).clone())
            .collect();

        // Step 2: Fill or truncate to target size
        if new_archive.len() < target_archive_size {
            // Collect dominated individuals with their fitness values (index into combined set)
            let mut dominated: Vec<(f64, &ParetoIndividual<U>)> = union
                .iter()
                .enumerate()
                .filter(|(i, _)| fitness[*i] >= 1.0)
                .map(|(i, ind)| (fitness[i], *ind))
                .collect();
            // Sort by fitness (lower is better) -- fill with best dominated
            dominated.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let needed = target_archive_size - new_archive.len();
            for (_, ind) in dominated.into_iter().take(needed) {
                new_archive.push(ind.clone());
            }
        } else if new_archive.len() > target_archive_size {
            // Truncate using Euclidean crowding criterion (D-03)
            Self::truncate_archive(&mut new_archive, target_archive_size);
        }

        new_archive
    }

    /// Selects a parent index via binary tournament from the archive.
    ///
    /// Falls back to the population when the archive has fewer than 2 entries
    /// (early generations). Lower rank wins; ties broken by random coin flip.
    fn binary_tournament_from_archive(
        archive: &[ParetoIndividual<U>],
        population: &[ParetoIndividual<U>],
        rng: &mut impl rand::Rng,
    ) -> usize {
        let pool = if archive.len() >= 2 {
            archive
        } else {
            // Fall back to population when archive has < 2 individuals (early generations)
            population
        };
        let n = pool.len();
        let i = rng.random_range(0..n);
        let j = rng.random_range(0..n);
        // Lower SPEA2 fitness is better -- use rank as a proxy (0 = non-dominated)
        // In the archive, all individuals are sorted by fitness; tournament picks by rank
        if pool[i].rank < pool[j].rank {
            i
        } else if pool[j].rank < pool[i].rank {
            j
        } else if rng.random::<bool>() {
            i
        } else {
            j
        }
    }
}
