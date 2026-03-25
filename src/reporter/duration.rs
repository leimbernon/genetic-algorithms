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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::stats::GenerationStats;

    fn make_stats(generation: usize) -> GenerationStats {
        GenerationStats::from_fitness_values(generation, &[1.0, 2.0, 3.0], true)
    }

    // Test 4: DurationReporter::new() creates with start = None
    #[test]
    fn duration_reporter_new_has_no_start() {
        let r = DurationReporter::new();
        assert!(r.start.is_none());
    }

    // Test 5: DurationReporter on_start sets start to Some(Instant)
    #[test]
    fn duration_reporter_on_start_sets_instant() {
        let mut r = DurationReporter::new();
        <DurationReporter as Reporter<BinaryChromosome>>::on_start(&mut r);
        assert!(r.start.is_some());
    }

    // Test 6: DurationReporter on_finish does not panic when start is None
    #[test]
    fn duration_reporter_on_finish_no_panic_without_start() {
        let mut r = DurationReporter::new();
        let stats = vec![make_stats(0), make_stats(1)];
        // Should not panic even if on_start was not called
        <DurationReporter as Reporter<BinaryChromosome>>::on_finish(
            &mut r,
            TerminationCause::GenerationLimitReached,
            &stats,
        );
    }

    // Test 7: DurationReporter on_finish does not panic when all_stats is empty
    #[test]
    fn duration_reporter_on_finish_empty_stats() {
        let mut r = DurationReporter::new();
        <DurationReporter as Reporter<BinaryChromosome>>::on_finish(
            &mut r,
            TerminationCause::GenerationLimitReached,
            &[],
        );
    }

    // Test 8: DurationReporter Default impl works
    #[test]
    fn duration_reporter_default() {
        let r = DurationReporter::default();
        assert!(r.start.is_none());
    }
}
