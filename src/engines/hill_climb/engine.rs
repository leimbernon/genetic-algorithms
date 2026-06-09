//! Hill-climbing engine implementation.

use std::sync::Arc;

use crate::configuration::ProblemSolving;
use crate::error::GaError;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::stats::GenerationStats;
use crate::traits::{LinearChromosome, Strategy};
use super::configuration::{HillClimbConfiguration, HillClimbMode};

/// Type alias for the neighbor-generation function stored inside [`HillClimbEngine`].
type NeighborFn<U> = Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>;

/// Hill-climbing search engine.
///
/// Performs local search from an initial solution by iteratively evaluating
/// neighbors. Supports two modes:
///
/// - [`HillClimbMode::Stochastic`] — accept the first improving neighbor (fast).
/// - [`HillClimbMode::SteepestAscent`] — evaluate all neighbors and accept the
///   best (more thorough, higher per-iteration cost).
///
/// Implements [`Strategy<U>`] for runtime swapping with other engines.
pub struct HillClimbEngine<U: LinearChromosome> {
    config: HillClimbConfiguration,
    neighbor_fn: NeighborFn<U>,
    current: Option<U>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> HillClimbEngine<U> {
    /// Create a new `HillClimbEngine` from the given configuration and initial solution.
    ///
    /// `neighbor_fn` must produce the neighborhood of any candidate solution.
    pub fn new(
        config: HillClimbConfiguration,
        initial: U,
        neighbor_fn: impl Fn(&U) -> Vec<U> + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            current: Some(initial),
            neighbor_fn: Arc::new(neighbor_fn),
            observer: None,
        }
    }

    /// Attach a lifecycle observer.
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

    /// Execute the hill-climbing search loop.
    ///
    /// Returns `Ok(())` on normal termination. The best candidate is available
    /// via [`HillClimbEngine::best`] after this call.
    pub fn run(&mut self) -> Result<(), GaError> {
        let mut current = self.current.take().ok_or_else(|| {
            GaError::ConfigurationError(
                "HillClimbEngine: no initial solution provided".to_string(),
            )
        })?;

        self.notify(|obs| obs.on_run_start());

        let mut no_improvement_count = 0usize;
        let mut iteration = 0usize;
        let mut all_stats: Vec<GenerationStats> =
            Vec::with_capacity(self.config.no_improvement_limit);

        loop {
            self.notify(|obs| obs.on_generation_start(iteration));

            let neighbors = (self.neighbor_fn)(&current);

            let best_neighbor = match self.config.mode {
                HillClimbMode::Stochastic => neighbors
                    .into_iter()
                    .find(|n| self.is_better(n.fitness(), current.fitness())),
                HillClimbMode::SteepestAscent => {
                    let current_fitness = current.fitness();
                    let is_better_fn = |a: &U, b: &U| -> std::cmp::Ordering {
                        if self.is_better(a.fitness(), b.fitness()) {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        }
                    };
                    neighbors
                        .into_iter()
                        .filter(|n| self.is_better(n.fitness(), current_fitness))
                        .max_by(is_better_fn)
                }
            };

            if let Some(next) = best_neighbor {
                self.notify(|obs| obs.on_new_best(iteration, &next));
                current = next;
                no_improvement_count = 0;
            } else {
                no_improvement_count += 1;
            }

            let stats = GenerationStats {
                generation: iteration,
                best_fitness: current.fitness(),
                worst_fitness: current.fitness(),
                avg_fitness: current.fitness(),
                fitness_std_dev: 0.0,
                population_size: 1,
                diversity: 0.0,
                dynamic_mutation_probability: None,
                avg_node_count: 0.0,
                cache_hits: None,
                cache_misses: None,
                true_fitness_calls: None,
            };
            self.notify(|obs| obs.on_generation_end(&stats));
            all_stats.push(stats);

            iteration += 1;

            // Early stop on fitness target.
            if let Some(target) = self.config.fitness_target {
                if self.is_better(current.fitness(), target) {
                    break;
                }
            }

            // No-improvement stopping criterion.
            let limit = match self.config.mode {
                HillClimbMode::Stochastic => self.config.no_improvement_limit,
                HillClimbMode::SteepestAscent => 1,
            };
            if no_improvement_count >= limit {
                break;
            }
        }

        self.current = Some(current);
        self.notify(|obs| obs.on_run_end(TerminationCause::GenerationLimitReached, &all_stats));

        Ok(())
    }

    /// Returns a reference to the best (current) solution, or `None` if
    /// [`HillClimbEngine::run`] has not been called.
    pub fn best(&self) -> Option<&U> {
        self.current.as_ref()
    }
}

impl<U: LinearChromosome + Clone> Strategy<U> for HillClimbEngine<U> {
    fn run(&mut self) -> Result<(), GaError> {
        HillClimbEngine::run(self)
    }

    fn best(&self) -> Option<&U> {
        HillClimbEngine::best(self)
    }
}
