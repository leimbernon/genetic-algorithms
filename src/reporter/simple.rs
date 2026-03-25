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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::stats::GenerationStats;

    fn make_stats(generation: usize) -> GenerationStats {
        GenerationStats::from_fitness_values(generation, &[1.0, 2.0, 3.0], true)
    }

    // Test 1: SimpleReporter::new(3) only fires at intervals of 3
    #[test]
    fn simple_reporter_fires_at_interval() {
        let mut r = SimpleReporter::new(3);
        // Simulate 9 generations — count should match at 3, 6, 9
        for i in 0..9 {
            <SimpleReporter as Reporter<BinaryChromosome>>::on_generation_complete(&mut r, &make_stats(i));
        }
        // count should be 9 after 9 calls
        assert_eq!(r.count, 9);
        // Only generations 3, 6, 9 should have triggered output (count % 3 == 0)
        // We verify via the count field (internal state)
        assert_eq!(r.count % r.interval, 0);
    }

    // Test 2: SimpleReporter count increments correctly per call
    #[test]
    fn simple_reporter_count_increments() {
        let mut r = SimpleReporter::new(5);
        for i in 0..4 {
            <SimpleReporter as Reporter<BinaryChromosome>>::on_generation_complete(&mut r, &make_stats(i));
        }
        assert_eq!(r.count, 4);
        // Not yet at interval, no print would have fired
        assert_ne!(r.count % r.interval, 0);
    }

    // Test 3: SimpleReporter on_finish runs without panic
    #[test]
    fn simple_reporter_on_finish_runs() {
        let mut r: SimpleReporter = SimpleReporter::new(10);
        let stats = vec![make_stats(0), make_stats(1)];
        // Should not panic
        <SimpleReporter as Reporter<BinaryChromosome>>::on_finish(
            &mut r,
            TerminationCause::GenerationLimitReached,
            &stats,
        );
    }

    // Test 4: SimpleReporter on_finish with empty stats does not panic
    #[test]
    fn simple_reporter_on_finish_empty_stats() {
        let mut r: SimpleReporter = SimpleReporter::new(10);
        <SimpleReporter as Reporter<BinaryChromosome>>::on_finish(
            &mut r,
            TerminationCause::GenerationLimitReached,
            &[],
        );
        // No panic — on_finish gracefully handles empty stats
    }
}
