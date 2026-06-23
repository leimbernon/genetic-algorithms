//! SMS-EMOA — S-Metric Selection Evolutionary Multi-Objective Algorithm.
//!
//! ## Description
//!
//! SMS-EMOA (Beume, Naujoks & Emmerich 2007) is a **steady-state** (μ+1)
//! multi-objective evolutionary algorithm that uses **hypervolume contribution**
//! to decide which individual is removed each generation. At each step, one
//! offspring is created via selection + crossover + mutation, then the
//! individual with the smallest contribution to the hypervolume of the worst
//! non-dominated front is removed.
//!
//! Per generation, SMS-EMOA:
//! 1. Create one offspring via binary tournament + crossover + mutation.
//! 2. Evaluate offspring objectives.
//! 3. Merge offspring into the population (μ+1).
//! 4. Non-dominated sort the population.
//! 5. Identify the **worst front** (highest rank).
//! 6. For each member of the worst front, compute **hypervolume contribution**
//!    — the reduction in hypervolume that removing that individual would cause.
//! 7. Remove the individual with the smallest hypervolume contribution
//!    (back to μ individuals).
//!
//! The hypervolume indicator simultaneously captures **convergence** (closer to
//! the true Pareto front → larger hypervolume) and **diversity** (well-spread
//! solutions → larger hypervolume), making SMS-EMOA a powerful unified approach.
//!
//! ## When to Use
//!
//! - **Problem type:** Multi-objective (2+ objectives, best at 2–3)
//! - **Variable type:** Continuous (real-valued), binary
//! - **Population structure:** Single population, steady-state
//! - **Key strength:** Hypervolume-based selection naturally balances
//!   convergence and diversity without requiring additional diversity
//!   preservation mechanisms.
//! - **Key weakness:** Hypervolume computation is expensive (O(N·2^M) for
//!   M objectives). Practical for 2–3 objectives; becomes prohibitive at 4+
//!   objectives. Requires a reference point that strictly dominates all
//!   solutions.
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Default | Description |
//! |-----------|------|---------|-------------|
//! | `num_objectives` | `usize` | `2` | Number of objectives (≥ 2). |
//! | `population_size` | `usize` | `100` | Population size (≥ 2). |
//! | `max_generations` | `usize` | `250` | Maximum generations. |
//! | `init_fn` | `Fn` | — | Chromosome initialisation. |
//! | `objective_fns` | `Vec<ObjectiveFn>` | — | One per objective. |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Default | Description |
//! |-----------|------|---------|-------------|
//! | `objective_directions` | `Vec<ObjectiveDirection>` | All `Minimize` | Per-objective Min/Max. |
//! | `hypervolume_reference_point` | `Option<Vec<f64>>` | `None` (auto-computed) | Reference for HV calc. |
//! | `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
//! | `observer` | `SmsEmoaObserver<U>` | `None` | Lifecycle observer. |
//!
//! ## Complete Example
//!
//! ```rust,no_run
//! // no_run: SMS-EMOA engine example — illustrative API usage, not a runnable benchmark
//! use genetic_algorithms::sms_emoa::SmsEmoaGa;
//! use genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration;
//! use genetic_algorithms::configuration::GaConfiguration;
//!
//! let sms_config = SmsEmoaConfiguration::new()
//!     .with_num_objectives(2)
//!     .with_population_size(100)
//!     .with_max_generations(250);
//!
//! let ga_config = GaConfiguration::default();
//! // let mut sms_emoa = SmsEmoaGa::<MyChromosome>::new(sms_config, ga_config)
//! //     .with_initialization_fn(|n, alleles, repeat| { /* ... */ })
//! //     .with_objective_fns(vec![
//! //         Box::new(|dna| { /* ZDT1 f1 */ 0.0 }),
//! //         Box::new(|dna| { /* ZDT1 f2 */ 0.0 }),
//! //     ])
//! //     .build()?;
//! //
//! // let pareto_front = sms_emoa.run()?;
//! // println!("Front size: {}", pareto_front.len());
//! ```
//!
//! ## Configuration Tips
//!
//! - If `hypervolume_reference_point` is not set, it is auto-computed from the
//!   initial population as `max(objective) + 1.0` per dimension. For better
//!   control, set it explicitly to a point that strictly dominates all expected
//!   solutions.
//! - SMS-EMOA produces one offspring per generation, so `max_generations` is
//!   the number of fitness evaluations. A typical setting is 10× population_size.
//! - Hypervolume computation allocates O(2^M) space; for M > 4 the `indicators`
//!   module is more efficient than the naive per-individual contribution calc.
//!
//! ## When to Choose This vs IBEA
//!
//! | Criterion | SMS-EMOA | IBEA |
//! |-----------|----------|------|
//! | Selection | Hypervolume contribution | I_eps+ indicator fitness |
//! | Indicator | Hypervolume (quality metric) | Additive epsilon (dominance shift) |
//! | Reference point | Required (auto or explicit) | Not needed |
//! | Objectives | 2–3 (HV cost) | 2+ (pairwise O(N²)) |
//! | Environmental sel. | One removal per gen | Iterative removal + recalculation |
//!
//! ## References
//!
//! - Beume, N., Naujoks, B., & Emmerich, M. (2007). SMS-EMOA: Multiobjective
//!   selection based on dominated hypervolume. _European Journal of Operational
//!   Research_, 181(3), 1653–1669.

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::indicators::hypervolume;
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::observer::SmsEmoaObserver;
use crate::operations::{crossover, mutation};
use crate::sms_emoa::configuration::SmsEmoaConfiguration;
use crate::traits::{InitializationFn, LinearChromosome, MutationOperator, VectorFitness};
use rand::Rng;
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// SMS-EMOA steady-state multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::sms_emoa::SmsEmoaGa;
/// use genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration;
/// use genetic_algorithms::configuration::GaConfiguration;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let sms_config = SmsEmoaConfiguration::default();
/// let ga_config = GaConfiguration::default();
/// let engine = SmsEmoaGa::<RangeChromosome<f64>>::new(sms_config, ga_config);
/// ```
pub struct SmsEmoaGa<U>
where
    U: LinearChromosome,
{
    /// SMS-EMOA-specific configuration.
    pub sms_config: SmsEmoaConfiguration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Optional structured lifecycle observer for SMS-EMOA-specific events.
    pub observer: Option<Arc<dyn SmsEmoaObserver<U> + Send + Sync>>,
}

