# Phase 57: PSO Engine - Research

**Researched:** 2026-06-02
**Domain:** Particle Swarm Optimization engine — Rust, real-valued, WASM-compatible
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Velocities stored in `PsoEngine`-internal `PsoState` struct as `Vec<Vec<f64>>` indexed parallel to population. Zero chromosome API changes. Pattern mirrors `CmaState`.
- **D-02:** Initial velocities auto-computed from gene bounds: `v_init_i ∈ [-(hi_i - lo_i), +(hi_i - lo_i)]`. No `v_init` config field.
- **D-03:** `PsoTopology` enum: `PsoTopology::Global` (gbest) and `PsoTopology::Ring { neighborhood_size: usize }` (lbest, index-based). Default: `PsoTopology::Global`.
- **D-04:** Ring neighborhood is index-based (not distance-based), O(k) lookup, ring-wrapped.
- **D-05:** `PsoInertia` enum: `PsoInertia::Constant(f64)` and `PsoInertia::LinearDecay { w_start: f64, w_end: f64 }`.
- **D-06:** `v_max` per gene auto-derived as `hi_i - lo_i`. Applied as post-update clamp on |velocity component| before position update.
- **D-07:** Absorbing boundary: clamp position to `[lo, hi]`, zero velocity component at bounds.
- **D-08:** `v_max` is not a config field — auto-derived.
- **D-09:** `PsoEngine` holds `Option<Arc<dyn GaObserver<U> + Send + Sync>>` with `with_observer()` builder from day 1. Five hooks: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`.
- **D-10:** Example is `pso_rastrigin` (10D, consistent with `cma_es_rastrigin`).
- Chromosome bound: `U: LinearChromosome`, `U::Gene: RealGene`.
- New module: `src/engines/pso/` with `engine.rs`, `configuration.rs`, `mod.rs`.
- Tests: `tests/engines/pso/test_pso.rs` (never inline).
- lib.rs: `#[path = "engines/pso/mod.rs"] pub mod pso;` and re-exports.

### Claude's Discretion

- `PsoState` field names and internal struct layout
- Personal best update logic (strictly improves vs ≥)
- `PsoResult<U>` field set (minimum: `population`, `best`, `best_fitness`, `generations`)
- Whether `GenerationStats` reuses existing fields or gets PSO-specific additions
- Default `neighborhood_size` for ring topology
- Whether `c1`/`c2` are hardcoded defaults or exposed in `PsoConfiguration` (recommend exposing, standard 2.0 each)

### Deferred Ideas (OUT OF SCOPE)

- Constriction factor (`PsoInertia::Constriction`)
- Velocity-based stagnation stopping
- Discrete PSO (BPSO)
- Adaptive c1/c2 (APSO variants)
</user_constraints>

---

## Summary

Phase 57 implements `PsoEngine<U>`, a Particle Swarm Optimization engine, following the established engine pattern from `CmaEngine`. The implementation is algorithmically simpler than CMA-ES: no matrix decomposition, no eigendecomposition — just per-particle velocity and position updates driven by three scalar coefficients (inertia `w`, cognitive `c1`, social `c2`).

The core design is entirely decided in CONTEXT.md. Research confirms that the existing trait surface (`RealGene`, `LinearChromosome`), the observer wiring pattern (`GaObserver<U>`), the internal state struct pattern (`CmaState`), and the re-export pattern (`lib.rs` `#[path]` alias) are all available and verified in the codebase. The only non-trivial implementation concern is **bounds access for velocity initialization and boundary enforcement**: `RealGene` does not expose `lo`/`hi` bounds, so the engine must either require `Range<f64>` genes directly or use a `Bounded` supertrait approach. The precedent from existing mutation operators is direct field access (`gene.ranges[0]`), which means PSO must accept this constraint at the concrete type level or introduce a minimal bounds-access extension to `RealGene`.

