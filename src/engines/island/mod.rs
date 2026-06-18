//! # Island Model (IslandGa)
//!
//! ## Description
//!
//! The Island Model is a coarse-grained parallel metaheuristic that runs **multiple
//! independent GA populations (islands) in parallel** using `rayon`. Periodically,
//! individuals migrate between islands according to a configurable topology. This
//! approach maintains diversity through geographic isolation — each island converges
//! toward different regions of the search space — while allowing beneficial traits
//! to spread across the archipelago via migration.
//!
//! Each island can use a **different GA configuration** (mutation rate, selection method,
//! crossover operator, etc.), enabling heterogeneous evolution strategies. When the
//! number of configurations is less than the number of islands, the last configuration
//! is reused for the remaining islands.
//!
//! The migration cycle works as follows:
//!
//! 1. Each island evolves independently for `migration_interval` generations
//! 2. Migrants are selected from each island according to the `migration_policy`
//! 3. Migrants are sent to neighboring islands (determined by the topology)
//! 4. Destination islands replace some individuals with the incoming migrants
//! 5. Evolution continues for another `migration_interval` generations
//!
//! ## When to Use
//!
//! - **Problem type:** Single-objective — continuous, binary, permutation, or symbolic
//! - **Number of objectives:** 1 (see [`crate::island::nsga2`] for multi-objective island model)
//! - **Variable type:** Any (delegated to per-island GA configurations)
//! - **Key strength:** Excellent diversity via geographic isolation; natural parallelization;
//!   heterogeneous operator configurations across islands
//! - **Key weakness:** Higher total population than single-population GA; migration overhead;
//!   tuning migration interval and rate requires experimentation
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `num_islands` | `usize` | Yes (via builder) | `4` | Number of sub-populations |
//! | `migration_interval` | `usize` | Yes (via builder) | `10` | Generations between migration events |
//! | `migration_count` | `usize` | Yes (via builder) | `1` | Individuals migrating per island per event |
//! | `ga_config` | `GaConfiguration` | Yes (constructor) | — | Base GA config for each island |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `topology` | `MigrationTopology` | No | `Ring` | Inter-island connection pattern |
//! | `migration_policy` | `MigrationPolicy` | No | `BestReplaceWorst` | Migrant selection and placement |
//! | `observer` | `Option<Arc<dyn IslandGaObserver<U>>>` | No | `None` | Island-specific lifecycle observer |
//!
//! ### Migration Topologies
//!
//! | Topology | Description | Connectivity |
//! |----------|-------------|-------------|
//! | [`Ring`](topology::MigrationTopology::Ring) | Each island sends to the next (circular) | 1 neighbor |
//! | [`FullyConnected`](topology::MigrationTopology::FullyConnected) | Each island sends to all others | N-1 neighbors |
//! | [`Grid`](topology::MigrationTopology::Grid) | 2D lattice with 4-neighbor connectivity | Up to 4 neighbors |
//! | [`Hypercube`](topology::MigrationTopology::Hypercube) | Binary-reflected neighbors (power-of-2 islands) | log2(N) neighbors |
//! | [`Custom`](topology::MigrationTopology::Custom) | User-defined adjacency list | Variable |
//!
//! ## Complete Example
//!
//! ```rust,no_run
//! // no_run: Island GA example — illustrative API usage, not a runnable benchmark
//! use genetic_algorithms::configuration::GaConfiguration;
//! use genetic_algorithms::island::configuration::IslandConfiguration;
//! use genetic_algorithms::island::topology::MigrationTopology;
//! use genetic_algorithms::island::IslandGa;
//! use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
//! use genetic_algorithms::chromosomes::{ChromosomeLength, Range as RangeChromosome};
//!
//! let island_config = IslandConfiguration::new()
//!     .with_num_islands(4)
//!     .with_migration_interval(10)
//!     .with_migration_count(2)
//!     .with_topology(MigrationTopology::Ring);
//!
//! let ga_config = GaConfiguration::default()
//!     .with_population_size(100)
//!     .with_max_generations(500)
//!     .with_chromosome_length(ChromosomeLength::Fixed(10));
//!
//! let mut island_ga: IslandGa<RangeChromosome<f64>> = IslandGa::new(island_config, ga_config);
//! // See examples/island_model.rs for a full runnable example
//! ```
//!
//! ## Configuration Tips
//!
//! - **Ring topology** is the best starting point — it creates slow gene flow that preserves
//!   island diversity while still allowing beneficial traits to spread
//! - **Migration interval of 5-20** generations is typical; shorter intervals increase gene
//!   flow (faster convergence), longer intervals preserve more diversity
//! - **1-2 migrants per island per event** is sufficient in most cases; more migrants =
//!   faster homogenization
//! - Use **heterogeneous configurations** (different mutation rates or selection methods)
//!   across islands to explore diverse search strategies simultaneously
//! - Total population = `num_islands * ga_config.population_size` — be mindful of total
//!   computational cost
//!
//! ## When to Choose This vs Cellular GA
//!
//! | Factor | IslandGa | CellularEngine |
//! |--------|----------|---------------|
//! | Granularity | Coarse-grained (islands) | Fine-grained (cells) |
//! | Gene flow | Periodic migration events | Continuous local neighborhood |
//! | Diversity mechanism | Geographic isolation | Spatial diffusion |
//! | Heterogeneous operators | Yes (per-island config) | No (single global config) |
//! | Parallel scaling | Excellent (island-level) | Good (cell-level) |
//! | Migration topologies | 5 options | 4 neighborhood types |
//!
//! ## References
//!
//! - Cantu-Paz, E. (2000). *Efficient and Accurate Parallel Genetic Algorithms*. Springer.
//! - Whitley, D., Rana, S., & Heckendorn, R. B. (1998). The Island Model Genetic Algorithm:
//!   On Separability, Population Size and Convergence. *Journal of Computing and Information
//!   Technology*.