impl<U> SmsEmoaGa<U>
where
    U: LinearChromosome,
{
    /// Creates a new `SmsEmoaGa` with the given configurations.
    pub fn new(sms_config: SmsEmoaConfiguration, ga_config: GaConfiguration) -> Self {
        SmsEmoaGa {
            sms_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives SMS-EMOA-specific hooks.
    pub fn with_observer(mut self, obs: Arc<dyn SmsEmoaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    pub(crate) fn notify<F: FnOnce(&dyn SmsEmoaObserver<U>)>(&self, f: F) {
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
        F: Fn(usize, Option<&[U::Gene]>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(f));
        self
    }

    /// Validates configuration and returns a ready-to-run instance.
    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the SMS-EMOA configuration.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.sms_config.num_objectives < 2 {
            return Err(GaError::InvalidSmsEmoaConfiguration(
                "num_objectives must be >= 2 for hypervolume-based selection".to_string(),
            ));
        }
        if self.sms_config.population_size < 2 {
            return Err(GaError::InvalidSmsEmoaConfiguration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidSmsEmoaConfiguration(
                "initialization_fn is required".to_string(),
            ));
        }
        if !self.sms_config.objective_directions.is_empty()
            && self.sms_config.objective_directions.len() != self.sms_config.num_objectives
        {
            return Err(GaError::InvalidSmsEmoaConfiguration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.sms_config.objective_directions.len(),
                self.sms_config.num_objectives
            )));
        }
        Ok(())
    }

    /// Computes hypervolume contribution for each individual in a set of points.
    ///
    /// The contribution of point `p` is: HV(front) - HV(front \ {p}).
    /// Uses the Phase 39 `hypervolume()` function with the given reference point.
    fn compute_hypervolume_contributions(
        points: &[ParetoIndividual<U>],
        reference_point: &[f64],
    ) -> Result<Vec<f64>, GaError> {
        let n = points.len();
        if n == 1 {
            // Only one point: the contribution is the full hypervolume
            return Ok(vec![hypervolume(
                &[points[0].objectives.clone()],
                reference_point,
            )?]);
        }

        let obj_slices: Vec<Vec<f64>> = points.iter().map(|p| p.objectives.clone()).collect();
        let total_hv = hypervolume(&obj_slices, reference_point)?;

        let mut contributions = Vec::with_capacity(n);
        for i in 0..n {
            let without: Vec<Vec<f64>> = obj_slices
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| v.clone())
                .collect();
            let hv_without = hypervolume(&without, reference_point)?;
            contributions.push(total_hv - hv_without);
        }

        Ok(contributions)
    }
}

