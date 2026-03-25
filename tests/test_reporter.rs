#![allow(deprecated)]
use std::sync::{Arc, Mutex};
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::reporter::Reporter;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ConfigurationT, SelectionConfig, CrossoverConfig, MutationConfig, StoppingConfig};
use genetic_algorithms::configuration::ProblemSolving;

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
    assert!(d.new_best_count >= 1, "on_new_best should fire at least once");
}

/// Test 4: on_new_best fires fewer times than on_generation_complete (not every gen improves)
#[test]
fn test_reporter_on_new_best_less_than_total_gens() {
    let data = Arc::new(Mutex::new(SpyData::default()));
    let spy = SpyReporter::new(Arc::clone(&data));
    // Use more generations and larger population so convergence is likely
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
    ga.run().expect("GA without reporter should complete without panic");
    assert_ne!(
        ga.termination_cause,
        TerminationCause::NotTerminated,
        "termination cause should be finalized after run"
    );
}
