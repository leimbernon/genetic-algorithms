//! Extracted from src/engines/ga.rs in phase 69-04 — Per-generation loop body (parent_crossover, extract_elite, reinsert_elite).

use super::*;
use crate::operations::DifferentialParams;

/// AOS and fitness parameters bundled for `parent_crossover` (D-07).
///
/// Groups the Adaptive Operator Selection state, operator portfolios,
/// fitness-context values, and per-offspring age assignment — the args that
/// do not belong to the core population/configuration inputs — so the
/// function signature stays below Clippy's `too_many_arguments` limit (7).
pub(crate) struct ParentCrossoverParams<'a, U: LinearChromosome> {
    /// Age assigned to each produced offspring.
    pub(crate) age: usize,
    /// Population-level maximum fitness (used by AGA crossover probability).
    pub(crate) f_max: f64,
    /// Population-level average fitness (used by AGA probability formulas).
    pub(crate) f_avg: f64,
    /// Dynamic mutation probability override (None → use configured static probability).
    pub(crate) dynamic_mutation_prob: Option<f64>,
    /// Current generation index (used by AOS selection strategy).
    pub(crate) generation: usize,
    /// Current best fitness across the population (used by AOS reward).
    pub(crate) best_fitness: f64,
    /// Whether the problem is a maximization problem (for AOS reward sign).
    pub(crate) is_maximization: bool,
    /// Per-chromosome fitness function (None in batch mode).
    pub(crate) fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
    /// Optional AOS crossover portfolio.
    pub(crate) crossover_portfolio: Option<&'a Vec<Crossover>>,
    /// Optional AOS mutation portfolio.
    pub(crate) mutation_portfolio: Option<&'a Vec<Mutation>>,
    /// Optional shared AOS crossover state.
    pub(crate) aos_crossover_state: Option<&'a Mutex<AosState>>,
    /// Optional shared AOS mutation state.
    pub(crate) aos_mutation_state: Option<&'a Mutex<AosState>>,
}

