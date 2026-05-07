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
use crate::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use crate::nsga2::crowding_distance::assign_crowding_distance;
use crate::nsga2::non_dominated_sort::{
    assign_ranks, non_dominated_sort_constrained, non_dominated_sort_with_directions,
};
use crate::nsga2::pareto::{ParetoFront, ParetoIndividual};
use crate::observer::Nsga2Observer;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

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
    /// Optional constraint functions. Each returns a violation amount (> 0 means violated).
    /// The total constraint violation is the sum of all individual violations.
    pub constraint_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for NSGA-II-specific events.
    pub observer: Option<Arc<dyn Nsga2Observer<U> + Send + Sync>>,
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
            constraint_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives NSGA-II-specific hooks.
    ///
    /// The observer is stored as an `Arc` for thread-safe sharing. All hooks receive
    /// `&self`, so observers needing interior mutability should use `Mutex` or atomics.
    ///
    /// See [`Nsga2Observer`] for the hook contract.
    pub fn with_observer(mut self, obs: Arc<dyn Nsga2Observer<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    fn notify<F: FnOnce(&dyn Nsga2Observer<U>)>(&self, f: F) {
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
    ///
    /// Each function evaluates one objective given the chromosome's DNA.
    pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.objective_fns = fns.into_iter().map(Arc::from).collect();
        self
    }

    /// Sets the constraint functions.
    ///
    /// Each function returns a violation amount for the given chromosome's DNA.
    /// A return value of `0.0` (or negative) means the constraint is satisfied.
    /// A positive value indicates the magnitude of the violation.
    /// The total constraint violation for an individual is the sum of all functions' outputs
    /// (clamped to >= 0).
    pub fn with_constraint_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.constraint_fns = fns.into_iter().map(Arc::from).collect();
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
        if !self.nsga2_config.objective_directions.is_empty()
            && self.nsga2_config.objective_directions.len() != self.nsga2_config.num_objectives
        {
            return Err(GaError::InvalidNsga2Configuration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.nsga2_config.objective_directions.len(),
                self.nsga2_config.num_objectives
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

        // Apply RNG seed if configured
        crate::rng::set_seed(self.ga_config.rng_seed);

        let pop_size = self.nsga2_config.population_size;
        let max_gens = self.nsga2_config.max_generations;
        let directions = self.nsga2_config.effective_directions();
        let has_constraints = !self.constraint_fns.is_empty();

        // Initialize population
        let mut population = self.initialize_population()?;

        for gen in 0..max_gens {
            // Non-dominated sorting (direction-aware, optionally constrained)
            let t_sort: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                { Some(Instant::now()) }
                #[cfg(target_arch = "wasm32")]
                { None }
            } else {
                None
            };
            let fronts = self.perform_sorting(&population, &directions, has_constraints);
            if let Some(start) = t_sort {
                self.notify(|obs| {
                    obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0)
                });
            }

            // Assign ranks
            let mut ranks = vec![0usize; population.len()];
            assign_ranks(&mut ranks, &fronts);
            for (i, &r) in ranks.iter().enumerate() {
                population[i].rank = r;
            }

            // Assign crowding distance per front
            let t_crowd: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                { Some(Instant::now()) }
                #[cfg(target_arch = "wasm32")]
                { None }
            } else {
                None
            };
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
            if let Some(start) = t_crowd {
                self.notify(|obs| {
                    obs.on_crowding_distance_calculated(gen, start.elapsed().as_secs_f64() * 1000.0)
                });
            }
            self.notify(|obs| obs.on_pareto_front_assigned(gen, fronts.len(), population.len()));

            // Binary tournament selection + crossover + mutation to create offspring
            let offspring = self.create_offspring(&population)?;

            // Combine parent + offspring
            population.extend(offspring);

            // Environmental selection: sort by (rank asc, crowding_distance desc), truncate
            // Re-evaluate ranks and crowding for combined population
            let combined_fronts = self.perform_sorting(&population, &directions, has_constraints);

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
        }

        // Extract the first Pareto front from the final population.
        // The population is already sorted by rank from environmental selection,
        // so individuals with rank 0 are the first Pareto front.
        let front_individuals: Vec<ParetoIndividual<U>> =
            population.into_iter().filter(|ind| ind.rank == 0).collect();

        Ok(ParetoFront::new(front_individuals))
    }

    /// Performs non-dominated sorting using the appropriate strategy.
    fn perform_sorting(
        &self,
        population: &[ParetoIndividual<U>],
        directions: &[ObjectiveDirection],
        has_constraints: bool,
    ) -> Vec<Vec<usize>> {
        let all_objectives: Vec<&[f64]> = population
            .iter()
            .map(|ind| ind.objectives.as_slice())
            .collect();

        if has_constraints {
            let violations: Vec<f64> = population
                .iter()
                .map(|ind| ind.constraint_violation)
                .collect();
            non_dominated_sort_constrained(&all_objectives, &violations, directions)
        } else {
            non_dominated_sort_with_directions(&all_objectives, directions)
        }
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
        let constraint_fns = &self.constraint_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let population = chromosomes
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
                let mut ind = ParetoIndividual::new(chrom, objectives);
                ind.constraint_violation = constraint_violation;
                ind
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let population = chromosomes
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
                let mut ind = ParetoIndividual::new(chrom, objectives);
                ind.constraint_violation = constraint_violation;
                ind
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

        let mut rng = crate::rng::make_rng();
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
                    if mutation_config.method == crate::operations::Mutation::Differential {
                        return Err(GaError::MutationError(
                            "Differential mutation is not supported in NSGA-II; \
                             use Cauchy, LevyFlight, Polynomial, or a standard mutation method instead."
                                .to_string(),
                        ));
                    } else if mutation_config.method == crate::operations::Mutation::Cauchy {
                        mutation::factory_with_params(
                            mutation_config.method,
                            child,
                            mutation_config.cauchy_scale,
                            None,
                        )?;
                    } else if mutation_config.method == crate::operations::Mutation::LevyFlight {
                        mutation::factory_with_params(
                            mutation_config.method,
                            child,
                            None,
                            mutation_config.levy_alpha,
                        )?;
                    } else if mutation_config.method == crate::operations::Mutation::Polynomial {
                        let eta = mutation_config.polynomial_eta.or(mutation_config.step);
                        mutation::factory_with_params(
                            mutation_config.method,
                            child,
                            eta,
                            None,
                        )?;
                    } else {
                        mutation::factory_with_params(
                            mutation_config.method,
                            child,
                            mutation_config.step,
                            mutation_config.sigma,
                        )?;
                    }
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
        let constraint_fns = &self.constraint_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let offspring = raw_offspring
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
                let mut ind = ParetoIndividual::new(chrom, objectives);
                ind.constraint_violation = constraint_violation;
                ind
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let offspring = raw_offspring
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
                let mut ind = ParetoIndividual::new(chrom, objectives);
                ind.constraint_violation = constraint_violation;
                ind
            })
            .collect();

        Ok(offspring)
    }

    /// Binary tournament selection: picks two random individuals and returns the
    /// index of the better one.
    ///
    /// When constraint violations are present, uses constrained tournament rules:
    /// 1. A feasible individual beats an infeasible one.
    /// 2. Among two infeasible individuals, the one with lower violation wins.
    /// 3. Among two feasible individuals (or when no constraints), lower rank wins,
    ///    then higher crowding distance breaks ties.
    fn binary_tournament(&self, population: &[ParetoIndividual<U>], rng: &mut impl Rng) -> usize {
        let n = population.len();
        let i = rng.random_range(0..n);
        let j = rng.random_range(0..n);

        let a = &population[i];
        let b = &population[j];

        // Constrained tournament: feasible beats infeasible
        let a_feasible = a.is_feasible();
        let b_feasible = b.is_feasible();

        match (a_feasible, b_feasible) {
            (true, false) => return i,
            (false, true) => return j,
            (false, false) => {
                // Both infeasible: prefer smaller constraint violation
                return if a.constraint_violation < b.constraint_violation {
                    i
                } else {
                    j
                };
            }
            (true, true) => {} // fall through to rank/crowding comparison
        }

        // Both feasible (or no constraints): standard rank + crowding distance
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

/// Evaluates all constraint functions for a given DNA and returns the total constraint violation.
///
/// Each constraint function returns a violation amount. Positive values are clamped to >= 0
/// and summed. If there are no constraint functions, returns 0.0.
fn evaluate_constraints<G>(dna: &[G], constraint_fns: &[Arc<ObjectiveFn<G>>]) -> f64
where
    G: Send + Sync,
{
    if constraint_fns.is_empty() {
        return 0.0;
    }
    constraint_fns.iter().map(|f| f(dna).max(0.0)).sum()
}
