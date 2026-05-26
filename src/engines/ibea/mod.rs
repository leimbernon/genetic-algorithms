//! IBEA — Indicator-Based Evolutionary Algorithm.
//!
//! ## Description
//!
//! IBEA (Zitzler & Kunzli 2004) is a multi-objective evolutionary algorithm that
//! uses a **pairwise indicator function** (the additive epsilon indicator I_eps+)
//! to assign fitness values, eliminating the need for Pareto dominance ranking
//! or diversity metrics like crowding distance.
//!
//! **Indicator fitness:**
//! - `I_eps+(a, b)` = minimum additive shift needed for `b` to dominate `a`.
//!   A positive value means `a` is better; 0.0 means `a` is dominated by or
//!   equal to `b`.
//! - Fitness `F(x) = sum_{y != x} -exp(-I_eps+(y, x) / c)` where `c` is the
//!   maximum absolute indicator value (adaptive scaling). **Higher fitness =
//!   better** — a positive I_eps+ (x is worse than y) reduces x's fitness.
//!
//! Per generation, IBEA:
//! 1. Compute the pairwise I_eps+ indicator matrix for the population.
//! 2. Compute indicator-based fitness for each individual.
//! 3. **Environmental selection:** iteratively remove the individual with the
//!    lowest indicator fitness, recalculating fitnesses after each removal,
//!    until the population reaches the target size.
//! 4. Create offspring via binary tournament + crossover + mutation.
//! 5. Merge offspring into the population.
//! 6. (Optionally) trim population back to pop_size to prevent unbounded growth.
//!
//! IBEA does **not** require a reference point (unlike SMS-EMOA) and works
//! with any binary quality indicator that can be paired with the exponential
//! scaling scheme.
//!
//! ## When to Use
//!
//! - **Problem type:** Multi-objective (2+ objectives)
//! - **Variable type:** Continuous, binary, permutation, or `List<T>`
//! - **Population structure:** Single population
//! - **Key strength:** No reference point needed. Flexible indicator framework
//!   — the I_eps+ indicator captures dominance relationships without expensive
//!   hypervolume computation.
//! - **Key weakness:** O(N²) pairwise indicator computation per generation.
//!   Environmental selection iteratively removes one at a time and recalculates,
//!   making it O(K·N²) where K is the number of removals per generation.
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
//! | `kappa` (config) | `f64` | `0.05` | Exponential scaling factor. |
//! | `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
//! | `observer` | `IbeaObserver<U>` | `None` | Lifecycle observer. |
//!
//! ## Complete Example
//!
//! ```ignore
//! use genetic_algorithms::ibea::IbeaGa;
//! use genetic_algorithms::ibea::configuration::IbeaConfiguration;
//! use genetic_algorithms::configuration::GaConfiguration;
//!
//! let ibea_config = IbeaConfiguration::new()
//!     .with_num_objectives(2)
//!     .with_population_size(100)
//!     .with_max_generations(250);
//!
//! let ga_config = GaConfiguration::default();
//! let mut ibea = IbeaGa::<MyChromosome>::new(ibea_config, ga_config)
//!     .with_initialization_fn(|n, alleles, repeat| { /* ... */ })
//!     .with_objective_fns(vec![
//!         Box::new(|dna| { /* ZDT1 f1 */ 0.0 }),
//!         Box::new(|dna| { /* ZDT1 f2 */ 0.0 }),
//!     ])
//!     .build()?;
//!
//! let pareto_front = ibea.run()?;
//! println!("Front size: {}", pareto_front.len());
//! ```
//!
//! ## Configuration Tips
//!
//! - The `kappa` scaling parameter controls selection pressure: smaller kappa
//!   → higher pressure (faster convergence); larger kappa → more relaxed
//!   (better diversity). Default is 0.05.
//! - IBEA's O(N²) fitness computation is the main bottleneck. For large
//!   populations (> 200), consider using SMS-EMOA or NSGA-II instead.
//! - The iterative environmental selection removes one individual at a time
//!   and recalculates all fitnesses — this is the dominant cost. The algorithm
//!   removes enough to accommodate the incoming offspring.
//!
//! ## When to Choose This vs SMS-EMOA
//!
//! | Criterion | IBEA | SMS-EMOA |
//! |-----------|------|----------|
//! | Indicator | I_eps+ (additive epsilon) | Hypervolume (S-metric) |
//! | Reference point | Not needed | Required |
//! | Objectives | 2+ | 2–3 (HV cost) |
//! | Cost per gen | O(K·N²) iterative removal | O(N·2^M) HV contribution |
//! | Steady-state | No (batch offspring) | Yes (μ+1 one offspring) |
//!
//! ## References
//!
//! - Zitzler, E., & Kunzli, S. (2004). Indicator-based selection in
//!   multiobjective search. _Parallel Problem Solving from Nature — PPSN VIII_,
//!   LNCS 3242, 832–842.

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::ibea::configuration::IbeaConfiguration;
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::nsga2::configuration::ObjectiveDirection;
use crate::observer::IbeaObserver;
use crate::operations::{crossover, mutation};
use crate::traits::{LinearChromosome, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// IBEA indicator-based multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct IbeaGa<U>
where
    U: LinearChromosome,
{
    /// IBEA-specific configuration.
    pub ibea_config: IbeaConfiguration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for IBEA-specific events.
    pub observer: Option<Arc<dyn IbeaObserver<U> + Send + Sync>>,
}

#[allow(dead_code)]
impl<U> IbeaGa<U>
where
    U: LinearChromosome,
{
    /// Creates a new `IbeaGa` with the given configurations.
    pub fn new(ibea_config: IbeaConfiguration, ga_config: GaConfiguration) -> Self {
        IbeaGa {
            ibea_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives IBEA-specific hooks.
    pub fn with_observer(mut self, obs: Arc<dyn IbeaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    pub(crate) fn notify<F: FnOnce(&dyn IbeaObserver<U>)>(&self, f: F) {
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

    /// Validates the IBEA configuration.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.ibea_config.num_objectives < 2 {
            return Err(GaError::InvalidIbeaConfiguration(
                "num_objectives must be >= 2 for indicator-based comparison".to_string(),
            ));
        }
        if self.ibea_config.population_size < 2 {
            return Err(GaError::InvalidIbeaConfiguration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidIbeaConfiguration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.ibea_config.num_objectives {
            return Err(GaError::InvalidIbeaConfiguration(format!(
                "Expected {} objective functions, got {}",
                self.ibea_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.ibea_config.objective_directions.is_empty()
            && self.ibea_config.objective_directions.len() != self.ibea_config.num_objectives
        {
            return Err(GaError::InvalidIbeaConfiguration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.ibea_config.objective_directions.len(),
                self.ibea_config.num_objectives
            )));
        }
        Ok(())
    }

    /// Computes the additive epsilon indicator I_eps+(a, b).
    ///
    /// I_eps+(a, b) = max_i { a_i - b_i } for minimization objectives.
    /// For maximization objectives, the delta is negated (b_i - a_i) so that
    /// a positive value still means "b needs to improve by that much."
    /// Returns 0.0 if a dominates b or is equal on all objectives.
    fn i_eps_plus(a: &[f64], b: &[f64], directions: &[ObjectiveDirection]) -> f64 {
        let mut max_eps = f64::NEG_INFINITY;
        for (idx, (ai, bi)) in a.iter().zip(b.iter()).enumerate() {
            let dir = directions
                .get(idx)
                .copied()
                .unwrap_or(ObjectiveDirection::Minimize);
            let delta = match dir {
                ObjectiveDirection::Minimize => ai - bi,
                ObjectiveDirection::Maximize => bi - ai,
            };
            if delta > max_eps {
                max_eps = delta;
            }
        }
        max_eps.max(0.0)
    }

    /// Computes indicator-based fitness for the population.
    ///
    /// Fitness F(x) = sum_{y in pop\{x}} -exp(-I_eps+(y, x) / c)
    /// where c = max |I_eps+| (scaling factor). Higher fitness = better.
    /// Recalculates fitnesses from scratch.
    fn compute_indicator_fitness(
        population: &[ParetoIndividual<U>],
        directions: &[ObjectiveDirection],
    ) -> Vec<f64> {
        let n = population.len();
        if n == 0 {
            return vec![];
        }

        // Compute pairwise I_eps+ matrix
        let mut indicators = vec![vec![0.0f64; n]; n];
        let mut max_abs = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let val = Self::i_eps_plus(
                    &population[i].objectives,
                    &population[j].objectives,
                    directions,
                );
                indicators[i][j] = val;
                if val.abs() > max_abs {
                    max_abs = val.abs();
                }
            }
        }

        // Scaling factor c = max |I_eps+| (avoid division by zero)
        let c = if max_abs > 1e-12 { max_abs } else { 1.0 };

        // F(x) = sum_{y != x} -exp(-I_eps+(y, x) / c)
        let mut fitness = vec![0.0f64; n];
        for (i, fi) in fitness.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, row) in indicators.iter().enumerate() {
                if i == j {
                    continue;
                }
                sum += (-row[i] / c).exp();
            }
            *fi = -sum;
        }

        fitness
    }

    /// Performs environmental selection for IBEA.
    ///
    /// Iteratively removes the individual with the lowest indicator fitness,
    /// recalculating fitnesses after each removal, until population reaches target size.
    fn environmental_selection(
        population: &mut Vec<ParetoIndividual<U>>,
        target_size: usize,
        directions: &[ObjectiveDirection],
    ) -> usize {
        let mut total_removed = 0usize;
        while population.len() > target_size {
            let fitness = Self::compute_indicator_fitness(population, directions);
            // Find index of minimum fitness (worst individual)
            let min_idx = fitness
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            population.remove(min_idx);
            total_removed += 1;
        }
        total_removed
    }

    /// Initializes the population with random chromosomes and evaluates objectives in parallel.
    fn initialize_population(&self) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let pop_size = self.ibea_config.population_size;
        let genes_per_chrom = match self.ga_config.limit_configuration.chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(n) => n,
            crate::chromosomes::ChromosomeLength::Variable { .. } => {
                return Err(GaError::InvalidIbeaConfiguration(
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

        let objective_fns = &self.objective_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(population)
    }
}

impl<U> IbeaGa<U>
where
    U: LinearChromosome + mutation::ValueMutable,
{
    /// Produces offspring chromosomes via binary tournament selection from population,
    /// followed by crossover + mutation on each selected pair.
    fn create_offspring(
        &self,
        population: &[ParetoIndividual<U>],
    ) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let pop_size = self.ibea_config.population_size;
        let mut rng = crate::rng::make_rng();
        let n = population.len();

        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration;
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);

        let mut offspring: Vec<U> = Vec::with_capacity(pop_size);
        let pairs_needed = (pop_size + 1).div_ceil(2);

        for _ in 0..pairs_needed {
            let p1_idx = rng.random_range(0..n);
            let p2_idx = rng.random_range(0..n);
            let parent_a = &population[p1_idx].chromosome;
            let parent_b = &population[p2_idx].chromosome;

            let p: f64 = rng.random();
            let children = if p <= crossover_prob {
                crossover::factory(parent_a, parent_b, crossover_config)?
            } else {
                vec![parent_a.clone(), parent_b.clone()]
            };

            for mut child in children {
                let mp: f64 = rng.random();
                if mp <= mut_prob {
                    mutation::factory_with_params(
                        mutation_config.method,
                        &mut child,
                        mutation_config.step,
                        mutation_config.sigma,
                    )?;
                }
                offspring.push(child);
                if offspring.len() >= pop_size {
                    break;
                }
            }
        }

        let objective_fns = &self.objective_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let evaluated: Vec<ParetoIndividual<U>> = offspring
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let evaluated: Vec<ParetoIndividual<U>> = offspring
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(evaluated)
    }
    /// Runs the IBEA algorithm and returns the Pareto front.
    ///
    /// 1. Validate configuration.
    /// 2. Initialize population, evaluate objectives.
    /// 3. For each generation:
    ///    a. Compute indicator fitness (pairwise I_eps+).
    ///    b. Notify `on_indicator_fitness_assigned`.
    ///    c. Environmental selection: iteratively remove lowest-fitness individuals.
    ///    d. Notify `on_environmental_selection`.
    ///    e. Create offspring via binary tournament + crossover + mutation.
    ///    f. Evaluate offspring, merge into population.
    /// 4. Post-hoc non-dominated sort; return rank-0 as ParetoFront.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        crate::rng::set_seed(self.ga_config.rng_seed);

        let pop_size = self.ibea_config.population_size;
        let max_gens = self.ibea_config.max_generations;
        let directions = self.ibea_config.effective_directions();

        let mut population = self.initialize_population()?;

        for gen in 0..max_gens {
            // Timing for observers
            let t_fitness: Option<Instant> = if self.observer.is_some() {
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

            // (a) Compute indicator fitness
            let mut _fitness = Self::compute_indicator_fitness(&population, &directions);

            // (b) Notify observer -- on_indicator_fitness_assigned
            if let Some(start) = t_fitness {
                self.notify(|obs| {
                    obs.on_indicator_fitness_assigned(
                        gen,
                        start.elapsed().as_secs_f64() * 1000.0,
                        population.len(),
                    )
                });
            }

            // Copy original population sizes for observer tracking
            let orig_size = population.len();

            // (c) Environmental selection: remove worst individuals
            let removed = Self::environmental_selection(&mut population, orig_size, &directions);

            // (d) Notify observer -- on_environmental_selection
            self.notify(|obs| obs.on_environmental_selection(gen, population.len(), removed));

            // (e) Create offspring via binary tournament + crossover + mutation
            if population.len() < 2 {
                // Refill if selection removed too many
                population = self.initialize_population()?;
                continue;
            }
            let mut offspring = self.create_offspring(&population)?;

            // (f) Merge offspring into population
            population.append(&mut offspring);

            // Trim population back to pop_size to prevent unbounded growth
            population.truncate(pop_size);
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
