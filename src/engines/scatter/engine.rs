//! # Scatter Search (ScatterEngine)
//!
//! ## Description
//!
//! Scatter Search is a population-based metaheuristic that maintains a small **reference set**
//! of elite and diverse solutions. Unlike the standard GA which relies on stochastic operators,
//! Scatter Search combines solutions **deterministically** through linear combinations and
//! optionally refines them with local search. The algorithm follows a five-method cycle:
//!
//! 1. **Diversification Generation** — create a pool of diverse candidate solutions
//! 2. **Improvement** (optional) — apply local search to refine each candidate
//! 3. **Reference Set (RefSet) Update** — select `b/2` high-quality and `b/2` diverse solutions
//! 4. **Subset Generation** — form all pairs from the reference set
//! 5. **Solution Combination** — create new candidates by linear combination of each pair
//!
//! The cycle repeats until `max_iterations` is reached or the fitness target is met. Because the
//! reference set is small (typically 10-20), Scatter Search excels at **intensification** around
//! promising regions, making it well-suited for expensive fitness evaluations.
//!
//! ## When to Use
//!
//! - **Problem type:** Single-objective, continuous (genes must implement [`DeGene`]
//!   for linear combination arithmetic)
//! - **Number of objectives:** 1
//! - **Variable type:** Real-valued
//! - **Key strength:** Excellent intensification around promising regions; small population reduces
//!   total fitness evaluations; deterministic combination operator
//! - **Key weakness:** Limited exploration compared to stochastic methods; not applicable to
//!   binary, permutation, or symbolic problems
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `population_size` | `usize` | Yes (via builder) | `50` | Total solutions generated during diversification |
//! | `reference_set_size` | `usize` | Yes (via builder) | `10` | Size of the reference set `b` (must be \< population_size) |
//! | `max_iterations` | `usize` | Yes (via builder) | `100` | Maximum scatter-search iterations |
//! | `init_fn` | `Fn(usize) -> Vec<U>` | Yes (constructor) | — | Population initialization closure |
//! | `fitness_fn` | `Fn(&[U::Gene]) -> f64` | Yes (constructor) | — | Fitness evaluation function |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `local_search` | `bool` | No | `false` | Enable local search refinement after combination |
//! | `local_search_steps` | `usize` | No | `20` | Maximum steps for hill-climbing refinement |
//! | `local_search_step_size` | `f64` | No | `0.1` | Perturbation magnitude for local search |
//! | `fitness_target` | `Option<f64>` | No | `None` | Stop when best fitness reaches this |
//! | `observer` | `Option<Arc<dyn GaObserver<U>>>` | No | `None` | Lifecycle observer |
//!
//! ## Complete Example
//!
//! ```rust,ignore
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::de::gene::DeGene;
//! use genetic_algorithms::genotypes::Range as RangeGenotype;
//! use genetic_algorithms::scatter::{ScatterConfiguration, ScatterEngine};
//! use genetic_algorithms::traits::ChromosomeT;
//!
//! // Rastrigin function (minimize toward 0.0 at origin)
//! let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
//!     let a = 10.0;
//!     let n = dna.len() as f64;
//!     a * n + dna.iter().map(|g| {
//!         g.de_value().powi(2) - a * (2.0 * std::f64::consts::PI * g.de_value()).cos()
//!     }).sum::<f64>()
//! };
//!
//! let config = ScatterConfiguration::default()
//!     .with_population_size(100)
//!     .with_reference_set_size(20)
//!     .with_max_iterations(200)
//!     .with_local_search(true);
//!
//! let init_fn = |n: usize| -> Vec<RangeChromosome<f64>> {
//!     (0..n).map(|_| {
//!         let mut c = RangeChromosome::new();
//!         for i in 0..10 {
//!             c.dna.push(RangeGenotype::new(i as i32, vec![(-5.12, 5.12)], 0.0));
//!         }
//!         c
//!     }).collect()
//! };
//!
//! let mut engine = ScatterEngine::new(config, init_fn, fitness_fn);
//! let result = engine.run();
//! println!("Best fitness: {:?}", result.best_fitness);
//! ```
//!
//! ## Configuration Tips
//!
//! - **Reference set size of 10-20** is the standard range; smaller = faster convergence,
//!   larger = better diversity but slower iteration
//! - Enable **local search** for problems with smooth fitness landscapes — the combination
//!   + refinement loop can converge very quickly
//! - **population_size** should be large (100-500) to ensure the diversification phase
//!   generates sufficient coverage of the search space
//! - Scatter Search is particularly effective when fitness evaluation is expensive — the
//!   small reference set minimizes total evaluations
//!
//! ## When to Choose This vs Standard GA
//!
//! | Factor | ScatterEngine | Ga |
//! |--------|---------------|-----|
//! | Population size | Small (10-20 ref set) | Large (50-500) |
//! | Combination | Deterministic (linear) | Stochastic (crossover) |
//! | Local search | Built-in (optional) | External (memetic config) |
//! | Total evaluations | Low (intensification) | Moderate-High (exploration) |
//! | Best for | Expensive fitness functions | General exploration |
//!
//! ## References
//!
//! - Glover, F. (1977). Heuristics for Integer Programming Using Surrogate Constraints.
//!   *Decision Sciences*, 8(1), 156–166.
//! - Laguna, M., & Marti, R. (2003). *Scatter Search: Methodology and Implementations in C.*
//!   Springer Science & Business Media.