pub mod configuration;
pub mod migration;
pub mod nsga2;
pub mod topology;

use crate::configuration::{GaConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::island::configuration::IslandConfiguration;
use crate::island::migration::migrate;
use crate::observer::IslandGaObserver;
use crate::operations::mutation;
use crate::population::Population;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, InitializationFn, LinearChromosome, MutationOperator};
use std::sync::Arc;

/// Island Model Genetic Algorithm orchestrator.
///
/// Runs multiple GA populations in parallel with periodic migration.
///
/// Each island can use a different `GaConfiguration` for heterogeneous evolution
/// strategies. When `ga_configs` has fewer entries than the number of islands,
/// the last entry is reused for the remaining islands.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::island::IslandGa;
/// use genetic_algorithms::island::configuration::IslandConfiguration;
/// use genetic_algorithms::configuration::GaConfiguration;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let island_config = IslandConfiguration::new()
///     .with_num_islands(4)
///     .with_migration_interval(10);
///
/// let ga_config = GaConfiguration::default();
///
/// let engine = IslandGa::<RangeChromosome<f64>>::new(island_config, ga_config)
///     .with_fitness_fn(|dna| dna.iter().map(|g| g.value() * g.value()).sum::<f64>());
/// ```
pub struct IslandGa<U>
where
    U: LinearChromosome,
{
    /// Island model configuration.
    pub island_config: IslandConfiguration,
    /// Per-island GA configurations. If fewer than `num_islands`, the last entry
    /// is cycled for the remaining islands.
    pub ga_configs: Vec<GaConfiguration>,
    /// The populations for each island.
    pub islands: Vec<Population<U>>,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Fitness function.
    pub fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
    /// Optional lifecycle observer for island-specific events.
    observer: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>,
}

