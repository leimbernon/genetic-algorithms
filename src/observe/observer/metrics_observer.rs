//! Metrics-facade observer that records per-generation gauges, operator timing
//! histograms, and event counters via the [`metrics`] crate.
//!
//! Enabled only when the `observer-metrics` feature flag is active.
//! When no metrics recorder is installed, all macro calls are zero-cost no-ops.
//!
//! # Metric Catalog
//!
//! | Metric | Kind | Hook |
//! |--------|------|------|
//! | `ga.generation.best_fitness` | gauge | `on_generation_end` |
//! | `ga.generation.mean_fitness` | gauge | `on_generation_end` |
//! | `ga.generation.diversity` | gauge | `on_generation_end` |
//! | `ga.operator.selection_ms` | histogram | `on_selection_complete` |
//! | `ga.operator.crossover_ms` | histogram | `on_crossover_complete` |
//! | `ga.operator.mutation_ms` | histogram | `on_mutation_complete` |
//! | `ga.operator.fitness_eval_ms` | histogram | `on_fitness_evaluation_complete` |
//! | `ga.operator.survivor_ms` | histogram | `on_survivor_selection_complete` |
//! | `ga.event.new_best` | counter | `on_new_best` |
//! | `ga.event.stagnation` | counter | `on_stagnation` |
//! | `ga.event.extension_triggered` | counter | `on_extension_triggered` |
//!
//! # Usage
//!
//! ```ignore
//! use std::sync::Arc;
//! use genetic_algorithms::MetricsObserver;
//!
//! let mut ga = Ga::new()
//!     // ... configuration ...
//!     .with_observer(Arc::new(MetricsObserver::new("experiment_42")))
//!     .build()
//!     .unwrap();
//! ```

use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use std::time::Duration;

/// Observer that records per-generation metrics via the [`metrics`] facade crate.
///
/// Construct with [`MetricsObserver::new`], passing a static string `run_id`
/// label that will be attached to every metric as a label/tag — used to
/// disambiguate runs in shared dashboards.
///
/// The `metrics` recorder (e.g., `metrics-exporter-prometheus`) is the user's
/// responsibility. When no recorder is installed, all metric calls are
/// zero-cost no-ops (noop recorder installed by default).
pub struct MetricsObserver {
    run_id: &'static str,
}

impl MetricsObserver {
    /// Create a new `MetricsObserver`.
    ///
    /// `run_id` is attached as a label to every emitted metric.
    /// Use a descriptive static string such as `"experiment_42"` or `"production_run"`.
    pub fn new(run_id: &'static str) -> Self {
        Self { run_id }
    }

    /// Returns the run_id this observer was constructed with.
    pub fn run_id(&self) -> &'static str {
        self.run_id
    }
}

impl Default for MetricsObserver {
    fn default() -> Self {
        Self::new("default")
    }
}

impl<U: ChromosomeT> GaObserver<U> for MetricsObserver {
    fn on_selection_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _population_size: usize,
    ) {
        metrics::histogram!("ga.operator.selection_ms", "run_id" => self.run_id)
            .record(duration.as_secs_f64() * 1000.0);
    }

    fn on_crossover_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _offspring_count: usize,
    ) {
        metrics::histogram!("ga.operator.crossover_ms", "run_id" => self.run_id)
            .record(duration.as_secs_f64() * 1000.0);
    }

    fn on_mutation_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _population_size: usize,
    ) {
        metrics::histogram!("ga.operator.mutation_ms", "run_id" => self.run_id)
            .record(duration.as_secs_f64() * 1000.0);
    }

    fn on_fitness_evaluation_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _population_size: usize,
    ) {
        metrics::histogram!("ga.operator.fitness_eval_ms", "run_id" => self.run_id)
            .record(duration.as_secs_f64() * 1000.0);
    }

    fn on_survivor_selection_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _population_size: usize,
    ) {
        metrics::histogram!("ga.operator.survivor_ms", "run_id" => self.run_id)
            .record(duration.as_secs_f64() * 1000.0);
    }

    fn on_new_best(&self, _generation: usize, _best: U) {
        metrics::counter!("ga.event.new_best", "run_id" => self.run_id).increment(1);
    }

    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {
        metrics::counter!("ga.event.stagnation", "run_id" => self.run_id).increment(1);
    }

    fn on_extension_triggered(&self, _event: ExtensionEvent) {
        metrics::counter!("ga.event.extension_triggered", "run_id" => self.run_id).increment(1);
    }

    fn on_generation_end(&self, stats: &GenerationStats) {
        metrics::gauge!("ga.generation.best_fitness", "run_id" => self.run_id)
            .set(stats.best_fitness);
        metrics::gauge!("ga.generation.mean_fitness", "run_id" => self.run_id)
            .set(stats.avg_fitness);
        metrics::gauge!("ga.generation.diversity", "run_id" => self.run_id).set(stats.diversity);
    }
}

impl<U: ChromosomeT> IslandGaObserver<U> for MetricsObserver {}

impl<U: ChromosomeT> Nsga2Observer<U> for MetricsObserver {}
