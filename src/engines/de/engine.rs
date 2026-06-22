//! `DeEngine` — the Differential Evolution execution loop.

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::configuration::{DeAdaptive, DeConfiguration, DeMutationStrategy};
use super::crossover::crossover;
use super::mutation::{mutate, DeMutationParams, JadeState, LShadeState};
use crate::configuration::ProblemSolving;
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::RealGene;
use crate::traits::{FitnessFn, LinearChromosome};
use rand::Rng;

/// Result returned by [`DeEngine::run`].
pub struct DeResult<U: LinearChromosome> {
    /// Final population (all individuals evaluated).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}

/// Differential Evolution engine.
///
/// Generic over the chromosome type `U`; `U::Gene` must implement [`RealGene`]
/// so that mutation arithmetic can be performed on gene values.
///
/// # Example
///
/// ```rust,no_run
/// // no_run: DE engine example — illustrative API usage, requires full initialization
/// use genetic_algorithms::de::{DeEngine, DeConfiguration, DeMutationStrategy};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::genotypes::Range as RangeGene;
/// use genetic_algorithms::traits::RealGene;
///
/// let config = DeConfiguration::default()
///     .with_population_size(50)
///     .with_max_generations(500);
///
/// let mut engine: DeEngine<RangeChromosome<f64>> = DeEngine::new(
///     config,
///     |n| (0..n).map(|_| /* initialise chromosome */ todo!()).collect(),
///     |dna: &[RangeGene<f64>]| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result = engine.run();
/// ```
pub struct DeEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: DeConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
}

