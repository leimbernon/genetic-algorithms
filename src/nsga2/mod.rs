//! NSGA-II multi-objective genetic algorithm.
//!
//! This module implements the Non-dominated Sorting Genetic Algorithm II
//! (NSGA-II) for multi-objective optimization. It uses a `ParetoIndividual<U>`
//! wrapper rather than modifying the existing `ChromosomeT` trait.
//!
//! # Example
//!
//! ```ignore
//! use genetic_algorithms::nsga2::Nsga2Ga;
//! use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
//! use genetic_algorithms::configuration::GaConfiguration;
//!
//! let nsga2_config = Nsga2Configuration::new()
//!     .with_num_objectives(2)
//!     .with_population_size(100)
//!     .with_max_generations(200);
//!
//! let ga_config = GaConfiguration::default();
//! let mut nsga2 = Nsga2Ga::<MyChromosome>::new(nsga2_config, ga_config)
//!     .with_initialization_fn(|n, alleles, repeat| { /* ... */ })
//!     .with_objective_fns(vec![
//!         Box::new(|dna| { /* objective 1 */ 0.0 }),
//!         Box::new(|dna| { /* objective 2 */ 0.0 }),
//!     ]);
//!
//! let pareto_front = nsga2.run().unwrap();
//! ```

pub mod configuration;
pub mod crowding_distance;
pub mod non_dominated_sort;
pub mod pareto;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::nsga2::configuration::Nsga2Configuration;
use crate::nsga2::crowding_distance::assign_crowding_distance;
use crate::nsga2::non_dominated_sort::{assign_ranks, non_dominated_sort};
use crate::nsga2::pareto::{ParetoFront, ParetoIndividual};
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use log::{debug, info};
use rand::Rng;
use rayon::prelude::*;
use std::sync::Arc;

/// Type alias for a single objective function.
pub type ObjectiveFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;

/// NSGA-II multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Nsga2Ga<U>
where
    U: ChromosomeT,
{
    /// NSGA-II specific configuration.
    pub nsga2_config: Nsga2Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
}

