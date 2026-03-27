use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
#[allow(deprecated)]
use super::Reporter;

/// Prints a one-line progress summary to stdout every N generations.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::reporter::SimpleReporter;
///
/// let ga = Ga::new()
///     // ...configuration...
///     .with_reporter(Box::new(SimpleReporter::new(10)))
///     .build()
///     .expect("valid config");
/// ```
pub struct SimpleReporter {
    interval: usize,
    count: usize,
}

impl SimpleReporter {
    /// Creates a reporter that prints every `interval` generations.
    ///
    /// Pass `1` to print every generation.
    pub fn new(interval: usize) -> Self {
        Self { interval, count: 0 }
    }
}

#[allow(deprecated)]
impl<U: ChromosomeT> Reporter<U> for SimpleReporter {
    fn on_generation_complete(&mut self, stats: &GenerationStats) {
        self.count += 1;
        if self.count % self.interval == 0 {
            println!(
                "[Gen {}] Best: {:.4} | Diversity: {:.4}",
                stats.generation + 1,
                stats.best_fitness,
                stats.diversity
            );
        }
    }

    fn on_finish(&mut self, _cause: TerminationCause, all_stats: &[GenerationStats]) {
        if let Some(last) = all_stats.last() {
            println!(
                "[Gen {}] Best: {:.4} | Diversity: {:.4} (finished)",
                last.generation + 1,
                last.best_fitness,
                last.diversity
            );
        }
    }
}

