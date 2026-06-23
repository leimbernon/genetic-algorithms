use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::observer::{ExtensionEvent, GaObserver, NoopObserver};
use genetic_algorithms::operations::{Crossover, Extension, Mutation, Selection, Survivor};
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, ExtensionConfig, MutationConfig, SelectionConfig,
    StoppingConfig,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct SpyData {
    run_start: AtomicUsize,
    generation_start: AtomicUsize,
    selection_complete: AtomicUsize,
    crossover_complete: AtomicUsize,
    mutation_complete: AtomicUsize,
    fitness_eval_complete: AtomicUsize,
    survivor_complete: AtomicUsize,
    new_best: AtomicUsize,
    stagnation: AtomicUsize,
    extension_triggered: AtomicUsize,
    generation_end: AtomicUsize,
    run_end: AtomicUsize,
    run_end_cause: std::sync::Mutex<Option<TerminationCause>>,
    run_end_stats_len: AtomicUsize,
}

struct SpyObserver {
    data: Arc<SpyData>,
}

impl SpyObserver {
    fn new(data: Arc<SpyData>) -> Self {
        Self { data }
    }
}

impl GaObserver<BinaryChromosome> for SpyObserver {
    fn on_run_start(&self) {
        self.data.run_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_start(&self, _generation: usize) {
        self.data.generation_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_selection_complete(&self, _generation: usize, _duration: Duration, _pop_size: usize) {
        self.data.selection_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_crossover_complete(&self, _generation: usize, _duration: Duration, _offspring: usize) {
        self.data.crossover_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_mutation_complete(&self, _generation: usize, _duration: Duration, _pop_size: usize) {
        self.data.mutation_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_fitness_evaluation_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _pop_size: usize,
    ) {
        self.data
            .fitness_eval_complete
            .fetch_add(1, Ordering::Relaxed);
    }
    fn on_survivor_selection_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _pop_size: usize,
    ) {
        self.data.survivor_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _generation: usize, _best: &BinaryChromosome) {
        self.data.new_best.fetch_add(1, Ordering::Relaxed);
    }
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {
        self.data.stagnation.fetch_add(1, Ordering::Relaxed);
    }
    fn on_extension_triggered(&self, _event: ExtensionEvent) {
        self.data
            .extension_triggered
            .fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.data.generation_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        self.data.run_end.fetch_add(1, Ordering::Relaxed);
        *self.data.run_end_cause.lock().unwrap() = Some(cause);
        self.data
            .run_end_stats_len
            .store(all_stats.len(), Ordering::Relaxed);
    }
}

fn build_test_ga_with_observer(
    max_gens: usize,
    observer: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>,
) -> Ga<BinaryChromosome> {
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
        .with_observer(observer)
        .build()
        .expect("valid config")
}

/// Test 1: on_run_start fires exactly once
#[test]
fn test_observer_on_run_start_fires_once() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert_eq!(data.run_start.load(Ordering::Relaxed), 1);
}

/// Test 2: on_generation_start fires exactly max_generations times
#[test]
fn test_observer_on_generation_start_count() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert_eq!(data.generation_start.load(Ordering::Relaxed), 10);
}

/// Test 3: on_generation_end fires exactly max_generations times
#[test]
fn test_observer_on_generation_end_count() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert_eq!(data.generation_end.load(Ordering::Relaxed), 10);
}

/// Test 4: on_run_end fires exactly once with GenerationLimitReached
#[test]
fn test_observer_on_run_end_fires_once() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert_eq!(data.run_end.load(Ordering::Relaxed), 1);
    assert_eq!(
        *data.run_end_cause.lock().unwrap(),
        Some(TerminationCause::GenerationLimitReached)
    );
    assert_eq!(data.run_end_stats_len.load(Ordering::Relaxed), 10);
}

/// Test 5: on_new_best fires at least once
///
/// Uses an all-false initializer so the initial best fitness is 0.0 and any
/// bit-flip mutation is guaranteed to trigger on_new_best.
#[test]
fn test_observer_on_new_best_fires() {
    fn all_false_init(size: usize, _alleles: Option<&[BinaryGene]>) -> Vec<BinaryGene> {
        (0..size)
            .map(|i| BinaryGene {
                id: i as i32,
                value: false,
            })
            .collect()
    }

    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_initialization_fn(all_false_init)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(10)
        .with_observer(spy)
        .build()
        .expect("valid config");
    ga.run().expect("GA run should succeed");
    assert!(
        data.new_best.load(Ordering::Relaxed) >= 1,
        "on_new_best should fire at least once when starting from all-zero fitness"
    );
}

