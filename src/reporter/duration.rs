use std::time::{Duration, Instant};
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
#[allow(deprecated)]
use super::Reporter;

/// Reports total wall-clock run time and per-generation average at the end of a run.
///
/// Timing is captured via `on_start` and `on_finish` using `std::time::Instant`.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::reporter::DurationReporter;
///
/// let ga = Ga::new()
///     // ...configuration...
///     .with_reporter(Box::new(DurationReporter::new()))
///     .build()
///     .expect("valid config");
/// ```
///
/// # Architectural Note
///
/// Per-operator phase timing (selection, crossover, mutation, survivor) would
/// require additional instrumentation hooks beyond the current four-hook
/// Reporter API (`on_start`, `on_generation_complete`, `on_new_best`,
/// `on_finish`). The four hooks fire at lifecycle boundaries, not around
/// individual operators within a generation. This reporter therefore provides
/// total wall-clock elapsed time and per-generation averages, which is the
/// most useful timing information achievable with the current hook set.
/// Per-operator timing is deferred to the Observability milestone (GaObserver
/// trait, issues #182-#186).
pub struct DurationReporter {
    start: Option<Instant>,
}

impl DurationReporter {
    /// Creates a new duration reporter.
    pub fn new() -> Self {
        Self { start: None }
    }
}

impl Default for DurationReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
impl<U: ChromosomeT> Reporter<U> for DurationReporter {
    fn on_start(&mut self) {
        self.start = Some(Instant::now());
    }

    fn on_finish(&mut self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        let elapsed = self
            .start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        let gens = all_stats.len();

        println!("Run complete ({:?}) in {:.2?} over {} generations", cause, elapsed, gens);
        if gens > 0 {
            let avg = elapsed / gens as u32;
            println!("  Avg per generation: {:.2?}", avg);
        }
        if self.start.is_none() {
            log::warn!("DurationReporter: on_start was not called before on_finish");
        }
    }
}