**Primary recommendation:** Follow `CmaEngine` exactly for struct layout, observer wiring, result type, and run loop. Add a `bounds()` method to `RealGene` as the cleanest non-breaking extension, returning `Option<(f64, f64)>` — this allows PSO velocity init and boundary enforcement without coupling to `Range<f64>` directly.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Velocity state storage | Engine-internal (`PsoState`) | — | Per-particle velocities are engine metadata, not chromosome data |
| Position update | Engine-internal (`run()` loop) | Chromosome (via `set_dna`) | Engine computes new positions, writes via `LinearChromosome::set_dna` |
| Personal best tracking | Engine-internal (`PsoState`) | — | Pbest is swarm-state, not fitness metadata |
| Global/ring best lookup | Engine-internal | — | Topology determines neighborhood scope |
| Fitness evaluation | Engine (calls `fitness_fn`) | — | Same pattern as `CmaEngine` |
| Observer notification | Engine (`notify()` helper) | Observer impl | Engine fires hooks; user impl receives them |
| Bounds enforcement | Engine-internal | Gene bounds data | Engine clamps positions + zeroes velocities |
| Re-export / API surface | `lib.rs` | `src/engines/pso/mod.rs` | Standard `#[path]` alias pattern |

---

## Standard Stack

No new external crates are required. All dependencies are already in `Cargo.toml`.

### Core (already present)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | existing | RNG for r1/r2 per-gene random draws | Used by all engines via `crate::rng::make_rng()` |
| `log` | existing | `log::warn!` for edge-case diagnostics | Project-wide logging pattern |

### No new dependencies

The PSO core loop is inherently sequential (no rayon needed — WASM-safe by default). No matrix library, no eigendecomposition, no special RNG transforms needed beyond uniform `[0,1]` draws.

**Installation:** None required.

---

## Package Legitimacy Audit

No new packages are introduced in this phase. Audit section: **N/A — no new dependencies.**

---

## Architecture Patterns

### System Architecture Diagram

```
init_fn(pop_size)
       │
       ▼
  Initial population
  ┌─────────────────┐
  │ Vec<U>          │──── fitness_fn ──► set_fitness
  └─────────────────┘
       │
       ▼
  PsoState::new(pop, config)
  ┌──────────────────────────────────────┐
  │ velocities: Vec<Vec<f64>>            │  ← random init from gene bounds
  │ pbest_positions: Vec<Vec<f64>>       │  ← copy of initial positions
  │ pbest_fitness: Vec<f64>              │  ← initial fitness values
  │ gbest_position: Vec<f64>             │  ← best initial position
  │ gbest_fitness: f64                   │
  └──────────────────────────────────────┘
       │
       ▼  for gen in 0..max_generations
  ┌────────────────────────────────────────────────────┐
  │ observer.on_generation_start(gen)                   │
  │                                                    │
  │ for each particle i:                               │
  │   w = inertia(gen)                                 │
  │   for each gene d:                                 │
  │     r1, r2 = uniform[0,1]                          │
  │     best_d = gbest[d] or lbest[i][d] (topology)   │
  │     v[i][d] = w*v + c1*r1*(pbest-x) + c2*r2*(best-x) │
  │     clamp |v[i][d]| ≤ v_max[d]                    │
  │     x[i][d] += v[i][d]                             │
  │     absorbing boundary: clamp x, zero v            │
  │   fitness = fitness_fn(new_dna)                    │
  │   update pbest if improved                         │
  │ update gbest/lbest                                 │
  │                                                    │
  │ observer.on_new_best(gen, best) if improved        │
  │ observer.on_generation_end(&stats)                 │
  │ early-stop check                                   │
  └────────────────────────────────────────────────────┘
       │
       ▼
  observer.on_run_end(cause, all_stats)
  PsoResult { population, best, best_fitness, generations }
```

### Recommended Project Structure

```
src/engines/pso/
├── engine.rs          # PsoEngine<U>, PsoState, PsoResult<U>
├── configuration.rs   # PsoConfiguration, PsoInertia, PsoTopology
└── mod.rs             # pub use re-exports (PsoEngine, PsoConfiguration, PsoResult, PsoTopology, PsoInertia)

tests/engines/pso/
└── test_pso.rs        # All PSO tests

examples/
└── pso_rastrigin.rs   # 10D Rastrigin convergence demo
```

### Pattern 1: CmaEngine Struct Layout (follow exactly)