impl<U> Nsga2Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `Nsga2Ga` with the given configurations.
    pub fn new(nsga2_config: Nsga2Configuration, ga_config: GaConfiguration) -> Self {
        Nsga2Ga {
            nsga2_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
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
    ///
    /// Each function evaluates one objective given the chromosome's DNA.
    pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.objective_fns = fns.into_iter().map(Arc::from).collect();
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

    /// Validates the NSGA-II configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidNsga2Configuration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.nsga2_config.num_objectives == 0 {
            return Err(GaError::InvalidNsga2Configuration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.nsga2_config.population_size < 2 {
            return Err(GaError::InvalidNsga2Configuration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidNsga2Configuration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.nsga2_config.num_objectives {
            return Err(GaError::InvalidNsga2Configuration(format!(
                "Expected {} objective functions, got {}",
                self.nsga2_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        Ok(())
    }
}

impl<U> Nsga2Ga<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    /// Runs the NSGA-II algorithm and returns the first Pareto front.
    ///
    /// # Returns
    ///
    /// `Ok(ParetoFront<U>)` containing the non-dominated solutions.
    ///
    /// # Errors
    ///
    /// Returns `GaError` on validation or operator failure.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;

        let pop_size = self.nsga2_config.population_size;
        let max_gens = self.nsga2_config.max_generations;

        // Initialize population
        let mut population = self.initialize_population()?;

        info!(
            target: "nsga2_events",
            "Starting NSGA-II: {} individuals, {} objectives, {} generations",
            pop_size,
            self.nsga2_config.num_objectives,
            max_gens
        );

        for gen in 0..max_gens {
            // Non-dominated sorting
            let all_objectives: Vec<&[f64]> = population
                .iter()
                .map(|ind| ind.objectives.as_slice())
                .collect();
            let fronts = non_dominated_sort(&all_objectives);

            // Assign ranks
            let mut ranks = vec![0usize; population.len()];
            assign_ranks(&mut ranks, &fronts);
            for (i, &r) in ranks.iter().enumerate() {
                population[i].rank = r;
            }

            // Assign crowding distance per front
            for front in &fronts {
                let front_objectives: Vec<&[f64]> = front
                    .iter()
                    .map(|&idx| population[idx].objectives.as_slice())
                    .collect();
                let mut front_crowding = vec![0.0; front.len()];
                assign_crowding_distance(&front_objectives, &mut front_crowding);
                for (local_idx, &global_idx) in front.iter().enumerate() {
                    population[global_idx].crowding_distance = front_crowding[local_idx];
                }
            }

            // Binary tournament selection + crossover + mutation to create offspring
            let offspring = self.create_offspring(&population)?;

            // Combine parent + offspring
            population.extend(offspring);

            // Environmental selection: sort by (rank asc, crowding_distance desc), truncate
            // Re-evaluate ranks and crowding for combined population
            let combined_objectives: Vec<&[f64]> = population
                .iter()
                .map(|ind| ind.objectives.as_slice())
                .collect();
            let combined_fronts = non_dominated_sort(&combined_objectives);

            let mut combined_ranks = vec![0usize; population.len()];
            assign_ranks(&mut combined_ranks, &combined_fronts);
            for (i, &r) in combined_ranks.iter().enumerate() {
                population[i].rank = r;
            }

            for front in &combined_fronts {
                let front_objectives: Vec<&[f64]> = front
                    .iter()
                    .map(|&idx| population[idx].objectives.as_slice())
                    .collect();
                let mut front_crowding = vec![0.0; front.len()];
                assign_crowding_distance(&front_objectives, &mut front_crowding);
                for (local_idx, &global_idx) in front.iter().enumerate() {
                    population[global_idx].crowding_distance = front_crowding[local_idx];
                }
            }

            // Sort: prefer lower rank, then higher crowding distance
            population.sort_by(|a, b| {
                a.rank.cmp(&b.rank).then_with(|| {
                    b.crowding_distance
                        .partial_cmp(&a.crowding_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });

            population.truncate(pop_size);

            debug!(
                target: "nsga2_events",
                "Generation {} complete, population size = {}", gen, population.len()
            );
        }

        // Extract the first Pareto front from the final population.
        // The population is already sorted by rank from environmental selection,
        // so individuals with rank 0 are the first Pareto front.
        let front_individuals: Vec<ParetoIndividual<U>> =
            population.into_iter().filter(|ind| ind.rank == 0).collect();

        Ok(ParetoFront::new(front_individuals))
    }

    /// Initializes the population with random chromosomes and evaluates objectives.
    fn initialize_population(&self) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let pop_size = self.nsga2_config.population_size;
        let genes_per_chrom = self.ga_config.limit_configuration.genes_per_chromosome;
        let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        // Create chromosomes without a single fitness fn (NSGA-II uses multiple objectives)
        let chromosomes: Vec<U> = crate::traits::initialize_chromosomes(
            pop_size,
            genes_per_chrom,
            alleles,
            Some(alleles_can_repeat),
            init_fn,
            None,
            0,
        );

        // Wrap each chromosome in a ParetoIndividual with evaluated objectives
        let objective_fns = &self.objective_fns;
        let population = chromosomes
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(population)
    }

    /// Creates offspring via binary tournament selection, crossover, and mutation.
    fn create_offspring(
        &self,
        population: &[ParetoIndividual<U>],
    ) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        use crate::operations::{crossover, mutation};

        let pop_size = self.nsga2_config.population_size;
        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration;
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);

        let mut rng = rand::rng();
        let mut raw_offspring: Vec<U> = Vec::with_capacity(pop_size);

        while raw_offspring.len() < pop_size {
            // Binary tournament selection
            let parent_a = self.binary_tournament(population, &mut rng);
            let parent_b = self.binary_tournament(population, &mut rng);

            let p: f64 = rng.random();
            let mut children = if p <= crossover_prob {
                crossover::factory(
                    &population[parent_a].chromosome,
                    &population[parent_b].chromosome,
                    crossover_config,
                )?
            } else {
                vec![
                    population[parent_a].chromosome.clone(),
                    population[parent_b].chromosome.clone(),
                ]
            };

            // Mutation
            for child in children.iter_mut() {
                let mp: f64 = rng.random();
                if mp <= mut_prob {
                    mutation::factory_with_params(
                        mutation_config.method,
                        child,
                        mutation_config.step,
                        mutation_config.sigma,
                    )?;
                }
            }

            for child in children {
                raw_offspring.push(child);
                if raw_offspring.len() >= pop_size {
                    break;
                }
            }
        }

        // Evaluate objectives in parallel
        let objective_fns = &self.objective_fns;
        let offspring = raw_offspring
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(offspring)
    }

    /// Binary tournament selection: picks two random individuals and returns the
    /// index of the better one (lower rank, or higher crowding distance if tied).
    fn binary_tournament(&self, population: &[ParetoIndividual<U>], rng: &mut impl Rng) -> usize {
        let n = population.len();
        let i = rng.random_range(0..n);
        let j = rng.random_range(0..n);

        let a = &population[i];
        let b = &population[j];

        if a.rank < b.rank {
            i
        } else if b.rank < a.rank {
            j
        } else if a.crowding_distance > b.crowding_distance {
            i
        } else {
            j
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::GaConfiguration;
    use crate::nsga2::configuration::Nsga2Configuration;

    #[test]
    fn test_nsga2_validate_no_init_fn() {
        let config = Nsga2Configuration::new().with_num_objectives(2);
        let ga_config = GaConfiguration::default();
        let nsga2 = Nsga2Ga::<crate::chromosomes::Binary>::new(config, ga_config);

        let result = nsga2.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_nsga2_validate_zero_objectives() {
        let config = Nsga2Configuration::new().with_num_objectives(0);
        let ga_config = GaConfiguration::default();
        let nsga2 = Nsga2Ga::<crate::chromosomes::Binary>::new(config, ga_config)
            .with_initialization_fn(|_, _, _| vec![]);

        let result = nsga2.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_nsga2_validate_mismatched_objective_fns() {
        let config = Nsga2Configuration::new().with_num_objectives(2);
        let ga_config = GaConfiguration::default();
        let nsga2 = Nsga2Ga::<crate::chromosomes::Binary>::new(config, ga_config)
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
        let nsga2 = Nsga2Ga::<crate::chromosomes::Binary>::new(config, ga_config)
            .with_initialization_fn(|_, _, _| vec![])
            .with_objective_fns(vec![Box::new(|_| 0.0)]);

        let result = nsga2.validate();
        assert!(result.is_err());
    }
}
