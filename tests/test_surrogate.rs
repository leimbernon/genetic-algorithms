// Wave 0 tests: SurrogateModel trait-level invariants and pure-math helpers.
//
// These tests verify the Phase 62 public contract established in Plan 01:
//   SC-1a: predict() is callable on a user-defined surrogate implementation
//   SC-1d: prescreening floor formula (max(1, floor(n * f))) is correct
//   SC-1g: NaN predictions are treated as worst score (NEG_INFINITY substitution)
//   SC-2c: GenerationStats.true_fitness_calls deserialises as None from old JSON
//
// Engine-dependent tests (SC-1b, SC-1c, SC-1e, SC-1f, SC-2a, SC-2b, SC-3) are
// intentionally absent. They will be added by Plan 02 once Ga::with_surrogate exists.
// Zero ignore attributes in this file.

use genetic_algorithms::traits::{ChromosomeT, GeneT};
use genetic_algorithms::SurrogateModel;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ─── Shared stub types ────────────────────────────────────────────────────────

/// Minimal gene — only GeneT required.
#[derive(Debug, Clone, Default)]
struct StubGene {
    id: i32,
}

impl GeneT for StubGene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

/// Minimal chromosome — carries a single fitness value.
#[derive(Debug, Clone, Default)]
struct StubChromosome {
    fitness: f64,
    age: usize,
}

impl ChromosomeT for StubChromosome {
    type Gene = StubGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn calculate_fitness(&mut self) {
        // no-op for stubs
    }

    fn age(&self) -> usize {
        self.age
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }
}

// ─── SC-1a: predict() is callable ────────────────────────────────────────────

/// Surrogate that counts the number of predict() invocations via an AtomicUsize.
struct CountingSurrogate {
    calls: Arc<AtomicUsize>,
}

impl SurrogateModel<StubChromosome> for CountingSurrogate {
    fn predict(&self, _chromosome: &StubChromosome) -> f64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        1.0
    }
}

/// SC-1a: A minimal SurrogateModel impl on a stub chromosome is callable.
/// Counts predict invocations via AtomicUsize and asserts the count matches.
#[test]
fn test_predict_called() {
    let calls = Arc::new(AtomicUsize::new(0));
    let surrogate = CountingSurrogate {
        calls: Arc::clone(&calls),
    };

    // Invoke predict 5 times on stub chromosomes.
    for _ in 0..5 {
        let c = StubChromosome::default();
        surrogate.predict(&c);
    }

    assert_eq!(calls.load(Ordering::SeqCst), 5, "predict must be called exactly 5 times");

    // Also verify the trait is storable in Arc<dyn SurrogateModel<_> + Send + Sync>
    // (compile-time proof that the Send + Sync bounds are satisfied).
    let _shared: Arc<dyn SurrogateModel<StubChromosome> + Send + Sync> =
        Arc::new(CountingSurrogate {
            calls: Arc::new(AtomicUsize::new(0)),
        });
}

// ─── SC-1d: prescreening floor formula ───────────────────────────────────────

/// Inline helper: the same formula Plan 02 will use in ga.rs.
/// Returns the number of offspring to retain after prescreening.
fn floor_keep(n: usize, f: f64) -> usize {
    ((n as f64 * f).floor() as usize).max(1)
}

/// SC-1d: The prescreening floor formula returns correct results for boundary inputs.
/// Pure-math test — no engine required.
#[test]
fn test_prescreening_floor() {
    // Near-zero fraction: must retain at least 1
    assert_eq!(floor_keep(10, 0.0001), 1, "floor(10 * 0.0001) == 0, max(0,1) == 1");

    // 50% fraction: retain exactly 5
    assert_eq!(floor_keep(10, 0.5), 5, "floor(10 * 0.5) == 5");

    // 100% fraction: retain all 10
    assert_eq!(floor_keep(10, 1.0), 10, "floor(10 * 1.0) == 10");

    // Exactly 0.0 fraction: must retain at least 1
    assert_eq!(floor_keep(10, 0.0), 1, "floor(10 * 0.0) == 0, max(0,1) == 1");

    // Larger population: n=100, f=0.3 → 30
    assert_eq!(floor_keep(100, 0.3), 30, "floor(100 * 0.3) == 30");

    // Single offspring: must always retain 1
    assert_eq!(floor_keep(1, 0.1), 1, "max(floor(0.1), 1) == 1");
}

