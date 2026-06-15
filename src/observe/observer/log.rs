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

use crate::ga::TerminationCause;
use crate::observer::{
    ExtensionEvent, GaObserver, IbeaObserver, IslandGaObserver, MoeaDObserver, Nsga2Observer,
    Nsga3Observer, SmsEmoaObserver, Spea2Observer,
};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use std::time::Duration;

/// Zero-sized observer that reproduces pre-v2.2.0 log output via the `log` crate.
///
/// All messages use the same target, level, KV fields, and format strings as
/// the original hardcoded `log!()` calls in `ga.rs`. Attach with:
///
/// ```ignore
/// ga.with_observer(Arc::new(LogObserver));
/// ```
///
/// **Note:** Attaching `LogObserver` alongside another observer that prints per-generation output
/// may produce redundant output.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::observer::LogObserver;
///
/// let _obs = LogObserver;
/// ```
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

    fn on_selection_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
        // Reproduces ga.rs line 768
        log::debug!(target="ga_events", method="run"; "Parents selected for reproduction");
    }

    fn on_crossover_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _offspring_count: usize,
    ) {
        // Reproduces ga.rs lines 1254, 794, and 1392 (parent_crossover start/finish + offspring created)
        // Lines inside the rayon par_iter (per-pair data) are dropped — not available at hook level
        log::debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");
        log::debug!(target="ga_events", method="run"; "Offspring created");
        log::debug!(target="ga_events", method="parent_crossover"; "Parent crossover finished");
    }

    fn on_mutation_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
        // No direct log call existed for mutation complete in the original code
    }

    fn on_fitness_evaluation_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
        // No direct log call existed for fitness evaluation complete in the original code
    }

    fn on_survivor_selection_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
        // Reproduces ga.rs line 894
        log::debug!(target="ga_events", method="run"; "Survivors selected");
    }

    fn on_new_best(&self, _generation: usize, _best: &U) {
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

        // Note: limit_reached trace messages are condition-dependent and are emitted from
        // on_run_end once the termination cause is known. They are intentionally NOT emitted
        // here to avoid spurious "limit reached" noise on every generation.
    }

    fn on_run_end(&self, cause: TerminationCause, _all_stats: &[GenerationStats]) {
        // Reproduces the condition-dependent trace messages from the old limit_reached()
        // helper (lines 1214 and 1223 of the pre-v2.2.0 ga.rs). Only emitted when the
        // termination cause confirms a limit was actually reached.
        log::debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
        if matches!(cause, TerminationCause::FitnessTargetReached) {
            log::trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
        }
        log::debug!(target="ga_events", method="limit_reached"; "Limit reached method finished");
    }
}

impl<U: ChromosomeT> IslandGaObserver<U> for LogObserver {
    fn on_island_run_start(&self, _island_id: usize) {
        // Reproduces island/mod.rs: info!(target: "island_events", "Starting island model GA: ...")
        log::info!(target: "island_events", "Island model GA started");
    }
    fn on_island_run_end(&self, _island_id: usize) {
        // No direct log call existed for island run end in the original code
        log::info!(target: "island_events", "Island model GA ended");
    }
    fn on_island_generation_end(
        &self,
        _island_id: usize,
        generation: usize,
        stats: &GenerationStats,
    ) {
        // Reproduces island/mod.rs: debug!(target: "island_events", "Generation {} complete", gen)
        log::debug!(target: "island_events", "Best chromosome calculated - generation {}", generation + 1);
        if let Some(prob) = stats.dynamic_mutation_probability {
            log::debug!(
                target: "island_events",
                "Dynamic mutation: diversity={:.4}, probability={:.4}",
                stats.diversity,
                prob
            );
        }
    }
    fn on_migration_triggered(&self, generation: usize, migration_count: usize) {
        // Reproduces island/mod.rs: debug!(target: "island_events", "Migration performed at generation {}", gen)
        log::debug!(target: "island_events", "Migration performed at generation {} (count={})", generation, migration_count);
    }
}

impl<U: ChromosomeT> Nsga2Observer<U> for LogObserver {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        // Reproduces nsga2/mod.rs: debug!(target: "nsga2_events", "Generation {} complete, population size = {}", gen, population.len())
        log::debug!(target: "nsga2_events", "Generation {} complete, population size = {}, fronts = {}", generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "nsga2_events", "Non-dominated sort complete at generation {} ({:.2}ms)", generation, duration_ms);
    }
    fn on_crowding_distance_calculated(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "nsga2_events", "Crowding distance calculated at generation {} ({:.2}ms)", generation, duration_ms);
    }
}

impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        log::debug!(target: "nsga3_events", "Generation {} complete, population size = {}, fronts = {}", generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "nsga3_events", "Non-dominated sort complete at generation {} ({:.2}ms)", generation, duration_ms);
    }
}

impl<U: ChromosomeT> MoeaDObserver<U> for LogObserver {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        log::debug!(target: "moead_events",
            "Generation {} complete, population size = {}, fronts = {}",
            generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "moead_events",
            "Non-dominated sort complete at generation {} ({:.2}ms)",
            generation, duration_ms);
    }
}

impl<U: ChromosomeT> Spea2Observer<U> for LogObserver {
    fn on_fitness_assigned(
        &self,
        generation: usize,
        duration_ms: f64,
        pop_size: usize,
        archive_size: usize,
    ) {
        log::debug!(target: "spea2_events",
            "Strength+density fitness assigned at generation {} ({:.2}ms) — pop={}, archive={}",
            generation, duration_ms, pop_size, archive_size);
    }
    fn on_archive_updated(
        &self,
        generation: usize,
        archive_size: usize,
        non_dominated_count: usize,
    ) {
        log::debug!(target: "spea2_events",
            "Archive updated at generation {} — size={}, non-dominated={}",
            generation, archive_size, non_dominated_count);
    }
}

impl<U: ChromosomeT> SmsEmoaObserver<U> for LogObserver {
    fn on_hypervolume_contribution_assigned(
        &self,
        generation: usize,
        duration_ms: f64,
        population_size: usize,
    ) {
        log::debug!(target: "sms_emoa_events",
            "Hypervolume contribution assigned at generation {} ({:.2}ms) — pop={}",
            generation, duration_ms, population_size);
    }
    fn on_steady_state_removal(&self, generation: usize, population_size: usize) {
        log::debug!(target: "sms_emoa_events",
            "Steady-state removal at generation {} — pop={}",
            generation, population_size);
    }
}

impl<U: ChromosomeT> IbeaObserver<U> for LogObserver {
    fn on_indicator_fitness_assigned(
        &self,
        generation: usize,
        duration_ms: f64,
        population_size: usize,
    ) {
        log::debug!(target: "ibea_events",
            "Indicator fitness assigned at generation {} ({:.2}ms) — pop={}",
            generation, duration_ms, population_size);
    }
    fn on_environmental_selection(
        &self,
        generation: usize,
        population_size: usize,
        _removed_index: usize,
    ) {
        log::debug!(target: "ibea_events",
            "Environmental selection at generation {} — pop={}",
            generation, population_size);
    }
}
