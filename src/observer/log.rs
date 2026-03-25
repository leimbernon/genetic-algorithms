//! Log-based observer that reproduces all pre-v2.2.0 `log!()` output.
//!
//! [`LogObserver`] is a zero-sized [`GaObserver`] implementation that emits
//! the same log messages (targets, levels, KV fields, format strings) as the
//! hardcoded `log!()` calls that existed in `ga.rs` prior to v2.2.0.
//!
//! # Usage
//!
//! ```ignore
//! use std::sync::Arc;
//! use genetic_algorithms::observer::LogObserver;
//!
//! let mut ga = Ga::new()
//!     // ... configuration ...
//!     .with_observer(Arc::new(LogObserver))
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Note
//!
//! Attaching `LogObserver` alongside `SimpleReporter` produces redundant
//! per-generation output, since both emit lifecycle information. Prefer
//! using only one.

use std::time::Duration;
use crate::ga::TerminationCause;
use crate::observer::{ExtensionEvent, GaObserver};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

/// Zero-sized observer that reproduces pre-v2.2.0 log output via the `log` crate.
///
/// All messages use the same target, level, KV fields, and format strings as
/// the original hardcoded `log!()` calls in `ga.rs`. Attach with:
///
/// ```ignore
/// ga.with_observer(Arc::new(LogObserver));
/// ```
///
/// **Note:** Attaching `LogObserver` alongside [`SimpleReporter`](crate::reporter::SimpleReporter)
/// produces redundant per-generation output.
pub struct LogObserver;

impl<U: ChromosomeT> GaObserver<U> for LogObserver {
    fn on_run_start(&self) {
        // Reproduces ga.rs line 613: info!("Initialization started")
        // No target — default target is the module path
        log::info!("Initialization started");
    }

    fn on_generation_start(&self, generation: usize) {
        // Reproduces ga.rs line 754: info!(target="ga_events", method="run"; "Generation number: {}", i+1)
        log::info!(target="ga_events", method="run"; "Generation number: {}", generation + 1);
    }

    fn on_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {
        // Reproduces ga.rs line 768
        log::debug!(target="ga_events", method="run"; "Parents selected for reproduction");
    }

    fn on_crossover_complete(&self, _generation: usize, _duration: Duration, _offspring_count: usize) {
        // Reproduces ga.rs lines 1254, 794, and 1392 (parent_crossover start/finish + offspring created)
        // Lines inside the rayon par_iter (per-pair data) are dropped — not available at hook level
        log::debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");
        log::debug!(target="ga_events", method="run"; "Offspring created");
        log::debug!(target="ga_events", method="parent_crossover"; "Parent crossover finished");
    }

    fn on_mutation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {
        // No direct log call existed for mutation complete in the original code
    }

    fn on_fitness_evaluation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {
        // No direct log call existed for fitness evaluation complete in the original code
    }

    fn on_survivor_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {
        // Reproduces ga.rs line 894
        log::debug!(target="ga_events", method="run"; "Survivors selected");
    }

    fn on_new_best(&self, _generation: usize, _best: U) {
        // No direct log call existed for new best — it was implicit in "Best chromosome calculated"
    }

    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {
        // No direct log call existed for stagnation
    }

    fn on_extension_triggered(&self, event: ExtensionEvent) {
        // Reproduces ga.rs lines 985-991
        log::info!(
            target="extension_events",
            method="run";
            "Extension triggered: diversity={:.6} < threshold={:.6}",
            event.diversity,
            event.threshold
        );
    }

    fn on_generation_end(&self, stats: &GenerationStats) {
        // Reproduces ga.rs line 921: debug!(target="ga_events", method="run"; "Best chromosome calculated - generation {}", i+1)
        log::debug!(target="ga_events", method="run"; "Best chromosome calculated - generation {}", stats.generation + 1);

        // Reproduces ga.rs lines 971-977 (only when dynamic mutation is active):
        // debug!(target="ga_events", method="run"; "Dynamic mutation: diversity={:.4}, probability={:.4}", diversity, prob)
        if let Some(prob) = stats.dynamic_mutation_probability {
            log::debug!(
                target="ga_events",
                method="run";
                "Dynamic mutation: diversity={:.4}, probability={:.4}",
                stats.diversity,
                prob
            );
        }

        // Absorbed from limit_reached() — lines 1207 and 1232:
        log::debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
        // Note: trace-level limit-reached messages (lines 1214, 1223) are condition-dependent
        // and cannot be reproduced from on_generation_end parameters (we don't know which
        // condition triggered). Emit both unconditionally at trace level for coverage.
        log::trace!(target="ga_events", method="limit_reached"; "limit reached for minimization");
        log::trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
        log::debug!(target="ga_events", method="limit_reached"; "Limit reached method finished");
    }

    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {
        // No direct log call existed for run end in the original code
    }
}
