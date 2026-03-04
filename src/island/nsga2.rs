//! Island Model + NSGA-II multi-objective genetic algorithm.
//!
//! This module combines the island model (multiple sub-populations with periodic
//! migration) with the NSGA-II algorithm (non-dominated sorting, crowding distance)
//! for multi-objective optimization.
//!
//! Each island runs an independent NSGA-II evolution loop. Periodically, the best
//! Pareto individuals (lowest rank, highest crowding distance) migrate between
//! islands according to the configured topology, replacing the worst individuals
//! in the destination island.
//!
//! # Example
//!
//! ```ignore
//! use genetic_algorithms::island::nsga2::IslandNsga2Ga;
//! use genetic_algorithms::island::configuration::IslandConfiguration;
//! use genetic_algorithms::island::topology::MigrationTopology;
//! use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
//! use genetic_algorithms::configuration::GaConfiguration;
//!
//! let island_config = IslandConfiguration::new()
//!     .with_num_islands(4)
//!     .with_migration_interval(10)
//!     .with_migration_count(2)
//!     .with_topology(MigrationTopology::Ring);
//!
//! let nsga2_config = Nsga2Configuration::new()
//!     .with_num_objectives(2)
//!     .with_population_size(100)
//!     .with_max_generations(200);
//!
//! let ga_config = GaConfiguration::default();
//!
//! let mut ga = IslandNsga2Ga::<MyChromosome>::new(island_config, nsga2_config, ga_config)
//!     .with_initialization_fn(|n, alleles, repeat| { /* ... */ })
//!     .with_objective_fns(vec![
//!         Box::new(|dna| { /* objective 1 */ 0.0 }),
//!         Box::new(|dna| { /* objective 2 */ 0.0 }),
//!     ])
//!     .build()
//!     .expect("Invalid configuration");
//!
//! let pareto_front = ga.run().unwrap();
//! ```

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::island::configuration::IslandConfiguration;
use crate::island::migration::migrate_pareto;
use crate::nsga2::configuration::Nsga2Configuration;
use crate::nsga2::crowding_distance::assign_crowding_distance;
use crate::nsga2::non_dominated_sort::{assign_ranks, non_dominated_sort};
use crate::nsga2::pareto::{ParetoFront, ParetoIndividual};
use crate::nsga2::ObjectiveFn;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use log::{debug, info};
use rand::Rng;
use std::sync::Arc;

/// Island Model + NSGA-II multi-objective genetic algorithm orchestrator.
///
/// Runs multiple NSGA-II populations in parallel with periodic Pareto-aware
/// migration. Returns the global Pareto front at the end of the run.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct IslandNsga2Ga<U>
where
    U: ChromosomeT,
{
    /// Island model configuration (num_islands, migration interval, count, topology).
    pub island_config: IslandConfiguration,
    /// NSGA-II configuration (num_objectives, population_size, max_generations).
    pub nsga2_config: Nsga2Configuration,
    /// Base GA configuration (operators — selection not used, crossover, mutation).
    pub ga_config: GaConfiguration,
    /// The island populations, each a flat `Vec<ParetoIndividual<U>>`.
    pub islands: Vec<Vec<ParetoIndividual<U>>>,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
}

impl<U> IslandNsga2Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `IslandNsga2Ga` with the given configurations.
    pub fn new(
        island_config: IslandConfiguration,
        nsga2_config: Nsga2Configuration,
        ga_config: GaConfiguration,
    ) -> Self {
        IslandNsga2Ga {
            island_config,
            nsga2_config,
            ga_config,
            islands: Vec::new(),
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
    /// Returns `GaError` if validation fails.
    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the combined island + NSGA-II configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        // Island validations
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

        // NSGA-II validations
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

        // Cross-config validation
        if self.island_config.migration_count >= self.nsga2_config.population_size {
            return Err(GaError::InvalidIslandConfiguration(format!(
                "migration_count ({}) must be < population_size ({})",
                self.island_config.migration_count, self.nsga2_config.population_size
            )));
        }

        // Required functions
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

    /// Initializes all islands with random populations wrapped in `ParetoIndividual`.
    fn initialize_islands(&mut self) -> Result<(), GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let num_islands = self.island_config.num_islands;
        let pop_size = self.nsga2_config.population_size;
        let genes_per_chrom = self.ga_config.limit_configuration.genes_per_chromosome;
        let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        self.islands = Vec::with_capacity(num_islands);

        for island_idx in 0..num_islands {
            // Create raw chromosomes (no scalar fitness fn for NSGA-II)
            let chromosomes: Vec<U> = crate::traits::initialize_chromosomes(
                pop_size,
                genes_per_chrom,
                alleles,
                Some(alleles_can_repeat),
                init_fn,
                None,
                0,
            );

            // Wrap in ParetoIndividual and evaluate objectives
            let population: Vec<ParetoIndividual<U>> = chromosomes
                .into_iter()
                .map(|chrom| {
                    let objectives = self.evaluate_objectives(&chrom);
                    ParetoIndividual::new(chrom, objectives)
                })
                .collect();

            self.islands.push(population);
            debug!(
                target: "island_events",
                "Initialized NSGA-II island {} with {} individuals", island_idx, pop_size
            );
        }

        Ok(())
    }

    /// Evaluates all objective functions for a chromosome.
    fn evaluate_objectives(&self, chromosome: &U) -> Vec<f64> {
        self.objective_fns
            .iter()
            .map(|f| f(chromosome.dna()))
            .collect()
    }

    /// Performs non-dominated sorting and crowding distance assignment on a population.
    fn rank_and_crowd(population: &mut [ParetoIndividual<U>]) {
        let all_objectives: Vec<&[f64]> = population
            .iter()
            .map(|ind| ind.objectives.as_slice())
            .collect();
        let fronts = non_dominated_sort(&all_objectives);

        let mut ranks = vec![0usize; population.len()];
        assign_ranks(&mut ranks, &fronts);
        for (i, &r) in ranks.iter().enumerate() {
            population[i].rank = r;
        }

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
    }
}