/// Test 6: operator hooks fire each generation
#[test]
fn test_observer_operator_hooks_fire_each_generation() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert_eq!(data.selection_complete.load(Ordering::Relaxed), 10);
    assert_eq!(data.crossover_complete.load(Ordering::Relaxed), 10);
    assert_eq!(data.mutation_complete.load(Ordering::Relaxed), 10);
    assert_eq!(data.fitness_eval_complete.load(Ordering::Relaxed), 10);
    assert_eq!(data.survivor_complete.load(Ordering::Relaxed), 10);
}

/// Test 7: GA without observer runs normally (Option is None, no panic)
#[test]
fn test_no_observer_default() {
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
        .expect("GA without observer should complete without panic");
    assert_ne!(ga.termination_cause, TerminationCause::NotTerminated);
}

/// Test 8: Partial observer implementation compiles and works
#[test]
fn test_observer_partial_impl_compiles() {
    struct CountingObserver(AtomicUsize);
    impl GaObserver<BinaryChromosome> for CountingObserver {
        fn on_generation_end(&self, _stats: &GenerationStats) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let obs = Arc::new(CountingObserver(AtomicUsize::new(0)));
    let obs_ref = Arc::clone(&obs);
    let mut ga = build_test_ga_with_observer(5, obs);
    ga.run().expect("GA should succeed with partial observer");
    assert_eq!(obs_ref.0.load(Ordering::Relaxed), 5);
}

/// Test 9: GaObserver is object-safe (Arc<dyn ...> compiles)
#[test]
fn test_observer_is_object_safe() {
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(NoopObserver);
    drop(obs);
}

/// Test 10: on_stagnation fires when no improvement occurs
#[test]
fn test_observer_stagnation_fires() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    // Run enough generations that stagnation is likely
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
        .with_observer(spy)
        .build()
        .expect("valid config");
    ga.run().expect("GA should succeed");
    // stagnation_count + new_best_count should equal max_generations
    let stag = data.stagnation.load(Ordering::Relaxed);
    let best = data.new_best.load(Ordering::Relaxed);
    assert_eq!(
        stag + best,
        50,
        "stagnation + new_best should equal total generations"
    );
}

/// LogObserver: implements GaObserver for BinaryChromosome (compile check)
#[cfg(feature = "logging")]
#[test]
fn test_log_observer_implements_trait() {
    use genetic_algorithms::observer::LogObserver;
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(LogObserver);
    drop(obs);
}

/// LogObserver: is Send + Sync
#[cfg(feature = "logging")]
#[test]
fn test_log_observer_is_send_sync() {
    use genetic_algorithms::observer::LogObserver;
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LogObserver>();
}

/// LogObserver: is a unit struct (zero-sized)
#[cfg(feature = "logging")]
#[test]
fn test_log_observer_is_unit_struct() {
    use genetic_algorithms::observer::LogObserver;
    assert_eq!(std::mem::size_of::<LogObserver>(), 0);
}

/// LogObserver: attaches to Ga<U> and GA run completes without panic
#[cfg(feature = "logging")]
#[test]
fn test_log_observer_attaches_and_runs() {
    use genetic_algorithms::observer::LogObserver;
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(LogObserver);
    let mut ga = build_test_ga_with_observer(5, obs);
    ga.run()
        .expect("GA with LogObserver should complete without panic");
}

/// LogObserver: is re-exported from crate root
#[cfg(feature = "logging")]
#[test]
fn test_log_observer_crate_reexport() {
    let _obs = genetic_algorithms::LogObserver;
}

/// Regression: no direct info!/debug!/trace! calls remain in engines/ga/mod.rs
/// (ga.rs was split into engines/ga/ directory module in phase 69-04)
#[test]
fn test_ga_has_no_direct_log_calls() {
    let ga_source = include_str!("../../../src/engines/ga/mod.rs");
    // Count occurrences of direct log macro invocations
    // The only allowed log call is log::warn! inside #[cfg(feature = "serde")]
    for line in ga_source.lines() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        // These macros must not appear outside comments
        assert!(
            !trimmed.starts_with("info!("),
            "Found direct info!() call in ga.rs: {}",
            trimmed
        );
        assert!(
            !trimmed.starts_with("debug!("),
            "Found direct debug!() call in ga.rs: {}",
            trimmed
        );
        assert!(
            !trimmed.starts_with("trace!("),
            "Found direct trace!() call in ga.rs: {}",
            trimmed
        );
        // log::info!, log::debug!, log::trace! forms
        assert!(
            !trimmed.starts_with("log::info!("),
            "Found direct log::info!() call in ga.rs: {}",
            trimmed
        );
        assert!(
            !trimmed.starts_with("log::debug!("),
            "Found direct log::debug!() call in ga.rs: {}",
            trimmed
        );
        assert!(
            !trimmed.starts_with("log::trace!("),
            "Found direct log::trace!() call in ga.rs: {}",
            trimmed
        );
    }
    // log::warn! is allowed for two exceptions:
    // 1. serde-gated checkpoint I/O failure (no on_checkpoint_failed hook yet)
    // 2. wasm32-gated max_duration_secs unsupported warning (fires before observer is ready)
    let warn_count = ga_source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("//") && (t.starts_with("log::warn!(") || t.starts_with("warn!("))
        })
        .count();
    assert!(
        warn_count <= 6,
        "Expected at most 6 warn!() calls in ga.rs: checkpoint + wasm32 exceptions + AOS portfolio warnings (Phase 43), found {}",
        warn_count
    );
}