use std::borrow::Cow;
use std::sync::Arc;

use crate::configuration::ProblemSolving;
use crate::de::gene::DeGene;
use super::configuration::ScatterConfiguration;
use crate::rng::make_rng;
use crate::traits::{ChromosomeT, FitnessFn};
use crate::observer::GaObserver;
use crate::stats::GenerationStats;
use crate::ga::TerminationCause;
use rand::Rng;

/// Result returned by [`ScatterEngine::run`].
pub struct ScatterResult<U: ChromosomeT> {
    /// Final reference set.
    pub reference_set: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of iterations completed.
    pub iterations: usize,
}

/// Scatter Search engine.
///
/// Generic over the chromosome type `U`; `U::Gene` must implement [`DeGene`]
/// so that linear combination of solutions can be computed.
///
/// # Algorithm
///
/// 1. **Diversification** — generate `population_size` random solutions and
///    evaluate their fitness.
/// 2. **Reference-set construction** — select `b/2` best solutions and `b/2`
///    diverse solutions to form the initial reference set.
/// 3. **Iteration** — for each iteration:
///    a. Generate all pairs from the reference set.
///    b. For each pair, create candidates by linear combination.
///    c. Optionally apply local search to each candidate.
///    d. Merge candidates with reference set; keep best `b` by quality and
///    diversity.
/// 4. Stop when `max_iterations` is reached or `fitness_target` is met.
pub struct ScatterEngine<U: ChromosomeT>
where
    U::Gene: DeGene,
{
    config: ScatterConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT + Clone> ScatterEngine<U>
where
    U::Gene: DeGene,
{
    /// Construct a new engine.
    pub fn new(
        config: ScatterConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
        }
    }

    /// Attach a lifecycle observer. Zero overhead when not set.
    pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(observer);
        self
    }

    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Run Scatter Search and return the result.
    pub fn run(&mut self) -> ScatterResult<U> {
        let mut rng = make_rng();
        let b = self.config.reference_set_size.max(2);

        // ── 1. Diversification phase ──────────────────────────────────────────
        let mut pool: Vec<U> = (self.init_fn)(self.config.population_size);
        for ind in &mut pool {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── 2. Reference-set construction ─────────────────────────────────────
        assert!(!pool.is_empty(), "ScatterEngine: init_fn returned an empty population");

        // Sort pool by fitness; take top b/2 as quality members
        self.sort_by_fitness(&mut pool);
        let quality_count = b / 2;
        let diverse_count = b - quality_count;

        // Clamp so both slices are always in-bounds regardless of pool size.
        let actual_quality = quality_count.min(pool.len());
        let (quality_slice, remaining_slice) = pool.split_at(actual_quality);
        let mut ref_set: Vec<U> = quality_slice.to_vec();

        // Fill the rest with the most diverse solutions (Euclidean distance)
        let mut remaining: Vec<U> = remaining_slice.to_vec();
        for _ in 0..diverse_count {
            if remaining.is_empty() { break; }
            let idx = self.most_diverse_index(&ref_set, &remaining);
            ref_set.push(remaining.remove(idx));
        }

        // ── 3. Track best ─────────────────────────────────────────────────────
        let (mut best_idx, mut best_fitness) = self.find_best(&ref_set);
        let mut best = ref_set[best_idx].clone();

        let mut iterations = 0usize;
        let mut target_reached = false;

        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
        let mut stats_history: Vec<GenerationStats> = Vec::new();
        let mut prev_best_fitness = best_fitness;
        self.notify(|obs| obs.on_run_start());

        // ── 4. Main loop ──────────────────────────────────────────────────────
        for iter in 0..self.config.max_iterations {
            self.notify(|obs| obs.on_generation_start(iter));

            let n = ref_set.len();
            let mut new_candidates: Vec<U> = Vec::new();

            // Generate candidates by combining all pairs
            for i in 0..n {
                for j in (i + 1)..n {
                    let candidates = self.combine(&ref_set[i], &ref_set[j], &mut rng);
                    new_candidates.extend(candidates);
                }
            }

            // Evaluate and optionally apply local search
            for cand in &mut new_candidates {
                let f = (self.fitness_fn)(cand.dna());
                cand.set_fitness(f);
                if self.config.local_search {
                    self.local_search_improve(cand, &mut rng);
                }
            }

            // Update reference set: merge, keep best b
            ref_set.extend(new_candidates);
            self.sort_by_fitness(&mut ref_set);
            ref_set.truncate(b);

            // Update best
            let (bi, bf) = self.find_best(&ref_set);
            if self.is_better(bf, best_fitness) {
                best_fitness = bf;
                best_idx = bi;
                best = ref_set[best_idx].clone();
            }

            iterations += 1;

            // Observer: on_new_best fires if global best improved this iteration
            if self.is_better(best_fitness, prev_best_fitness) {
                prev_best_fitness = best_fitness;
                self.notify(|obs| obs.on_new_best(iter, best.clone()));
            }

            // Observer: on_generation_end with stats from ref_set
            let fitness_values: Vec<f64> = ref_set.iter().map(|c| c.fitness()).collect();
            let gen_stats = GenerationStats::from_fitness_values(iter, &fitness_values, is_maximization);
            stats_history.push(gen_stats);
            self.notify(|obs| obs.on_generation_end(stats_history.last().unwrap()));

            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    target_reached = true;
                    break;
                }
            }
        }

        let cause = if target_reached {
            TerminationCause::FitnessTargetReached
        } else {
            TerminationCause::GenerationLimitReached
        };
        self.notify(|obs| obs.on_run_end(cause, &stats_history));

        ScatterResult { reference_set: ref_set, best, best_fitness, iterations }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Produce two linear-combination candidates from a pair of solutions.
    ///
    /// Candidate A: `α * x1 + (1-α) * x2`  with α ~ Uniform(0, 1)
    /// Candidate B: `(1-α) * x1 + α * x2`
    fn combine(&self, x1: &U, x2: &U, rng: &mut impl Rng) -> Vec<U> {
        let alpha: f64 = rng.random::<f64>();
        let beta = 1.0 - alpha;
        let dim = x1.dna().len().min(x2.dna().len());

        let dna_a: Vec<U::Gene> = (0..dim)
            .map(|j| {
                let v = alpha * x1.dna()[j].de_value() + beta * x2.dna()[j].de_value();
                x1.dna()[j].with_de_value(v)
            })
            .collect();
        let dna_b: Vec<U::Gene> = (0..dim)
            .map(|j| {
                let v = beta * x1.dna()[j].de_value() + alpha * x2.dna()[j].de_value();
                x1.dna()[j].with_de_value(v)
            })
            .collect();

        let mut ca = x1.clone();
        ca.set_dna(Cow::Owned(dna_a));
        let mut cb = x2.clone();
        cb.set_dna(Cow::Owned(dna_b));
        vec![ca, cb]
    }

    /// Simple local search: hill-climbing with random perturbations.
    fn local_search_improve(&self, ind: &mut U, rng: &mut impl Rng) {
        let step = self.config.local_search_step_size;
        let mut current_fitness = ind.fitness();
        let dim = ind.dna().len();

        for _ in 0..self.config.local_search_steps {
            let j = rng.random_range(0..dim);
            let delta: f64 = (rng.random::<f64>() * 2.0 - 1.0) * step;
            let old_val = ind.dna()[j].de_value();
            let new_gene = ind.dna()[j].with_de_value(old_val + delta);
            ind.set_gene(j, new_gene);
            let new_fitness = (self.fitness_fn)(ind.dna());
            if self.is_better(new_fitness, current_fitness) {
                current_fitness = new_fitness;
                ind.set_fitness(new_fitness);
            } else {
                // Revert
                let revert = ind.dna()[j].with_de_value(old_val);
                ind.set_gene(j, revert);
            }
        }
        // Sync the chromosome's stored fitness to the final accepted value.
        ind.set_fitness(current_fitness);
    }

    fn sort_by_fitness(&self, pop: &mut [U]) {
        match self.config.problem_solving {
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                pop.sort_unstable_by(|a, b| {
                    a.fitness().partial_cmp(&b.fitness()).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProblemSolving::Maximization => {
                pop.sort_unstable_by(|a, b| {
                    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }

    fn find_best(&self, pop: &[U]) -> (usize, f64) {
        assert!(!pop.is_empty(), "find_best called on empty population");
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

    /// Index into `candidates` of the individual most distant from `ref_set`.
    fn most_diverse_index(&self, ref_set: &[U], candidates: &[U]) -> usize {
        candidates
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let min_dist = ref_set
                    .iter()
                    .map(|r| euclidean_distance(r.dna(), c.dna()))
                    .fold(f64::MAX, f64::min);
                (i, min_dist)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
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

fn euclidean_distance<G: DeGene>(a: &[G], b: &[G]) -> f64 {
    let len = a.len().min(b.len());
    (0..len)
        .map(|i| {
            let d = a[i].de_value() - b[i].de_value();
            d * d
        })
        .sum::<f64>()
        .sqrt()
}
