use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::observer::{ExtensionEvent, GaObserver, NoopObserver};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ConfigurationT, SelectionConfig, CrossoverConfig, MutationConfig, StoppingConfig};
use genetic_algorithms::configuration::ProblemSolving;

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
    fn on_fitness_evaluation_complete(&self, _generation: usize, _duration: Duration, _pop_size: usize) {
        self.data.fitness_eval_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_survivor_selection_complete(&self, _generation: usize, _duration: Duration, _pop_size: usize) {
        self.data.survivor_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _generation: usize, _best: BinaryChromosome) {
        self.data.new_best.fetch_add(1, Ordering::Relaxed);
    }
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {
        self.data.stagnation.fetch_add(1, Ordering::Relaxed);
    }
    fn on_extension_triggered(&self, _event: ExtensionEvent) {
        self.data.extension_triggered.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.data.generation_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        self.data.run_end.fetch_add(1, Ordering::Relaxed);
        *self.data.run_end_cause.lock().unwrap() = Some(cause);
        self.data.run_end_stats_len.store(all_stats.len(), Ordering::Relaxed);
    }
}

fn build_test_ga_with_observer(max_gens: usize, observer: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>) -> Ga<BinaryChromosome> {
    Ga::new()
        .with_population_size(20)
        .with_genes_per_chromosome(8)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| {
            dna.iter().filter(|g| g.value).count() as f64
        })
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
#[test]
fn test_observer_on_new_best_fires() {
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver::new(Arc::clone(&data)));
    let mut ga = build_test_ga_with_observer(10, spy);
    ga.run().expect("GA run should succeed");
    assert!(data.new_best.load(Ordering::Relaxed) >= 1);
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
        .with_genes_per_chromosome(8)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| {
            dna.iter().filter(|g| g.value).count() as f64
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(10)
        .build()
        .expect("valid config");
    ga.run().expect("GA without observer should complete without panic");
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
        .with_genes_per_chromosome(8)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| {
            dna.iter().filter(|g| g.value).count() as f64
        })
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
    assert_eq!(stag + best, 50, "stagnation + new_best should equal total generations");
}

/// LogObserver: implements GaObserver for BinaryChromosome (compile check)
#[test]
fn test_log_observer_implements_trait() {
    use genetic_algorithms::observer::LogObserver;
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(LogObserver);
    drop(obs);
}

/// LogObserver: is Send + Sync
#[test]
fn test_log_observer_is_send_sync() {
    use genetic_algorithms::observer::LogObserver;
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LogObserver>();
}

/// LogObserver: is a unit struct (zero-sized)
#[test]
fn test_log_observer_is_unit_struct() {
    use genetic_algorithms::observer::LogObserver;
    assert_eq!(std::mem::size_of::<LogObserver>(), 0);
}

/// LogObserver: attaches to Ga<U> and GA run completes without panic
#[test]
fn test_log_observer_attaches_and_runs() {
    use genetic_algorithms::observer::LogObserver;
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(LogObserver);
    let mut ga = build_test_ga_with_observer(5, obs);
    ga.run().expect("GA with LogObserver should complete without panic");
}

/// LogObserver: is re-exported from crate root
#[test]
fn test_log_observer_crate_reexport() {
    let _obs = genetic_algorithms::LogObserver;
}