// ============================================================================
// OrderingSpyObserver — records event ordering and operator Duration values
// ============================================================================

#[derive(Default)]
struct OrderingSpyData {
    /// Events recorded in order of occurrence.
    events: Mutex<Vec<String>>,
    /// Duration received by on_mutation_complete.
    mutation_duration: Mutex<Option<Duration>>,
    /// Duration received by on_fitness_evaluation_complete.
    fitness_eval_duration: Mutex<Option<Duration>>,
}

struct OrderingSpyObserver {
    data: Arc<OrderingSpyData>,
}

impl OrderingSpyObserver {
    fn new(data: Arc<OrderingSpyData>) -> Self {
        Self { data }
    }
}

impl GaObserver<BinaryChromosome> for OrderingSpyObserver {
    fn on_extension_triggered(&self, _event: ExtensionEvent) {
        self.data
            .events
            .lock()
            .unwrap()
            .push("extension_triggered".to_string());
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.data
            .events
            .lock()
            .unwrap()
            .push("generation_end".to_string());
    }
    fn on_mutation_complete(&self, _generation: usize, duration: Duration, _pop_size: usize) {
        *self.data.mutation_duration.lock().unwrap() = Some(duration);
    }
    fn on_fitness_evaluation_complete(
        &self,
        _generation: usize,
        duration: Duration,
        _pop_size: usize,
    ) {
        *self.data.fitness_eval_duration.lock().unwrap() = Some(duration);
    }
}

/// Test 12: on_extension_triggered fires before on_generation_end within the same generation.
///
/// Sets diversity_threshold high (100.0) so the extension always triggers, then verifies
/// that in the event stream "extension_triggered" always precedes "generation_end".
#[test]
fn test_extension_fires_before_generation_end() {
    let data = Arc::new(OrderingSpyData::default());
    let spy = Arc::new(OrderingSpyObserver::new(Arc::clone(&data)));

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
        .with_max_generations(5)
        // Set threshold very high so the extension always triggers (diversity is std dev of fitness)
        .with_extension_method(Extension::MassExtinction)
        .with_extension_diversity_threshold(100.0)
        .with_observer(spy)
        .build()
        .expect("valid config");

    ga.run().expect("GA should succeed");

    let events = data.events.lock().unwrap();
    // Every "extension_triggered" must appear before the immediately following "generation_end"
    let mut last_ext_idx: Option<usize> = None;
    for (idx, event) in events.iter().enumerate() {
        if event == "extension_triggered" {
            last_ext_idx = Some(idx);
        } else if event == "generation_end" {
            if let Some(ext_idx) = last_ext_idx.take() {
                assert!(
                    ext_idx < idx,
                    "extension_triggered (at {}) must come before generation_end (at {})",
                    ext_idx,
                    idx
                );
            }
        }
    }
}

/// Test 13: on_mutation_complete receives a Duration > Duration::ZERO.
#[test]
fn test_mutation_timing_nonzero() {
    let data = Arc::new(OrderingSpyData::default());
    let spy = Arc::new(OrderingSpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(3, spy);
    ga.run().expect("GA should succeed");
    let d = data.mutation_duration.lock().unwrap();
    assert!(d.is_some(), "on_mutation_complete should have been called");
}

/// Test 14: on_fitness_evaluation_complete receives a Duration > Duration::ZERO.
#[test]
fn test_fitness_eval_timing_nonzero() {
    let data = Arc::new(OrderingSpyData::default());
    let spy = Arc::new(OrderingSpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(3, spy);
    ga.run().expect("GA should succeed");
    let d = data.fitness_eval_duration.lock().unwrap();
    assert!(
        d.is_some(),
        "on_fitness_evaluation_complete should have been called"
    );
}
