//! Reporter — deprecated lifecycle hooks for the GA execution loop.
//!
//! The [`Reporter`] trait provides four hooks that fire at key execution points:
//! [`on_start`](Reporter::on_start), [`on_generation_complete`](Reporter::on_generation_complete),
//! [`on_new_best`](Reporter::on_new_best), and [`on_finish`](Reporter::on_finish).
//!
//! **Deprecated in v2.2.0.** Superseded by the observer system (see the
//! [`observer`] module). The [`GaObserver<U>`] trait provides 11 lifecycle hooks
//! compared to Reporter's 4, supports `Arc<dyn>` sharing across rayon threads,
//! and uses `&self` instead of `&mut self`. Reporter will be removed in v3.0.0.
//!
//! Built-in implementations:
//! - [`NoopReporter`] — default, zero overhead (all hooks are no-ops)
//! - [`SimpleReporter`] — prints progress to stdout every N generations
//! - [`DurationReporter`] — reports total wall-clock timing at run end

mod duration;
mod noop;
mod simple;

pub use duration::DurationReporter;
pub use noop::NoopReporter;
pub use simple::SimpleReporter;

use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

/// Lifecycle observer for `Ga<U>`.
///
/// All methods have default no-op implementations, so implementors only need
/// to override the hooks they care about.
///
/// The `Send` supertrait allows reporters to be used with the island model
/// in future without API changes.
#[deprecated(
    since = "2.2.0",
    note = "use GaObserver<U> instead. Reporter will be removed in v3.0.0."
)]
pub trait Reporter<U: ChromosomeT>: Send {
    /// Called once before the first generation.
    fn on_start(&mut self) {}

    /// Called at the end of every generation, after statistics are collected.
    fn on_generation_complete(&mut self, _stats: &GenerationStats) {}

    /// Called whenever the population's best fitness improves.
    ///
    /// `generation` is the 0-based generation index. `best` is a clone of
    /// the new best chromosome.
    fn on_new_best(&mut self, _generation: usize, _best: U) {}

    /// Called once after the GA loop exits with the final termination cause
    /// and the full per-generation statistics history.
    fn on_finish(&mut self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}