impl<U: LinearChromosome + Clone> DeEngine<U>
where
    U::Gene: RealGene,
{
    /// Construct a new engine.
    ///
    /// * `config` — algorithm parameters.
    /// * `init_fn` — called once with `population_size`; must return that many
    ///   initialised chromosomes (fitness is ignored — the engine re-evaluates).
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: DeConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            fitness_cache: None,
        }
    }

    /// Run the DE algorithm and return the result.
    pub fn run(&mut self) -> DeResult<U>
    where
        U::Gene: Debug,
    {
        let mut rng = make_rng();
        let pop_size = self.config.population_size;
        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);

        // D-05: bootstrap cache handle at run() start.
        if let Some(size) = self.config.fitness_cache_size {
            if self.fitness_cache.is_none() {
                let (wrapped_fn, cache_handle) =
                    crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
                self.fitness_fn = wrapped_fn;
                self.fitness_cache = Some(cache_handle);
            }
        }

        // ── Initialise ────────────────────────────────────────────────────────
        let mut pop: Vec<U> = (self.init_fn)(pop_size);
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── Best tracking ─────────────────────────────────────────────────────
        let (mut best_idx, mut best_fitness) = self.find_best(&pop);
        let mut best = pop[best_idx].clone();

        // ── Adaptive state ────────────────────────────────────────────────────
        let mut jade = JadeState::new();
        let mut lshade = match &self.config.adaptive {
            DeAdaptive::LShade { history_size } => Some(LShadeState::new(*history_size)),
            _ => None,
        };

        // Archive used by JADE (inferior solutions replaced by better ones)
        let mut archive: Vec<U> = Vec::new();

        let mut generations = 0usize;
        let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);

        // Cache snapshot for per-generation delta stats.
        let (mut prev_cache_hits, mut prev_cache_misses) = match &self.fitness_cache {
            Some(ch) => {
                let c = ch.lock().expect("fitness cache lock poisoned");
                (c.hits(), c.misses())
            }
            None => (0, 0),
        };

        // ── Main loop ─────────────────────────────────────────────────────────
        for _gen in 0..self.config.max_generations {
            // Determine effective strategy (JADE uses current-to-pbest/1)
            let eff_strategy = match &self.config.adaptive {
                DeAdaptive::Jade { p, .. } => {
                    let _ = p; // used below when selecting pbest
                    DeMutationStrategy::CurrentToBest1 // overridden to pbest below
                }
                _ => self.config.mutation_strategy.clone(),
            };

            for i in 0..pop_size {
                // Draw F and CR (adaptive or static)
                let (f, cr) = match &self.config.adaptive {
                    DeAdaptive::None => (self.config.mutation_factor, self.config.crossover_rate),
                    DeAdaptive::Jade { .. } => (jade.draw_f(&mut rng), jade.draw_cr(&mut rng)),
                    DeAdaptive::LShade { .. } => {
                        let ls = lshade.as_ref().unwrap();
                        (ls.draw_f(&mut rng), ls.draw_cr(&mut rng))
                    }
                };

                // Determine "best" vector (pbest for JADE)
                let eff_best = match &self.config.adaptive {
                    DeAdaptive::Jade { p, .. } => {
                        let p_count = ((pop_size as f64 * p).ceil() as usize).max(1);
                        self.select_pbest(&pop, p_count, &mut rng)
                    }
                    _ => best_idx,
                };

                // Mutate
                let arc_ref: Option<&[U]> = match &self.config.adaptive {
                    DeAdaptive::Jade { .. } if !archive.is_empty() => Some(&archive),
                    _ => None,
                };
                let mutant = mutate(
                    &DeMutationParams {
                        strategy: &eff_strategy,
                        target_idx: i,
                        best_idx: eff_best,
                        f,
                    },
                    &pop,
                    &mut rng,
                    arc_ref,
                );

                // Crossover
                let trial_dna = crossover(
                    &self.config.crossover_mode,
                    pop[i].dna(),
                    &mutant,
                    cr,
                    &mut rng,
                );
                let trial_fitness = (self.fitness_fn)(&trial_dna);

                // Greedy selection
                let improved = self.is_better(trial_fitness, pop[i].fitness());
                if improved {
                    // Archive the replaced individual (JADE)
                    if matches!(&self.config.adaptive, DeAdaptive::Jade { .. }) {
                        archive.push(pop[i].clone());
                        if archive.len() > pop_size {
                            let drop = rng.random_range(0..archive.len());
                            archive.swap_remove(drop);
                        }
                        jade.record_success(f, cr);
                    }
                    if let Some(ref mut ls) = lshade {
                        ls.record_success(f, cr);
                    }

                    let mut trial = pop[i].clone();
                    trial.set_dna(Cow::Owned(trial_dna));
                    trial.set_fitness(trial_fitness);
                    pop[i] = trial;

                    if self.is_better(trial_fitness, best_fitness) {
                        best_fitness = trial_fitness;
                        best_idx = i;
                        best = pop[i].clone();
                    }
                }
            }

            // Update adaptive state at end of generation
            match &self.config.adaptive {
                DeAdaptive::Jade { c, .. } => jade.update(*c),
                DeAdaptive::LShade { .. } => lshade.as_mut().unwrap().update(),
                DeAdaptive::None => {}
            }

            generations += 1;

            // Per-generation stats + cache delta.
            let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
            let mut stats =
                GenerationStats::from_fitness_values(generations, &fitness_values, is_maximization);
            if let Some(ref ch) = self.fitness_cache {
                let c = ch.lock().expect("fitness cache lock poisoned");
                stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
                stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
                prev_cache_hits = c.hits();
                prev_cache_misses = c.misses();
            }
            all_stats.push(stats);

            // Re-locate best (index may have shifted from swaps above)
            let (bi, bf) = self.find_best(&pop);
            if self.is_better(bf, best_fitness) {
                best_fitness = bf;
                best_idx = bi;
                best = pop[bi].clone();
            }

            // Early stopping
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    break;
                }
            }
        }

        DeResult {
            population: pop,
            best,
            best_fitness,
            generations,
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn find_best(&self, pop: &[U]) -> (usize, f64) {
        let mut best_idx = 0;
        let mut best_fit = pop[0].fitness();
        for (i, ind) in pop.iter().enumerate().skip(1) {
            if self.is_better(ind.fitness(), best_fit) {
                best_fit = ind.fitness();
                best_idx = i;
            }
        }
        (best_idx, best_fit)
    }

    /// Select a random index from the top `p_count` individuals (pbest for JADE).
    fn select_pbest(&self, pop: &[U], p_count: usize, rng: &mut impl rand::Rng) -> usize {
        let mut indexed: Vec<(usize, f64)> = pop
            .iter()
            .enumerate()
            .map(|(i, u)| (i, u.fitness()))
            .collect();
        if matches!(self.config.problem_solving, ProblemSolving::Minimization) {
            indexed.sort_unstable_by(|a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            indexed.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        let top = indexed[..p_count.min(indexed.len())].to_vec();
        let pick = rng.random_range(0..top.len());
        top[pick].0
    }

    fn is_better(&self, candidate: f64, current: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => candidate < current,
            ProblemSolving::Maximization => candidate > current,
            ProblemSolving::FixedFitness => {
                if let Some(t) = self.config.fitness_target {
                    (candidate - t).abs() < (current - t).abs()
                } else {
                    candidate < current
                }
            }
        }
    }

    fn reached_target(&self, fitness: f64, target: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => fitness <= target,
            ProblemSolving::Maximization => fitness >= target,
            ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
        }
    }
}
