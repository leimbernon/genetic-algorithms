/// Integration tests for COMP-01: CompositeObserver fan-out across all three
/// observer traits (GaObserver, IslandGaObserver, Nsga2Observer).
///
/// No cfg gate — CompositeObserver is always available without feature flags.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::observer::{GaObserver, IslandGaObserver, Nsga2Observer};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::{AllObserver, CompositeObserver};

// ============================================================================
// CountingAllObserver — implements all three traits
// ============================================================================

#[derive(Default)]
struct CountingAllObserver {
    ga_hooks: AtomicUsize,
    island_hooks: AtomicUsize,
    nsga2_hooks: AtomicUsize,
}

impl GaObserver<BinaryChromosome> for CountingAllObserver {
    fn on_run_start(&self) {
        self.ga_hooks.fetch_add(1, Ordering::Relaxed);
    }
}

impl IslandGaObserver<BinaryChromosome> for CountingAllObserver {
    fn on_island_run_start(&self, _island_id: usize) {
        self.island_hooks.fetch_add(1, Ordering::Relaxed);
    }
}

impl Nsga2Observer<BinaryChromosome> for CountingAllObserver {
    fn on_pareto_front_assigned(&self, _generation: usize, _front_count: usize, _population_size: usize) {
        self.nsga2_hooks.fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// Helper: build a standard onemax Ga for integration tests
// ============================================================================

fn build_ga(
    observer: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>,
    max_generations: usize,
) -> Ga<BinaryChromosome> {
    Ga::new()
        .with_genes_per_chromosome(8)
        .with_population_size(20)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(max_generations)
        .with_observer(observer)
        .build()
        .unwrap()
}

// ============================================================================
// COMP-01 Test 1: GaObserver hooks fan out to all inner observers
// ============================================================================

/// COMP-01: CompositeObserver fans out GaObserver hooks to every inner observer.
#[test]
fn test_composite_observer_ga_hooks() {
    let a = Arc::new(CountingAllObserver::default());
    let b = Arc::new(CountingAllObserver::default());

    let composite = CompositeObserver::<BinaryChromosome>::new()
        .add(Arc::clone(&a) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>)
        .add(Arc::clone(&b) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>);

    let mut ga = build_ga(Arc::new(composite), 5);
    ga.run().expect("GA run should succeed");

    assert!(
        a.ga_hooks.load(Ordering::Relaxed) >= 1,
        "observer a should receive on_run_start"
    );
    assert!(
        b.ga_hooks.load(Ordering::Relaxed) >= 1,
        "observer b should receive on_run_start"
    );
}

// ============================================================================
// COMP-01 Test 2: IslandGaObserver hooks fan out to all inner observers
// ============================================================================

/// COMP-01: CompositeObserver fans out IslandGaObserver hooks to every inner observer.
#[test]
fn test_composite_observer_island_hooks() {
    let a = Arc::new(CountingAllObserver::default());
    let b = Arc::new(CountingAllObserver::default());

    let composite = CompositeObserver::<BinaryChromosome>::new()
        .add(Arc::clone(&a) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>)
        .add(Arc::clone(&b) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>);

    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(3)
        .with_migration_count(1);

    let ga_config = GaConfiguration::new()
        .with_population_size(10)
        .with_genes_per_chromosome(8)
        .with_max_generations(5)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    let mut island_ga = IslandGa::<BinaryChromosome>::new(island_config, ga_config)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_observer(Arc::new(composite) as Arc<dyn IslandGaObserver<BinaryChromosome> + Send + Sync>)
        .build()
        .expect("IslandGa configuration should be valid");

    island_ga.run().expect("IslandGa run should succeed");

    assert!(
        a.island_hooks.load(Ordering::Relaxed) > 0,
        "observer a should receive on_island_run_start"
    );
    assert!(
        b.island_hooks.load(Ordering::Relaxed) > 0,
        "observer b should receive on_island_run_start"
    );
}

// ============================================================================
// COMP-01 Test 3: Nsga2Observer hooks fan out to all inner observers
// ============================================================================

/// COMP-01: CompositeObserver fans out Nsga2Observer hooks to every inner observer.
#[test]
fn test_composite_observer_nsga2_hooks() {
    let a = Arc::new(CountingAllObserver::default());
    let b = Arc::new(CountingAllObserver::default());

    let composite = CompositeObserver::<BinaryChromosome>::new()
        .add(Arc::clone(&a) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>)
        .add(Arc::clone(&b) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>);

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(10)
        .with_max_generations(5);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip);

    let mut nsga2 = Nsga2Ga::<BinaryChromosome>::new(nsga2_config, ga_config)
        .with_initialization_fn(binary_random_initialization)
        .with_objective_fns(vec![
            Box::new(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64),
            Box::new(|dna: &[BinaryGene]| dna.iter().filter(|g| !g.value).count() as f64),
        ])
        .with_observer(Arc::new(composite) as Arc<dyn Nsga2Observer<BinaryChromosome> + Send + Sync>);

    nsga2.run().expect("Nsga2Ga run should succeed");

    assert!(
        a.nsga2_hooks.load(Ordering::Relaxed) > 0,
        "observer a should receive on_pareto_front_assigned"
    );
    assert!(
        b.nsga2_hooks.load(Ordering::Relaxed) > 0,
        "observer b should receive on_pareto_front_assigned"
    );
}

// ============================================================================
// COMP-01 Test 4: AllObserver bounds compile-time assertion
// ============================================================================

/// COMP-01: CompositeObserver satisfies AllObserver<U> — compile-time check.
///
/// If this test compiles, the AllObserver supertrait bound is satisfied.
#[test]
fn test_all_observer_bounds() {
    fn assert_all_observer<U: ChromosomeT, T: AllObserver<U>>() {}
    assert_all_observer::<BinaryChromosome, CompositeObserver<BinaryChromosome>>();
}

// ============================================================================
// COMP-01 Test 5: Fan-out delivers to every added observer (including duplicates)
// ============================================================================

/// COMP-01: Adding the same observer twice causes on_run_start to fire exactly twice.
///
/// This verifies fan-out semantics: CompositeObserver dispatches to all registered
/// inner observers, even if the same logical observer appears more than once.
#[test]
fn test_composite_fan_out_order() {
    let observer = Arc::new(CountingAllObserver::default());

    // Add the same underlying observer twice via two separate Arcs.
    let arc1 = Arc::clone(&observer) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>;
    let arc2 = Arc::clone(&observer) as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>;

    let composite = CompositeObserver::<BinaryChromosome>::new()
        .add(arc1)
        .add(arc2);

    let mut ga = build_ga(Arc::new(composite), 1);
    ga.run().expect("GA run should succeed");

    // on_run_start fires once per run, but dispatched to 2 inner observers.
    assert_eq!(
        observer.ga_hooks.load(Ordering::Relaxed),
        2,
        "on_run_start should fire exactly 2 times (once per registered observer)"
    );
}