**What:** Engine holds `config`, `init_fn: Arc<Fn>`, `fitness_fn: Arc<FitnessFn>`, `observer: Option<Arc<dyn GaObserver<U>>>`.
**When to use:** All new engines in this project.

```rust
// Source: src/engines/cma/engine.rs (VERIFIED: codebase)
pub struct PsoEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: PsoConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> PsoEngine<U>
where
    U::Gene: RealGene,
{
    pub fn new(
        config: PsoConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self { ... }

    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }
}
```

### Pattern 2: PsoState Internal Struct

**What:** Private struct allocated once before the loop, holds all PSO bookkeeping.
**When to use:** Mirrors `CmaState` pattern from `src/engines/cma/engine.rs`.

```rust
// Pattern: src/engines/cma/engine.rs CmaState (VERIFIED: codebase)
struct PsoState {
    /// Problem dimension (genes per chromosome).
    dim: usize,
    /// Number of particles.
    n_particles: usize,
    /// Velocities: [particle][gene].
    velocities: Vec<Vec<f64>>,
    /// Personal best positions: [particle][gene].
    pbest_positions: Vec<Vec<f64>>,
    /// Personal best fitness values: [particle].
    pbest_fitness: Vec<f64>,
    /// Global best position (gbest).
    gbest_position: Vec<f64>,
    /// Global best fitness.
    gbest_fitness: f64,
    /// v_max per gene (= hi_i - lo_i).
    v_max: Vec<f64>,
}
```

### Pattern 3: Observer Wiring

**What:** `notify()` helper dispatches to observer if present. Five hooks fired in the PSO loop.
**When to use:** Matches `CmaEngine::notify()` pattern exactly.

```rust
// Source: src/engines/cma/engine.rs (VERIFIED: codebase)
// on_run_start — before everything
self.notify(|obs| obs.on_run_start());

// on_generation_start — top of each generation
self.notify(|obs| obs.on_generation_start(gen));

// on_new_best — when global best improves
let best_clone = best.clone();
self.notify(|obs| obs.on_new_best(gen, best_clone));

// on_generation_end — after stats computed
self.notify(|obs| obs.on_generation_end(&stats));

// on_run_end — after loop exits
self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));
```

### Pattern 4: WASM-Safe Timing Gate

**What:** `Instant::now()` is forbidden on WASM. Gate at call site.
**When to use:** Any code that would measure wall-clock time.

```rust
// Source: src/engines/gp/engine.rs and CLAUDE.md (VERIFIED: codebase)
#[cfg(not(target_arch = "wasm32"))]
let t0 = std::time::Instant::now();
// ... work ...
#[cfg(not(target_arch = "wasm32"))]
let elapsed = t0.elapsed();
```

Note: PSO has no rayon usage (sequential by nature), so no `par_iter` WASM gate is needed.

### Pattern 5: lib.rs Re-export

**What:** New engines are added via `#[path]` alias and `pub use`.
**When to use:** Adding any new top-level module to the library.

```rust
// Source: src/lib.rs (VERIFIED: codebase)
#[path = "engines/pso/mod.rs"]
pub mod pso;

// In the pub use block:
pub use pso::{PsoEngine, PsoConfiguration, PsoResult, PsoTopology, PsoInertia};
```

### Pattern 6: Velocity Update Formula

**What:** Standard PSO update per particle `i`, per gene `d`, per generation.

```rust
// Standard PSO (CITED: Kennedy & Eberhart 1995; Shi & Eberhart 1998)
// As specified in 57-CONTEXT.md velocity formula
let w = inertia_weight(gen, &config.inertia, config.max_generations);
let r1: f64 = rng.random();
let r2: f64 = rng.random();
let best_d = match config.topology {
    PsoTopology::Global => state.gbest_position[d],
    PsoTopology::Ring { neighborhood_size } => lbest_position(i, d, neighborhood_size, &state),
};
let new_v = w * state.velocities[i][d]
    + config.c1 * r1 * (state.pbest_positions[i][d] - x_curr)
    + config.c2 * r2 * (best_d - x_curr);
// Clamp velocity magnitude
let new_v = new_v.clamp(-state.v_max[d], state.v_max[d]);
// Update position
let new_x = x_curr + new_v;
// Absorbing boundary
let (lo, hi) = gene_bounds(d);
let (new_x, new_v) = if new_x < lo {
    (lo, 0.0)
} else if new_x > hi {
    (hi, 0.0)
} else {
    (new_x, new_v)
};
```

