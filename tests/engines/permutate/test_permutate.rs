//! Integration tests for PermutateEngine — STR-04.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use genetic_algorithms::{GaObserver, PermutateConfiguration, PermutateEngine};

// ─── Observer ────────────────────────────────────────────────────────────────

struct RecordingObserver {
    events: Mutex<Vec<String>>,
}

impl RecordingObserver {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Mutex::new(Vec::new()),
        })
    }

    fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

impl<U: ChromosomeT> GaObserver<U> for RecordingObserver {
    fn on_run_start(&self) {
        self.events.lock().unwrap().push("run_start".to_string());
    }

    fn on_generation_start(&self, _g: usize) {
        self.events.lock().unwrap().push("gen_start".to_string());
    }

    fn on_new_best(&self, _g: usize, _best: &U) {
        self.events.lock().unwrap().push("new_best".to_string());
    }

    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.events.lock().unwrap().push("gen_end".to_string());
    }

    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {
        self.events.lock().unwrap().push("run_end".to_string());
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_candidate(value: f64) -> RangeChromosome<f64> {
    let gene = RangeGene::new(0, vec![(-10.0, 10.0)], value);
    let mut c = <RangeChromosome<f64>>::default();
    c.set_dna(Cow::Owned(vec![gene]));
    c.set_fitness(value); // fitness IS the value directly
    c
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_permutate_finds_best_candidate() {
    let candidates = vec![
        make_candidate(3.0),
        make_candidate(1.0),
        make_candidate(4.0),
        make_candidate(1.5),
        make_candidate(2.0),
    ];
    let config = PermutateConfiguration::default().with_safety_gate(1000);

    let mut engine = PermutateEngine::new(config, candidates);
    engine.run().expect("run must succeed");

    let best = engine.best().expect("best must be Some after run");
    assert_eq!(
        best.fitness(),
        1.0,
        "minimization should select the candidate with fitness 1.0"
    );
}

#[test]
fn test_permutate_maximization() {
    let candidates = vec![
        make_candidate(1.0),
        make_candidate(5.0),
        make_candidate(2.0),
    ];
    let config =
        PermutateConfiguration::default().with_problem_solving(ProblemSolving::Maximization);

    let mut engine = PermutateEngine::new(config, candidates);
    engine.run().expect("run must succeed");

    let best = engine.best().expect("best must be Some after run");
    assert_eq!(
        best.fitness(),
        5.0,
        "maximization should select the candidate with fitness 5.0"
    );
}

#[test]
fn test_permutate_safety_gate_triggers() {
    let candidates: Vec<RangeChromosome<f64>> = (0..10).map(|i| make_candidate(i as f64)).collect();

    let config = PermutateConfiguration::default().with_safety_gate(3);

    let mut engine = PermutateEngine::new(config, candidates);
    let result = engine.run();

    assert!(
        result.is_ok(),
        "run must return Ok even when safety gate triggers"
    );
    assert!(
        engine.best().is_some(),
        "best must be Some even when gate triggers"
    );
}

#[test]
fn test_permutate_observer_hooks_per_candidate() {
    let observer = RecordingObserver::new();
    let candidates = vec![
        make_candidate(3.0),
        make_candidate(1.0),
        make_candidate(2.0),
    ];
    let config = PermutateConfiguration::default();

    let mut engine = PermutateEngine::new(config, candidates)
        .with_observer(observer.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);

    engine.run().expect("run must succeed");

    let events = observer.events();

    let run_start_count = events.iter().filter(|e| *e == "run_start").count();
    assert_eq!(run_start_count, 1, "run_start must fire exactly once");

    let gen_start_count = events.iter().filter(|e| *e == "gen_start").count();
    assert_eq!(
        gen_start_count, 3,
        "gen_start must fire once per candidate (3 candidates)"
    );

    let run_end_count = events.iter().filter(|e| *e == "run_end").count();
    assert_eq!(run_end_count, 1, "run_end must fire exactly once");

    let new_best_count = events.iter().filter(|e| *e == "new_best").count();
    assert!(new_best_count >= 1, "new_best must fire at least once");

    for event in &events {
        assert_ne!(
            event, "selection_complete",
            "selection_complete must not fire in PermutateEngine"
        );
    }
}

#[test]
fn test_permutate_best_before_run_returns_none() {
    let candidates = vec![make_candidate(1.0), make_candidate(2.0)];
    let config = PermutateConfiguration::default();

    let engine = PermutateEngine::new(config, candidates);

    // Do NOT call run()
    assert!(
        engine.best().is_none(),
        "best() must return None before run() is called"
    );
}

#[test]
fn test_permutate_fitness_target_early_stop() {
    let candidates: Vec<RangeChromosome<f64>> = vec![
        make_candidate(5.0),
        make_candidate(4.0),
        make_candidate(3.0),
        make_candidate(2.0),
        make_candidate(1.0),
        make_candidate(0.5),
        make_candidate(0.1),
        make_candidate(0.0),
        make_candidate(-0.1),
        make_candidate(-0.2),
    ];

    let config = PermutateConfiguration::default().with_fitness_target(1.5);

    let mut engine = PermutateEngine::new(config, candidates);
    let result = engine.run();

    assert!(result.is_ok(), "run must succeed with fitness target");

    let best = engine.best().expect("best must be Some after run");
    assert!(
        best.fitness() <= 1.5,
        "early stop: best fitness {} should be <= 1.5",
        best.fitness()
    );
}
