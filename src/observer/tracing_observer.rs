//! Tracing-based observer that emits structured spans and events for GA lifecycle hooks.
//!
//! [`TracingObserver`] implements [`GaObserver`] using the [`tracing`] crate, enabling
//! integration with OpenTelemetry, Jaeger, or any subscriber compatible with the tracing
//! ecosystem.
//!
//! # Span Hierarchy
//!
//! ```text
//! ga_run  (INFO span — wraps the entire GA run)
//!   └── ga_generation  (DEBUG span — wraps one generation)
//!         ├── trace: selection_complete
//!         ├── trace: crossover_complete
//!         ├── trace: mutation_complete
//!         ├── trace: fitness_evaluation_complete
//!         ├── trace: survivor_selection_complete
//!         ├── info:  new_best
//!         ├── warn:  stagnation detected
//!         ├── info:  extension_triggered
//!         └── debug: generation_end
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use std::sync::Arc;
//! use genetic_algorithms::TracingObserver;
//!
//! let mut ga = Ga::new()
//!     // ... configuration ...
//!     .with_observer(Arc::new(TracingObserver::new()))
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Safety
//!
//! `TracingObserver` stores `Mutex<Option<Span>>` (not `EnteredSpan`) because
//! `EnteredSpan` is `!Send`. This design satisfies the `GaObserver: Send + Sync`
//! supertrait requirements while correctly tracking the parent-child span relationship.
//!
//! # No `log::*` calls
//!
//! This file intentionally contains zero `log::*` calls to prevent infinite recursion
//! when a `LogTracer` is installed (which routes `log` events into the `tracing`
//! subscriber — emitting `log::*` here would loop).

use std::sync::Mutex;
use std::time::Duration;
use tracing::Span;
use crate::ga::TerminationCause;
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

/// Tracing-based [`GaObserver`] that emits structured spans and events for all 12 hooks.
///
/// Attach via `Arc::new(TracingObserver::new())` or `Arc::new(TracingObserver::default())`.
///
/// Each GA run produces a top-level `ga_run` INFO span. Each generation produces a
/// nested `ga_generation` DEBUG span. Individual operator timings are emitted as TRACE
/// events inside the generation span.
///
/// # Thread Safety
///
/// `Mutex<Option<Span>>` is used instead of `Option<EnteredSpan>` because `EnteredSpan`
/// is `!Send`. The span hierarchy (parent-child relationship) is established when
/// `on_generation_start` enters the `run_span` before creating `gen_span`.
pub struct TracingObserver {
    run_span: Mutex<Option<Span>>,
    gen_span: Mutex<Option<Span>>,
}

impl TracingObserver {
    /// Create a new `TracingObserver` with no active spans.
    pub fn new() -> Self {
        Self {
            run_span: Mutex::new(None),
            gen_span: Mutex::new(None),
        }
    }
}

impl Default for TracingObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: ChromosomeT> GaObserver<U> for TracingObserver {
    /// Creates the top-level `ga_run` INFO span and emits a run-start event.
    fn on_run_start(&self) {
        let span = tracing::info_span!("ga_run");
        tracing::info!(parent: &span, "ga run started");
        if let Ok(mut guard) = self.run_span.lock() {
            *guard = Some(span);
        }
    }

    /// Creates a `ga_generation` DEBUG span nested inside `ga_run`.
    ///
    /// The `run_span` is entered first to establish the parent-child relationship
    /// so the subscriber correctly nests `ga_generation` under `ga_run`.
    fn on_generation_start(&self, generation: usize) {
        let run_guard = self.run_span.lock().ok();
        // Enter the run span so that the new gen span inherits it as parent.
        let _run_entered = run_guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());

        let gen_span = tracing::debug_span!("ga_generation", generation);
        if let Ok(mut guard) = self.gen_span.lock() {
            *guard = Some(gen_span);
        }
        // _run_entered drops here, which is fine — the parent relationship is
        // already recorded by the subscriber when the gen span was created.
    }

    /// Emits a TRACE event with selection timing inside the current generation span.
    fn on_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let duration_ms = duration.as_secs_f64() * 1000.0;
        tracing::trace!(generation, duration_ms, population_size, "selection_complete");
    }

    /// Emits a TRACE event with crossover timing inside the current generation span.
    fn on_crossover_complete(&self, generation: usize, duration: Duration, offspring_count: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let duration_ms = duration.as_secs_f64() * 1000.0;
        tracing::trace!(generation, duration_ms, offspring_count, "crossover_complete");
    }

    /// Emits a TRACE event with mutation timing inside the current generation span.
    fn on_mutation_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let duration_ms = duration.as_secs_f64() * 1000.0;
        tracing::trace!(generation, duration_ms, population_size, "mutation_complete");
    }

    /// Emits a TRACE event with fitness evaluation timing inside the current generation span.
    fn on_fitness_evaluation_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let duration_ms = duration.as_secs_f64() * 1000.0;
        tracing::trace!(generation, duration_ms, population_size, "fitness_evaluation_complete");
    }

    /// Emits a TRACE event with survivor selection timing inside the current generation span.
    fn on_survivor_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let duration_ms = duration.as_secs_f64() * 1000.0;
        tracing::trace!(generation, duration_ms, population_size, "survivor_selection_complete");
    }

    /// Emits an INFO event with the new best fitness inside the current generation span.
    fn on_new_best(&self, generation: usize, best: U) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let fitness = best.fitness();
        tracing::info!(generation, fitness, "new_best");
    }

    /// Emits a WARN event with stagnation count inside the current generation span.
    fn on_stagnation(&self, generation: usize, stagnation_count: usize) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        tracing::warn!(generation, stagnation_count, "stagnation detected");
    }

    /// Emits an INFO event with extension metadata inside the current generation span.
    fn on_extension_triggered(&self, event: ExtensionEvent) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        tracing::info!(
            generation = event.generation,
            diversity = event.diversity,
            extension_type = event.extension_type,
            threshold = event.threshold,
            "extension_triggered"
        );
    }

    /// Emits a DEBUG event with full generation statistics, then closes the generation span.
    fn on_generation_end(&self, stats: &GenerationStats) {
        let guard = self.gen_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        tracing::debug!(
            generation = stats.generation,
            best_fitness = stats.best_fitness,
            avg_fitness = stats.avg_fitness,
            worst_fitness = stats.worst_fitness,
            fitness_std_dev = stats.fitness_std_dev,
            population_size = stats.population_size,
            diversity = stats.diversity,
            "generation_end"
        );
        // Drop the entered guard before locking gen_span to clear the span.
        drop(_entered);
        drop(guard);
        if let Ok(mut gen_guard) = self.gen_span.lock() {
            *gen_guard = None;
        }
    }

    /// Emits an INFO event with termination cause and total generations, then closes the run span.
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        let guard = self.run_span.lock().ok();
        let _entered = guard
            .as_deref()
            .and_then(|opt| opt.as_ref())
            .map(|span| span.enter());
        let total_generations = all_stats.len();
        tracing::info!(cause = ?cause, total_generations, "ga run ended");
        // Drop the entered guard before clearing the run span.
        drop(_entered);
        drop(guard);
        if let Ok(mut run_guard) = self.run_span.lock() {
            *run_guard = None;
        }
    }
}

impl<U: ChromosomeT> IslandGaObserver<U> for TracingObserver {}

impl<U: ChromosomeT> Nsga2Observer<U> for TracingObserver {}
