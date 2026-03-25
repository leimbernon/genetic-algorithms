//! Structured lifecycle observer for the GA execution loop.
//!
//! This module provides the [`GaObserver`] trait, which exposes 12 hooks that
//! fire at precise points in the GA run loop. Unlike the legacy [`Reporter`]
//! trait, observers:
//!
//! - Are stored as `Arc<dyn GaObserver + Send + Sync>` (sharable across island threads)
//! - Receive `&self` (not `&mut self`), enabling interior mutability patterns
//! - Cover 12 lifecycle, operator-timing, and special-event hooks (vs 4 in Reporter)
//! - Require `Send + Sync` for safe use in rayon parallel regions
//!
//! # Hooks
//!
//! | Hook | When it fires |
//! |------|--------------|
//! | `on_run_start` | Once before the first generation |
//! | `on_generation_start` | Start of each generation, before any operators |
//! | `on_selection_complete` | After parent selection |
//! | `on_crossover_complete` | After crossover produces offspring |
//! | `on_mutation_complete` | After mutation is applied |
//! | `on_fitness_evaluation_complete` | After fitness evaluation of new population |
//! | `on_survivor_selection_complete` | After survivor selection prunes population |
//! | `on_new_best` | When the population's best fitness improves |
//! | `on_stagnation` | Each time the stagnation counter increments |
//! | `on_extension_triggered` | When an extension strategy fires |
//! | `on_generation_end` | End of each generation, after statistics collected |
//! | `on_run_end` | Once after the GA loop exits |
//!
//! [`Reporter`]: crate::reporter::Reporter

use std::time::Duration;
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

/// Payload for the [`GaObserver::on_extension_triggered`] hook.
///
/// Stack-allocated and `Copy`-able — zero heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct ExtensionEvent {
    /// The generation at which the extension fired.
    pub generation: usize,
    /// Population diversity at the time of extension.
    pub diversity: f64,
    /// Name of the extension strategy (e.g. `"MassExtinction"`).
    pub extension_type: &'static str,
}

/// Structured lifecycle observer for [`Ga<U>`](crate::ga::Ga).
///
/// All methods have default no-op implementations — implement only the hooks
/// you need. The `Send + Sync` supertraits are required for safe sharing
/// across rayon threads (island model) via `Arc`.
///
/// # Differences from [`Reporter`](crate::reporter::Reporter)
///
/// | Aspect | Reporter | GaObserver |
/// |--------|----------|------------|
/// | Storage | `Box<dyn Reporter + Send>` | `Arc<dyn GaObserver + Send + Sync>` |
/// | Mutability | `&mut self` | `&self` |
/// | Hooks | 4 lifecycle | 12 (lifecycle + operator + special) |
/// | Thread safety | `Send` only | `Send + Sync` |
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    /// Called once before the first generation.
    fn on_run_start(&self) {}
    /// Called at the start of each generation, before any operators run.
    fn on_generation_start(&self, _generation: usize) {}
    /// Called after parent selection completes.
    fn on_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    /// Called after crossover produces offspring.
    fn on_crossover_complete(&self, _generation: usize, _duration: Duration, _offspring_count: usize) {}
    /// Called after mutation is applied to offspring.
    fn on_mutation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    /// Called after fitness evaluation of the new population.
    fn on_fitness_evaluation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    /// Called after survivor selection prunes the population.
    fn on_survivor_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    /// Called when the population's best fitness improves.
    fn on_new_best(&self, _generation: usize, _best: U) {}
    /// Called each time the stagnation counter increments.
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {}
    /// Called when an extension strategy fires due to low diversity.
    fn on_extension_triggered(&self, _event: ExtensionEvent) {}
    /// Called at the end of each generation, after statistics are collected.
    fn on_generation_end(&self, _stats: &GenerationStats) {}
    /// Called once after the GA loop exits.
    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}

/// Zero-sized no-op observer. All hooks use their default empty bodies.
///
/// Useful as a compile-check type or as a placeholder.
pub struct NoopObserver;

impl<U: ChromosomeT> GaObserver<U> for NoopObserver {}
