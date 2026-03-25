//! Lifecycle reporters for the GA execution loop.
//!
//! The [`Reporter`] trait provides four hooks that fire at key execution points:
//! [`on_start`](Reporter::on_start), [`on_generation_complete`](Reporter::on_generation_complete),
//! [`on_new_best`](Reporter::on_new_best), and [`on_finish`](Reporter::on_finish).
//!
//! Built-in implementations:
//! - [`NoopReporter`] — default, zero overhead (all hooks are no-ops)
//! - [`SimpleReporter`] — prints progress to stdout every N generations
//! - [`DurationReporter`] — reports total wall-clock timing at run end

mod noop;
mod simple;
mod duration;

pub use noop::NoopReporter;
pub use simple::SimpleReporter;
pub use duration::DurationReporter;

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

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::stats::GenerationStats;

    // Test 1: NoopReporter implements Reporter<BinaryChromosome>
    #[test]
    fn noop_reporter_satisfies_reporter_trait() {
        let mut r: NoopReporter = NoopReporter;
        // All calls should compile and do nothing
        <NoopReporter as Reporter<BinaryChromosome>>::on_start(&mut r);
        let stats = GenerationStats::from_fitness_values(0, &[1.0, 2.0], false);
        <NoopReporter as Reporter<BinaryChromosome>>::on_generation_complete(&mut r, &stats);
        <NoopReporter as Reporter<BinaryChromosome>>::on_finish(
            &mut r,
            TerminationCause::GenerationLimitReached,
            &[stats],
        );
    }

    // Test 2: Reporter trait has 4 hooks with default no-op bodies (compile check)
    #[test]
    fn reporter_trait_has_four_default_hooks() {
        struct AllDefaults;
        impl Reporter<BinaryChromosome> for AllDefaults {}

        let mut r = AllDefaults;
        r.on_start();
        let stats = GenerationStats::from_fitness_values(0, &[0.5], false);
        r.on_generation_complete(&stats);
        r.on_finish(TerminationCause::FitnessTargetReached, &[]);
    }

    // Test 3: Custom reporter can override only on_generation_complete, leave others as defaults
    #[test]
    fn partial_override_only_on_generation_complete() {
        struct CountingReporter {
            count: usize,
        }
        impl Reporter<BinaryChromosome> for CountingReporter {
            fn on_generation_complete(&mut self, _stats: &GenerationStats) {
                self.count += 1;
            }
        }

        let mut r = CountingReporter { count: 0 };
        r.on_start(); // default no-op
        let stats = GenerationStats::from_fitness_values(1, &[0.1, 0.2, 0.3], false);
        r.on_generation_complete(&stats);
        r.on_generation_complete(&stats);
        assert_eq!(r.count, 2);
        r.on_finish(TerminationCause::StagnationReached, &[]); // default no-op
    }

    // Test 4: Box<dyn Reporter<BinaryChromosome> + Send> is object-safe (compiles)
    #[test]
    fn reporter_is_object_safe() {
        let r: Box<dyn Reporter<BinaryChromosome> + Send> = Box::new(NoopReporter);
        // Just confirm it compiles and can be held as a trait object
        drop(r);
    }
}
