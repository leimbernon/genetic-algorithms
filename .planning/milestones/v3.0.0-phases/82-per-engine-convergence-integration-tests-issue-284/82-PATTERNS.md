# Phase 82: Per-Engine Convergence Integration Tests - Pattern Map

**Mapped:** 2026-06-22
**Files analyzed:** 6 (all modifications — adding test functions to existing test files)
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `tests/engines/de/test_de.rs` | test | request-response | same file (lines 60-71) | exact |
| `tests/engines/scatter/test_scatter.rs` | test | request-response | same file (lines 39-54) | exact |
| `tests/engines/cellular/test_cellular.rs` | test | request-response | same file (lines 68-77) | exact |
| `tests/engines/alps/test_alps.rs` | test | request-response | same file (lines 59-66) | exact |
| `tests/engines/cma/test_cma.rs` | test | request-response | same file (lines 124-147) | exact |
| `tests/engines/pso/test_pso.rs` | test | request-response | same file (lines 346-362) | exact |

## Pattern Assignments

### `tests/engines/de/test_de.rs` — add `test_de_convergence`

**Analog:** same file, `test_de_rand1_binomial_converges` (lines 60-71)

**Existing convergence pattern** (lines 60-71):
```rust
#[test]
fn test_de_rand1_binomial_converges() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(
        result.best_fitness < 5.0,
        "DE/rand/1 binomial should reduce sphere fitness; got {}",
        result.best_fitness
    );
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
}
```

**Engine construction pattern** — reuse existing `sphere_engine` helper (lines 41-56):
```rust
fn sphere_engine(
    strategy: DeMutationStrategy,
    mode: DeCrossoverMode,
) -> DeEngine<RangeChromosome<f64>> {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_mutation_factor(0.8)
        .with_crossover_rate(0.9)
        .with_mutation_strategy(strategy)
        .with_crossover_mode(mode)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0); // stop early once good enough

    DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
}
```

