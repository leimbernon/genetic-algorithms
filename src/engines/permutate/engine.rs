//! `PermutateEngine` — exhaustive evaluation of a user-supplied candidate set.

use std::sync::Arc;

use crate::configuration::ProblemSolving;
use crate::error::GaError;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::stats::GenerationStats;
use crate::traits::{ChromosomeT, Strategy};
use super::configuration::PermutateConfiguration;

/// Exhaustive permutation search engine.
///
/// Iterates over a user-supplied candidate set in order, tracking the best
/// candidate found so far. Implements [`Strategy<U>`] for runtime swapping
/// with other engines via `Box<dyn Strategy<U>>`.
///
/// Candidates must have fitness pre-evaluated before being passed to the engine —
/// `chromosome.fitness()` is called directly, not a fitness function.
pub struct PermutateEngine<U: ChromosomeT> {
    config: PermutateConfiguration,
    candidates: Vec<U>,
    best: Option<U>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT + Clone> PermutateEngine<U> {
    /// Construct a new `PermutateEngine`.
    ///
    /// * `config` — algorithm parameters.
    /// * `candidates` — pre-evaluated candidate set. `chromosome.fitness()` is
    ///   read directly; no fitness function is applied.
    pub fn new(config: PermutateConfiguration, candidates: Vec<U>) -> Self {
        Self {
            config,
            candidates,
            best: None,
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

    /// Execute the permutation search loop.
    ///
    /// Returns `Ok(())` on normal termination. The best candidate is available
    /// via [`PermutateEngine::best`] after this call.
    pub fn run(&mut self) -> Result<(), GaError> {
        self.notify(|obs| obs.on_run_start());

        let gate = self.config.safety_gate;
        let mut best: Option<U> = None;
        let mut all_stats: Vec<GenerationStats> =
            Vec::with_capacity(self.candidates.len().min(gate));

        for (idx, candidate) in self.candidates.iter().enumerate() {
            // Safety gate: stop if too many candidates evaluated.
            if idx >= gate {
                log::warn!(
                    target: "ga_events",
                    "PermutateEngine: safety gate of {} reached, stopping after {} candidates",
                    gate,
                    idx
                );
                break;
            }

            self.notify(|obs| obs.on_generation_start(idx));

            let is_new_best = match &best {
                None => true,
                Some(b) => self.is_better(candidate.fitness(), b.fitness()),
            };

            if is_new_best {
                self.notify(|obs| obs.on_new_best(idx, candidate));
                best = Some(candidate.clone());
            }

            let best_fit = best.as_ref().map(|b| b.fitness()).unwrap_or(f64::NAN);
            let stats = GenerationStats {
                generation: idx,
                best_fitness: best_fit,
                worst_fitness: candidate.fitness(),
                avg_fitness: candidate.fitness(),
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

            // Early stop on fitness target.
            if let Some(target) = self.config.fitness_target {
                if is_new_best && self.is_better(candidate.fitness(), target) {
                    break;
                }
            }
        }

        self.best = best;
        self.notify(|obs| obs.on_run_end(TerminationCause::GenerationLimitReached, &all_stats));

        Ok(())
    }

    /// Returns a reference to the best candidate found, or `None` if
    /// [`PermutateEngine::run`] has not been called.
    pub fn best(&self) -> Option<&U> {
        self.best.as_ref()
    }
}

impl<U: ChromosomeT + Clone> Strategy<U> for PermutateEngine<U> {
    fn run(&mut self) -> Result<(), GaError> {
        PermutateEngine::run(self)
    }

    fn best(&self) -> Option<&U> {
        PermutateEngine::best(self)
    }
}