impl<U> IslandNsga2Ga<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    /// Runs the Island-NSGA-II algorithm and returns the global Pareto front.
    ///
    /// The algorithm:
    /// 1. Initializes all islands with random populations.
    /// 2. For each generation, evolves each island one NSGA-II generation
    ///    (non-dominated sort, crowding distance, binary tournament, crossover,
    ///    mutation, environmental selection).
    /// 3. At migration intervals, performs Pareto-aware migration between islands.
    /// 4. Returns the global Pareto front (rank-0 individuals across all islands).
    ///
    /// # Returns
    ///
    /// `Ok(ParetoFront<U>)` containing the global non-dominated solutions.
    ///
    /// # Errors
    ///
    /// Returns `GaError` on validation, initialization, operator, or migration failure.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        self.initialize_islands()?;

        let max_gens = self.nsga2_config.max_generations;
        let pop_size = self.nsga2_config.population_size;

        info!(
            target: "island_events",
            "Starting Island-NSGA-II: {} islands, {} individuals/island, {} objectives, {} generations",
            self.island_config.num_islands,
            pop_size,
            self.nsga2_config.num_objectives,
            max_gens
        );

        // Initial ranking for all islands
        {
            use rayon::prelude::*;
            self.islands
                .par_iter_mut()
                .for_each(|island| Self::rank_and_crowd(island));
        }

        for gen in 0..max_gens {
            // Evolve each island one NSGA-II generation
            self.evolve_islands_one_generation(pop_size)?;

            // Migration at configured intervals
            if gen > 0
                && self.island_config.migration_interval > 0
                && gen % self.island_config.migration_interval == 0
            {
                migrate_pareto(&mut self.islands, &self.island_config)?;
                debug!(
                    target: "island_events",
                    "Pareto migration at generation {}", gen
                );
            }
        }

        // Build global Pareto front: merge all islands, re-sort, extract rank 0
        Ok(self.global_pareto_front())
    }

    /// Evolves each island for one NSGA-II generation.
    ///
    /// For each island:
    /// 1. Binary tournament selection + crossover + mutation -> offspring.
    /// 2. Combine parent + offspring.
    /// 3. Non-dominated sort + crowding distance on combined.
    /// 4. Environmental selection: sort by (rank asc, crowding desc), truncate to `pop_size`.
    fn evolve_islands_one_generation(&mut self, pop_size: usize) -> Result<(), GaError> {
        use crate::operations::{crossover, mutation};
        use rayon::prelude::*;

        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration;
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);
        let objective_fns = &self.objective_fns;

        self.islands.par_iter_mut().try_for_each(|island| {
            let mut rng = rand::rng();
            let mut offspring: Vec<ParetoIndividual<U>> = Vec::with_capacity(pop_size);

            while offspring.len() < pop_size {
                // Binary tournament selection
                let parent_a = binary_tournament(island, &mut rng);
                let parent_b = binary_tournament(island, &mut rng);

                let p: f64 = rng.random();
                let mut children = if p <= crossover_prob {
                    crossover::factory(
                        &island[parent_a].chromosome,
                        &island[parent_b].chromosome,
                        crossover_config,
                    )?
                } else {
                    vec![
                        island[parent_a].chromosome.clone(),
                        island[parent_b].chromosome.clone(),
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

                // Evaluate objectives and wrap in ParetoIndividual
                for child in children {
                    let objectives: Vec<f64> =
                        objective_fns.iter().map(|f| f(child.dna())).collect();
                    offspring.push(ParetoIndividual::new(child, objectives));
                    if offspring.len() >= pop_size {
                        break;
                    }
                }
            }

            // Combine parent + offspring
            island.extend(offspring);

            // Non-dominated sort + crowding on combined
            Self::rank_and_crowd(island);

            // Environmental selection: sort by (rank asc, crowding desc), truncate
            island.sort_by(|a, b| {
                a.rank.cmp(&b.rank).then_with(|| {
                    b.crowding_distance
                        .partial_cmp(&a.crowding_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });

            island.truncate(pop_size);

            Ok(())
        })
    }

    /// Merges all islands and returns the global Pareto front (rank-0 individuals).
    fn global_pareto_front(&self) -> ParetoFront<U> {
        // Merge all individuals from all islands
        let mut combined: Vec<ParetoIndividual<U>> = self
            .islands
            .iter()
            .flat_map(|island| island.iter().cloned())
            .collect();

        if combined.is_empty() {
            return ParetoFront::new(vec![]);
        }

        // Re-sort the combined population to get a global ranking
        Self::rank_and_crowd(&mut combined);

        // Extract rank-0 individuals
        let front_individuals: Vec<ParetoIndividual<U>> =
            combined.into_iter().filter(|ind| ind.rank == 0).collect();

        ParetoFront::new(front_individuals)
    }
}

/// Binary tournament selection for Pareto individuals.
///
/// Picks two random individuals and returns the index of the better one
/// (lower rank, or higher crowding distance if tied).
fn binary_tournament<U>(population: &[ParetoIndividual<U>], rng: &mut impl Rng) -> usize
where
    U: ChromosomeT,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::GaConfiguration;
    use crate::island::configuration::IslandConfiguration;
    use crate::nsga2::configuration::Nsga2Configuration;

    #[test]
    fn test_island_nsga2_validate_zero_islands() {
        let island_config = IslandConfiguration::new().with_num_islands(0);
        let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

        assert!(ga.validate().is_err());
    }

    #[test]
    fn test_island_nsga2_validate_zero_objectives() {
        let island_config = IslandConfiguration::new().with_num_islands(2);
        let nsga2_config = Nsga2Configuration::new().with_num_objectives(0);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![]);

        assert!(ga.validate().is_err());
    }

    #[test]
    fn test_island_nsga2_validate_no_init_fn() {
        let island_config = IslandConfiguration::new().with_num_islands(2);
        let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

        assert!(ga.validate().is_err());
    }

    #[test]
    fn test_island_nsga2_validate_mismatched_objectives() {
        let island_config = IslandConfiguration::new().with_num_islands(2);
        let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]); // only 1, need 2

        assert!(ga.validate().is_err());
    }

    #[test]
    fn test_island_nsga2_validate_migration_count_exceeds_pop() {
        let island_config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(200);
        let nsga2_config = Nsga2Configuration::new()
            .with_num_objectives(2)
            .with_population_size(100);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

        assert!(ga.validate().is_err());
    }

    #[test]
    fn test_island_nsga2_validate_ok() {
        let island_config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(2);
        let nsga2_config = Nsga2Configuration::new()
            .with_num_objectives(2)
            .with_population_size(20);
        let ga_config = GaConfiguration::default();
        let ga = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

        assert!(ga.validate().is_ok());
    }

    #[test]
    fn test_island_nsga2_build_ok() {
        let island_config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_count(2);
        let nsga2_config = Nsga2Configuration::new()
            .with_num_objectives(2)
            .with_population_size(20);
        let ga_config = GaConfiguration::default();
        let result = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)])
        .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_island_nsga2_build_fails_invalid() {
        let island_config = IslandConfiguration::new().with_num_islands(0);
        let nsga2_config = Nsga2Configuration::new();
        let ga_config = GaConfiguration::default();
        let result = IslandNsga2Ga::<crate::chromosomes::Binary>::new(
            island_config,
            nsga2_config,
            ga_config,
        )
        .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_binary_tournament_prefers_lower_rank() {
        use crate::traits::ChromosomeT;
        use std::borrow::Cow;

        #[derive(Debug, Clone, Default)]
        struct SimpleChrom {
            dna: Vec<crate::genotypes::Binary>,
        }

        impl ChromosomeT for SimpleChrom {
            type Gene = crate::genotypes::Binary;
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
            fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
            where
                F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
            {
                self
            }
            fn calculate_fitness(&mut self) {}
            fn fitness(&self) -> f64 {
                0.0
            }
            fn set_fitness(&mut self, _: f64) -> &mut Self {
                self
            }
            fn set_age(&mut self, _: usize) -> &mut Self {
                self
            }
            fn age(&self) -> usize {
                0
            }
        }

        let pop = vec![
            ParetoIndividual {
                chromosome: <SimpleChrom as Default>::default(),
                objectives: vec![1.0],
                rank: 2,
                crowding_distance: 5.0,
            },
            ParetoIndividual {
                chromosome: <SimpleChrom as Default>::default(),
                objectives: vec![1.0],
                rank: 0,
                crowding_distance: 1.0,
            },
        ];

        // With only 2 individuals, tournament always picks between index 0 and 1.
        // rank 0 < rank 2, so index 1 should be preferred.
        let mut wins = [0usize; 2];
        let mut rng = rand::rng();
        for _ in 0..100 {
            let idx = binary_tournament(&pop, &mut rng);
            wins[idx] += 1;
        }
        // The rank-0 individual (index 1) should win the majority
        assert!(
            wins[1] > wins[0],
            "Rank-0 individual should win more often: wins={:?}",
            wins
        );
    }
}