impl<U> IslandGa<U>
where
    U: LinearChromosome,
{
    /// Creates a new `IslandGa` with a single shared GA configuration for all islands.
    ///
    /// # Arguments
    ///
    /// * `island_config` - Configuration for the island model.
    /// * `ga_config` - Base GA configuration applied to each island.
    ///
    /// # Returns
    ///
    /// A new `IslandGa` instance.
    pub fn new(island_config: IslandConfiguration, ga_config: GaConfiguration) -> Self {
        IslandGa {
            island_config,
            ga_configs: vec![ga_config],
            islands: Vec::new(),
            alleles: Vec::new(),
            initialization_fn: None,
            fitness_fn: None,
            observer: None,
        }
    }

    /// Creates a new `IslandGa` with per-island GA configurations.
    ///
    /// When `configs` has fewer entries than `num_islands`, the last entry is
    /// repeated for the remaining islands. This allows heterogeneous evolution
    /// strategies — e.g. different mutation rates or selection methods per island.
    ///
    /// # Arguments
    ///
    /// * `island_config` - Configuration for the island model.
    /// * `configs` - One or more GA configurations. Must not be empty.
    ///
    /// # Returns
    ///
    /// A new `IslandGa` instance.
    pub fn with_heterogeneous_configs(
        island_config: IslandConfiguration,
        configs: Vec<GaConfiguration>,
    ) -> Self {
        IslandGa {
            island_config,
            ga_configs: configs,
            islands: Vec::new(),
            alleles: Vec::new(),
            initialization_fn: None,
            fitness_fn: None,
            observer: None,
        }
    }

    /// Returns the effective GA configuration for a given island index.
    ///
    /// If `ga_configs` has fewer entries than the number of islands, the last
    /// entry is returned for out-of-range indices.
    pub fn config_for_island(&self, island_index: usize) -> &GaConfiguration {
        if island_index < self.ga_configs.len() {
            &self.ga_configs[island_index]
        } else {
            self.ga_configs
                .last()
                .expect("ga_configs must not be empty")
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
        F: Fn(usize, Option<&[U::Gene]>) -> Vec<U::Gene> + Send + Sync + 'static,
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

    /// Attaches a lifecycle observer that receives island-specific hooks during execution.
    ///
    /// The observer is stored as an `Arc` for thread-safe sharing across rayon island threads.
    /// All hooks receive `&self`, so observers that need interior mutability should use
    /// `Mutex`, `AtomicU64`, or similar.
    ///
    /// See [`IslandGaObserver`] for the hook contract.
    pub fn with_observer(mut self, obs: Arc<dyn IslandGaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an island observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    fn notify<F: FnOnce(&dyn IslandGaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
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
        if self.ga_configs.is_empty() {
            return Err(GaError::InvalidIslandConfiguration(
                "ga_configs must not be empty".to_string(),
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
        // Check migration count against the smallest population size across all configs
        for (i, config) in self.ga_configs.iter().enumerate() {
            let pop_size = config.limit_configuration.population_size;
            if self.island_config.migration_count >= pop_size {
                return Err(GaError::InvalidIslandConfiguration(format!(
                    "migration_count ({}) must be < population_size ({}) for config index {}",
                    self.island_config.migration_count, pop_size, i
                )));
            }
        }
        Ok(())
    }

    /// Initializes all islands with random populations.
    ///
    /// Each island uses its own `GaConfiguration` (from `ga_configs`) to determine
    /// population size and gene parameters.
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

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        self.islands = Vec::with_capacity(num_islands);

        for island_idx in 0..num_islands {
            let cfg = self.config_for_island(island_idx);
            let pop_size = cfg.limit_configuration.population_size;
            let genes_per_chrom = match cfg.limit_configuration.chromosome_length {
                crate::chromosomes::ChromosomeLength::Fixed(n) => n,
                crate::chromosomes::ChromosomeLength::Variable { .. } => {
                    return Err(GaError::InvalidIslandConfiguration(
                        "ChromosomeLength::Variable is not yet supported (Phase 52). Use ChromosomeLength::Fixed.".into(),
                    ));
                }
            };

            let chromosomes = crate::traits::initialize_chromosomes::<U>(
                pop_size,
                genes_per_chrom,
                alleles,
                init_fn,
                Some(fitness_fn),
                0,
            );

            self.islands.push(Population::new(chromosomes));
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
    U: LinearChromosome + mutation::ValueMutable + crate::traits::RealValuedMutation,
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

        // Apply RNG seed from the first island config if configured
        let base_config = self.config_for_island(0);
        crate::rng::set_seed(base_config.rng_seed);

        // Use the first config's limit settings for the global run parameters
        let max_generations = base_config.limit_configuration.max_generations;
        let problem_solving = base_config.limit_configuration.problem_solving;
        let fitness_target = base_config.limit_configuration.fitness_target;

        self.notify(|obs| obs.on_island_run_start(0));

        for gen in 0..max_generations {
            // Evolve each island for one generation
            self.evolve_islands_one_generation(gen, problem_solving)?;

            // Check fitness target
            if let Some(target) = fitness_target {
                let best = self.global_best(problem_solving);
                let dist = (best.fitness() - target).abs();
                if dist < 1e-10 {
                    self.notify(|obs| obs.on_island_run_end(0));
                    return Ok(best);
                }
            }

            // Migration
            if gen > 0
                && self.island_config.migration_interval > 0
                && gen % self.island_config.migration_interval == 0
            {
                let migration_count = self.island_config.migration_count;
                migrate(&mut self.islands, &self.island_config, problem_solving)?;
                self.notify(|obs| obs.on_migration_triggered(gen, migration_count));
            }
        }

        self.notify(|obs| obs.on_island_run_end(0));
        Ok(self.global_best(problem_solving))
    }

    /// Performs one generation of evolution on each island.
    ///
    /// Each island uses its own `GaConfiguration` for operator parameters.
    fn evolve_islands_one_generation(
        &mut self,
        gen: usize,
        problem_solving: ProblemSolving,
    ) -> Result<(), GaError> {
        use crate::operations::{crossover, mutation, selection, survivor};
        use rand::Rng;

        let fitness_fn = self
            .fitness_fn
            .as_ref()
            .ok_or_else(|| GaError::ConfigurationError("No fitness function set".to_string()))?;

        let fitness_fn = Arc::clone(fitness_fn);

        // Clone observer Arc once before entering the parallel region.
        let observer_clone: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>> =
            self.observer.as_ref().map(Arc::clone);

        let is_maximization = matches!(problem_solving, ProblemSolving::Maximization);

        // Build a per-island config snapshot so we can move into the parallel closure.
        // Each tuple holds (selection, crossover, mutation, survivor, limit, num_threads).
        let island_configs: Vec<_> = (0..self.islands.len())
            .map(|i| {
                let cfg = self.config_for_island(i);
                (
                    cfg.selection_configuration,
                    cfg.crossover_configuration,
                    cfg.mutation_configuration.clone(),
                    cfg.survivor,
                    cfg.limit_configuration,
                    cfg.number_of_threads,
                )
            })
            .collect();

        #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
        {
            use rayon::prelude::*;
            self.islands
                .par_iter_mut()
                .enumerate()
                .try_for_each(|(idx, island)| {
                    let (
                        selection_config,
                        crossover_config,
                        mutation_config,
                        survivor_method,
                        limit_config,
                        num_threads,
                    ) = island_configs[idx].clone();
                    let pop_size = limit_config.population_size;

                    // Selection: returns Vec<Vec<usize>> parent index groups (island uses 2-parent crossover)
                    let parent_pairs =
                        selection::factory(&island.chromosomes, selection_config, num_threads, 2)?;

                    // Crossover: iterate over parent groups
                    let mut rng = crate::rng::make_rng();
                    let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);

                    let mut offspring: Vec<U> = Vec::new();
                    for group in &parent_pairs {
                        let idx_a = group[0];
                        let idx_b = group[1];
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
                            match &mutation_config.method {
                                crate::operations::Mutation::Insertion
                                | crate::operations::Mutation::Deletion => {
                                    mutation::factory_with_chromosome_length(
                                        mutation_config.method.clone(),
                                        child,
                                        None,
                                    )?;
                                }
                                other => {
                                    other.mutate(child, other)?;
                                }
                            }
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

                    // Fire per-island generation hook if an observer is attached
                    if let Some(ref obs) = observer_clone {
                        let fitness_values: Vec<f64> =
                            island.chromosomes.iter().map(|c| c.fitness()).collect();
                        let stats = GenerationStats::from_fitness_values(
                            gen,
                            &fitness_values,
                            is_maximization,
                        );
                        obs.on_island_generation_end(idx, gen, &stats);
                    }

                    Ok(())
                })
        } // end #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]

        #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
        {
            self.islands
                .iter_mut()
                .enumerate()
                .try_for_each(|(idx, island)| {
                    let (
                        selection_config,
                        crossover_config,
                        mutation_config,
                        survivor_method,
                        limit_config,
                        num_threads,
                    ) = island_configs[idx].clone();
                    let pop_size = limit_config.population_size;

                    // Selection: returns Vec<Vec<usize>> parent index groups (island uses 2-parent crossover)
                    let parent_pairs =
                        selection::factory(&island.chromosomes, selection_config, num_threads, 2)?;

                    // Crossover: iterate over parent groups
                    let mut rng = crate::rng::make_rng();
                    let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);

                    let mut offspring: Vec<U> = Vec::new();
                    for group in &parent_pairs {
                        let idx_a = group[0];
                        let idx_b = group[1];
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
                            match &mutation_config.method {
                                crate::operations::Mutation::Insertion
                                | crate::operations::Mutation::Deletion => {
                                    mutation::factory_with_chromosome_length(
                                        mutation_config.method.clone(),
                                        child,
                                        None,
                                    )?;
                                }
                                other => {
                                    other.mutate(child, other)?;
                                }
                            }
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

                    // Fire per-island generation hook if an observer is attached
                    if let Some(ref obs) = observer_clone {
                        let fitness_values: Vec<f64> =
                            island.chromosomes.iter().map(|c| c.fitness()).collect();
                        let stats = GenerationStats::from_fitness_values(
                            gen,
                            &fitness_values,
                            is_maximization,
                        );
                        obs.on_island_generation_end(idx, gen, &stats);
                    }

                    Ok(())
                })
        } // end #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
    }
}
