//! NSGA-III many-objective genetic algorithm.
//!
//! NSGA-III extends NSGA-II for problems with three or more objectives by
//! replacing crowding-distance survivor selection with reference-point niche
//! association on the unit hyperplane. Reference points are either
//! auto-generated via the Das-Dennis simplex lattice
//! ([`Nsga3Configuration::with_reference_points_auto`](configuration::Nsga3Configuration::with_reference_points_auto))
//! or user-supplied
//! ([`Nsga3Configuration::with_reference_points`](configuration::Nsga3Configuration::with_reference_points)).
//!
//! Reference: Deb & Jain 2014 (IEEE-TEC 18(4):577-601).

pub mod configuration;
pub mod das_dennis;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::non_dominated_sort::{assign_ranks, non_dominated_sort_with_directions};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::nsga2::configuration::ObjectiveDirection;
use crate::nsga3::configuration::Nsga3Configuration;
use crate::observer::Nsga3Observer;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// NSGA-III many-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Nsga3Ga<U>
where
    U: ChromosomeT,
{
    /// NSGA-III specific configuration.
    pub nsga3_config: Nsga3Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for NSGA-III-specific events.
    pub observer: Option<Arc<dyn Nsga3Observer<U> + Send + Sync>>,
}

impl<U> Nsga3Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `Nsga3Ga` with the given configurations.
    pub fn new(nsga3_config: Nsga3Configuration, ga_config: GaConfiguration) -> Self {
        Nsga3Ga {
            nsga3_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives NSGA-III-specific hooks.
    pub fn with_observer(mut self, obs: Arc<dyn Nsga3Observer<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    fn notify<F: FnOnce(&dyn Nsga3Observer<U>)>(&self, f: F) {
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

    /// Validates the NSGA-III configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidNsga3Configuration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.nsga3_config.num_objectives == 0 {
            return Err(GaError::InvalidNsga3Configuration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.nsga3_config.population_size < 2 {
            return Err(GaError::InvalidNsga3Configuration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidNsga3Configuration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.nsga3_config.num_objectives {
            return Err(GaError::InvalidNsga3Configuration(format!(
                "Expected {} objective functions, got {}",
                self.nsga3_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.nsga3_config.objective_directions.is_empty()
            && self.nsga3_config.objective_directions.len() != self.nsga3_config.num_objectives
        {
            return Err(GaError::InvalidNsga3Configuration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.nsga3_config.objective_directions.len(),
                self.nsga3_config.num_objectives
            )));
        }
        // Reference points must be configured (auto or custom).
        let ref_points = self.nsga3_config.effective_reference_points();
        match ref_points {
            None => {
                return Err(GaError::InvalidNsga3Configuration(
                    "reference points must be configured via with_reference_points_auto(p) or with_reference_points(points)".to_string(),
                ));
            }
            Some(points) => {
                for (i, pt) in points.iter().enumerate() {
                    if pt.len() != self.nsga3_config.num_objectives {
                        return Err(GaError::InvalidNsga3Configuration(format!(
                            "reference point {} has dimension {}, expected {}",
                            i,
                            pt.len(),
                            self.nsga3_config.num_objectives
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

impl<U> Nsga3Ga<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    /// Runs the NSGA-III algorithm and returns the first Pareto front.
    ///
    /// Implements the Deb & Jain 2014 generation loop:
    /// 1. Non-dominated sorting on the parent population (for tournament selection ranks).
    /// 2. Binary tournament selection + crossover + mutation to create offspring.
    /// 3. Combine parent + offspring (size 2N).
    /// 4. Non-dominated sort on the combined population.
    /// 5. Reference-point environmental selection (normalize → associate → niche-select).
    ///
    /// User objective functions should return finite `f64` values. NaN/Infinity values
    /// propagate through non-dominated sorting with `Ordering::Equal` fallback.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidNsga3Configuration` when reference points are not configured,
    /// or `GaError::MutationError` / `GaError::InitializationError` on operator failures.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        crate::rng::set_seed(self.ga_config.rng_seed);

        let pop_size = self.nsga3_config.population_size;
        let max_gens = self.nsga3_config.max_generations;
        let directions = self.nsga3_config.effective_directions();
        let reference_points = self
            .nsga3_config
            .effective_reference_points()
            .ok_or_else(|| {
                GaError::InvalidNsga3Configuration(
                    "reference points must be configured before run()".to_string(),
                )
            })?;

        let mut population = self.initialize_population()?;

        for gen in 0..max_gens {
            // Step 1: non-dominated sorting on the parent population (for tournament selection rank).
            let t_sort: Option<Instant> = if self.observer.is_some() {
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
            let parent_objs: Vec<&[f64]> =
                population.iter().map(|i| i.objectives.as_slice()).collect();
            let parent_fronts = non_dominated_sort_with_directions(&parent_objs, &directions);
            let mut parent_ranks = vec![0usize; population.len()];
            assign_ranks(&mut parent_ranks, &parent_fronts);
            for (i, &r) in parent_ranks.iter().enumerate() {
                population[i].rank = r;
            }
            if let Some(start) = t_sort {
                self.notify(|obs| {
                    obs.on_non_dominated_sort_complete(
                        gen,
                        start.elapsed().as_secs_f64() * 1000.0,
                    )
                });
            }

            // Step 2: create offspring via binary tournament + crossover + mutation.
            let offspring = self.create_offspring(&population)?;

            // Step 3: combine parent + offspring (size 2N).
            let mut combined = population;
            combined.extend(offspring);

            // Step 4: non-dominated sort on the combined population.
            let combined_objs: Vec<&[f64]> =
                combined.iter().map(|i| i.objectives.as_slice()).collect();
            let combined_fronts = non_dominated_sort_with_directions(&combined_objs, &directions);
            let mut combined_ranks = vec![0usize; combined.len()];
            assign_ranks(&mut combined_ranks, &combined_fronts);
            for (i, &r) in combined_ranks.iter().enumerate() {
                combined[i].rank = r;
            }

            // Step 5: reference-point environmental selection.
            population = nsga3_environmental_selection(
                combined,
                combined_fronts,
                pop_size,
                &reference_points,
                &directions,
            );

            // Step 6: observer notifications.
            // Re-derive front_count from the final selected population's ranks.
            let front_count = population
                .iter()
                .map(|i| i.rank)
                .max()
                .map(|m| m + 1)
                .unwrap_or(0);
            self.notify(|obs| {
                obs.on_pareto_front_assigned(gen, front_count, population.len())
            });
        }

        let front_individuals: Vec<ParetoIndividual<U>> =
            population.into_iter().filter(|ind| ind.rank == 0).collect();
        Ok(ParetoFront::new(front_individuals))
    }

    /// Initializes the population with random chromosomes and evaluates objectives in parallel.
    fn initialize_population(&self) -> Result<Vec<ParetoIndividual<U>>, GaError> {
        let init_fn = self.initialization_fn.as_ref().ok_or_else(|| {
            GaError::InitializationError("No initialization function set".to_string())
        })?;

        let pop_size = self.nsga3_config.population_size;
        let genes_per_chrom = self.ga_config.limit_configuration.genes_per_chromosome;
        let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;

        let alleles = if self.alleles.is_empty() {
            None
        } else {
            Some(self.alleles.as_slice())
        };

        let chromosomes: Vec<U> = crate::traits::initialize_chromosomes(
            pop_size,
            genes_per_chrom,
            alleles,
            Some(alleles_can_repeat),
            init_fn,
            None,
            0,
        );

        let objective_fns = &self.objective_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> =
                    objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let population: Vec<ParetoIndividual<U>> = chromosomes
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> =
                    objective_fns.iter().map(|f| f(chrom.dna())).collect();
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

        let pop_size = self.nsga3_config.population_size;
        let crossover_config = self.ga_config.crossover_configuration;
        let mutation_config = self.ga_config.mutation_configuration;
        let crossover_prob = crossover_config.probability_max.unwrap_or(1.0);
        let mut_prob = mutation_config.probability_max.unwrap_or(0.1);

        let mut rng = crate::rng::make_rng();
        let mut raw_offspring: Vec<U> = Vec::with_capacity(pop_size);

        while raw_offspring.len() < pop_size {
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

            for child in children.iter_mut() {
                let mp: f64 = rng.random();
                if mp <= mut_prob {
                    if mutation_config.method == crate::operations::Mutation::Differential {
                        return Err(GaError::MutationError(
                            "Differential mutation is not supported in NSGA-III; \
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

        let objective_fns = &self.objective_fns;
        #[cfg(not(target_arch = "wasm32"))]
        let offspring: Vec<ParetoIndividual<U>> = raw_offspring
            .into_par_iter()
            .map(|chrom| {
                let objectives: Vec<f64> =
                    objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();
        #[cfg(target_arch = "wasm32")]
        let offspring: Vec<ParetoIndividual<U>> = raw_offspring
            .into_iter()
            .map(|chrom| {
                let objectives: Vec<f64> =
                    objective_fns.iter().map(|f| f(chrom.dna())).collect();
                ParetoIndividual::new(chrom, objectives)
            })
            .collect();

        Ok(offspring)
    }

    /// Binary tournament: lower rank wins; ties broken randomly (NSGA-III does not use crowding distance).
    fn binary_tournament(
        &self,
        population: &[ParetoIndividual<U>],
        rng: &mut impl Rng,
    ) -> usize {
        let n = population.len();
        let i = rng.random_range(0..n);
        let j = rng.random_range(0..n);
        if population[i].rank < population[j].rank {
            i
        } else if population[j].rank < population[i].rank {
            j
        } else if rng.random::<bool>() {
            i
        } else {
            j
        }
    }
}

/// NSGA-III reference-point environmental selection.
///
/// Selects exactly `pop_size` survivors from `combined` using the algorithm
/// of Deb & Jain 2014 (Procedure 1 + Procedure 2):
///   1. Take all fronts that fit entirely.
///   2. From the splitting front Fl, select remaining slots via niche-preservation:
///      - normalize the union (St) onto the unit hyperplane,
///      - associate each individual to its nearest reference point (perpendicular distance),
///      - count niche occupancy among already-selected individuals,
///      - repeatedly pick from the under-populated niches.
fn nsga3_environmental_selection<U: ChromosomeT>(
    combined: Vec<ParetoIndividual<U>>,
    fronts: Vec<Vec<usize>>,
    pop_size: usize,
    reference_points: &[Vec<f64>],
    directions: &[ObjectiveDirection],
) -> Vec<ParetoIndividual<U>> {
    // Take fronts that fit entirely; identify the splitting front Fl.
    let mut next_indices: Vec<usize> = Vec::with_capacity(pop_size);
    let mut splitting_front: Vec<usize> = Vec::new();
    for front in &fronts {
        if next_indices.len() + front.len() <= pop_size {
            next_indices.extend_from_slice(front);
            if next_indices.len() == pop_size {
                splitting_front.clear();
                break;
            }
        } else {
            splitting_front = front.clone();
            break;
        }
    }

    // If no splitting front needed, return the selected individuals.
    if splitting_front.is_empty() {
        return next_indices
            .into_iter()
            .map(|idx| combined[idx].clone())
            .collect();
    }

    // St = next_indices ∪ splitting_front
    let mut st_indices: Vec<usize> = next_indices.clone();
    st_indices.extend_from_slice(&splitting_front);

    // Normalize St onto the unit hyperplane (translate by ideal point, scale by intercepts).
    let st_normalized = normalize_st(&combined, &st_indices, directions);

    // Associate each individual in St to its nearest reference point (perpendicular distance).
    let association: Vec<(usize, f64)> =
        associate_to_reference_points(&st_normalized, reference_points);
    // association[k] corresponds to st_indices[k].

    // Build niche counts ρ_j over already-selected individuals (those in next_indices).
    let mut niche_count = vec![0usize; reference_points.len()];
    for &(ref_idx, _) in &association[..next_indices.len()] {
        niche_count[ref_idx] += 1;
    }

    // Map: candidate index in splitting_front -> position k in st_indices/association.
    // splitting_front candidates occupy positions [next_indices.len()..st_indices.len()] in association.
    let split_offset = next_indices.len();
    // remaining[j] = list of (candidate_idx_in_splitting_front, perp_distance) for ref point j.
    let mut remaining: Vec<Vec<(usize, f64)>> = vec![Vec::new(); reference_points.len()];
    for (i_in_split, _) in splitting_front.iter().enumerate() {
        let k = split_offset + i_in_split;
        let (ref_idx, perp) = association[k];
        remaining[ref_idx].push((i_in_split, perp));
    }

    let mut rng = crate::rng::make_rng();
    while next_indices.len() < pop_size {
        // Find ref points with minimum niche count among those that still have candidates.
        let mut min_niche = usize::MAX;
        for (j, slot) in remaining.iter().enumerate() {
            if !slot.is_empty() && niche_count[j] < min_niche {
                min_niche = niche_count[j];
            }
        }
        // Collect all tied indices.
        let tied: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter_map(|(j, slot)| {
                if !slot.is_empty() && niche_count[j] == min_niche {
                    Some(j)
                } else {
                    None
                }
            })
            .collect();
        if tied.is_empty() {
            // No more candidates to draw from — defensive break (should not happen given splitting_front.len() suffices).
            break;
        }
        // Pick a tied ref point j*.
        let j_star = tied[rng.random_range(0..tied.len())];

        let chosen_position_in_slot = if niche_count[j_star] == 0 {
            // Pick the candidate with minimum perpendicular distance (ties broken randomly via slot order).
            let mut best_pos = 0;
            let mut best_dist = remaining[j_star][0].1;
            for (pos, &(_idx, dist)) in remaining[j_star].iter().enumerate().skip(1) {
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = pos;
                }
            }
            best_pos
        } else {
            // Pick uniformly at random.
            rng.random_range(0..remaining[j_star].len())
        };
        let (cand_idx_in_split, _) = remaining[j_star].swap_remove(chosen_position_in_slot);
        let global_idx = splitting_front[cand_idx_in_split];
        next_indices.push(global_idx);
        niche_count[j_star] += 1;
    }

    next_indices
        .into_iter()
        .map(|idx| combined[idx].clone())
        .collect()
}

/// Normalizes the sub-population indexed by `st_indices` onto the unit hyperplane.
///
/// Translates by the per-objective ideal (min) and scales by intercepts derived
/// from the M extreme points (ASF minimization). Falls back to the per-objective
/// nadir when the hyperplane is degenerate. Intercepts are clamped to `f64::EPSILON`
/// to prevent division by zero (RESEARCH.md Pitfall 1).
///
/// For Maximize objectives, signs are flipped before normalization so all dimensions
/// behave as minimization (smaller is better in the normalized frame).
fn normalize_st<U: ChromosomeT>(
    combined: &[ParetoIndividual<U>],
    st_indices: &[usize],
    directions: &[ObjectiveDirection],
) -> Vec<Vec<f64>> {
    let m = directions.len();
    if st_indices.is_empty() {
        return Vec::new();
    }

    // Step A: build raw objective vectors with maximize objectives sign-flipped.
    let raw: Vec<Vec<f64>> = st_indices
        .iter()
        .map(|&i| {
            (0..m)
                .map(|d| match directions[d] {
                    ObjectiveDirection::Minimize => combined[i].objectives[d],
                    ObjectiveDirection::Maximize => -combined[i].objectives[d],
                })
                .collect()
        })
        .collect();

    // Step B: ideal point (per-objective minimum across St).
    let mut ideal = vec![f64::INFINITY; m];
    for v in &raw {
        for d in 0..m {
            if v[d] < ideal[d] {
                ideal[d] = v[d];
            }
        }
    }

    // Step C: translate by the ideal point.
    let translated: Vec<Vec<f64>> = raw
        .iter()
        .map(|v| (0..m).map(|d| v[d] - ideal[d]).collect())
        .collect();

    // Step D: extreme points via ASF (Achievement Scalarizing Function).
    // ASF(x, axis_d) = max over k of x[k] / w[k] where w[d]=1, w[k!=d]=1e-6.
    let mut intercepts = vec![0.0f64; m];
    let mut degenerate = false;
    for axis in 0..m {
        let mut best_idx: usize = 0;
        let mut best_asf = f64::INFINITY;
        for (i, v) in translated.iter().enumerate() {
            let mut asf = f64::NEG_INFINITY;
            for (k, &vk) in v.iter().enumerate() {
                let w = if k == axis { 1.0 } else { 1.0e-6 };
                let val = vk / w;
                if val > asf {
                    asf = val;
                }
            }
            if asf < best_asf {
                best_asf = asf;
                best_idx = i;
            }
        }
        // Intercept on axis `axis` is the value of the extreme point on that axis.
        intercepts[axis] = translated[best_idx][axis];
        if intercepts[axis].abs() < f64::EPSILON {
            degenerate = true;
        }
    }

    // Step E: degenerate fallback — use nadir (per-objective max of translated).
    if degenerate {
        let mut nadir = vec![f64::NEG_INFINITY; m];
        for v in &translated {
            for d in 0..m {
                if v[d] > nadir[d] {
                    nadir[d] = v[d];
                }
            }
        }
        for d in 0..m {
            intercepts[d] = nadir[d].max(f64::EPSILON);
        }
    }

    // Step F: clamp and divide.
    for intercept in intercepts.iter_mut() {
        if *intercept < f64::EPSILON {
            *intercept = f64::EPSILON;
        }
    }
    translated
        .iter()
        .map(|v| (0..m).map(|d| v[d] / intercepts[d]).collect())
        .collect()
}

/// For each normalized point, returns `(nearest_ref_point_idx, perpendicular_distance)`.
///
/// Perpendicular distance from point `f''` to the line through origin and reference point `r`:
///   d_perp = || f'' - ((f''·r) / (r·r)) * r ||
fn associate_to_reference_points(
    normalized: &[Vec<f64>],
    reference_points: &[Vec<f64>],
) -> Vec<(usize, f64)> {
    // Pre-compute r·r for each reference point.
    let r_dot_r: Vec<f64> = reference_points
        .iter()
        .map(|r| r.iter().map(|x| x * x).sum::<f64>().max(f64::EPSILON))
        .collect();

    normalized
        .iter()
        .map(|f| {
            let mut best_idx = 0usize;
            let mut best_dist = f64::INFINITY;
            for (j, r) in reference_points.iter().enumerate() {
                // f·r
                let fr: f64 = f.iter().zip(r.iter()).map(|(a, b)| a * b).sum();
                let scale = fr / r_dot_r[j];
                // perpendicular vector = f - scale * r
                let dist_sq: f64 = f
                    .iter()
                    .zip(r.iter())
                    .map(|(a, b)| {
                        let diff = a - scale * b;
                        diff * diff
                    })
                    .sum();
                let dist = dist_sq.sqrt();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = j;
                }
            }
            (best_idx, best_dist)
        })
        .collect()
}
