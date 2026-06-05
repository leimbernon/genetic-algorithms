//! Fan-out observer that dispatches all 20 lifecycle hooks to every inner observer in insertion order.
//!
//! [`CompositeObserver<U>`] lets you combine multiple observers — for example,
//! `LogObserver` and a custom `MetricsObserver` — without writing manual dispatch
//! code. Each inner observer must implement all three observer traits
//! ([`GaObserver<U>`], [`IslandGaObserver<U>`], [`Nsga2Observer<U>`]), which is
//! captured by the [`AllObserver<U>`] supertrait.
//!
//! # Builder Pattern
//!
//! ```ignore
//! use std::sync::Arc;
//! use genetic_algorithms::{AllObserver, CompositeObserver};
//! use genetic_algorithms::observer::LogObserver;
//!
//! let composite = CompositeObserver::new()
//!     .add(Arc::new(LogObserver))
//!     .add(Arc::new(my_metrics_observer));
//!
//! ga.with_observer(Arc::new(composite));
//! ```
//!
//! # Fan-out semantics
//!
//! Every hook iterates `self.observers` in insertion order and calls the
//! corresponding hook on each inner observer. Errors in one observer do not
//! affect the others (all hooks return `()`).

use crate::cma::restart::RestartEvent;
use crate::ga::TerminationCause;
use crate::observer::{AllObserver, ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use std::sync::Arc;
use std::time::Duration;

/// Fan-out observer that dispatches all 20 lifecycle hooks to every inner
/// observer in insertion order.
///
/// Build with [`CompositeObserver::new()`] and chain [`CompositeObserver::add()`]
/// calls to register inner observers. The composite itself satisfies
/// [`AllObserver<U>`] via the blanket impl in `src/observer/mod.rs`, so it can
/// be attached to `Ga<U>`, `IslandGa<U>`, or `Nsga2Ga<U>` using the same
/// `Arc<dyn GaObserver<U>>` or `Arc<dyn IslandGaObserver<U>>` fields.
pub struct CompositeObserver<U: ChromosomeT> {
    observers: Vec<Arc<dyn AllObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT> CompositeObserver<U> {
    /// Creates an empty `CompositeObserver` with no inner observers.
    pub fn new() -> Self {
        CompositeObserver {
            observers: Vec::new(),
        }
    }

    /// Registers an inner observer and returns `self` for chaining.
    ///
    /// Observers are called in the order they are added.
    // `add` is the idiomatic builder-pattern name here and does not implement
    // arithmetic — suppress the false-positive `should_implement_trait` lint.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, obs: Arc<dyn AllObserver<U> + Send + Sync>) -> Self {
        self.observers.push(obs);
        self
    }

    /// Returns the number of observers currently registered.
    pub fn observer_count(&self) -> usize {
        self.observers.len()
    }
}

impl<U: ChromosomeT> Default for CompositeObserver<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: ChromosomeT> Clone for CompositeObserver<U> {
    /// Clones the composite by cloning each `Arc` (cheap reference-count increment).
    ///
    /// This allows the same composite to be attached to multiple GA engines
    /// without duplicating the underlying observer state.
    fn clone(&self) -> Self {
        CompositeObserver {
            observers: self.observers.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// GaObserver — 13 hooks
// ---------------------------------------------------------------------------

impl<U: ChromosomeT> GaObserver<U> for CompositeObserver<U> {
    fn on_run_start(&self) {
        for obs in &self.observers {
            obs.on_run_start();
        }
    }

    fn on_generation_start(&self, generation: usize) {
        for obs in &self.observers {
            obs.on_generation_start(generation);
        }
    }

    fn on_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        for obs in &self.observers {
            obs.on_selection_complete(generation, duration, population_size);
        }
    }

    fn on_crossover_complete(&self, generation: usize, duration: Duration, offspring_count: usize) {
        for obs in &self.observers {
            obs.on_crossover_complete(generation, duration, offspring_count);
        }
    }

    fn on_mutation_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        for obs in &self.observers {
            obs.on_mutation_complete(generation, duration, population_size);
        }
    }

    fn on_fitness_evaluation_complete(
        &self,
        generation: usize,
        duration: Duration,
        population_size: usize,
    ) {
        for obs in &self.observers {
            obs.on_fitness_evaluation_complete(generation, duration, population_size);
        }
    }

    fn on_survivor_selection_complete(
        &self,
        generation: usize,
        duration: Duration,
        population_size: usize,
    ) {
        for obs in &self.observers {
            obs.on_survivor_selection_complete(generation, duration, population_size);
        }
    }

    fn on_new_best(&self, generation: usize, best: U) {
        for obs in &self.observers {
            obs.on_new_best(generation, best.clone());
        }
    }

    fn on_stagnation(&self, generation: usize, stagnation_count: usize) {
        for obs in &self.observers {
            obs.on_stagnation(generation, stagnation_count);
        }
    }

    fn on_extension_triggered(&self, event: ExtensionEvent) {
        for obs in &self.observers {
            obs.on_extension_triggered(event);
        }
    }

    fn on_restart(&self, event: &RestartEvent) {
        for obs in &self.observers {
            obs.on_restart(event);
        }
    }

    fn on_generation_end(&self, stats: &GenerationStats) {
        for obs in &self.observers {
            obs.on_generation_end(stats);
        }
    }

    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        for obs in &self.observers {
            obs.on_run_end(cause, all_stats);
        }
    }
}

// ---------------------------------------------------------------------------
// IslandGaObserver — 4 hooks
// ---------------------------------------------------------------------------

impl<U: ChromosomeT> IslandGaObserver<U> for CompositeObserver<U> {
    fn on_island_run_start(&self, island_id: usize) {
        for obs in &self.observers {
            obs.on_island_run_start(island_id);
        }
    }

    fn on_island_run_end(&self, island_id: usize) {
        for obs in &self.observers {
            obs.on_island_run_end(island_id);
        }
    }

    fn on_island_generation_end(
        &self,
        island_id: usize,
        generation: usize,
        stats: &GenerationStats,
    ) {
        for obs in &self.observers {
            obs.on_island_generation_end(island_id, generation, stats);
        }
    }

    fn on_migration_triggered(&self, generation: usize, migration_count: usize) {
        for obs in &self.observers {
            obs.on_migration_triggered(generation, migration_count);
        }
    }
}

// ---------------------------------------------------------------------------
// Nsga2Observer — 3 hooks
// ---------------------------------------------------------------------------

impl<U: ChromosomeT> Nsga2Observer<U> for CompositeObserver<U> {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        for obs in &self.observers {
            obs.on_pareto_front_assigned(generation, front_count, population_size);
        }
    }

    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        for obs in &self.observers {
            obs.on_non_dominated_sort_complete(generation, duration_ms);
        }
    }

    fn on_crowding_distance_calculated(&self, generation: usize, duration_ms: f64) {
        for obs in &self.observers {
            obs.on_crowding_distance_calculated(generation, duration_ms);
        }
    }
}

// Note: CompositeObserver<U> automatically satisfies AllObserver<U> via the
// blanket impl in src/observer/mod.rs — no explicit `impl AllObserver` needed.
