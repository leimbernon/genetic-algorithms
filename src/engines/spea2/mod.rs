//! SPEA2 — Strength Pareto Evolutionary Algorithm 2.
//!
//! ## Description
//!
//! SPEA2 (Zitzler, Laumanns & Thiele 2001) is a multi-objective evolutionary
//! algorithm that maintains a **fixed-size external archive** of non-dominated
//! solutions alongside the main population.
//!
//! Fitness assignment combines two components:
//! - **Raw strength** `R(i)` — sum of strength values of all individuals that
//!   dominate `i`, where `strength(j) = count(j dominates all others)`.
//! - **Density** `D(i)` — estimated via k-nearest-neighbour distance in
//!   objective space: `D(i) = 1 / (σ_k + 2)`, where `k = floor(sqrt(N))` and
//!   `σ_k` is the distance to the k-th nearest neighbour.
//!
//! **Final fitness** `F(i) = R(i) + D(i)` — lower is better.
//!
//! Per generation, SPEA2:
//! 1. Compute fitness on the combined population + archive set.
//! 2. **Environmental selection:** copy all non-dominated solutions (fitness <
//!    1.0) to the new archive. If under capacity, fill with best-dominated by
//!    fitness. If over capacity, truncate via iterative nearest-neighbour
//!    Euclidean removal (lexicographic tie-breaking).
//! 3. **Binary tournament** from the archive — create new offspring population
//!    via crossover + mutation.
//! 4. Replace the population with the offspring. The archive persists across
//!    generations.
//!
//! ## When to Use
//!
//! - **Problem type:** Multi-objective (2+ objectives)
//! - **Variable type:** Continuous, binary, permutation, or `List<T>`
//! - **Population structure:** Single population with external archive
//! - **Key strength:** The external archive preserves non-dominated solutions
//!   even if the population loses them. The k-NN density provides fine-grained
//!   diversity preservation across the entire Pareto front.
//! - **Key weakness:** The fitness computation is O(N²) per generation (pairwise
//!   domination checks). Archive truncation via iterative nearest-neighbour
//!   removal is also O(N²). Slower than NSGA-II for large populations.
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Default | Description |
//! |-----------|------|---------|-------------|
//! | `num_objectives` | `usize` | `2` | Number of objectives. |
//! | `population_size` | `usize` | `100` | Main population size (≥ 2). |
//! | `archive_size` | `usize` | `100` | External archive size (≤ pop_size). |
//! | `max_generations` | `usize` | `250` | Maximum generations. |
//! | `init_fn` | `Fn` | — | Chromosome initialisation. |
//! | `objective_fns` | `Vec<ObjectiveFn>` | — | One per objective. |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Default | Description |
//! |-----------|------|---------|-------------|
//! | `objective_directions` | `Vec<ObjectiveDirection>` | All `Minimize` | Per-objective Min/Max. |
//! | `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
//! | `observer` | `Spea2Observer<U>` | `None` | Lifecycle observer. |
//!
//! ## Complete Example
//!
//! ```ignore
//! use genetic_algorithms::spea2::Spea2Ga;
//! use genetic_algorithms::spea2::configuration::Spea2Configuration;
//! use genetic_algorithms::configuration::GaConfiguration;
//!
//! let spea2_config = Spea2Configuration::new()
//!     .with_num_objectives(2)
//!     .with_population_size(100)
//!     .with_archive_size(100)
//!     .with_max_generations(250);
//!
//! let ga_config = GaConfiguration::default();
//! let mut spea2 = Spea2Ga::<MyChromosome>::new(spea2_config, ga_config)
//!     .with_initialization_fn(|n, alleles, repeat| { /* ... */ })
//!     .with_objective_fns(vec![
//!         Box::new(|dna| { /* ZDT1 f1 */ 0.0 }),
//!         Box::new(|dna| { /* ZDT1 f2 */ 0.0 }),
//!     ])
//!     .build()?;
//!
//! let pareto_front = spea2.run()?;
//! println!("Front size: {}", pareto_front.len());
//! ```
//!
//! ## Configuration Tips
//!
//! - `archive_size` must be ≤ `population_size`. The canonical SPEA2 uses
//!   `archive_size = population_size`. Smaller archives converge faster but
//!   may lose diversity.
//! - SPEA2 works well with moderate population sizes (50–200). The O(N²)
//!   fitness computation becomes noticeable beyond typical sizes.
//! - The archive truncation uses lexicographic nearest-neighbour removal,
//!   which preserves boundary points (they have the largest distances).
//!
//! ## When to Choose This vs NSGA-II
//!
//! | Criterion | SPEA2 | NSGA-II |
//! |-----------|-------|---------|
//! | Archive | External fixed-size archive | None (elitism from sorting) |
//! | Diversity | k-NN density | Crowding distance |
//! | Fitness cost | O(N²) domination + O(N²) NN | O(M·N²) sort + O(N log N) crowd |
//! | Archive quality | Preserves non-dominated across gens | Best front from combined set |
//! | Parameter | archive_size added | Simpler config |
//!
//! ## References
//!
//! - Zitzler, E., Laumanns, M., & Thiele, L. (2001). SPEA2: Improving the
//!   strength Pareto evolutionary algorithm. _TIK-Report 103_, ETH Zurich.

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::nsga2::configuration::ObjectiveDirection;
use crate::observer::Spea2Observer;
use crate::operations::{crossover, mutation};
use crate::spea2::configuration::Spea2Configuration;
use crate::traits::{InitializationFn, LinearChromosome, MutationOperator, VectorFitness};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// SPEA2 strength-Pareto multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Spea2Ga<U>
where
    U: LinearChromosome,
{
    /// SPEA2-specific configuration.
    pub spea2_config: Spea2Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Optional structured lifecycle observer for SPEA2-specific events.
    pub observer: Option<Arc<dyn Spea2Observer<U> + Send + Sync>>,
}

