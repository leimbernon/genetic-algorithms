//! Island Model for parallel multi-population genetic algorithm evolution.
//!
//! The island model runs multiple independent populations (islands) that
//! evolve in parallel using `rayon`. Periodically, the best individuals
//! from each island migrate to neighboring islands according to a
//! configurable topology.
//!
//! # Example
//!
//! ```ignore
//! use genetic_algorithms::island::configuration::IslandConfiguration;
//! use genetic_algorithms::island::topology::MigrationTopology;
//! use genetic_algorithms::island::IslandGa;
//!
//! let island_config = IslandConfiguration::new()
//!     .with_num_islands(4)
//!     .with_migration_interval(10)
//!     .with_migration_count(2)
//!     .with_topology(MigrationTopology::Ring);
//! ```

pub mod configuration;
pub mod migration;
pub mod nsga2;
pub mod topology;

use crate::configuration::{GaConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::island::configuration::IslandConfiguration;
use crate::island::migration::migrate;
use crate::operations::mutation;
use crate::population::Population;
use crate::traits::{ChromosomeT, FitnessFn, InitializationFn};
use log::{debug, info};
use std::sync::Arc;

/// Island Model Genetic Algorithm orchestrator.
///
/// Runs multiple GA populations in parallel with periodic migration.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct IslandGa<U>
where
    U: ChromosomeT,
{
    /// Island model configuration.
    pub island_config: IslandConfiguration,
    /// Base GA configuration applied to each island.
    pub ga_config: GaConfiguration,
    /// The populations for each island.
    pub islands: Vec<Population<U>>,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Fitness function.
    pub fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
}

impl<U> IslandGa<U>
where
    U: ChromosomeT,
{
    /// Creates a new `IslandGa` with the given configurations.
    ///
    /// # Arguments
    ///
    /// * `island_config` - Configuration for the island model.
    /// * `ga_config` - Base GA configuration for each island.
    ///
    /// # Returns
    ///
    /// A new `IslandGa` instance.
    pub fn new(island_config: IslandConfiguration, ga_config: GaConfiguration) -> Self {
        IslandGa {
            island_config,
            ga_config,
            islands: Vec::new(),
            alleles: Vec::new(),
            initialization_fn: None,
            fitness_fn: None,
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

    /// Sets the fitness function.
    pub fn with_fitness_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Some(Arc::new(f));
        self
    }

    /// Validates configuration and returns a ready-to-run instance.
    ///
    /// Call this after setting all builder options and before calling `run()`.
    ///
    /// # Errors
    ///
    /// Returns `GaError` if validation fails (see [`validate`](Self::validate)).
    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the island configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(GaError)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidIslandConfiguration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.island_config.num_islands == 0 {
            return Err(GaError::InvalidIslandConfiguration(
                "num_islands must be > 0".to_string(),
            ));
        }
        if self.island_config.migration_interval == 0 {
            return Err(GaError::InvalidIslandConfiguration(
                "migration_interval must be > 0".to_string(),
            ));
        }
        if self.island_config.migration_count == 0 {
            return Err(GaError::InvalidIslandConfiguration(
                "migration_count must be > 0".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidIslandConfiguration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.fitness_fn.is_none() {
            return Err(GaError::InvalidIslandConfiguration(
                "fitness_fn is required".to_string(),
            ));
        }
        let pop_size = self.ga_config.limit_configuration.population_size;
        if self.island_config.migration_count >= pop_size {
            return Err(GaError::InvalidIslandConfiguration(format!(
                "migration_count ({}) must be < population_size ({})",
                self.island_config.migration_count, pop_size
            )));
        }
        Ok(())
    }

    /// Initializes all islands with random populations.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InitializationError` if initialization fails.
    pub fn initialize(&mut self) -> Result<(), GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;
        let fitness_fn = self
            .fitness_fn
            .as_ref()
            .ok_or_else(|| GaError::InitializationError("No fitness function set".to_string()))?;

        let num_islands = self.island_config.num_islands;
        let pop_size = self.ga_config.limit_configuration.population_size;
        let genes_per_chrom = self.ga_config.limit_configuration.genes_per_chromosome;
        let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        self.islands = Vec::with_capacity(num_islands);

        for island_idx in 0..num_islands {
            let chromosomes = crate::traits::initialize_chromosomes::<U>(
                pop_size,
                genes_per_chrom,
                alleles,
                Some(alleles_can_repeat),
                init_fn,
                Some(fitness_fn),
                0,
            );

            self.islands.push(Population::new(chromosomes));
            debug!(
                target: "island_events",
                "Initialized island {} with {} chromosomes", island_idx, pop_size
            );
        }

        Ok(())
    }

    /// Returns the best chromosome across all islands.
    fn global_best(&self, problem_solving: ProblemSolving) -> U {
        let mut best: Option<&U> = None;

        for island in &self.islands {
            for chrom in &island.chromosomes {
                let is_better = match best {
                    None => true,
                    Some(current_best) => match problem_solving {
                        ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                            chrom.fitness() < current_best.fitness()
                        }
                        ProblemSolving::Maximization => chrom.fitness() > current_best.fitness(),
                    },
                };
                if is_better {
                    best = Some(chrom);
                }
            }
        }

        // Safety: we always initialize at least one island with at least one chromosome
        best.expect("Islands should not be empty after initialization")
            .clone()
    }
}

impl<U> IslandGa<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    /// Runs the island model GA and returns the best chromosome found across all islands.
    ///
    /// # Returns
    ///
    /// `Ok(U)` - The best chromosome found across all islands.
    ///
    /// # Errors
    ///
    /// Returns `GaError` if validation, initialization, or migration fails.
    pub fn run(&mut self) -> Result<U, GaError> {
        self.validate()?;
        self.initialize()?;

        let max_generations = self.ga_config.limit_configuration.max_generations;
        let problem_solving = self.ga_config.limit_configuration.problem_solving;
        let fitness_target = self.ga_config.limit_configuration.fitness_target;

        info!(
            target: "island_events",
            "Starting island model GA: {} islands, {} generations",
            self.island_config.num_islands,
            max_generations
        );

        for gen in 0..max_generations {
            // Evolve each island for one generation
            self.evolve_islands_one_generation(problem_solving)?;

            // Check fitness target
            if let Some(target) = fitness_target {
                let best = self.global_best(problem_solving);
                let dist = (best.fitness() - target).abs();
                if dist < 1e-10 {
                    info!(
                        target: "island_events",
                        "Fitness target reached at generation {}", gen
                    );
                    return Ok(best);
                }
            }

            // Migration
            if gen > 0
                && self.island_config.migration_interval > 0
                && gen % self.island_config.migration_interval == 0
            {
                migrate(&mut self.islands, &self.island_config, problem_solving)?;
                debug!(
                    target: "island_events",
                    "Migration performed at generation {}", gen
                );
            }
        }

        Ok(self.global_best(problem_solving))
    }

    /// Performs one generation of evolution on each island.
    ///
    /// Uses the configured selection, crossover, mutation and survivor operators
    /// via the standard factory functions.
    fn evolve_islands_one_generation(
        &mut self,
        _problem_solving: ProblemSolving,
    ) -> Result<(), GaError> {
        use crate::operations::{crossover, mutation, selection, survivor};
        use rand::Rng;
        use rayon::prelude::*;

        let selection_config = self.ga_config.selection_configuration;
        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration;
        let survivor_method = self.ga_config.survivor;
        let limit_config = self.ga_config.limit_configuration;
        let pop_size = limit_config.population_size;
        let fitness_fn = self
            .fitness_fn
            .as_ref()
            .ok_or_else(|| GaError::ConfigurationError("No fitness function set".to_string()))?;

        let fitness_fn = Arc::clone(fitness_fn);
        let num_threads = self.ga_config.number_of_threads;

        self.islands.par_iter_mut().try_for_each(|island| {
            // Selection: returns Vec<(usize, usize)> parent index pairs
            let parent_pairs =
                selection::factory(&island.chromosomes, selection_config, num_threads)?;

            // Crossover: iterate over parent pairs
            let mut rng = rand::rng();
            let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);

            let mut offspring: Vec<U> = Vec::new();
            for &(idx_a, idx_b) in &parent_pairs {
                let p: f64 = rng.random();
                if p <= crossover_prob {
                    let children = crossover::factory(
                        &island.chromosomes[idx_a],
                        &island.chromosomes[idx_b],
                        crossover_config,
                    )?;
                    offspring.extend(children);
                } else {
                    offspring.push(island.chromosomes[idx_a].clone());
                    offspring.push(island.chromosomes[idx_b].clone());
                }
            }

            // Mutation
            let mut_prob = mutation_config.probability_max.unwrap_or(0.1);
            for child in offspring.iter_mut() {
                let p: f64 = rng.random();
                if p <= mut_prob {
                    mutation::factory_with_params(
                        mutation_config.method,
                        child,
                        mutation_config.step,
                        mutation_config.sigma,
                    )?;
                }
            }

            // Assign fitness to offspring
            for child in offspring.iter_mut() {
                let ff = Arc::clone(&fitness_fn);
                child.set_fitness_fn(move |genes| ff(genes));
                child.calculate_fitness();
            }

            // Combine parent population with offspring
            island.chromosomes.append(&mut offspring);

            // Survivor selection: trims in-place to pop_size
            survivor::factory(
                survivor_method,
                &mut island.chromosomes,
                pop_size,
                limit_config,
            )?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::GaConfiguration;
    use crate::island::configuration::IslandConfiguration;

    #[test]
    fn test_island_ga_validate_no_init_fn() {
        let config = IslandConfiguration::new().with_num_islands(2);
        let ga_config = GaConfiguration::default();
        let island_ga: IslandGa<crate::chromosomes::Binary> = IslandGa::new(config, ga_config);

        let result = island_ga.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_island_ga_validate_zero_islands() {
        let config = IslandConfiguration::new().with_num_islands(0);
        let ga_config = GaConfiguration::default();
        let island_ga = IslandGa::<crate::chromosomes::Binary>::new(config, ga_config)
            .with_initialization_fn(|_, _, _| vec![])
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
        let island_ga = IslandGa::<crate::chromosomes::Binary>::new(config, ga_config)
            .with_initialization_fn(|_, _, _| vec![])
            .with_fitness_fn(|_| 0.0);

        let result = island_ga.validate();
        assert!(result.is_err());
    }
}
