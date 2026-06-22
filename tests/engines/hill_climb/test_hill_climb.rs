//! Integration tests for HillClimbEngine — STR-02, STR-03.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use genetic_algorithms::{GaObserver, HillClimbConfiguration, HillClimbEngine, HillClimbMode};

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

fn make_neighbor_fn(step: f64) -> impl Fn(&RangeChromosome<f64>) -> Vec<RangeChromosome<f64>> {
    move |c| {
        let current_val = c.dna()[0].value();
        let lo = -100.0;
        let hi = 100.0;
        let val_down = current_val - step;
        let val_up = current_val + step;

        let mut n_down = <RangeChromosome<f64>>::default();
        n_down.set_dna(Cow::Owned(vec![RangeGene::new(
            0,
            vec![(lo, hi)],
            val_down,
        )]));
        n_down.set_fitness(val_down.abs());

        let mut n_up = <RangeChromosome<f64>>::default();
        n_up.set_dna(Cow::Owned(vec![RangeGene::new(0, vec![(lo, hi)], val_up)]));
        n_up.set_fitness(val_up.abs());

        vec![n_down, n_up]
    }
}

fn make_initial(value: f64) -> RangeChromosome<f64> {
    let gene = RangeGene::new(0, vec![(-100.0, 100.0)], value);
    let mut c = <RangeChromosome<f64>>::default();
    c.set_dna(Cow::Owned(vec![gene]));
    c.set_fitness(value.abs());
    c
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_stochastic_finds_improvement() {
    let initial = make_initial(5.0);
    let initial_fitness = initial.fitness();

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::Stochastic)
        .with_no_improvement_limit(10);

    let mut engine = HillClimbEngine::new(config, initial, make_neighbor_fn(0.1));
    engine.run().expect("run must succeed");

    let best = engine.best().expect("best must be Some after run");
    assert!(
        best.fitness() < initial_fitness,
        "stochastic hill climb should improve from {} to something less",
        initial_fitness
    );
}

#[test]
fn test_stochastic_stops_on_no_improvement_limit() {
    // Neighbor fn always returns a chromosome with fitness worse than current
    let initial = make_initial(5.0);

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::Stochastic)
        .with_no_improvement_limit(3);

    let mut engine = HillClimbEngine::new(config, initial, |c| {
        let val = c.dna()[0].value();
        let lo = -100.0;
        let hi = 100.0;
        // Return a neighbor with strictly worse fitness (higher abs value)
        let worse_val = val + 10.0;
        let mut n = <RangeChromosome<f64>>::default();
        n.set_dna(Cow::Owned(vec![RangeGene::new(
            0,
            vec![(lo, hi)],
            worse_val,
        )]));
        n.set_fitness(worse_val.abs());
        vec![n]
    });

    engine.run().expect("engine must terminate without panic");
    assert!(
        engine.best().is_some(),
        "best() must return Some even when no improvement occurs"
    );
}

#[test]
fn test_stochastic_observer_hooks_order() {
    let observer = RecordingObserver::new();
    let initial = make_initial(2.0);

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::Stochastic)
        .with_no_improvement_limit(1);

    let mut engine = HillClimbEngine::new(config, initial, make_neighbor_fn(0.1))
        .with_observer(observer.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);

    engine.run().expect("run must succeed");

    let events = observer.events();

    // run_start must be first
    assert_eq!(events[0], "run_start", "first event must be run_start");

    // run_end must be last
    assert_eq!(
        events.last().unwrap(),
        "run_end",
        "last event must be run_end"
    );

    // gen_start must appear before gen_end
    let gen_start_pos = events
        .iter()
        .position(|e| e == "gen_start")
        .expect("gen_start missing");
    let gen_end_pos = events
        .iter()
        .position(|e| e == "gen_end")
        .expect("gen_end missing");
    assert!(
        gen_start_pos < gen_end_pos,
        "gen_start must precede gen_end"
    );

    // GA-specific hooks must NOT appear
    for event in &events {
        assert_ne!(
            event, "selection_complete",
            "selection_complete must not fire"
        );
        assert_ne!(
            event, "crossover_complete",
            "crossover_complete must not fire"
        );
        assert_ne!(
            event, "mutation_complete",
            "mutation_complete must not fire"
        );
    }
}

#[test]
fn test_steepest_ascent_converges() {
    let initial = make_initial(5.0);
    let initial_fitness = initial.fitness(); // 5.0

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::SteepestAscent)
        .with_no_improvement_limit(1);

    // Neighbor fn returns 5 neighbors at various step sizes
    let mut engine = HillClimbEngine::new(config, initial, |c| {
        let val = c.dna()[0].value();
        let lo = -100.0;
        let hi = 100.0;
        let steps = [0.1, -0.1, 0.2, -0.2, 0.5];
        steps
            .iter()
            .map(|&s| {
                let nv = val + s;
                let mut n = <RangeChromosome<f64>>::default();
                n.set_dna(Cow::Owned(vec![RangeGene::new(0, vec![(lo, hi)], nv)]));
                n.set_fitness(nv.abs());
                n
            })
            .collect()
    });

    engine.run().expect("run must succeed");

    let best = engine.best().expect("best must be Some after run");
    assert!(
        best.fitness() < initial_fitness,
        "SteepestAscent should improve fitness: got {} >= {}",
        best.fitness(),
        initial_fitness
    );
}

#[test]
fn test_steepest_ascent_stops_on_no_improvement() {
    let initial = make_initial(5.0);

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::SteepestAscent)
        .with_no_improvement_limit(1);

    // Always return neighbors worse than current
    let mut engine = HillClimbEngine::new(config, initial, |c| {
        let val = c.dna()[0].value();
        let lo = -100.0;
        let hi = 100.0;
        let worse_val = val + 1.0;
        let mut n = <RangeChromosome<f64>>::default();
        n.set_dna(Cow::Owned(vec![RangeGene::new(
            0,
            vec![(lo, hi)],
            worse_val,
        )]));
        n.set_fitness(worse_val.abs());
        vec![n]
    });

    engine.run().expect("engine must terminate without panic");
    assert!(
        engine.best().is_some(),
        "best() must be Some even when no improvement found"
    );
}

#[test]
fn test_steepest_ascent_empty_neighbor_list() {
    let initial = make_initial(3.0);

    let config = HillClimbConfiguration::default()
        .with_mode(HillClimbMode::SteepestAscent)
        .with_no_improvement_limit(1);

    // Neighbor fn returns empty list — engine must not panic
    let mut engine = HillClimbEngine::new(config, initial, |_c| vec![]);

    engine.run().expect("empty neighbors must not panic");

    let best = engine.best();
    assert!(
        best.is_some(),
        "best() must return Some (the initial chromosome)"
    );
    assert_eq!(
        best.unwrap().fitness(),
        3.0,
        "best should be the initial chromosome with fitness 3.0"
    );
}