### Anti-Patterns to Avoid

- **Storing velocities in chromosomes:** Velocities are engine-internal state (D-01). Never add velocity fields to `ChromosomeT` or `LinearChromosome`.
- **Accessing bounds via `RealGene` trait:** `RealGene` has only `real_value()` and `with_real_value()` — no bounds methods. Bounds must come from gene metadata (`.ranges` field on `Range<f64>`) or a new optional `bounds()` method on `RealGene`.
- **Using `par_iter` in the PSO loop:** PSO velocity updates are sequential (each particle is independent but the topology lookup reads shared state). Even if per-particle evaluation is parallelizable, the global state mutation makes naive rayon usage incorrect.
- **Using `Instant::now()` unconditionally:** Gate all timing behind `#[cfg(not(target_arch = "wasm32"))]`.
- **Inline tests:** All tests go in `tests/engines/pso/test_pso.rs`, never `#[cfg(test)]` in `src/`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RNG | Custom PRNG | `crate::rng::make_rng()` | Project-standard RNG with optional seeding; consistent across all engines |
| Fitness tracking | Custom best-tracker | `is_better()` helper (same pattern as `CmaEngine`) | Handles Minimize/Maximize/FixedFitness uniformly |
| Stats collection | Custom struct | `GenerationStats::from_fitness_values()` | Standardized stats format passed to observer |
| Observer dispatch | Direct calls | `notify()` helper pattern | No-op when `observer = None`; avoids `if let` repetition |
| Gene construction | Manual struct | `gene.with_real_value(v)` from `RealGene` | Zero-copy gene construction with preserved metadata |

**Key insight:** PSO is algorithmically simpler than CMA-ES but must follow the same structural conventions. The complexity is in velocity bookkeeping and bounds management, not in the algorithm itself.

---

## Critical Finding: Bounds Access

`RealGene` (verified at `src/traits/real_gene.rs`) exposes only:
- `fn real_value(&self) -> f64`
- `fn with_real_value(&self, value: f64) -> Self`

No `bounds()`, `lo()`, or `hi()` method exists. [VERIFIED: codebase]

PSO requires per-gene `(lo, hi)` for:
1. Initial velocity computation: `v_init ∈ [-(hi-lo), +(hi-lo)]` (D-02)
2. `v_max` per gene: `hi - lo` (D-08)
3. Absorbing boundary enforcement (D-07)

**Two options — planner must choose:**

| Option | Approach | Pro | Con |
|--------|----------|-----|-----|
| A | Add `fn bounds(&self) -> Option<(f64, f64)>` to `RealGene` | Cleanest, generic, maintains trait abstraction | Minor trait change (non-breaking — default impl returns `None`) |
| B | Auto-extract bounds from initial population values (observe min/max per gene across init pop) | No trait changes | Unreliable (init pop may not span full range); not robust |
| C | Require user to pass explicit bounds via `PsoConfiguration` | Simple | Duplicates what the gene already knows; poor ergonomics |