impl<U> SmsEmoaGa<U>
where
    U: LinearChromosome
        + mutation::ValueMutable
        + VectorFitness
        + crate::traits::RealValuedMutation,
{
    /// Initializes the population with random chromosomes and evaluates objectives in parallel.
    fn initialize_population(&self) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let pop_size = self.sms_config.population_size;
        let genes_per_chrom = match self.ga_config.limit_configuration.chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(n) => n,
            crate::chromosomes::ChromosomeLength::Variable { .. } => {
                return Err(GaError::InvalidSmsEmoaConfiguration(
                    "ChromosomeLength::Variable is not yet supported (Phase 52). Use ChromosomeLength::Fixed.".into(),
                ));
            }
        };

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        let chromosomes: Vec<U> = crate::traits::initialize_chromosomes(
            pop_size,
            genes_per_chrom,
            alleles,
            init_fn,
            None,
            0,
        );

        #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_par_iter()
            .map(|mut chrom| {
                chrom.calculate_fitness();
                let objectives = chrom.fitness_values().to_vec();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_iter()
            .map(|mut chrom| {
                chrom.calculate_fitness();
                let objectives = chrom.fitness_values().to_vec();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(population)
    }

    /// Creates one offspring via binary tournament selection, crossover, and mutation.
    fn create_one_offspring(&self, population: &[ParetoIndividual<U>]) -> Result<U, GaError> {
        let mut rng = crate::rng::make_rng();
        let n = population.len();

        let p1_idx = rng.random_range(0..n);
        let p2_idx = rng.random_range(0..n);
        let parent_a = &population[p1_idx].chromosome;
        let parent_b = &population[p2_idx].chromosome;

        let crossover_config = self.ga_config.crossover_configuration;
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let p: f64 = rng.random();
        let mut children = if p <= crossover_prob {
            crossover::factory(parent_a, parent_b, crossover_config)?
        } else {
            vec![parent_a.clone()]
        };

        let mutation_config = &self.ga_config.mutation_configuration;
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);
        for child in children.iter_mut() {
            let mp: f64 = rng.random();
            if mp <= mut_prob {
                mutation_config
                    .method
                    .mutate(child, &mutation_config.method)?;
            }
        }

        // Return first child (steady-state: one offspring per generation)
        Ok(children
            .into_iter()
            .next()
            .unwrap_or_else(|| parent_a.clone()))
    }

    /// Runs the SMS-EMOA algorithm and returns the Pareto front.
    ///
    /// Implements Beume, Naujoks & Emmerich 2007 (SMS-EMOA):
    /// 1. Validate configuration, extract directions.
    /// 2. Initialize population and evaluate objectives.
    /// 3. For each generation:
    ///    a. Select 2 parents via binary tournament + crossover + mutation -> 1 offspring.
    ///    b. Evaluate offspring objectives.
    ///    c. Merge offspring into population (mu+1).
    ///    d. Non-dominated sort the population.
    ///    e. Identify worst front, compute HV contributions for worst-front members.
    ///    f. Remove individual with smallest HV contribution (steady-state back to mu).
    ///    g. Notify observer hooks.
    /// 4. Post-hoc non-dominated sort; return rank-0 as ParetoFront.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        crate::rng::set_seed(self.ga_config.rng_seed);

        let max_gens = self.sms_config.max_generations;
        let directions = self.sms_config.effective_directions();

        // Initialize population
        let mut population = self.initialize_population()?;

        // Runtime objective-count guard
        if let Some(first) = population.first() {
            let got = first.chromosome.fitness_values().len();
            if got != self.sms_config.num_objectives {
                return Err(GaError::InvalidSmsEmoaConfiguration(format!(
                    "Expected {} objectives from fitness_values(), got {}",
                    self.sms_config.num_objectives, got
                )));
            }
        }

        // Auto-compute reference point if not provided
        // Use max of each objective across the initial population + 1.0 margin
        let reference_point: Vec<f64> =
            if let Some(ref rp) = self.sms_config.hypervolume_reference_point {
                rp.clone()
            } else {
                let mut ref_pt = vec![f64::NEG_INFINITY; self.sms_config.num_objectives];
                for ind in &population {
                    for (j, &val) in ind.objectives.iter().enumerate() {
                        if val > ref_pt[j] {
                            ref_pt[j] = val;
                        }
                    }
                }
                // Add margin of 1.0 per objective to ensure strict domination
                for val in ref_pt.iter_mut().take(self.sms_config.num_objectives) {
                    *val += 1.0;
                }
                ref_pt
            };

        // Generation loop
        for gen in 0..max_gens {
            // (a) Create one offspring via binary tournament + crossover + mutation
            let offspring_chrom = self.create_one_offspring(&population)?;

            // (b) Evaluate offspring objectives
            let mut offspring_chrom = offspring_chrom;
            offspring_chrom.calculate_fitness();
            let offspring_obj = offspring_chrom.fitness_values().to_vec();
            let offspring = ParetoIndividual::new(offspring_chrom, offspring_obj);

            // (c) Merge offspring into population (mu+1)
            population.push(offspring);

            // (d) Non-dominated sort
            let obj_slices: Vec<&[f64]> = population
                .iter()
                .map(|ind| ind.objectives.as_slice())
                .collect();
            let fronts =
                crate::multi_objective::non_dominated_sort::non_dominated_sort_with_directions(
                    &obj_slices,
                    &directions,
                );
            let mut ranks = vec![0usize; population.len()];
            crate::multi_objective::non_dominated_sort::assign_ranks(&mut ranks, &fronts);
            for (i, &r) in ranks.iter().enumerate() {
                population[i].rank = r;
            }

            // (e) Identify worst front indices
            let max_rank = *ranks.iter().max().unwrap_or(&0);
            let worst_front_indices: Vec<usize> = (0..population.len())
                .filter(|&i| ranks[i] == max_rank)
                .collect();

            // Timing for observer
            let t_hvc: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Some(Instant::now())
                }
                #[cfg(target_arch = "wasm32")]
                {
                    None
                }
            } else {
                None
            };

            // Compute HV contributions for worst front members
            let worst_front_individuals: Vec<&ParetoIndividual<U>> = worst_front_indices
                .iter()
                .map(|&i| &population[i])
                .collect();
            let hv_contributions = Self::compute_hypervolume_contributions(
                &worst_front_individuals
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                &reference_point,
            )?;

            // Notify observer -- on_hypervolume_contribution_assigned
            if let Some(start) = t_hvc {
                self.notify(|obs| {
                    obs.on_hypervolume_contribution_assigned(
                        gen,
                        start.elapsed().as_secs_f64() * 1000.0,
                        worst_front_indices.len(),
                    )
                });
            }

            // (f) Remove individual with smallest HV contribution
            let min_idx = hv_contributions
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            let remove_pop_idx = worst_front_indices[min_idx];
            population.remove(remove_pop_idx);

            // (g) Notify observer -- on_steady_state_removal
            self.notify(|obs| obs.on_steady_state_removal(gen, population.len()));
        }

        // Post-hoc non-dominated sort over final population -> ParetoFront
        let obj_slices: Vec<&[f64]> = population
            .iter()
            .map(|ind| ind.objectives.as_slice())
            .collect();
        let fronts = crate::multi_objective::non_dominated_sort::non_dominated_sort_with_directions(
            &obj_slices,
            &directions,
        );
        let mut ranks = vec![0usize; population.len()];
        crate::multi_objective::non_dominated_sort::assign_ranks(&mut ranks, &fronts);
        for (i, &r) in ranks.iter().enumerate() {
            population[i].rank = r;
        }
        let front_individuals: Vec<ParetoIndividual<U>> =
            population.into_iter().filter(|ind| ind.rank == 0).collect();
        Ok(ParetoFront::new(front_individuals))
    }
}