**New test to add** — tighter threshold, matching decisions D-08/D-13:
```rust
/// Convergence regression test: DE/rand/1/binomial must reach sphere minimum < 1.0
/// on 5 dimensions within 300 generations. Prevents silent regressions in search dynamics.
#[test]
fn test_de_convergence() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(
        result.best_fitness < 1.0,
        "DE should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**Key details:**
- `DeEngine::new()` does NOT return Result — no `.expect()` needed
- `engine.run()` returns result directly — no `.expect()` needed
- Reuses `sphere_engine` helper which already has `fitness_target(1.0)`, `population_size(30)`, `max_generations(300)`
- Seed 42 already embedded in `sphere_engine` via `random_pop(n, 5, -5.0, 5.0, 42)`

---

### `tests/engines/scatter/test_scatter.rs` — add `test_scatter_convergence`

**Analog:** same file, `test_scatter_basic_convergence` (lines 39-54)

**Existing convergence pattern** (lines 39-54):
```rust
#[test]
fn test_scatter_basic_convergence() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(50)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();

    assert!(!result.reference_set.is_empty());
    assert!(result.iterations > 0);
    assert!(result.best_fitness < 125.0);
}
```

**New test to add:**
```rust
/// Convergence regression test: Scatter must reach sphere minimum < 1.0
/// on 5 dimensions within 300 iterations. Prevents silent regressions in search dynamics.
#[test]
fn test_scatter_convergence() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(300)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();

    assert!(
        result.best_fitness < 1.0,
        "Scatter should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**Key details:**
- Scatter uses `with_max_iterations()` NOT `with_max_generations()` — critical distinction
- `ScatterEngine::new()` does NOT return Result — no `.expect()` needed
- `engine.run()` returns result directly — no `.expect()` needed
- Must include `.with_reference_set_size(6)` — required by ScatterConfiguration
- Seed 42 via `random_pop(n, 5, -5.0, 5.0, 42)`

---

### `tests/engines/cellular/test_cellular.rs` — add `test_cellular_convergence`

**Analog:** same file, `make_engine` helper (lines 43-62) + `test_von_neumann_async_reduces_fitness` (lines 68-77)

**Existing engine construction pattern** (lines 43-62):
```rust
fn make_engine(
    rows: usize,
    cols: usize,
    neighborhood: Neighborhood,
    update_mode: UpdateMode,
) -> CellularEngine<RangeChromosome<f64>> {
    let config = CellularConfiguration::default()
        .with_grid(rows, cols)
        .with_neighborhood(neighborhood)
        .with_update_mode(update_mode)
        .with_max_generations(100)
        .with_selection(Selection::Tournament)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) }))
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(50.0);

    CellularEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .expect("valid test config")
}
```

**New test to add:**
```rust
/// Convergence regression test: Cellular GA must reach sphere minimum < 1.0
/// on 5 dimensions within 300 generations. Prevents silent regressions in search dynamics.
#[test]
fn test_cellular_convergence() {
    let config = CellularConfiguration::default()
        .with_grid(6, 6)
        .with_neighborhood(Neighborhood::Moore)
        .with_update_mode(UpdateMode::Asynchronous)
        .with_max_generations(300)
        .with_selection(Selection::Tournament)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) }))
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);

    let mut engine = CellularEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .expect("valid test config");
    let result = engine.run();

    assert!(
        result.best_fitness < 1.0,
        "Cellular should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**Key details:**
- `CellularEngine::new()` RETURNS Result — MUST use `.expect("valid test config")`
- `engine.run()` returns result directly — no `.expect()` needed
- Cellular requires: `.with_grid()`, `.with_neighborhood()`, `.with_update_mode()`, `.with_selection()`, `.with_crossover()`, `.with_mutation()`
- Grid size 6×6 = 36 individuals (population_size = rows × cols)
- Uses `GaussianParams { sigma: Some(0.5) }` for mutation — matches existing pattern
- Seed 42 via `random_pop(n, 5, -5.0, 5.0, 42)`

---

### `tests/engines/alps/test_alps.rs` — add `test_alps_convergence`

**Analog:** same file, `make_engine` helper (lines 39-53) + `test_linear_age_scheme_runs` (lines 59-66)

**Existing engine construction pattern** (lines 39-53):
```rust
fn make_engine(scheme: AlpsAgeScheme) -> AlpsEngine<RangeChromosome<f64>> {
    let config = AlpsConfiguration::default()
        .with_n_layers(4)
        .with_layer_size(15)
        .with_age_scheme(scheme)
        .with_age_gap(5)
        .with_injection_interval(10)
        .with_max_generations(100)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) }))
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(50.0);

    AlpsEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere).expect("valid test config")
}
```

**New test to add:**
```rust
/// Convergence regression test: ALPS must reach sphere minimum < 1.0
/// on 5 dimensions within 300 generations. Prevents silent regressions in search dynamics.
#[test]
fn test_alps_convergence() {
    let config = AlpsConfiguration::default()
        .with_n_layers(4)
        .with_layer_size(15)
        .with_age_scheme(AlpsAgeScheme::Linear)
        .with_age_gap(5)
        .with_injection_interval(10)
        .with_max_generations(300)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) }))
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);

    let mut engine = AlpsEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .expect("valid test config");
    let result = engine.run();

    assert!(
        result.best_fitness < 1.0,
        "ALPS should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**Key details:**
- `AlpsEngine::new()` RETURNS Result — MUST use `.expect("valid test config")`
- `engine.run()` returns result directly — no `.expect()` needed
- ALPS requires: `.with_n_layers()`, `.with_layer_size()`, `.with_age_scheme()`, `.with_age_gap()`, `.with_injection_interval()`, `.with_crossover()`, `.with_mutation()`
- Uses `AlpsAgeScheme::Linear` — simplest scheme, matches existing test pattern
- Seed 42 via `random_pop(n, 5, -5.0, 5.0, 42)`

---

### `tests/engines/cma/test_cma.rs` — add `test_cma_convergence` and `test_cma_ipop_convergence`

**Analog:** same file, `test_cma_sphere_converges` (lines 124-147) for basic convergence, `test_cma_ipop_restarts` (lines 401-433) for IPOP convergence

**Existing basic convergence pattern** (lines 124-147):
```rust
#[test]
fn test_cma_sphere_converges() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3);

    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness < 5.0,
        "CMA-ES should converge to < 5.0 on 5D sphere within 500 generations, got {}",
        result.best_fitness
    );
    assert!(
        result.generations > 0,
        "Should have run at least one generation"
    );
    assert!(
        !result.population.is_empty(),
        "Population should be non-empty"
    );
}
```

**Existing IPOP restart pattern** (lines 401-433):
```rust
#[test]
fn test_cma_ipop_restarts() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts: 2,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .with_observer(spy.clone());

    let result = engine.run().expect("engine run should succeed");

    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "on_restart should fire at least once with stagnation_threshold=5 and max_restarts=2"
    );
    assert!(
        result.total_restarts >= 1,
        "total_restarts should be >= 1 after IPOP restart, got {}",
        result.total_restarts
    );
}
```

**SpyObserver pattern** (lines 50-118):
```rust
struct SpyObserver {
    new_best_count: AtomicUsize,
    run_start_count: AtomicUsize,
    run_end_count: AtomicUsize,
    generation_start_count: AtomicUsize,
    generation_end_count: AtomicUsize,
    restart_count: AtomicUsize,
    last_restart_kind: Mutex<Option<RestartKind>>,
    restart_kinds: Mutex<Vec<RestartKind>>,
    last_restart_number: AtomicUsize,
    last_population_size_after: AtomicUsize,
}

impl Default for SpyObserver {
    fn default() -> Self {
        Self {
            new_best_count: AtomicUsize::new(0),
            run_start_count: AtomicUsize::new(0),
            run_end_count: AtomicUsize::new(0),
            generation_start_count: AtomicUsize::new(0),
            generation_end_count: AtomicUsize::new(0),
            restart_count: AtomicUsize::new(0),
            last_restart_kind: Mutex::new(None),
            restart_kinds: Mutex::new(Vec::new()),
            last_restart_number: AtomicUsize::new(0),
            last_population_size_after: AtomicUsize::new(0),
        }
    }
}

impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    fn on_restart(&self, event: &RestartEvent) {
        self.restart_count.fetch_add(1, Ordering::SeqCst);
        *self.last_restart_kind.lock().unwrap() = Some(event.kind);
        self.restart_kinds.lock().unwrap().push(event.kind);
        self.last_restart_number.store(event.restart_number, Ordering::SeqCst);
        self.last_population_size_after.store(event.population_size_after, Ordering::SeqCst);
    }
    // ... other observer methods ...
}
```

**New test 1 — `test_cma_convergence`:**
```rust
/// Convergence regression test: CMA-ES must reach sphere minimum < 1.0
/// on 5 dimensions (no restart). Prevents silent regressions in search dynamics.
#[test]
fn test_cma_convergence() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(300)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_fitness_target(1.0);

    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness < 1.0,
        "CMA-ES should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**New test 2 — `test_cma_ipop_convergence`:**
```rust
/// Convergence regression test: CMA-ES with IPOP restart must reach sphere minimum < 1.0
/// on 5 dimensions and trigger at least one restart. Prevents silent regressions in
/// restart-augmented search dynamics.
#[test]
fn test_cma_ipop_convergence() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_fitness_target(1.0)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 10,
            max_restarts: 3,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .with_observer(spy.clone());

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness < 1.0,
        "CMA with IPOP should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "IPOP should trigger at least one restart"
    );
}
```

**Key details:**
- `CmaEngine::new()` does NOT return Result — no `.expect()` on construction
- `engine.run()` RETURNS Result — MUST use `.expect("engine run should succeed")`
- Uses `CmaConfiguration::default_for_dim(5)` NOT `CmaConfiguration::default()` — critical!
- Must include `.with_sigma0(0.3)` — required for CMA initialization
- IPOP test reuses existing `SpyObserver` (already defined in file, lines 50-118)
- For IPOP: use `with_max_generations(500)` and `stagnation_threshold: 10` (higher than existing tests to allow convergence before restart triggers)
- Seed 42 via `random_pop(n, 5, -5.0, 5.0, 42)`

---

### `tests/engines/pso/test_pso.rs` — add `test_pso_convergence`

**Analog:** same file, `test_pso_sphere_converges` (lines 346-362)

**Existing convergence pattern** (lines 346-362):
```rust
#[test]
fn test_pso_sphere_converges() {
    rng::set_seed(Some(42));
    let init_pop = random_pop(30, 10, -5.12, 5.12, 42);
    let config = PsoConfiguration::default()
        .with_population_size(30)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1e-2);
    let mut engine = PsoEngine::new(config, move |_n| init_pop.clone(), sphere);
    let result = engine.run().expect("engine run should succeed");
    assert!(
        result.best_fitness < 1e-2 || result.generations < 500,
        "PSO must converge on 10D Sphere: best_fitness={:.6} after {} generations",
        result.best_fitness,
        result.generations
    );
}
```

**New test to add:**
```rust
/// Convergence regression test: PSO must reach sphere minimum < 1.0
/// on 5 dimensions within 300 generations. Prevents silent regressions in search dynamics.
#[test]
fn test_pso_convergence() {
    rng::set_seed(Some(42));
    let init_pop = random_pop(30, 5, -5.0, 5.0, 42);
    let config = PsoConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);
    let mut engine = PsoEngine::new(config, move |_n| init_pop.clone(), sphere);
    let result = engine.run().expect("engine run should succeed");
    assert!(
        result.best_fitness < 1.0,
        "PSO should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

**Key details:**
- `PsoEngine::new()` does NOT return Result — no `.expect()` on construction
- `engine.run()` RETURNS Result — MUST use `.expect("engine run should succeed")`
- PSO uses `move |_n| init_pop.clone()` pattern — population created BEFORE engine, passed via closure
- MUST call `rng::set_seed(Some(42))` BEFORE `random_pop()` — PSO tests require explicit seed
- Uses 5 dimensions (not 10) per decision D-11
- Bounds `-5.0, 5.0` (not `-5.12, 5.12`) per decision D-11
- Seed 42 via `random_pop(30, 5, -5.0, 5.0, 42)`

---

## Shared Patterns

### Sphere Helper
**Source:** Each test file has its own `sphere` function (identical implementation)
**Apply to:** All new convergence tests
```rust
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}
```
NOTE: PSO uses `g.real_value()` instead of `g.value()` — both are equivalent for `RangeGene<f64>` but must match the file's existing convention.

### Random Population Helper
**Source:** Each test file has its own `random_pop` function (identical implementation)
**Apply to:** All new convergence tests
```rust
fn random_pop(n: usize, dim: usize, lo: f64, hi: f64, seed: u64) -> Vec<RangeChromosome<f64>> {
    rng::set_seed(Some(seed));
    let mut r = rng::make_rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| {
                    let v = r.random::<f64>() * (hi - lo) + lo;
                    RangeGene::new(j as i32, vec![(lo, hi)], v)
                })
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}
```

### Convergence Assertion Pattern
**Source:** All existing convergence tests
**Apply to:** All new convergence tests
```rust
assert!(
    result.best_fitness < 1.0,
    "Engine should converge to sphere minimum < 1.0; got {}",
    result.best_fitness
);
```

### RNG Determinism Pattern
**Source:** All existing tests
**Apply to:** All new convergence tests
```rust
rng::set_seed(Some(42));  // Only needed explicitly for PSO; others pass seed to random_pop
```

### Engine Construction Variations

| Engine | Construction returns Result? | `run()` returns Result? | Config factory |
|--------|------------------------------|------------------------|----------------|
| DeEngine | No | No | `DeConfiguration::default()` |
| ScatterEngine | No | No | `ScatterConfiguration::default()` |
| CellularEngine | **Yes** — needs `.expect()` | No | `CellularConfiguration::default()` |
| AlpsEngine | **Yes** — needs `.expect()` | No | `AlpsConfiguration::default()` |
| CmaEngine | No | **Yes** — needs `.expect()` | `CmaConfiguration::default_for_dim(5)` |
| PsoEngine | No | **Yes** — needs `.expect()` | `PsoConfiguration::default()` |

## No Analog Found

None — all 6 test files have existing convergence or smoke tests that serve as exact analogs.

## Metadata

**Analog search scope:** `tests/engines/{de,scatter,cellular,alps,cma,pso}/`
**Files scanned:** 6
**Pattern extraction date:** 2026-06-22