impl<U> Spea2Ga<U>
where
    U: LinearChromosome,
{
    /// Creates a new `Spea2Ga` with the given configurations.
    pub fn new(spea2_config: Spea2Configuration, ga_config: GaConfiguration) -> Self {
        Spea2Ga {
            spea2_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
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
                self.spea2_config.archive_size, self.spea2_config.population_size
            )));
        }
        Ok(())
    }

    /// Euclidean distance between two objective vectors.
    fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
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
        let union: Vec<&ParetoIndividual<U>> = population.iter().chain(archive.iter()).collect();
        let n = union.len();
        // D-02: k = floor(sqrt(N_pop + N_archive))
        let k = (n as f64).sqrt().floor() as usize;

        // Step 1: Compute strength S(i) = count of individuals that i dominates
        let mut strength = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                if i != j
                    && crate::multi_objective::pareto::dominates_with_directions(
                        &union[i].objectives,
                        &union[j].objectives,
                        directions,
                    )
                {
                    strength[i] += 1.0;
                }
            }
        }

        // Step 2: Compute raw fitness R(i) = sum of strengths of individuals dominating i
        let mut raw_fitness = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                if i != j
                    && crate::multi_objective::pareto::dominates_with_directions(
                        &union[j].objectives,
                        &union[i].objectives,
                        directions,
                    )
                {
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
    fn truncate_archive(archive: &mut Vec<ParetoIndividual<U>>, target_size: usize) {
        while archive.len() > target_size {
            let n = archive.len();
            let mut remove_idx = 0usize;
            let mut remove_dist_list: Vec<f64> = Vec::new();

            // For each individual, compute sorted distances to all others.
            // Find the one with the lexicographically smallest sorted distance list.
            for i in 0..n {
                let mut dists: Vec<f64> = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| {
                        Self::euclidean_distance(&archive[i].objectives, &archive[j].objectives)
                    })
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
        let union: Vec<&ParetoIndividual<U>> = population.iter().chain(archive.iter()).collect();

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
        // Compare by SPEA2 fitness stored in crowding_distance (lower is better).
        // `rank` is always 0 during the generation loop (assigned only in post-hoc sort).
        // `crowding_distance` is set to the SPEA2 fitness value just before create_offspring.
        let fi = pool[i].crowding_distance;
        let fj = pool[j].crowding_distance;
        if fi < fj {
            i
        } else if fj < fi {
            j
        } else if rng.random::<bool>() {
            i
        } else {
            j
        }
    }
}

impl<U> Spea2Ga<U>
where
    U: LinearChromosome + mutation::ValueMutable + VectorFitness,
{
    /// Runs the SPEA2 algorithm and returns the Pareto front from the final archive.
    ///
    /// Implements Zitzler, Laumanns & Thiele 2001 Algorithm 1:
    /// 1. Validate configuration and extract directions.
    /// 2. Initialize population and evaluate objectives (parallel via rayon).
    /// 3. Initialize empty archive.
    /// 4. For each generation:
    ///    a. Compute SPEA2 fitness (strength + density) on population + archive.
    ///    b. Notify `on_fitness_assigned` observer hook.
    ///    c. Environmental selection: copy non-dominated to archive, fill or truncate.
    ///    d. Notify `on_archive_updated` observer hook.
    ///    e. Binary tournament selection from archive -> create offspring population.
    /// 5. Post-hoc non-dominated sort over the final archive; return rank-0 as ParetoFront.
    ///
    /// # Errors
    ///
    /// - `GaError::InvalidSpea2Configuration` when configuration is incomplete.
    /// - `GaError::CrossoverError` / `GaError::MutationError` / `GaError::InitializationError` on operator failures.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        crate::rng::set_seed(self.ga_config.rng_seed);

        let archive_size = self.spea2_config.archive_size;
        let max_gens = self.spea2_config.max_generations;
        let directions = self.spea2_config.effective_directions();

        // Step 2: Initialize population and evaluate objectives in parallel
        let mut population = self.initialize_population()?;

        // Runtime objective-count guard
        if let Some(first) = population.first() {
            let got = first.chromosome.fitness_values().len();
            if got != self.spea2_config.num_objectives {
                return Err(GaError::InvalidSpea2Configuration(format!(
                    "Expected {} objectives from fitness_values(), got {}",
                    self.spea2_config.num_objectives, got
                )));
            }
        }

        // Step 3: Initialize empty archive
        let mut archive: Vec<ParetoIndividual<U>> = Vec::with_capacity(archive_size);

        // Step 4: Generation loop
        for gen in 0..max_gens {
            // Timing for observer -- Instant::now() gated per WASM requirement
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

            // 4a: Compute SPEA2 fitness on combined population + archive
            let fitness = Self::assign_spea2_fitness(&population, &archive, &directions);

            // 4b: Observer -- on_fitness_assigned
            if let Some(start) = t_fitness {
                self.notify(|obs| {
                    obs.on_fitness_assigned(
                        gen,
                        start.elapsed().as_secs_f64() * 1000.0,
                        population.len(),
                        archive.len(),
                    )
                });
            }

            // 4c: Environmental selection -- build new archive
            archive = Self::environmental_selection(&population, &archive, &fitness, archive_size);

            // Compute non-dominated count for observer
            let nd_count = {
                let mut nd = 0usize;
                for i in 0..archive.len() {
                    let mut dominated = false;
                    for j in 0..archive.len() {
                        if j != i
                            && crate::multi_objective::pareto::dominates_with_directions(
                                &archive[j].objectives,
                                &archive[i].objectives,
                                &directions,
                            )
                        {
                            dominated = true;
                            break;
                        }
                    }
                    if !dominated {
                        nd += 1;
                    }
                }
                nd
            };

            // 4d: Observer -- on_archive_updated
            self.notify(|obs| obs.on_archive_updated(gen, archive.len(), nd_count));

            // Tag each archive member with its SPEA2 fitness so binary_tournament_from_archive
            // can compare by fitness (lower = better) instead of rank which is always 0 here.
            // We use `crowding_distance` as a scratch field since the SPEA2 loop does not use it.
            let archive_fitness =
                Self::assign_spea2_fitness(&archive, &[], &directions);
            for (i, ind) in archive.iter_mut().enumerate() {
                ind.crowding_distance = archive_fitness[i];
            }

            // 4e: Binary tournament from archive -> produce new population
            population = self.create_offspring(&archive)?;
        }

        // Step 5: Post-hoc non-dominated sort on final archive -> ParetoFront
        let obj_slices: Vec<&[f64]> = archive
            .iter()
            .map(|ind| ind.objectives.as_slice())
            .collect();
        let fronts = crate::multi_objective::non_dominated_sort::non_dominated_sort_with_directions(
            &obj_slices,
            &directions,
        );
        let mut ranks = vec![0usize; archive.len()];
        crate::multi_objective::non_dominated_sort::assign_ranks(&mut ranks, &fronts);
        for (i, &r) in ranks.iter().enumerate() {
            archive[i].rank = r;
        }
        let front_individuals: Vec<ParetoIndividual<U>> =
            archive.into_iter().filter(|ind| ind.rank == 0).collect();
        Ok(ParetoFront::new(front_individuals))
    }

    /// Initializes the population with random chromosomes and evaluates objectives in parallel.
    fn initialize_population(&self) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let pop_size = self.spea2_config.population_size;
        let genes_per_chrom = match self.ga_config.limit_configuration.chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(n) => n,
            crate::chromosomes::ChromosomeLength::Variable { .. } => {
                return Err(GaError::InvalidSpea2Configuration(
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

        #[cfg(not(target_arch = "wasm32"))]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_par_iter()
            .map(|mut chrom| {
                chrom.calculate_fitness();
                let objectives = chrom.fitness_values().to_vec();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
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

    /// Produces offspring chromosomes via binary tournament selection from archive,
    /// followed by crossover + mutation on each selected pair.
    fn create_offspring(
        &self,
        archive: &[ParetoIndividual<U>],
    ) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let pop_size = self.spea2_config.population_size;
        let mut rng = crate::rng::make_rng();

        // For the tournament fallback: a temporary empty population vec.
        let population: Vec<ParetoIndividual<U>> = Vec::new();

        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration.clone();
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);

        let mut offspring: Vec<U> = Vec::with_capacity(pop_size);
        // Produce pop_size offspring via tournament from archive, pairing to produce 2 children each
        let pairs_needed = (pop_size + 1).div_ceil(2);

        for _ in 0..pairs_needed {
            let p1_idx = Self::binary_tournament_from_archive(archive, &population, &mut rng);
            let p2_idx = Self::binary_tournament_from_archive(archive, &population, &mut rng);

            let parent_a = &archive[p1_idx].chromosome;
            let parent_b = &archive[p2_idx].chromosome;

            let p: f64 = rng.random();
            let children = if p <= crossover_prob {
                crossover::factory(parent_a, parent_b, crossover_config)?
            } else {
                vec![parent_a.clone(), parent_b.clone()]
            };

            for mut child in children {
                let mp: f64 = rng.random();
                if mp <= mut_prob {
                    mutation_config.method.mutate(&mut child, &mutation_config.method)?;
                }
                offspring.push(child);
                if offspring.len() >= pop_size {
                    break;
                }
            }
        }

        // Evaluate objectives for all offspring
        #[cfg(not(target_arch = "wasm32"))]
        let evaluated: Vec<ParetoIndividual<U>> = offspring
            .into_par_iter()
            .map(|mut chrom| {
                chrom.calculate_fitness();
                let objectives = chrom.fitness_values().to_vec();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let evaluated: Vec<ParetoIndividual<U>> = offspring
            .into_iter()
            .map(|mut chrom| {
                chrom.calculate_fitness();
                let objectives = chrom.fitness_values().to_vec();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(evaluated)
    }
}