// ─── SC-1g: NaN predictions treated as worst ─────────────────────────────────

/// Surrogate that returns NaN for the chromosome at the target index,
/// and a finite score otherwise.
struct NanSurrogate {
    nan_index: usize,
    call_count: AtomicUsize,
}

impl SurrogateModel<StubChromosome> for NanSurrogate {
    fn predict(&self, _chromosome: &StubChromosome) -> f64 {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        if idx == self.nan_index {
            f64::NAN
        } else {
            idx as f64
        }
    }
}

/// SC-1g: NaN predicted scores are substituted with NEG_INFINITY so they sort last.
/// Reproduces the documented substitution inline (pure data manipulation; no engine).
#[test]
fn test_nan_prediction_treated_as_worst() {
    // Build a small offspring vector (5 chromosomes).
    let offspring: Vec<StubChromosome> = (0..5).map(|_| StubChromosome::default()).collect();

    let surrogate = NanSurrogate {
        nan_index: 2, // chromosome at index 2 gets NaN
        call_count: AtomicUsize::new(0),
    };

    // Score each offspring; reproduce the NaN → NEG_INFINITY substitution
    // exactly as Plan 02 will do it in ga.rs.
    let mut scored: Vec<(usize, f64)> = offspring
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let raw = surrogate.predict(c);
            let score = if raw.is_nan() { f64::NEG_INFINITY } else { raw };
            (i, score)
        })
        .collect();

    // Sort descending by score (best-first, worst-last).
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // The NaN entry (original index 2) must be last after substitution.
    let last_original_index = scored.last().unwrap().0;
    assert_eq!(
        last_original_index, 2,
        "NaN entry (original index 2) must sort last (treated as NEG_INFINITY)"
    );

    // All other entries must have finite scores.
    for (orig_idx, score) in &scored[..scored.len() - 1] {
        assert!(
            score.is_finite(),
            "entry {orig_idx} should have a finite score, got {score}"
        );
    }

    // Last entry must be NEG_INFINITY (the substituted NaN).
    assert_eq!(
        scored.last().unwrap().1,
        f64::NEG_INFINITY,
        "substituted NaN must be NEG_INFINITY"
    );
}

// ─── SC-2c: serde default for true_fitness_calls ─────────────────────────────

/// SC-2c: Deserialising a GenerationStats JSON payload that lacks the
/// `true_fitness_calls` field yields None (serde(default) backward-compat).
///
/// Gated behind #[cfg(feature = "serde")] — only runs with `--features serde`.
#[cfg(feature = "serde")]
#[test]
fn stats_serde_default() {
    use genetic_algorithms::stats::GenerationStats;

    // Minimal GenerationStats JSON without the new field.
    // Matches the shape of existing checkpoints created before Phase 62.
    let json = r#"{
        "generation": 0,
        "best_fitness": 1.0,
        "worst_fitness": 3.0,
        "avg_fitness": 2.0,
        "fitness_std_dev": 0.816,
        "population_size": 3,
        "diversity": 0.816,
        "dynamic_mutation_probability": null,
        "avg_node_count": 0.0,
        "cache_hits": null,
        "cache_misses": null
    }"#;

    let parsed: GenerationStats = serde_json::from_str(json)
        .expect("GenerationStats must deserialise from JSON lacking true_fitness_calls");

    assert!(
        parsed.true_fitness_calls.is_none(),
        "true_fitness_calls must be None when absent from checkpoint JSON, got {:?}",
        parsed.true_fitness_calls
    );
}
