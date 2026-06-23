# Phase 59: Restart Strategies — IPOP / BIPOP - Pattern Map

**Mapped:** 2026-06-05
**Files analyzed:** 9 (2 new source, 3 modified source, 2 new test, 1 modified test, 1 new example)
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/cma/restart.rs` | types/config | — | `src/observe/observer/mod.rs` (ExtensionEvent) | role-match |
| `src/engines/cma/configuration.rs` | config | — | `src/engines/cma/configuration.rs` itself (existing builder pattern) | exact (self-extension) |
| `src/engines/cma/mod.rs` | module | — | `src/engines/cma/mod.rs` itself + `src/lib.rs` (re-export pattern) | exact (self-extension) |
| `src/engines/cma/engine.rs` | engine | request-response | `src/engines/cma/engine.rs` itself (outer-loop wrap) | exact (self-extension) |
| `src/observe/observer/mod.rs` | observer trait | event-driven | `src/observe/observer/mod.rs` (ExtensionEvent / on_extension_triggered) | exact (same file) |
| `src/lib.rs` | re-exports | — | `src/lib.rs` lines 372–373 (pso/eda re-export pattern) | exact |
| `tests/engines/cma/test_cma.rs` | test | — | `tests/engines/cma/test_cma.rs` (SpyObserver pattern, CMA-05/06) | exact (extend) |
| `tests/test_engines.rs` | test module | — | `tests/test_engines.rs` lines 18–20 (cma module declaration) | exact (extend) |
| `examples/ipop_rastrigin.rs` | example | — | `examples/cma_es_rastrigin.rs` | exact |

---

## Pattern Assignments

### `src/engines/cma/restart.rs` (new file — types module)

**Analog:** `src/observe/observer/mod.rs` lines 33–46 (`ExtensionEvent`)

**Type definition pattern** (lines 33–46 of observer/mod.rs):
```rust
/// Payload for the [`GaObserver::on_extension_triggered`] hook.
///
/// Stack-allocated and `Copy`-able — zero heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct ExtensionEvent {
    /// The generation at which the extension fired.
    pub generation: usize,
    /// Population diversity at the time of extension.
    pub diversity: f64,
    /// Name of the extension strategy (e.g. `"MassExtinction"`).
    pub extension_type: &'static str,
    /// Diversity threshold that triggered the extension.
    pub threshold: f64,
}
```

**Apply exactly this pattern for:**
```rust
// src/engines/cma/restart.rs
#[derive(Debug, Clone, Copy)]
pub enum RestartStrategy {
    Ipop {
        population_scale: f64,
        stagnation_threshold: usize,
        max_restarts: usize,
    },
    Bipop {
        population_scale: f64,
        small_population_size: usize,
        stagnation_threshold: usize,
        max_restarts: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RestartEvent {
    pub restart_number: usize,
    pub generation: usize,
    pub population_size_before: usize,
    pub population_size_after: usize,
    pub kind: RestartKind,
}

#[derive(Debug, Clone, Copy)]
pub enum RestartKind {
    Ipop,
    BipopLarge,
    BipopSmall,
}
```

Note: `RestartStrategy` carries owned data (no `&'static str`), but `RestartEvent` and `RestartKind` are `Copy` — same derive set as `ExtensionEvent`. `RestartStrategy` itself is NOT `Copy` (contains `f64` and `usize` — it can be `Clone`; `Copy` is fine too since all fields are `Copy`).

---

### `src/engines/cma/configuration.rs` (modified — add field + builder)

**Analog:** `src/engines/cma/configuration.rs` lines 107–176 (existing builder methods)

**Existing field pattern** (lines 40–44):
```rust
    /// Optional fitness target — engine stops early when reached.
    ///
    /// `None` means the engine runs until `max_generations` is exhausted.
    pub fitness_target: Option<f64>,
```

**Existing Default impl pattern** (lines 71–85):
```rust
impl Default for CmaConfiguration {
    fn default() -> Self {
        Self {
            sigma0: 0.3,
            population_size: 0,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            cc: None,
            cs: None,
            c1: None,
            cmu: None,
        }
    }
}
```

**Existing builder method pattern** (lines 135–141):
```rust
    /// Builder: set fitness target for early stopping.
    ///
    /// The engine stops as soon as the best fitness satisfies the target
    /// condition for the current `problem_solving` direction.
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }
```

**New field to add** (after `cmu: Option<f64>` at line 68):
```rust
    /// Restart strategy for escaping local optima (IPOP or BIPOP).
    ///
    /// `None` (default) disables restarts — the engine runs a single CMA-ES
    /// run and stops at `max_generations` or `fitness_target`. Set to
    /// [`RestartStrategy::Ipop`] or [`RestartStrategy::Bipop`] to enable
    /// automatic restarts on stagnation.
    pub restart_strategy: Option<RestartStrategy>,
```

**New builder method to add** (after `with_cmu` at line 175):
```rust
    /// Builder: set restart strategy (IPOP or BIPOP).
    ///
    /// When set, the engine automatically restarts on stagnation, scaling
    /// the population according to the chosen strategy.
    pub fn with_restart_strategy(mut self, strategy: RestartStrategy) -> Self {
        self.restart_strategy = Some(strategy);
        self
    }
```

**Default impl update** — add `restart_strategy: None,` to the `Default` impl.

**Import to add** at top of `configuration.rs`:
```rust
use super::restart::RestartStrategy;
```

---

### `src/engines/cma/mod.rs` (modified — add re-exports)

**Analog:** `src/engines/cma/mod.rs` lines 1–7 (existing pattern)

**Current file** (lines 1–7):
```rust
//! CMA-ES engine. Covariance Matrix Adaptation Evolution Strategy for real-valued black-box continuous optimization.

pub mod configuration;
pub mod engine;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
```

**New version — add restart module and its public re-exports:**
```rust
//! CMA-ES engine. Covariance Matrix Adaptation Evolution Strategy for real-valued black-box continuous optimization.

pub mod configuration;
pub mod engine;
pub mod restart;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
pub use restart::{RestartEvent, RestartKind, RestartStrategy};
```

---

### `src/engines/cma/engine.rs` (modified — outer restart loop + CmaResult field)

**Analog:** `src/engines/cma/engine.rs` itself — the restart loop wraps the existing generation loop.

**CmaResult extension** (lines 297–306 — add `total_restarts` field):
```rust
pub struct CmaResult<U: LinearChromosome> {
    /// Final population (all individuals evaluated).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
    // NEW:
    /// Total number of restarts that fired during the run.
    ///
    /// Always `0` when no `restart_strategy` is configured.
    pub total_restarts: usize,
}
```

**Import to add** (after line 25 `use super::configuration::CmaConfiguration;`):
```rust
use super::restart::{RestartEvent, RestartKind, RestartStrategy};
```

**notify() helper pattern** (lines 372–376 — already exists, reuse for on_restart):
```rust
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }
```

**Existing initial setup block to preserve** (lines 426–500 — peek, lambda computation, initial pop evaluation, mean computation, CmaState::new, eigendecomposition). The restart loop calls this same block on each restart with `current_lambda` instead of `lambda`.

**CmaState reset on restart** — call `CmaState::new(n, current_lambda, &self.config, new_mean)` exactly as done at line 488, then the three eigendecomposition lines (491–494). This is the full state reset per D-07.

**Outer loop structure to wrap around the existing generation loop** (insert before `for gen in 0..self.config.max_generations` at line 510):

Key variables to declare outside the outer loop:
```rust
let mut total_restarts: usize = 0;
let default_lambda = lambda;  // capture before outer loop (needed for BIPOP small formula)
let mut current_lambda = lambda;
// global_best tracking replaces single-run best/best_fitness after this point:
let mut global_best_fitness = best_fitness;
let mut global_best: U = best.clone();
```

Variables to declare inside the outer restart loop (fresh each restart):
```rust
let mut restart_best_fitness = /* find_best result for this restart's initial pop */;
let mut stagnation_count: usize = 0;
```

**Restart trigger block** (inside inner generation loop, after best tracking, before statistics):
```rust
if let Some(ref strategy) = self.config.restart_strategy {
    let (threshold, max_r) = match strategy {
        RestartStrategy::Ipop { stagnation_threshold, max_restarts, .. } => (*stagnation_threshold, *max_restarts),
        RestartStrategy::Bipop { stagnation_threshold, max_restarts, .. } => (*stagnation_threshold, *max_restarts),
    };
    if stagnation_count >= threshold {
        if total_restarts >= max_r {
            break 'restart_loop;
        }
        let pop_before = current_lambda;
        current_lambda = compute_next_lambda(strategy, current_lambda, default_lambda, total_restarts);
        let kind = restart_kind(strategy, total_restarts);
        total_restarts += 1;
        let event = RestartEvent {
            restart_number: total_restarts,
            generation: gen,
            population_size_before: pop_before,
            population_size_after: current_lambda,
            kind,
        };
        self.notify(|obs| obs.on_restart(&event));
        break; // break inner loop → outer loop re-inits
    }
}
```

**Helper functions to add** (private, after `find_best` at line 416):
```rust
fn compute_next_lambda(
    strategy: &RestartStrategy,
    current_lambda: usize,
    default_lambda: usize,
    restart_count: usize,
) -> usize {
    match strategy {
        RestartStrategy::Ipop { population_scale, .. } => {
            ((current_lambda as f64) * population_scale).floor() as usize
        }
        RestartStrategy::Bipop { population_scale, small_population_size, .. } => {
            let next_restart_number = restart_count + 1;
            if next_restart_number % 2 == 1 {
                ((current_lambda as f64) * population_scale).floor() as usize
            } else if *small_population_size == 0 {
                (default_lambda / 5).max(1)
            } else {
                *small_population_size
            }
        }
    }
}

fn restart_kind(strategy: &RestartStrategy, restart_count: usize) -> RestartKind {
    let next_restart_number = restart_count + 1;
    match strategy {
        RestartStrategy::Ipop { .. } => RestartKind::Ipop,
        RestartStrategy::Bipop { .. } => {
            if next_restart_number % 2 == 1 { RestartKind::BipopLarge }
            else { RestartKind::BipopSmall }
        }
    }
}
```

Note: these are free functions (not methods), placed after `find_best` and before `run`. They take `&RestartStrategy` so no `self` needed. WASM-safe: only arithmetic.

**CmaResult construction** (lines 714–719 — add `total_restarts`):
```rust
CmaResult {
    population: pop,
    best: global_best,
    best_fitness: global_best_fitness,
    generations,
    total_restarts,
}
```

**`on_restart` call via notify** (same pattern as `on_new_best` at line 692):
```rust
// existing on_new_best pattern (lines 691-693):
let best_clone = best.clone();
self.notify(|obs| obs.on_new_best(gen, best_clone));
// on_restart pattern (same shape):
self.notify(|obs| obs.on_restart(&event));
```

---

### `src/observe/observer/mod.rs` (modified — add 13th hook)

**Analog:** `src/observe/observer/mod.rs` lines 113–114 (`on_extension_triggered`)

**Existing hook pattern** (lines 113–114):
```rust
    /// Called when an extension strategy fires due to low diversity.
    fn on_extension_triggered(&self, _event: ExtensionEvent) {}
```

**New hook to add** (after `on_extension_triggered`, before `on_generation_end` at line 116):
```rust
    /// Called when the CMA-ES engine triggers an automatic restart.
    ///
    /// Fires once per restart event, after state has been reset and before
    /// the next restart's generation loop begins.
    fn on_restart(&self, _event: &crate::engines::cma::restart::RestartEvent) {}
```

**Alternative** (if `RestartEvent` is imported at top of observer/mod.rs):
```rust
use crate::engines::cma::restart::RestartEvent;
// then:
fn on_restart(&self, _event: &RestartEvent) {}
```

Note: `on_extension_triggered` takes `ExtensionEvent` by value (it's `Copy`). `RestartEvent` is also `Copy`, so it can be taken by value the same way. Either `&RestartEvent` or `RestartEvent` (by copy) is correct — prefer `&RestartEvent` for consistency with other hooks that take references.

**Module-level doc comment update** (lines 12–27 — add `on_restart` row to the hooks table):
```rust
//! | `on_restart` | When the CMA-ES engine triggers an automatic restart |
```

---

### `src/lib.rs` (modified — add 3 re-exports)

**Analog:** `src/lib.rs` lines 372–373 (pso/eda re-export pattern)

**Existing re-export pattern** (lines 372–373):
```rust
pub use pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoResult, PsoTopology};
pub use eda::{EdaConfiguration, EdaEngine, EdaModel, EdaRealEngine, EdaResult};
```

**New line to add** (after line 373):
```rust
pub use cma::{RestartEvent, RestartKind, RestartStrategy};
```

Note: `CmaEngine`, `CmaConfiguration`, `CmaResult` are NOT re-exported at the crate root (users access them via `genetic_algorithms::cma::{...}`). The new restart types should follow the same granularity as `ExtensionEvent` (line 349), which IS re-exported at the crate root via `pub use observer::ExtensionEvent;`. Follow that precedent and re-export all three new types.

---

### `tests/engines/cma/test_cma.rs` (modified — extend with restart tests)

**Analog:** `tests/engines/cma/test_cma.rs` lines 49–80 (SpyObserver) and lines 195–218 (CMA-05 observer test)

**SpyObserver extension pattern** (add `restart_count` field):
```rust
#[derive(Default)]
struct SpyObserver {
    new_best_count: AtomicUsize,
    run_start_count: AtomicUsize,
    run_end_count: AtomicUsize,
    generation_start_count: AtomicUsize,
    generation_end_count: AtomicUsize,
    // NEW for phase 59:
    restart_count: AtomicUsize,
    last_restart_kind: std::sync::Mutex<Option<genetic_algorithms::RestartKind>>,
}
```

**SpyObserver impl extension** (add `on_restart` method):
```rust
impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    // existing methods unchanged ...

    fn on_restart(&self, event: &genetic_algorithms::RestartEvent) {
        self.restart_count.fetch_add(1, Ordering::SeqCst);
        *self.last_restart_kind.lock().unwrap() = Some(event.kind);
    }
}
```

**Test import additions** (after existing imports at lines 7–20):
```rust
use genetic_algorithms::cma::RestartStrategy;
use genetic_algorithms::{RestartEvent, RestartKind};
```

**Test structure pattern** (copy from CMA-05 at lines 195–218):
```rust
#[test]
fn test_cma_ipop_restarts() {
    let config = CmaConfiguration::default_for_dim(3)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts: 2,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 3, -5.0, 5.0, 42),
        sphere,
    )
    .with_observer(spy.clone());

    let result = engine.run();

    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "on_restart should fire at least once with low stagnation_threshold"
    );
    assert!(
        result.total_restarts >= 1,
        "total_restarts should be >= 1"
    );
}
```

---

### `tests/test_engines.rs` (modified — add cma_restart sub-module)

**Analog:** `tests/test_engines.rs` lines 18–20 (cma module declaration)

**Existing pattern** (lines 18–20):
```rust
    mod cma {
        mod test_cma;
    }
```

Note: The RESEARCH.md says tests go in `tests/engines/cma/test_cma.rs` (extend existing), NOT in a new `cma_restart/` subdirectory. The CONTEXT.md file list mentions `tests/engines/cma_restart/` but the RESEARCH.md Wave 0 notes say "extend existing `tests/engines/cma/test_cma.rs`". Use the existing `test_cma.rs` — no new subdirectory, no `test_engines.rs` modification needed.

---

### `examples/ipop_rastrigin.rs` (new file)

**Analog:** `examples/cma_es_rastrigin.rs` — exact structural match

**File structure pattern** (from `examples/cma_es_rastrigin.rs`):
```rust
/*!
# IPOP-CMA-ES: Rastrigin Minimization with Restarts
...
*/

use std::f64::consts::PI;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use genetic_algorithms::LogObserver;
use rand::Rng;
use std::borrow::Cow;
```

**`init_population` helper** (lines 49–65 of `cma_es_rastrigin.rs`) — copy verbatim, update `DIMENSIONS` constant to `10` (higher dimension shows restart benefit better on Rastrigin).

**`main` function pattern** (lines 67–96):
```rust
fn main() {
    let config = CmaConfiguration::default_for_dim(DIMENSIONS)
        .with_sigma0(0.5)
        .with_max_generations(200)  // per restart
        .with_problem_solving(ProblemSolving::Minimization)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 50,
            max_restarts: 3,
        });

    let mut engine = CmaEngine::new(config, init_population, rastrigin)
        .with_observer(Arc::new(LogObserver));

    let result = engine.run();

    println!("Total restarts: {}", result.total_restarts);
    println!("Generations:    {}", result.generations);
    println!("Best fitness:   {:.6}", result.best_fitness);
    assert!(result.best_fitness.is_finite());
}
```

**Additional import** (not in `cma_es_rastrigin.rs`):
```rust
use genetic_algorithms::cma::RestartStrategy;
```

---

## Shared Patterns

### `notify()` Observer Dispatch
**Source:** `src/engines/cma/engine.rs` lines 371–376
**Apply to:** `engine.rs` restart trigger
```rust
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
// Usage:
self.notify(|obs| obs.on_restart(&event));
```

### Builder Method Pattern
**Source:** `src/engines/cma/configuration.rs` lines 135–141
**Apply to:** `with_restart_strategy()` builder method
```rust
pub fn with_<field>(mut self, value: T) -> Self {
    self.<field> = Some(value);  // or direct assignment
    self
}
```

### Default No-Op Hook Pattern
**Source:** `src/observe/observer/mod.rs` line 114
**Apply to:** `fn on_restart` in `GaObserver` trait
```rust
fn on_extension_triggered(&self, _event: ExtensionEvent) {}
// → mirrors as:
fn on_restart(&self, _event: &RestartEvent) {}
```

### WASM Guard Pattern
**Source:** `CLAUDE.md` — mandatory for all new code
**Apply to:** Any par_iter or Instant::now calls — none introduced in this phase (pure arithmetic restart loop). Verify with `cargo check --target wasm32-unknown-unknown` per project mandate.

### Test SpyObserver with AtomicUsize
**Source:** `tests/engines/cma/test_cma.rs` lines 49–80
**Apply to:** New restart test hooks — extend existing `SpyObserver` with `restart_count: AtomicUsize` and `on_restart` impl.

---

## No Analog Found

All 9 files have sufficient analogs. No files require falling back to RESEARCH.md patterns exclusively.

The outer restart loop structure in `engine.rs` has no exact analog in the codebase (no other engine has a restart loop), but RESEARCH.md Pattern 1 (lines 148–229) provides a detailed pseudo-code template, and the inner loop to be wrapped already exists verbatim.

---

## Key Observations for Planner

1. **`test_engines.rs` is NOT modified** — restart tests go in the existing `tests/engines/cma/test_cma.rs` file per RESEARCH.md Wave 0 guidance. The `mod cma { mod test_cma; }` declaration already covers new tests added to that file.

2. **`RestartEvent` placement decision** — the RESEARCH.md recommends `src/engines/cma/restart.rs` (keeping CMA-specific types out of the general observer module). The `on_restart` hook in `observer/mod.rs` imports via `crate::engines::cma::restart::RestartEvent`. This creates a one-directional dependency: observer → cma types. Check for circular imports: `observer/mod.rs` does NOT currently import from `engines/cma/`. Adding `use crate::engines::cma::restart::RestartEvent;` in `observer/mod.rs` is safe as long as `restart.rs` does NOT import from `observer/`.

3. **`CmaResult` construction site** — verified single construction site at engine.rs line 714. Adding `total_restarts` field is non-breaking.

4. **`on_new_best` pitfall** — gate initial-best notification inside the outer restart loop with `is_better(restart_initial_fitness, global_best_fitness)` per RESEARCH.md Pitfall 3. Only fire `on_new_best` when the restart's initial pop beats the global record.

5. **`on_run_start` / `on_run_end`** — fire once per `run()` call, not per restart. They remain at lines 424 and 712 of `engine.rs`, outside the outer restart loop.

---

## Metadata

**Analog search scope:** `src/engines/cma/`, `src/observe/observer/`, `src/lib.rs`, `tests/engines/cma/`, `examples/`
**Files read:** `engine.rs`, `configuration.rs`, `mod.rs`, `observer/mod.rs`, `lib.rs`, `test_cma.rs`, `test_engines.rs`, `cma_es_rastrigin.rs`, `eda/engine.rs` (partial)
**Pattern extraction date:** 2026-06-05
