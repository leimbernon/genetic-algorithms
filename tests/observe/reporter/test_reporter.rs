#![allow(deprecated)]
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::reporter::Reporter;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct SpyData {
    start_count: usize,
    generation_complete_count: usize,
    new_best_count: usize,
    finish_count: usize,
    finish_cause: Option<TerminationCause>,
    finish_stats_len: usize,
}

struct SpyReporter {
    data: Arc<Mutex<SpyData>>,
}

impl SpyReporter {
    fn new(data: Arc<Mutex<SpyData>>) -> Self {
        Self { data }
    }
}

impl Reporter<BinaryChromosome> for SpyReporter {
    fn on_start(&mut self) {
        self.data.lock().unwrap().start_count += 1;
    }
    fn on_generation_complete(&mut self, _stats: &GenerationStats) {
        self.data.lock().unwrap().generation_complete_count += 1;
    }
    fn on_new_best(&mut self, _generation: usize, _best: BinaryChromosome) {
        self.data.lock().unwrap().new_best_count += 1;
    }
    fn on_finish(&mut self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        let mut d = self.data.lock().unwrap();
        d.finish_count += 1;
        d.finish_cause = Some(cause);
        d.finish_stats_len = all_stats.len();
    }
}

fn build_test_ga(max_gens: usize, spy: SpyReporter) -> Ga<BinaryChromosome> {
    Ga::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(max_gens)
        .with_reporter(Box::new(spy))
        .build()
        .expect("valid config")
}

/// Test 1: on_start fires exactly once
#[test]
fn test_reporter_on_start_fires_once() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert_eq!(d.start_count, 1, "on_start should fire exactly once");
}

/// Test 2: on_generation_complete fires exactly max_generations times
#[test]
fn test_reporter_on_generation_complete_count() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert_eq!(
        d.generation_complete_count, 10,
        "on_generation_complete should fire once per generation"
    );
}

/// Test 3: on_new_best fires at least once (first generation always improves from default)
#[test]
fn test_reporter_on_new_best_fires() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert!(
        d.new_best_count >= 1,
        "on_new_best should fire at least once"
    );
}

/// Test 4: on_new_best fires fewer times than on_generation_complete (not every gen improves)
#[test]
fn test_reporter_on_new_best_less_than_total_gens() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    // Use more generations and larger population so convergence is likely
    let mut ga = Ga::new()
        .with_population_size(50)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(50)
        .with_reporter(Box::new(spy))
        .build()
        .expect("valid config");
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert!(
        d.new_best_count < d.generation_complete_count,
        "on_new_best ({}) should fire fewer times than on_generation_complete ({})",
        d.new_best_count,
        d.generation_complete_count
    );
}

/// Test 5: on_finish fires exactly once
#[test]
fn test_reporter_on_finish_fires_once() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert_eq!(d.finish_count, 1, "on_finish should fire exactly once");
}

/// Test 6: on_finish receives correct TerminationCause when running to generation limit
#[test]
fn test_reporter_on_finish_termination_cause() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert_eq!(
        d.finish_cause,
        Some(TerminationCause::GenerationLimitReached),
        "termination cause should be GenerationLimitReached"
    );
}

/// Test 7: on_finish receives all_stats with length == number of generations run
#[test]
fn test_reporter_on_finish_stats_length() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    let mut ga = build_test_ga(10, spy);
    ga.run().expect("GA run should succeed");
    let d = data.lock().unwrap();
    assert_eq!(
        d.finish_stats_len, 10,
        "all_stats passed to on_finish should have one entry per generation"
    );
}

/// Test 8: Ga without reporter runs normally (no panic, Option is None)
#[test]
fn test_no_reporter_default() {
    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(10)
        .build()
        .expect("valid config");
    ga.run()
        .expect("GA without reporter should complete without panic");
    assert_ne!(
        ga.termination_cause,
        TerminationCause::NotTerminated,
        "termination cause should be finalized after run"
    );
}

// ── Unit tests migrated from src/reporter/duration.rs ────────────────────────

use genetic_algorithms::reporter::DurationReporter;

fn make_duration_stats(generation: usize) -> GenerationStats {
    GenerationStats::from_fitness_values(generation, &[1.0, 2.0, 3.0], true)
}

// Test: DurationReporter::new() constructs successfully (no panic, returns DurationReporter)
#[test]
fn duration_reporter_new_no_panic() {
    let _r = DurationReporter::new();
}

// Test: DurationReporter on_start does not panic
#[test]
fn duration_reporter_on_start_no_panic() {
    let mut r = DurationReporter::new();
    <DurationReporter as Reporter<BinaryChromosome>>::on_start(&mut r);
    // on_start should complete without panic
}

// Test: DurationReporter on_finish does not panic when start is None
#[test]
fn duration_reporter_on_finish_no_panic_without_start() {
    let mut r = DurationReporter::new();
    let stats = vec![make_duration_stats(0), make_duration_stats(1)];
    // Should not panic even if on_start was not called
    <DurationReporter as Reporter<BinaryChromosome>>::on_finish(
        &mut r,
        TerminationCause::GenerationLimitReached,
        &stats,
    );
}

// Test: DurationReporter on_finish does not panic when all_stats is empty
#[test]
fn duration_reporter_on_finish_empty_stats() {
    let mut r = DurationReporter::new();
    <DurationReporter as Reporter<BinaryChromosome>>::on_finish(
        &mut r,
        TerminationCause::GenerationLimitReached,
        &[],
    );
}

// Test: DurationReporter Default impl works
#[test]
fn duration_reporter_default_no_panic() {
    let _r = DurationReporter::default();
}

// ── Unit tests migrated from src/reporter/mod.rs ─────────────────────────────

use genetic_algorithms::reporter::NoopReporter;

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

// ── Unit tests migrated from src/reporter/simple.rs ──────────────────────────

use genetic_algorithms::reporter::SimpleReporter;

fn make_simple_stats(generation: usize) -> GenerationStats {
    GenerationStats::from_fitness_values(generation, &[1.0, 2.0, 3.0], true)
}

// Test 1: SimpleReporter fires at interval — behavioral: no panic after 9 calls
#[test]
fn simple_reporter_fires_at_interval() {
    let mut r = SimpleReporter::new(3);
    // Simulate 9 generations — should not panic
    for i in 0..9 {
        <SimpleReporter as Reporter<BinaryChromosome>>::on_generation_complete(
            &mut r,
            &make_simple_stats(i),
        );
    }
    // Behavioral assertion: on_generation_complete completes 9 times without panic
}

// Test 2: SimpleReporter does not panic when called fewer times than interval
#[test]
fn simple_reporter_below_interval_no_panic() {
    let mut r = SimpleReporter::new(5);
    for i in 0..4 {
        <SimpleReporter as Reporter<BinaryChromosome>>::on_generation_complete(
            &mut r,
            &make_simple_stats(i),
        );
    }
    // Behavioral assertion: 4 calls with interval=5 complete without panic
}

// Test 3: SimpleReporter on_finish runs without panic
#[test]
fn simple_reporter_on_finish_runs() {
    let mut r: SimpleReporter = SimpleReporter::new(10);
    let stats = vec![make_simple_stats(0), make_simple_stats(1)];
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