/// Performs parent crossover using the configured crossover and mutation strategies.
///
/// Behavior:
/// - Splits work among threads considering available parent pairs.
/// - Computes adaptive probabilities when enabled; otherwise uses static ones.
/// - Produces children, mutates them, computes their fitness, and returns the offspring.
pub(crate) fn parent_crossover<U>(
    parents: &[Vec<usize>],
    chromosomes: &[U],
    configuration: &GaConfiguration,
    params: ParentCrossoverParams<'_, U>,
) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome
        + Send
        + Sync
        + 'static
        + Clone
        + mutation::ValueMutable
        + crate::traits::RealValuedMutation,
{
    // Destructure the population-level and AOS params bundle (D-07)
    let ParentCrossoverParams {
        age,
        f_max,
        f_avg,
        dynamic_mutation_prob,
        generation,
        best_fitness,
        is_maximization,
        fitness_fn,
        crossover_portfolio,
        mutation_portfolio,
        aos_crossover_state,
        aos_mutation_state,
    } = params;

    /*
        Gets the static crossover probability config and the static mutation probability config
        This way we avoid of passing by these conditions at each thread if it's not necessary
    */
    let crossover_probability_config =
        if let Some(p) = configuration.crossover_configuration.probability_max {
            if !configuration.adaptive_ga {
                Some(p)
            } else {
                None
            }
        } else {
            Some(1.0)
        };

    let mutation_probability_config = if let Some(dp) = dynamic_mutation_prob {
        // Dynamic mutation overrides static probability
        Some(dp)
    } else if let Some(p) = configuration.mutation_configuration.probability_max {
        if !configuration.adaptive_ga {
            Some(p)
        } else {
            None
        }
    } else {
        Some(1.0)
    };

    // Create AOS reward accumulators (Phase 43)
    // These are shared across rayon threads via Arc<Mutex<Vec<(usize, f64)>>>
    let crossover_reward_acc: RewardAccumulator = if aos_crossover_state.is_some() {
        Some(Arc::new(Mutex::new(Vec::new())))
    } else {
        None
    };
    let mutation_reward_acc: RewardAccumulator = if aos_mutation_state.is_some() {
        Some(Arc::new(Mutex::new(Vec::new())))
    } else {
        None
    };

    // Shared per-group closure: produces children from one N-ary parent group.
    // cfg-gated only for the iterator kind (par_iter on native, iter on wasm32).
    let process_pair = |group: &Vec<usize>| -> Result<Vec<U>, GaError> {
        let mut rng = crate::rng::make_rng();

        // T-54-01: guard against out-of-bounds or too-small group (minimum 2 parents)
        if group.len() < 2 {
            return Err(GaError::SelectionError(format!(
                "Selection group has fewer than 2 parents (got {})",
                group.len()
            )));
        }
        let key = group[0];
        let value = group[1];

        // Getting the parent 1 and 2 for crossover
        let parent_1 = chromosomes.get(key).ok_or_else(|| {
            GaError::SelectionError(format!(
                "Selection returned out-of-bounds index {} (population size {})",
                key,
                chromosomes.len()
            ))
        })?;
        let parent_2 = chromosomes.get(value).ok_or_else(|| {
            GaError::SelectionError(format!(
                "Selection returned out-of-bounds index {} (population size {})",
                value,
                chromosomes.len()
            ))
        })?;

        // Select operators via AOS if portfolios are configured (Phase 43)
        // AOS returns (operator_index, operator_enum) for reward tracking
        let selected_crossover: Option<(usize, Crossover)> = if let (
            Some(portfolio),
            Some(aos_state),
        ) =
            (crossover_portfolio, aos_crossover_state)
        {
            let mut state = aos_state.lock().unwrap();
            let op_idx = state.select_operator(&mut rng, generation);
            Some((op_idx, portfolio[op_idx]))
        } else {
            None
        };

        let selected_mutation: Option<(usize, Mutation)> =
            if let (Some(portfolio), Some(aos_state)) = (mutation_portfolio, aos_mutation_state) {
                let mut state = aos_state.lock().unwrap();
                let op_idx = state.select_operator(&mut rng, generation);
                Some((op_idx, portfolio[op_idx].clone()))
            } else {
                None
            };

        // Making the crossover of the parents when the random number is below or equal to the given probability
        let crossover_probability = rng.random_range(0.0..1.0);
        let effective_crossover_prob = if let Some(p) = crossover_probability_config {
            p
        } else {
            crossover::aga_probability(
                parent_1,
                parent_2,
                f_max,
                f_avg,
                configuration
                    .crossover_configuration
                    .probability_max
                    .unwrap_or(1.0),
                configuration
                    .crossover_configuration
                    .probability_min
                    .unwrap_or(0.0),
            )
        };

        // Making the mutation of each child when the random number is below or equal the given probability
        let mut mutation_probability = rng.random_range(0.0..1.0);
        let effective_mutation_prob = if let Some(p) = mutation_probability_config {
            p
        } else {
            mutation::aga_probability(
                parent_1,
                parent_2,
                f_avg,
                configuration
                    .mutation_configuration
                    .probability_max
                    .unwrap_or(1.0),
                configuration
                    .mutation_configuration
                    .probability_min
                    .unwrap_or(0.0),
            )
        };

        let mut child_1: U;
        let mut child_2: U;

        if crossover_probability <= effective_crossover_prob {
            // Determine the effective crossover method (AOS-selected or user-configured)
            let effective_method = selected_crossover
                .map(|(_, op)| op)
                .unwrap_or(configuration.crossover_configuration.method);

            // Dispatch crossover by group size: groups of 2 use the standard 2-parent path;
            // larger groups use the multi-parent dispatch (UNDX/SPX/PCX via group.len() > 2).
            let mut children = if group.len() > 2 {
                // Multi-parent crossover path: collect all parents from the group
                let mut parent_refs: Vec<&U> = Vec::with_capacity(group.len());
                for &idx in group.iter() {
                    let p = chromosomes.get(idx).ok_or_else(|| {
                        GaError::SelectionError(format!(
                            "Selection returned out-of-bounds index {} (population size {})",
                            idx,
                            chromosomes.len()
                        ))
                    })?;
                    parent_refs.push(p);
                }
                let mut cx_config = configuration.crossover_configuration;
                cx_config.method = effective_method;
                // Returns 1 offspring per D-04 (single-offspring contract)
                crossover::factory_multi_parent_dispatch(&parent_refs, cx_config)?
            } else {
                // Standard 2-parent crossover path — all variants with group.len() == 2
                let mut cx_config = configuration.crossover_configuration;
                cx_config.method = effective_method;
                crossover::factory(parent_1, parent_2, cx_config)?
            };

            // factory_multi_parent_dispatch returns 1 child; factory returns 2.
            // For the 1-child path, child_1 gets the actual offspring; child_2 falls back to
            // parent_1.clone() (D-04 / Pitfall 1). For the 2-child path, both pops succeed.
            child_1 = children.pop().ok_or_else(|| {
                GaError::CrossoverError("Crossover returned no children".to_string())
            })?;
            child_2 = children.pop().unwrap_or_else(|| parent_1.clone());
        } else {
            child_1 = parent_1.clone();
            child_2 = parent_2.clone();
        }

        // Determine mutation method: AOS-selected or configured single operator
        let selected_mutation_idx = selected_mutation.as_ref().map(|(idx, _)| *idx);
        let mutation_method = selected_mutation
            .map(|(_, op)| op)
            .unwrap_or_else(|| configuration.mutation_configuration.method.clone());

        if mutation_probability <= effective_mutation_prob {
            match &mutation_method {
                Mutation::Differential(DifferentialParams { f }) => {
                    let f_val = f.unwrap_or(0.5);
                    crate::operations::mutation::differential::differential_mutation(
                        &mut child_1,
                        chromosomes,
                        key,
                        f_val,
                    )?;
                }
                Mutation::Insertion | Mutation::Deletion => {
                    mutation::factory_with_chromosome_length(
                        mutation_method.clone(),
                        &mut child_1,
                        Some(configuration.limit_configuration.chromosome_length),
                    )?;
                }
                _ => {
                    mutation_method.mutate(&mut child_1, &mutation_method)?;
                }
            }
        }

        mutation_probability = rng.random_range(0.0..1.0);
        if mutation_probability <= effective_mutation_prob {
            match &mutation_method {
                Mutation::Differential(DifferentialParams { f }) => {
                    let f_val = f.unwrap_or(0.5);
                    crate::operations::mutation::differential::differential_mutation(
                        &mut child_2,
                        chromosomes,
                        value,
                        f_val,
                    )?;
                }
                Mutation::Insertion | Mutation::Deletion => {
                    mutation::factory_with_chromosome_length(
                        mutation_method.clone(),
                        &mut child_2,
                        Some(configuration.limit_configuration.chromosome_length),
                    )?;
                }
                _ => {
                    mutation_method.mutate(&mut child_2, &mutation_method)?;
                }
            }
        }

        // Inject fitness function into children built via U::new() (which start with the
        // default no-op fitness fn). Children from parent.clone() (the else branch above)
        // already carry the correct fitness fn from their parent.
        if let Some(ref ff) = fitness_fn {
            let ff1 = Arc::clone(ff);
            child_1.set_fitness_fn(move |genes| ff1(genes));
            let ff2 = Arc::clone(ff);
            child_2.set_fitness_fn(move |genes| ff2(genes));
        }

        // Calculate the fitness of both children and set their age
        child_1.calculate_fitness();
        child_2.calculate_fitness();

        child_1.set_age(age);
        child_2.set_age(age);

        // Accumulate AOS rewards (Phase 43)
        // Crossover reward: compare parent vs child fitness
        if let Some(ref acc) = crossover_reward_acc {
            if let Some((c_op_idx, _)) = selected_crossover {
                let (p, c) = if is_maximization {
                    (child_1.fitness(), parent_1.fitness())
                } else {
                    (parent_1.fitness(), child_1.fitness())
                };
                let reward = crate::aos::compute_normalized_reward(p, c, best_fitness);
                acc.lock().unwrap().push((c_op_idx, reward));
            }
        }
        // Mutation reward: compare parent vs child fitness
        if let Some(ref acc) = mutation_reward_acc {
            if let Some(m_op_idx) = selected_mutation_idx {
                let (p, c) = if is_maximization {
                    (child_1.fitness(), parent_1.fitness())
                } else {
                    (parent_1.fitness(), child_1.fitness())
                };
                let reward = crate::aos::compute_normalized_reward(p, c, best_fitness);
                acc.lock().unwrap().push((m_op_idx, reward));
            }
        }

        Ok(vec![child_1, child_2])
    };

    // Use rayon to process parent pairs in parallel (sequential fallback on wasm32 or parallel=off)
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    let results: Vec<Result<Vec<U>, GaError>> = parents.par_iter().map(process_pair).collect();
    #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
    let results: Vec<Result<Vec<U>, GaError>> = parents.iter().map(process_pair).collect();

    // Check for any errors and flatten the results
    let mut offspring = Vec::new();
    for result in results {
        offspring.extend(result?);
    }

    // Apply AOS reward updates after collecting all rewards (Phase 43)
    if let Some(acc) = crossover_reward_acc {
        let rewards = acc.lock().unwrap().drain(..).collect::<Vec<_>>();
        if !rewards.is_empty() {
            if let Some(aos_state) = aos_crossover_state {
                let mut state = aos_state.lock().unwrap();
                state.record_rewards(&rewards);
                state.update();
            }
        }
    }
    if let Some(acc) = mutation_reward_acc {
        let rewards = acc.lock().unwrap().drain(..).collect::<Vec<_>>();
        if !rewards.is_empty() {
            if let Some(aos_state) = aos_mutation_state {
                let mut state = aos_state.lock().unwrap();
                state.record_rewards(&rewards);
                state.update();
            }
        }
    }

    Ok(offspring)
}

