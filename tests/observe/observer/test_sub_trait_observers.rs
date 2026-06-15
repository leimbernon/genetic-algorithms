/// Integration tests for SUB-01 (IslandGaObserver), SUB-02 (Nsga2Observer),
/// and SUB-03 (LogObserver implements all three traits).
use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::traits::{LinearChromosome, VectorFitness};

// Custom 2-objective binary chromosome for NSGA-II observer tests
#[derive(Debug, Clone, Default)]
struct MoBinaryChromosome {
    dna: Vec<BinaryGene>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl genetic_algorithms::traits::ChromosomeT for MoBinaryChromosome {
    type Gene = BinaryGene;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        let t = self.dna.iter().filter(|g| g.value).count() as f64;
        self.fitness_values = vec![t, self.dna.len() as f64 - t];
        self.fitness = t;
    }
}
impl LinearChromosome for MoBinaryChromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self { self.dna = dna.into_owned(); self }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}
impl VectorFitness for MoBinaryChromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, v: Vec<f64>) { self.fitness_values = v; }
}
impl genetic_algorithms::operations::mutation::ValueMutable for MoBinaryChromosome {}
impl genetic_algorithms::traits::OperatorCompat for MoBinaryChromosome {}
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::observer::{GaObserver, IslandGaObserver, Nsga2Observer};
#[cfg(feature = "logging")]
use genetic_algorithms::observer::LogObserver;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};

// ============================================================================
// SUB-01: IslandGaObserver hooks fire on IslandGa
// ============================================================================

#[derive(Default)]
struct IslandCounters {
    run_start: AtomicUsize,
    run_end: AtomicUsize,
    generation_end: AtomicUsize,
    migration_triggered: AtomicUsize,
}

struct CountingIslandObserver {
    counters: Arc<IslandCounters>,
}

impl IslandGaObserver<BinaryChromosome> for CountingIslandObserver {
    fn on_island_run_start(&self, _island_id: usize) {
        self.counters.run_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_island_run_end(&self, _island_id: usize) {
        self.counters.run_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_island_generation_end(
        &self,
        _island_id: usize,
        _generation: usize,
        _stats: &GenerationStats,
    ) {
        self.counters.generation_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_migration_triggered(&self, _generation: usize, _migration_count: usize) {
        self.counters
            .migration_triggered
            .fetch_add(1, Ordering::Relaxed);
    }
}

/// SUB-01: IslandGa observer hooks fire during a run.
#[test]
fn test_island_observer_hooks_fire() {
    let counters = Arc::new(IslandCounters::default());
    let observer = Arc::new(CountingIslandObserver {
        counters: Arc::clone(&counters),
    });

    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(2)
        .with_migration_count(1);

    let ga_config = GaConfiguration::new()
        .with_population_size(10)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
        .with_max_generations(5)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    let mut island_ga = IslandGa::<BinaryChromosome>::new(island_config, ga_config)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_observer(observer as Arc<dyn IslandGaObserver<BinaryChromosome> + Send + Sync>)
        .build()
        .expect("IslandGa configuration should be valid");

    island_ga.run().expect("IslandGa run should succeed");

    assert!(
        counters.run_start.load(Ordering::Relaxed) >= 1,
        "on_island_run_start should fire at least once"
    );
    assert!(
        counters.run_end.load(Ordering::Relaxed) >= 1,
        "on_island_run_end should fire at least once"
    );
    assert!(
        counters.generation_end.load(Ordering::Relaxed) >= 1,
        "on_island_generation_end should fire at least once"
    );
    // migration_interval=2, max_generations=5 => migration fires at gen 2 and gen 4
    assert!(
        counters.migration_triggered.load(Ordering::Relaxed) >= 1,
        "on_migration_triggered should fire at least once"
    );
}

// ============================================================================
// SUB-02: Nsga2Observer hooks fire on Nsga2Ga
// ============================================================================

#[derive(Default)]
struct Nsga2Counters {
    pareto_front: AtomicUsize,
    sort_complete: AtomicUsize,
    crowding_distance: AtomicUsize,
}

struct CountingNsga2Observer {
    counters: Arc<Nsga2Counters>,
}

impl Nsga2Observer<MoBinaryChromosome> for CountingNsga2Observer {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
        self.counters.pareto_front.fetch_add(1, Ordering::Relaxed);
    }
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {
        self.counters.sort_complete.fetch_add(1, Ordering::Relaxed);
    }
    fn on_crowding_distance_calculated(&self, _generation: usize, _duration_ms: f64) {
        self.counters
            .crowding_distance
            .fetch_add(1, Ordering::Relaxed);
    }
}

/// SUB-02: Nsga2Ga observer hooks fire during a run.
#[test]
fn test_nsga2_observer_hooks_fire() {
    let counters = Arc::new(Nsga2Counters::default());
    let observer = Arc::new(CountingNsga2Observer {
        counters: Arc::clone(&counters),
    });

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(10)
        .with_max_generations(5);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip);

    let mut nsga2 = Nsga2Ga::<MoBinaryChromosome>::new(nsga2_config, ga_config)
        .with_initialization_fn(binary_random_initialization)
        .with_observer(observer as Arc<dyn Nsga2Observer<MoBinaryChromosome> + Send + Sync>);

    nsga2.run().expect("Nsga2Ga run should succeed");

    assert!(
        counters.pareto_front.load(Ordering::Relaxed) >= 1,
        "on_pareto_front_assigned should fire at least once"
    );
    assert!(
        counters.sort_complete.load(Ordering::Relaxed) >= 1,
        "on_non_dominated_sort_complete should fire at least once"
    );
    assert!(
        counters.crowding_distance.load(Ordering::Relaxed) >= 1,
        "on_crowding_distance_calculated should fire at least once"
    );
}

// ============================================================================
// SUB-03: LogObserver implements all three observer traits (compile-time check)
// ============================================================================

/// SUB-03: LogObserver satisfies all three observer trait bounds simultaneously.
#[cfg(feature = "logging")]
#[test]
fn test_logobserver_implements_all_three_traits() {
    fn assert_ga_observer<U: ChromosomeT, T: GaObserver<U>>() {}
    fn assert_island_observer<U: ChromosomeT, T: IslandGaObserver<U>>() {}
    fn assert_nsga2_observer<U: ChromosomeT, T: Nsga2Observer<U>>() {}

    assert_ga_observer::<BinaryChromosome, LogObserver>();
    assert_island_observer::<BinaryChromosome, LogObserver>();
    assert_nsga2_observer::<BinaryChromosome, LogObserver>();
}