**Recommendation (Claude's discretion):** Option A. Add `fn bounds(&self) -> Option<(f64, f64)>` to `RealGene` with default impl returning `None`. Existing mutation operators already access `gene.ranges` directly on the concrete type — adding a trait method formalizes this pattern. `Range<f64>` impl returns `Some((ranges[0].0, ranges[0].1))`. When `bounds()` returns `None`, PSO falls back to `v_max = 1.0` per gene (safe default). This is a non-breaking addition to `RealGene`.

---

## Common Pitfalls

### Pitfall 1: Ring Topology Index Wrapping Off-by-One

**What goes wrong:** `(i ± k/2) mod n` wraps incorrectly for odd `neighborhood_size` or small swarms where `k >= n`.
**Why it happens:** Integer division of `k/2` drops remainders; modular arithmetic on negative indices panics in Rust (`-1 % n` is not `n-1` in Rust).
**How to avoid:** Use `(i + n - offset) % n` for left-side wrap. Clamp `neighborhood_size` to `n - 1` if `neighborhood_size >= n`. Test with swarm size smaller than neighborhood.
**Warning signs:** Test with `n=3`, `neighborhood_size=5` — should see each particle see its entire neighborhood without panic.

### Pitfall 2: Personal Best Not Initialized Correctly

**What goes wrong:** Personal best positions not copied from initial population — initialized to zeros or defaults instead.
**Why it happens:** `PsoState::new()` allocates pbest_positions as `vec![vec![0.0; dim]; n]` without copying actual gene values.
**How to avoid:** In `PsoState::new()`, extract `pbest_positions[i][d] = pop[i].dna()[d].real_value()` and `pbest_fitness[i] = pop[i].fitness()` directly from the evaluated initial population.
**Warning signs:** PSO converges to origin on functions where origin is not optimal.

### Pitfall 3: v_max Not Applied Before Position Update

**What goes wrong:** Position update overshoots massively; absorbing boundary fires every generation; particles cluster at bounds.
**Why it happens:** The D-06/D-08 sequence is: (1) compute new velocity, (2) clamp |v| ≤ v_max, (3) update position, (4) absorbing boundary. Applying absorbing boundary before velocity clamp, or forgetting velocity clamp entirely, causes explosive divergence.
**How to avoid:** Enforce the exact sequence from the CONTEXT.md velocity formula spec: velocity clamp first, then position update, then absorbing boundary.
**Warning signs:** All particles at the edges of the search space after a few generations.

### Pitfall 4: Gbest Not Updated After Each Particle

**What goes wrong:** Global best only updated once per generation (after all particles update) rather than after each particle's fitness is evaluated.
**Why it happens:** Standard PSO updates gbest after the generation-wide sweep; some implementations update it mid-sweep (synchronous vs. asynchronous PSO). Both are valid, but mixing them creates subtle bugs.
**How to avoid:** Use synchronous update (update gbest once at the end of each generation's particle sweep). Document this choice. The CONTEXT.md formula is consistent with synchronous gbest.
**Warning signs:** Non-deterministic convergence behavior; different results with same seed between runs.

### Pitfall 5: LinearDecay Inertia Division by Zero

**What goes wrong:** `max_generations = 1` causes division by zero in `(w_start - w_end) / (max_generations - 1)`.
**Why it happens:** Linear interpolation formula requires at least 2 generations.
**How to avoid:** Guard: `if max_generations <= 1 { return w_end }` in the inertia computation function.
**Warning signs:** Panic on single-generation runs; caught in boundary tests.

---

## Code Examples

### PsoConfiguration Structure (mirrors CmaConfiguration)

```rust
// Source: src/engines/cma/configuration.rs pattern (VERIFIED: codebase)
#[derive(Debug, Clone)]
pub struct PsoConfiguration {
    pub population_size: usize,        // swarm size; 0 = auto (heuristic: 20-50 for 10D)
    pub max_generations: usize,        // default: 1000
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    pub inertia: PsoInertia,           // default: LinearDecay { w_start: 0.9, w_end: 0.4 }
    pub c1: f64,                       // cognitive coefficient; default: 2.0
    pub c2: f64,                       // social coefficient; default: 2.0
    pub topology: PsoTopology,         // default: PsoTopology::Global
}

#[derive(Debug, Clone)]
pub enum PsoInertia {
    Constant(f64),
    LinearDecay { w_start: f64, w_end: f64 },
}

#[derive(Debug, Clone)]
pub enum PsoTopology {
    Global,
    Ring { neighborhood_size: usize },
}
```

### PsoResult (mirrors CmaResult)

```rust
// Source: src/engines/cma/engine.rs CmaResult pattern (VERIFIED: codebase)
pub struct PsoResult<U: LinearChromosome> {
    pub population: Vec<U>,
    pub best: U,
    pub best_fitness: f64,
    pub generations: usize,
}
```

### mod.rs Re-export Pattern

```rust
// Source: src/engines/cma/mod.rs (VERIFIED: codebase)
pub mod configuration;
pub mod engine;

pub use configuration::{PsoConfiguration, PsoInertia, PsoTopology};
pub use engine::{PsoEngine, PsoResult};
```

### lib.rs Integration

```rust
// Source: src/lib.rs pattern (VERIFIED: codebase)
// In module declarations block:
#[path = "engines/pso/mod.rs"]
pub mod pso;

// In pub use block:
pub use pso::{PsoEngine, PsoConfiguration, PsoResult, PsoTopology, PsoInertia};
```

### pso_rastrigin Example Structure (follow cma_es_rastrigin.rs)

```rust
// Source: examples/cma_es_rastrigin.rs (VERIFIED: codebase)
// Pattern to follow for pso_rastrigin.rs:
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::LogObserver;

const DIMENSIONS: usize = 10;  // 10D (vs 5D for CMA)
const SEARCH_LO: f64 = -5.12;
const SEARCH_HI: f64 = 5.12;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `DeGene` trait | `RealGene` trait | Phase 56 | PSO uses `RealGene` bound, not the old name |
| `crate::observe::observer::GaObserver` import | `crate::observer::GaObserver` | v2.4.0 (phase 36+) | Use `crate::observer::GaObserver` in imports |
| Inline tests `#[cfg(test)]` | Tests in `tests/` directory | Project-wide policy | All PSO tests go in `tests/engines/pso/test_pso.rs` |

**Deprecated/outdated:**
- `DeGene`: fully renamed to `RealGene` in phase 56 — do not use `DeGene` anywhere in PSO code.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `RealGene::bounds()` is the best approach for bound access | Critical Finding section | If approach B or C chosen, implementation differs; low risk since all three options are viable |
| A2 | Default ring `neighborhood_size` of 3 is appropriate | Discretion area | Minor UX impact; 5 is also common; either is fine |
| A3 | `c1` and `c2` should be exposed in `PsoConfiguration` (not hardcoded) | Discretion area | Low risk — both approaches work; exposed gives more user control |

**Bounds-access note:** The decision of which option (A/B/C) to use for bounds access is left to the planner per the "Claude's Discretion" scope. Option A (`bounds()` on `RealGene`) is recommended but requires a one-line addition to the `RealGene` trait.

---

## Open Questions

1. **Bounds access approach**
   - What we know: `RealGene` has no bounds method; `Range<f64>.ranges` stores `(lo, hi)` pairs
   - What's unclear: Whether to extend `RealGene` (Option A), derive bounds from init pop (Option B), or add explicit config bounds (Option C)
   - Recommendation: Option A — add `fn bounds(&self) -> Option<(f64, f64)>` with default `None`; implement for `Range<f64>` and `MultiRangeGenotype<f64>`

2. **Population size default for PSO**
   - What we know: CMA-ES uses `4 + floor(3*ln(n))` (Hansen formula); PSO literature typically uses 20-50 particles independent of dimension
   - What's unclear: Whether to use a dimension-aware formula or a fixed default
   - Recommendation: Default `population_size = 30` (independent of dimension); expose `with_population_size()` builder

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies. PSO uses only existing Rust stdlib and already-present crate dependencies. `cargo check --target wasm32-unknown-unknown` is a CI gate (`.github/workflows/wasm-check.yml` already present).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (project uses `Cargo.toml` test discovery) |
| Quick run command | `cargo test --test test_pso 2>&1 \| tail -20` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements to Test Map

| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| `PsoEngine::new(config, init_fn, fitness_fn).run()` returns `PsoResult<U>` | Integration | `cargo test --test test_pso test_pso_run_returns_result` | No — Wave 0 |
| Personal best updated when fitness improves | Unit | `cargo test --test test_pso test_pso_pbest_update` | No — Wave 0 |
| Observer `on_run_start` fires exactly once | Integration | `cargo test --test test_pso test_pso_observer_run_start` | No — Wave 0 |
| Observer `on_generation_start` fires once per generation | Integration | `cargo test --test test_pso test_pso_observer_generation_count` | No — Wave 0 |
| Observer `on_new_best` fires when best improves | Integration | `cargo test --test test_pso test_pso_observer_new_best` | No — Wave 0 |
| Observer `on_run_end` fires exactly once | Integration | `cargo test --test test_pso test_pso_observer_run_end` | No — Wave 0 |
| Ring topology neighborhood wraps correctly at boundaries | Unit | `cargo test --test test_pso test_pso_ring_wrap` | No — Wave 0 |
| Absorbing boundary zeroes velocity at gene bounds | Unit | `cargo test --test test_pso test_pso_absorbing_boundary` | No — Wave 0 |
| LinearDecay inertia produces w_start at gen 0 and w_end at max | Unit | `cargo test --test test_pso test_pso_linear_decay` | No — Wave 0 |
| Sphere function minimized (convergence smoke test) | Integration | `cargo test --test test_pso test_pso_sphere_converges` | No — Wave 0 |
| `cargo check --target wasm32-unknown-unknown` passes | WASM CI | CI gate | CI existing |

### Sampling Rate

- **Per task commit:** `cargo test --test test_pso 2>&1 | tail -20`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green + WASM check before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/pso/` directory (create)
- [ ] `tests/engines/pso/test_pso.rs` — stub tests with `#[ignore]` markers, all PSO behaviors
- [ ] `src/engines/pso/` directory (create)
- [ ] `src/engines/pso/engine.rs`, `configuration.rs`, `mod.rs` — stub files

---

## Security Domain

Security enforcement for this phase: not applicable. PSO is a pure computation engine with no I/O, no user-controlled inputs beyond the fitness function, and no network or filesystem access. No ASVS categories apply.

---

## Sources

### Primary (HIGH confidence)
- `src/engines/cma/engine.rs` — CmaEngine struct layout, CmaState pattern, observer wiring, run loop structure, is_better helper, find_best helper, notify helper [VERIFIED: codebase]
- `src/engines/cma/configuration.rs` — CmaConfiguration pattern, builder methods, Default impl [VERIFIED: codebase]
- `src/engines/cma/mod.rs` — Module wiring pattern [VERIFIED: codebase]
- `src/traits/real_gene.rs` — `RealGene` trait surface (only `real_value` + `with_real_value`) [VERIFIED: codebase]
- `src/traits/linear_chromosome.rs` — `LinearChromosome` trait surface [VERIFIED: codebase]
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait, all hook signatures [VERIFIED: codebase]
- `src/lib.rs` — `#[path]` re-export pattern, observer import path `crate::observer::GaObserver` [VERIFIED: codebase]
- `src/types/genotypes/range.rs` — `Range<T>` struct with `.ranges: Arc<[(T, T)]>` field [VERIFIED: codebase]
- `src/stats.rs` — `GenerationStats` struct and `from_fitness_values()` constructor [VERIFIED: codebase]
- `examples/cma_es_rastrigin.rs` — Example structure to follow for `pso_rastrigin.rs` [VERIFIED: codebase]
- `tests/engines/cma/test_cma.rs` — Test file pattern (SpyObserver, helper functions, structure) [VERIFIED: codebase]
- `.planning/phases/57-pso-engine/57-CONTEXT.md` — All locked decisions [VERIFIED: codebase]

### Secondary (MEDIUM confidence)
- Standard PSO algorithm: Kennedy & Eberhart (1995), Shi & Eberhart (1998) TPSO — velocity formula, linear decay defaults (w_start=0.9, w_end=0.4, c1=c2=2.0) are well-established literature values [ASSUMED: training knowledge, but universally consistent across PSO literature]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps; everything existing and verified
- Architecture patterns: HIGH — CmaEngine is the direct template, fully read
- Trait surface: HIGH — RealGene and LinearChromosome verified in codebase
- Observer wiring: HIGH — GaObserver trait verified, CmaEngine notify() pattern verified
- Bounds access pitfall: HIGH — verified by reading RealGene source; mutation operators provide precedent
- PSO algorithm defaults: MEDIUM — well-known literature values, marked [ASSUMED]

**Research date:** 2026-06-02
**Valid until:** 2026-07-02 (stable internal codebase; no external deps)