/// Extracts the top `count` individuals from the population by fitness.
///
/// Only clones the selected elite individuals instead of the whole population.
pub(crate) fn extract_elite<U: LinearChromosome>(
    chromosomes: &[U],
    count: usize,
    problem_solving: ProblemSolving,
) -> Vec<U> {
    if count == 0 || chromosomes.is_empty() {
        return Vec::new();
    }
    let k = count.min(chromosomes.len());

    // Build index array and partially sort so the best `k` are at the front.
    let mut indices: Vec<usize> = (0..chromosomes.len()).collect();
    let cmp_fn = |a: &usize, b: &usize| {
        let cmp = chromosomes[*a]
            .fitness()
            .partial_cmp(&chromosomes[*b].fitness())
            .unwrap_or(std::cmp::Ordering::Equal);
        match problem_solving {
            ProblemSolving::Maximization => cmp.reverse(),
            _ => cmp,
        }
    };
    indices.select_nth_unstable_by(k - 1, cmp_fn);
    // The first `k` elements are the best (unordered among themselves).
    indices.truncate(k);

    indices.iter().map(|&i| chromosomes[i].clone()).collect()
}

/// Reinserts elite individuals into the population, replacing the worst if already at capacity.
pub(crate) fn reinsert_elite<U: LinearChromosome>(
    chromosomes: &mut [U],
    elite: Vec<U>,
    problem_solving: ProblemSolving,
) {
    let k = elite.len().min(chromosomes.len());
    if k == 0 {
        return;
    }

    // Partition so the k worst chromosomes end up at indices 0..k (O(n) instead of O(n log n)).
    // The comparator puts the worst individuals first:
    //   - Maximization: natural order (lower fitness first) = worst first
    //   - Minimization/FixedFitness: reversed order (higher fitness first) = worst first
    chromosomes.select_nth_unstable_by(k - 1, |a, b| {
        let cmp = a
            .fitness()
            .partial_cmp(&b.fitness())
            .unwrap_or(std::cmp::Ordering::Equal);
        match problem_solving {
            ProblemSolving::Maximization => cmp,
            _ => cmp.reverse(),
        }
    });

    // Overwrite the k worst slots with the elite individuals.
    for (i, elite_individual) in elite.into_iter().take(k).enumerate() {
        chromosomes[i] = elite_individual;
    }
}
