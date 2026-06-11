# Phase 56: CMA-ES Engine - Research

**Researched:** 2026-06-01
**Domain:** CMA-ES algorithm implementation in Rust, gene trait rename, observer wiring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `DeGene` is hard-renamed to `RealGene` in this phase. Methods: `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()`. All `DeEngine` bounds update to `RealGene`. `CmaEngine` also bounds on `U::Gene: RealGene`. No deprecated alias.
- **D-02:** The `DeGene` impl on `Range<f64>` becomes a `RealGene` impl. File placement of `RealGene` is Claude's discretion.
- **D-03:** No restart logic in this phase. `CmaEngine` runs a fixed `max_generations` loop with optional `fitness_target` early stopping. Restart strategies deferred to issue #255.
- **D-04:** `CmaConfiguration` exposes optional tuning: `cc`, `cs`, `c1`, `cmu` (all `Option<f64>`, default `None` = Hansen's auto formulas). Builder methods: `.with_cc()`, `.with_cs()`, `.with_c1()`, `.with_cmu()`.
- **D-05:** Required config fields: `sigma0: f64` (default 0.3), `population_size: usize` (λ, auto-computed from `n` if 0 or via `default_for_dim(n)`), `max_generations: usize`, `problem_solving: ProblemSolving`, `fitness_target: Option<f64>`.
- **D-06:** `CmaEngine` includes `Option<Arc<dyn GaObserver<U> + Send + Sync>>` from day 1. Hooks fire: `on_run_start`, `on_generation_start`/`on_generation_end` per generation (with `GenerationStats`), `on_new_best` when best improves, `on_run_end` after loop. Mandatory per CLAUDE.md.
- **D-07:** Example benchmark: left to planner/executor. Natural choice: `cma_es_rastrigin`.

### Claude's Discretion

- File placement of `RealGene` trait (new shared module vs. kept in `src/engines/de/gene.rs` with re-export)
- Whether `GenerationStats` fields are populated from CMA-ES internal state or computed separately
- Internal CMA-ES bookkeeping structures (pc, ps, C, eigendecomposition scheduling)
- Example benchmark choice

### Deferred Ideas (OUT OF SCOPE)

- Restart strategies (IPOP/BIPOP) — Issue #255
- Active CMA-ES (negative update for bad steps)
- CMA-ES in multi-objective mode (CMA-ES-MO, MO-CMA-ES)
</user_constraints>

---

## Summary

CMA-ES (Covariance Matrix Adaptation Evolution Strategy) is a well-defined black-box continuous optimization algorithm described fully in Hansen's tutorial (arXiv:1604.00772). Its full-precision default formulas for all hyperparameters are derived from problem dimension `n`, making implementation deterministic once the algorithm skeleton and data structures are understood. No external linear algebra dependencies (ndarray/nalgebra) are needed — the algorithm works on flat `Vec<f64>` arrays and requires only a pure-Rust Cholesky/eigendecomposition or an incremental Jacobi method on the covariance matrix `C`.

The rename from `DeGene` to `RealGene` is a mechanical cascade touching 5 files in `src/` and 0 test files (tests only use `DeEngine` and `Range<f64>`, never reference `DeGene` by name). The `ScatterEngine` also bounds on `DeGene`, making it a 6th file to update.

The `GpGa` engine (most recently added) is the correct observer hook integration template: it stores `Option<Arc<dyn GaObserver<U> + Send + Sync>>`, uses a private `notify()` helper, and gates `Instant::now()` with `#[cfg(not(target_arch = "wasm32"))]`. CMA-ES is inherently sequential per generation (no rayon parallelism needed in the core loop), so WASM compatibility is straightforward.

**Primary recommendation:** Implement CMA-ES as a standalone, self-contained `src/engines/cma/` module with all state (mean, sigma, pc, ps, C, eigendecomposition) in an internal `CmaState` struct. Move `RealGene` to `src/traits/real_gene.rs` (parallel to `real_valued.rs`) and re-export from both `src/traits/mod.rs` and `crate::de` for backward compatibility at the module level (the trait itself has no backward compat — it's a hard rename per D-01).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CMA-ES parameter update (mean, C, sigma) | `CmaEngine::run()` | — | All state is local to the engine run loop; no shared state |
| Eigendecomposition scheduling | `CmaState` internal struct | — | O(n³) operation; scheduled every `floor(1/(10·n·sqrt(n)))` generations |
| Gene arithmetic (real_value/with_real_value) | `RealGene` trait | — | Shared trait in `src/traits/` used by `DeEngine`, `ScatterEngine`, `CmaEngine` |
| Configuration validation | `CmaConfiguration::build()` / `run()` | — | Mirrors `GpConfiguration::build()` pattern |
| Observer hooks | `CmaEngine::notify()` | `GaObserver<U>` | All 5 required hooks wired from day 1 per D-06 |
| GenerationStats population | `CmaEngine::run()` | `GenerationStats::from_fitness_values()` | Use existing `from_fitness_values()` factory; diversity = fitness std_dev |
| Public API surface | `src/engines/cma/mod.rs` + `src/lib.rs` | — | Pattern: `pub mod cma` via `#[path]` in lib.rs |

---

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` (existing) | workspace | RNG — `make_rng()` | Already used by all engines |
| `log` (existing) | workspace | Logging with `target:` | Used by all engines |
| `rayon` (existing, NOT used in CMA-ES core) | workspace | Not needed in CMA-ES | CMA-ES updates are sequential per generation |

[VERIFIED: codebase] All three are already in `Cargo.toml`. No new dependencies are required for this phase.

### Supporting (pure stdlib)

| Item | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `Vec<f64>` | stdlib | Covariance matrix C (n×n stored row-major), evolution paths pc, ps, mean m | All CMA-ES internal state |
| `std::borrow::Cow` | stdlib | `set_dna()` zero-copy DNA update | When writing new offspring chromosomes |
| `#[cfg(not(target_arch = "wasm32"))]` | stdlib | Gate `Instant::now()` calls | Any timing code inside observer notifications |

### No New External Crates

The CMA-ES eigendecomposition is the only mathematically complex step. The standard approach in constrained Rust (no nalgebra) is a Jacobi iteration on the symmetric matrix C. For n ≤ 100 (typical CMA-ES use), this converges in < 20 iterations per scheduling window. The implementation fits in ~60 lines of safe Rust.

**Alternative if eigendecomposition is undesirable:** Use Cholesky factorization and update the Cholesky factor `A` directly (Cholesky-CMA variant). This avoids eigendecomposition entirely and is numerically stable. The trade-off is slightly different adaptation behavior — standard CMA-ES uses eigendecomposition; Cholesky-CMA is a recognized variant. [ASSUMED — which variant to use is Claude's discretion per D-02]

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pure Rust Jacobi eigen | `nalgebra` crate | nalgebra adds ~1MB binary, requires WASM compat check; Jacobi is 60 lines and sufficient for n ≤ 100 |
| Pure Rust Jacobi eigen | `ndarray` + `ndarray-linalg` | `ndarray-linalg` links LAPACK — incompatible with `wasm32-unknown-unknown` |
| Row-major Vec<f64> for C | 2D array of arrays | `Vec<f64>` is idiomatic and cache-friendly for BLAS-style loops |

**Installation:** No new packages. All required dependencies are already in `Cargo.toml`.

---

## Package Legitimacy Audit

No new packages are installed in this phase. All code uses existing workspace dependencies.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (none) | — | — | — | — | — | — |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User code
    │ CmaEngine::new(config, init_fn, fitness_fn)
    │ CmaEngine::with_observer(obs)
    ▼
CmaEngine<U>
    ├── config: CmaConfiguration
    ├── init_fn: Arc<Fn(usize) -> Vec<U>>
    ├── fitness_fn: Arc<Fn(&[U::Gene]) -> f64>
    └── observer: Option<Arc<dyn GaObserver<U>>>
    │
    │ .run() → CmaResult<U>
    ▼
on_run_start ──────────────────────────────────► GaObserver
    │
    │  initialize population (init_fn)
    │  evaluate fitness (fitness_fn per individual)
    │  compute initial mean m = weighted average of μ best
    │
    │  CmaState { m, sigma, pc, ps, C, D, B, invsqrtC, eigeneval }
    │
    ├── for gen in 0..max_generations:
    │       on_generation_start(gen) ────────────► GaObserver
    │       │
    │       │  sample λ offspring from N(m, sigma²·C)
    │       │  ─── z_k ~ N(0, I), y_k = B·D·z_k, x_k = m + sigma·y_k
    │       │  evaluate fitness for each x_k
    │       │  sort by fitness → select μ best
    │       │
    │       │  update mean: m' = Σ w_i · x_{i:λ}
    │       │  update evolution paths: ps, pc
    │       │  update covariance matrix: C ← rank-one + rank-mu
    │       │  update step size: sigma via CSA
    │       │  schedule eigendecomposition of C (every T_eigen generations)
    │       │
    │       │  on_new_best(gen, best_clone) ──────► GaObserver (if improved)
    │       │  collect GenerationStats
    │       │  on_generation_end(&stats) ──────────► GaObserver
    │       │
    │       └── check fitness_target early stop
    │
    │  on_run_end(cause, &all_stats) ───────────► GaObserver
    ▼
CmaResult<U> { population, best, best_fitness, generations }
```

### Recommended Project Structure

```
src/engines/cma/
├── mod.rs              # pub use engine::{CmaEngine, CmaResult}; pub use configuration::CmaConfiguration
├── engine.rs           # CmaEngine<U>, CmaResult<U>, CmaState (private)
└── configuration.rs    # CmaConfiguration, builder methods

src/traits/
├── real_gene.rs        # RealGene trait (renamed from DeGene) + Range<f64> impl
└── mod.rs              # + pub mod real_gene; pub use real_gene::RealGene;

tests/engines/cma/
└── test_cma.rs         # All CMA-ES tests

tests/test_engines.rs   # + mod cma { mod test_cma; }
```

### Pattern 1: CmaState Internal Bookkeeping

**What:** All CMA-ES adaptation state lives in a private `CmaState` struct initialized before the loop and mutated in-place each generation.

**When to use:** Keeps the `run()` method readable; state fields are not user-visible.

```rust
// Source: Hansen CMA-ES tutorial, arXiv:1604.00772, Section 4
struct CmaState {
    n: usize,           // problem dimension
    lambda: usize,      // population size λ
    mu: usize,          // number of parents μ
    weights: Vec<f64>,  // recombination weights w_i (normalized, positive only)
    mu_eff: f64,        // effective selection mass
    // Step-size control
    cs: f64,            // step-size cumulation (σ)
    ds: f64,            // step-size damping
    chi_n: f64,         // E[||N(0,I)||] ≈ sqrt(n)*(1 - 1/(4n) + 1/(21n²))
    // Covariance update
    cc: f64,            // covariance cumulation (pc)
    c1: f64,            // rank-one update rate
    cmu: f64,           // rank-mu update rate
    // Mutable state
    mean: Vec<f64>,     // distribution mean m (length n)
    sigma: f64,         // global step size σ
    ps: Vec<f64>,       // evolution path for σ-control (length n)
    pc: Vec<f64>,       // evolution path for rank-one update (length n)
    c_mat: Vec<f64>,    // covariance matrix C (n×n, row-major)
    // Eigendecomposition cache (updated on schedule)
    b_mat: Vec<f64>,    // eigenvectors B (n×n)
    d_vec: Vec<f64>,    // sqrt(eigenvalues) D (length n)
    invsqrtc: Vec<f64>, // C^{-1/2} = B · diag(1/D) · B^T (n×n)
    eigeneval: usize,   // generation counter when eigen was last computed
}
```

[ASSUMED — exact field names; logic derived from Hansen's tutorial arXiv:1604.00772]

### Pattern 2: Hansen Default Parameter Formulas

**What:** All CMA-ES parameters have closed-form defaults in terms of `n` (dimension) and `λ`.

**When to use:** When `CmaConfiguration` fields are `None`, compute these at the start of `run()`.

```rust
// Source: Hansen, arXiv:1604.00772, Table 1
// λ = 4 + floor(3 * ln(n))
let lambda = population_size.max(4 + (3.0 * (n as f64).ln()).floor() as usize);
// μ = floor(λ/2)
let mu = lambda / 2;
// Weights: w_i = ln((λ+1)/2) - ln(i)  for i = 1..=mu  (un-normalized)
let weights_raw: Vec<f64> = (1..=mu).map(|i| ((lambda as f64 + 1.0) / 2.0).ln() - (i as f64).ln()).collect();
let w_sum: f64 = weights_raw.iter().sum();
let weights: Vec<f64> = weights_raw.iter().map(|w| w / w_sum).collect();
// μ_eff = (Σ w_i)² / Σ w_i²  = 1 / Σ w_i²  (since Σ w_i = 1)
let mu_eff = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

// Step-size control
let cs = config.cs.unwrap_or((mu_eff + 2.0) / (n as f64 + mu_eff + 5.0));
let ds = 1.0 + 2.0 * (0.0_f64).max((mu_eff - 1.0) / (n as f64 + 1.0)).sqrt() + cs;
let chi_n = (n as f64).sqrt() * (1.0 - 1.0 / (4.0 * n as f64) + 1.0 / (21.0 * (n as f64).powi(2)));

// Covariance
let cc = config.cc.unwrap_or((4.0 + mu_eff / n as f64) / (n as f64 + 4.0 + 2.0 * mu_eff / n as f64));
let c1 = config.c1.unwrap_or(2.0 / ((n as f64 + 1.3).powi(2) + mu_eff));
let cmu = config.cmu.unwrap_or(
    (2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / ((n as f64 + 2.0).powi(2) + mu_eff))
        .min(1.0 - c1)
);

// Eigendecomposition interval (Hansen arXiv:1604.00772)
// Canonical formula: t_eigen = max(1, floor(n^1.5 * 10 / lambda))
let t_eigen = ((n as f64).powf(1.5) * 10.0 / lambda as f64).floor() as usize;
let t_eigen = t_eigen.max(1);
```

[ASSUMED — formulas transcribed from Hansen's tutorial; should be cross-verified against arXiv:1604.00772 Table 1 during implementation]

### Pattern 3: Offspring Sampling

**What:** Sample λ candidate points from the current distribution `N(m, σ²·C)`.

**When to use:** Each generation, after parameter initialization.

```rust
// Source: Hansen, arXiv:1604.00772, Section 4
// x_k = m + σ · B · D · z_k    where z_k ~ N(0, I)
// B columns are eigenvectors of C; D diagonal = sqrt(eigenvalues)
for k in 0..lambda {
    // Box-Muller inline (no rand_distr dependency):
    let z: Vec<f64> = (0..n).map(|_| standard_normal(&mut rng)).collect();
    // y = B · D · z   (n×n matrix-vector products)
    let y: Vec<f64> = (0..n).map(|i| {
        (0..n).map(|j| state.b_mat[i * n + j] * state.d_vec[j] * z[j]).sum()
    }).collect();
    let x: Vec<f64> = (0..n).map(|i| state.mean[i] + state.sigma * y[i]).collect();
    // write x back into offspring[k].dna via with_real_value()
    // ...
}
```

[ASSUMED — exact indexing depends on row-major vs column-major B storage choice]

### Pattern 4: CMA-ES Update Equations

**What:** Per-generation state update for mean, evolution paths, covariance, step size.

```rust
// Source: Hansen, arXiv:1604.00772, Equations (1)–(6)

// 1. Update mean
let new_mean: Vec<f64> = (0..n).map(|i|
    weights.iter().zip(sorted_offspring.iter())
        .map(|(w, x)| w * x[i]).sum()
).collect();

// 2. Update ps (step-size evolution path, isotropic)
// ps = (1-cs)*ps + sqrt(cs*(2-cs)*mu_eff) * invsqrtC * (new_mean - old_mean) / sigma
let step = ... // (new_mean - old_mean) / sigma
let invsqrtC_step: Vec<f64> = matvec(&state.invsqrtc, &step, n);
state.ps = (0..n).map(|i|
    (1.0 - cs) * state.ps[i] + (cs * (2.0 - cs) * mu_eff).sqrt() * invsqrtC_step[i]
).collect();

// 3. h_sigma indicator (prevents too-large ps update from influencing pc)
let ps_norm = state.ps.iter().map(|x| x * x).sum::<f64>().sqrt();
let h_sigma = if ps_norm / (1.0 - (1.0 - cs).powi(2 * (gen + 1))).sqrt() / chi_n
    < 1.4 + 2.0 / (n as f64 + 1.0) { 1.0 } else { 0.0 };

// 4. Update pc (covariance evolution path)
// pc = (1-cc)*pc + h_sigma * sqrt(cc*(2-cc)*mu_eff) * (new_mean - old_mean) / sigma
state.pc = (0..n).map(|i|
    (1.0 - cc) * state.pc[i] + h_sigma * (cc * (2.0 - cc) * mu_eff).sqrt() * step[i]
).collect();

// 5. Update C: rank-one + rank-mu
// C = (1 - c1 - cmu) * C
//   + c1 * (pc · pc^T + (1-h_sigma)*cc*(2-cc)*C)
//   + cmu * Σ_i w_i * y_i:λ · (y_i:λ)^T
// where y_i:λ = (x_i:λ - old_mean) / sigma

// 6. Update sigma via Cumulative Step-size Adaptation (CSA)
// sigma *= exp((cs/ds) * (||ps|| / chi_n - 1))
state.sigma *= ((cs / ds) * (ps_norm / chi_n - 1.0)).exp();
```

[ASSUMED — formulas transcribed from Hansen tutorial; verify signs and indices during implementation]

### Pattern 5: Observer Wiring (from GpGa reference)

**What:** The exact pattern used in `GpGa` — copy this verbatim.

```rust
// Source: src/engines/gp/engine.rs
use crate::observe::observer::GaObserver;
// Note: lib.rs re-exports as crate::observer::GaObserver

pub struct CmaEngine<U: LinearChromosome> where U::Gene: RealGene {
    config: CmaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> CmaEngine<U> where U::Gene: RealGene {
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

// In run():
self.notify(|obs| obs.on_run_start());
// ...
self.notify(|obs| obs.on_generation_start(gen));
// ...
let best_clone = best.clone();
self.notify(|obs| obs.on_new_best(gen, best_clone));
// ...
self.notify(|obs| obs.on_generation_end(&stats));
// ...
self.notify(|obs| obs.on_run_end(termination_cause, &all_stats));
```

[VERIFIED: codebase — exact pattern from `src/engines/gp/engine.rs`]

### Pattern 6: WASM-Gated Timing

**What:** `Instant::now()` must be gated — copied from GpGa.

```rust
// Source: src/engines/gp/engine.rs (lines 250-260 pattern)
let t_fit: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
// ... work ...
if let Some(t) = t_fit {
    let count = pop.len();
    self.notify(|obs| obs.on_fitness_evaluation_complete(gen, t.elapsed(), count));
}
```

[VERIFIED: codebase — exact pattern from `src/engines/gp/engine.rs`]

### Pattern 7: RealGene Trait (Rename from DeGene)

**What:** Exact interface kept, method names changed.

```rust
// Source: derived from src/engines/de/gene.rs
pub trait RealGene: GeneT {
    fn real_value(&self) -> f64;
    fn with_real_value(&self, value: f64) -> Self;
}

impl RealGene for Range<f64> {
    #[inline]
    fn real_value(&self) -> f64 { self.value }
    #[inline]
    fn with_real_value(&self, value: f64) -> Self {
        let mut g = self.clone();
        g.value = value;
        g
    }
}
```

[VERIFIED: codebase — derived from src/engines/de/gene.rs, method names changed per D-01]

### Anti-Patterns to Avoid

- **Using `par_iter()` in the CMA-ES core loop:** CMA-ES is inherently sequential per generation (the covariance update requires all offspring sorted by fitness). Parallelism is only possible in the fitness evaluation step — and even then, a simple `iter_mut()` is fine since CMA-ES typically uses small populations (λ = 4-20 for n ≤ 10).
- **Calling eigendecomposition every generation:** For n = 10, eigendecomposition of a 10×10 matrix is cheap. But for n = 50+, it's O(n³). Schedule it every `max(1, floor(lambda / (10 * n * sqrt(n))))` generations.
- **Storing the full D matrix instead of the diagonal:** `D` is diagonal — store as `Vec<f64>` of length n, not n×n.
- **Forgetting to re-extract DNA from chromosomes to build the mean:** The engine must call `individual.dna()[j].real_value()` to get the f64 coordinates for mean/covariance computation.
- **Not clamping sigma or eigenvalues:** Without numerical guards, CMA-ES can diverge. Clamp `sigma > 0`, `d_vec[i] > 1e-20`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Statistics from fitness values | Custom mean/std | `GenerationStats::from_fitness_values()` | Already handles empty pop, is/max direction; used by all engines |
| RNG initialization | Custom seeding | `crate::rng::make_rng()` | Respects global seed set by tests via `rng::set_seed()` |
| Normal distribution sampling | rolling your own without spec | Box-Muller inline helper (RESOLVED per Q2) | `rand_distr` is not a workspace dep; Box-Muller is 4 lines and well-specified |
| Observer notification pattern | Ad-hoc per-hook checks | Private `notify()` helper (GpGa pattern) | One line per hook call; zero overhead when `observer` is `None` |
| `ProblemSolving` comparison | Custom is_better logic | Inline match on `ProblemSolving` (DeEngine pattern) | DeEngine's `is_better()` helper is the correct model; replicate it |

**Key insight:** CMA-ES constructs offspring by transforming the distribution — there is no separate "crossover" or "mutation" step from the standard operator API. The engine is fully self-contained; it does not use `selection::factory`, `crossover::factory`, or `mutation::factory`. This is correct and intentional.

---

## Runtime State Inventory

This is a greenfield engine addition combined with a trait rename. No persistent runtime state exists for CMA-ES since it does not exist yet. The rename (`DeGene` → `RealGene`) is purely a source-code change with no database, registry, or stored data implications.

Nothing found in any category — verified by audit above.

---

## Breaking Change Cascade: DeGene → RealGene

### Files Requiring Changes in `src/`

| File | Change Required |
|------|----------------|
| `src/engines/de/gene.rs` | Rename trait `DeGene` → `RealGene`, rename methods `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()`. Move file to `src/traits/real_gene.rs` (Claude's discretion — see placement recommendation below). |
| `src/engines/de/engine.rs` | Update `use super::gene::DeGene` → `use crate::traits::RealGene` (or new path). Update all `DeGene` bounds to `RealGene`. |
| `src/engines/de/mutation.rs` | Update `use super::gene::DeGene` → new path. Update all `DeGene` bounds and all `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()` call sites (7 occurrences). |
| `src/engines/de/crossover.rs` | Update `use super::gene::DeGene` → new path. Update all `DeGene` bounds. No method call changes in crossover (it delegates to mutation). |
| `src/engines/de/mod.rs` | Remove `pub use gene::DeGene`. Add `pub use crate::traits::RealGene` (or `pub use crate::de::gene::RealGene` if file stays in `de/`). |
| `src/engines/scatter/engine.rs` | Update `use crate::de::gene::DeGene` → new path. Update `DeGene` bounds. Update `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()` (4 call sites). |
| `src/lib.rs` | Add `pub mod cma` via `#[path = "engines/cma/mod.rs"]`. Add `pub use traits::RealGene` (or re-export from `cma` module). If `RealGene` moves to `src/traits/`, add to traits re-exports. |

**Test files:** Zero changes required. Tests reference `DeEngine` (not `DeGene` directly) and the `Range<f64>` chromosome type. Confirmed by grep: no test file mentions `DeGene`, `de_value`, or `with_de_value`.

[VERIFIED: codebase — confirmed via grep of tests/ directory returning empty results]

### Recommended Placement for RealGene

Move `src/engines/de/gene.rs` → `src/traits/real_gene.rs`. Rationale:
- `RealGene` is a shared trait used by `DeEngine`, `ScatterEngine`, and now `CmaEngine` — it is not DE-specific.
- `src/traits/real_valued.rs` already exists as a precedent for gene-level marker traits.
- `src/engines/de/mod.rs` can re-export `pub use crate::traits::RealGene` (and alias `pub use RealGene as DeGene` is NOT added per D-01).
- Add to `src/traits/mod.rs`: `pub mod real_gene; pub use real_gene::RealGene;`
- Add to `src/lib.rs` traits re-exports: `pub use traits::RealGene;`

[ASSUMED — final placement is Claude's discretion per D-02, but this is the most architecturally sound option]

### MultiRangeGenotype: Needs RealGene Impl

`MultiRangeGenotype<f64>` has a `value: f64` field (confirmed by source). Per CONTEXT.md canonical refs, `MultiRangeChromosome<T>` also implements `RealValued` and will need a `RealGene` impl. This impl goes in the same `src/traits/real_gene.rs` file alongside the `Range<f64>` impl.

[VERIFIED: codebase — MultiRangeGenotype has `pub value: T` field confirmed in src/types/genotypes/multi_range.rs]

---

## Common Pitfalls

### Pitfall 1: Eigendecomposition of Non-PSD Matrix C

**What goes wrong:** After many generations, numerical drift can make C slightly non-positive-semidefinite, causing complex eigenvalues from a Jacobi solver.
**Why it happens:** Floating-point accumulation in the rank-one/rank-mu update terms.
**How to avoid:** After each eigendecomposition, clamp all diagonal elements of D (sqrt eigenvalues) to `max(d_i, 1e-10 * max(d_j))`. Mirror C after update to enforce symmetry: `C[i][j] = C[j][i] = (C[i][j] + C[j][i]) / 2`.
**Warning signs:** `d_vec` contains NaN or negative values; `sigma` → 0 or infinity.

### Pitfall 2: `FitnessFn<U::Gene>` Type Alias Confusion

**What goes wrong:** The fitness function in `DeEngine` takes `&[U::Gene]`, not `&U`. CMA-ES also needs a function over gene slices (to compute fitness from f64 coordinates). The `FitnessFn<G>` type alias is `Arc<dyn Fn(&[G]) -> f64 + Send + Sync>` — must match this signature exactly.
**Why it happens:** `GpGa` uses a different fitness fn signature (`&Node<N>`) — do not copy its `GpFitnessFn` type alias.
**How to avoid:** Use `use crate::traits::FitnessFn;` and `Arc<FitnessFn<U::Gene>>` as in `DeEngine`.

[VERIFIED: codebase — confirmed in src/engines/de/engine.rs]

### Pitfall 3: DNA Extraction for CMA-ES Arithmetic

**What goes wrong:** CMA-ES needs raw `f64` coordinates (not gene objects) for mean/covariance arithmetic. Forgetting to call `.real_value()` on each gene to get the float.
**Why it happens:** The chromosome stores `Vec<U::Gene>`, not `Vec<f64>`. You cannot do arithmetic on gene objects directly.
**How to avoid:** Write a helper `fn extract_coords(chr: &U) -> Vec<f64>` that maps `chr.dna().iter().map(|g| g.real_value()).collect()`. Use this consistently.

### Pitfall 4: Observer import path

**What goes wrong:** `use crate::observer::GaObserver` fails if the observer is imported from the wrong module.
**Why it happens:** The actual trait lives in `src/observe/observer/mod.rs` but is re-exported by `lib.rs` as `crate::observer`. The internal crate path used in engine files is `crate::observe::observer::GaObserver` (following the internal module path, not the public re-export).
**How to avoid:** Follow exactly what `GpGa` does: `use crate::observer::GaObserver;`. The `#[path]` alias in `lib.rs` makes `crate::observer` resolve to `src/observe/observer/mod.rs`.

[VERIFIED: codebase — confirmed in src/engines/gp/engine.rs line 11]

### Pitfall 5: TerminationCause Required for on_run_end

**What goes wrong:** `on_run_end` takes a `TerminationCause` — this must be imported and tracked.
**Why it happens:** `TerminationCause` is defined in `src/engines/ga.rs` and re-exported via `lib.rs` as `crate::ga::TerminationCause`.
**How to avoid:** Import `use crate::ga::TerminationCause;` and track a `termination_cause` variable (initialized to `GenerationLimitReached`, overwritten to `FitnessTargetReached` on early stop). Mirror `GpGa` exactly.

[VERIFIED: codebase — confirmed in src/engines/gp/engine.rs]

### Pitfall 6: Chromosome Reconstruction After CMA-ES Sample

**What goes wrong:** CMA-ES constructs new individuals from sampled f64 vectors. The init_fn creates seed chromosomes, but each generation needs new chromosomes built from `m + sigma * B * D * z`. The mechanism to write these back is `set_dna(Cow::Owned(...))` with genes constructed via `with_real_value()`.
**Why it happens:** `CmaEngine` doesn't have a `crossover` operator — it must construct offspring manually. The typical approach is to clone a template chromosome and overwrite its DNA.
**How to avoid:** After sampling `x_k: Vec<f64>`, use `pop[0].clone()` as a template, then call `template.set_dna(Cow::Owned(x_k.iter().enumerate().map(|(j, &v)| pop[0].dna()[j].with_real_value(v)).collect()))`. The gene `id` and `bounds` are preserved from the template; only the value changes.

---

## Code Examples

### CMA-ES Module Layout (mod.rs)

```rust
// Source: src/engines/de/mod.rs pattern
//! CMA-ES engine.

pub mod configuration;
pub mod engine;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
```

### Configuration Struct Skeleton

```rust
// Source: src/engines/de/configuration.rs pattern + D-04/D-05 from CONTEXT.md
use crate::configuration::ProblemSolving;

#[derive(Debug, Clone)]
pub struct CmaConfiguration {
    pub sigma0: f64,
    pub population_size: usize,   // 0 = auto-compute from dimension
    pub max_generations: usize,
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    // Optional tuning — None uses Hansen's auto formula
    pub cc: Option<f64>,
    pub cs: Option<f64>,
    pub c1: Option<f64>,
    pub cmu: Option<f64>,
}

impl Default for CmaConfiguration {
    fn default() -> Self {
        Self {
            sigma0: 0.3,
            population_size: 0,  // auto
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            cc: None, cs: None, c1: None, cmu: None,
        }
    }
}

impl CmaConfiguration {
    pub fn default_for_dim(n: usize) -> Self {
        let lambda = 4 + (3.0 * (n as f64).ln()).floor() as usize;
        Self { population_size: lambda, ..Self::default() }
    }
    // Builder methods: with_sigma0, with_population_size, with_max_generations,
    // with_problem_solving, with_fitness_target, with_cc, with_cs, with_c1, with_cmu
}
```

### Test File Structure (tests/engines/cma/test_cma.rs)

```rust
// Source: tests/engines/de/test_de.rs pattern
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::configuration::ProblemSolving;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn random_pop(n: usize, dim: usize) -> Vec<RangeChromosome<f64>> { ... }

#[test]
fn test_cma_sphere_converges() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_fitness_target(1e-6)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5), sphere);
    let result = engine.run();
    assert!(result.best_fitness < 1.0, "CMA-ES should reduce sphere fitness; got {}", result.best_fitness);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `DeGene` trait in `src/engines/de/gene.rs` | `RealGene` trait in `src/traits/real_gene.rs` | Phase 56 (v3.0.0) | All engines needing real arithmetic share one trait |
| No CMA-ES engine in library | `CmaEngine<U>` in `src/engines/cma/` | Phase 56 (v3.0.0) | Library covers main real-valued black-box optimization algorithm |
| 12 engines listed in lib.rs | 13 engines (+ `CmaEngine`) | Phase 56 | Update engine count in lib.rs top-level doc comment |

**Deprecated/outdated:**
- `DeGene`: hard-renamed to `RealGene` in v3.0.0 per D-01. No alias. External users who implemented `DeGene` must update. This is a documented breaking change in the v3.0.0 migration guide.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `rand_distr::StandardNormal` is available as a transitive dependency | Standard Stack | Superseded by Q2 RESOLVED: use Box-Muller inline; no `rand_distr` dependency added |
| A2 | Jacobi iteration (pure Rust) is the eigendecomposition approach | Architecture Patterns | RESOLVED via Q1 — Jacobi confirmed |
| A3 | `RealGene` moves to `src/traits/real_gene.rs` | Breaking Change Cascade | If kept in `src/engines/de/gene.rs`, all import paths differ but behavior identical |
| A4 | `MultiRangeGenotype<f64>` needs `RealGene` impl added in this phase | Breaking Change Cascade | If deferred, `MultiRangeChromosome<f64>` won't work with `CmaEngine` out-of-box |
| A5 | CMA-ES formulas from Hansen arXiv:1604.00772 | Architecture Patterns | Formulas are well-established; verification against the paper is straightforward |
| A6 | Eigendecomposition interval: `max(1, floor(n^1.5 * 10 / lambda))` | Architecture Patterns | Hansen arXiv:1604.00772 canonical scheduling; off-by-one in scheduling won't affect correctness |

---

## Open Questions (RESOLVED)

1. **Eigendecomposition approach: Jacobi vs Cholesky-CMA** — **RESOLVED: Jacobi iteration**
   - What we know: Both are valid. Jacobi is standard CMA-ES; Cholesky-CMA is a recognized variant.
   - Resolution: Use Jacobi iteration — closer to the reference algorithm, 60 lines of safe Rust, sufficient for typical n ≤ 100. No prior eigendecomposition in codebase, so we follow the reference algorithm directly. Codified in Plan 03 Task 1.

2. **`rand_distr` availability** — **RESOLVED: Box-Muller inline (rand_distr absent, no new dep needed)**
   - What we know: `rand` is in the workspace but `rand_distr` is NOT a direct workspace dependency (verified by inspecting `Cargo.toml` — only `rand` is listed).
   - Resolution: Use Box-Muller transform inline (2 lines per sample) rather than adding `rand_distr` as a new dependency. This avoids a new external crate, keeps the WASM compatibility surface unchanged, and matches the "no new dependencies" goal of this phase. Codified in Plan 03 Task 1 — `standard_normal()` helper.

3. **Example naming (D-07 left to planner)** — **RESOLVED: `cma_es_rastrigin`**
   - Resolution: `examples/cma_es_rastrigin.rs` — demonstrates CMA-ES vs GA on a multimodal benchmark; directly shows the library's new capability. Codified in Plan 04.

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies identified. This phase adds a new Rust source module and renames an existing trait. No CLI tools, databases, or runtime services are required.

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treating as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | none (uses `Cargo.toml` test settings) |
| Quick run command | `cargo test engines::cma` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CMA-01 | `CmaEngine` converges on sphere function (5D) | integration | `cargo test test_cma_sphere_converges` | ❌ Wave 0 |
| CMA-02 | `CmaEngine` early-stops when `fitness_target` reached | integration | `cargo test test_cma_early_stopping` | ❌ Wave 0 |
| CMA-03 | `CmaConfiguration::default_for_dim(n)` computes λ correctly | unit | `cargo test test_cma_default_for_dim` | ❌ Wave 0 |
| CMA-04 | `CmaResult` fields populated correctly (population, best, generations) | unit | `cargo test test_cma_result_fields` | ❌ Wave 0 |
| CMA-05 | `GaObserver` `on_new_best` fires when fitness improves | integration | `cargo test test_cma_observer_new_best` | ❌ Wave 0 |
| CMA-06 | `GaObserver` `on_run_start`/`on_run_end` fire once each | integration | `cargo test test_cma_observer_lifecycle` | ❌ Wave 0 |
| CMA-07 | `DeEngine` still compiles and passes tests after `DeGene → RealGene` rename | regression | `cargo test engines::de` | ❌ Wave 0 (rename cascade) |
| CMA-08 | `ScatterEngine` still compiles and passes tests after rename | regression | `cargo test engines::scatter` | ❌ Wave 0 (rename cascade) |
| CMA-09 | `cargo check --target wasm32-unknown-unknown` passes | WASM | `cargo check --target wasm32-unknown-unknown` | ❌ Wave 0 |
| CMA-10 | `Range<f64>` implements `RealGene` (replaces `DeGene` impl) | unit | `cargo test test_real_gene_range_f64` | ❌ Wave 0 |
| CMA-11 | Maximization mode converges (problem_solving = Maximization) | integration | `cargo test test_cma_maximization` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test engines::cma`
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** Full suite green + `cargo clippy` zero warnings + `cargo check --target wasm32-unknown-unknown` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/cma/test_cma.rs` — covers CMA-01 through CMA-11
- [ ] `tests/test_engines.rs` — add `mod cma { mod test_cma; }` entry
- [ ] WASM target check: `rustup target add wasm32-unknown-unknown` if not present

---

## Security Domain

This phase adds a pure algorithmic Rust library engine. There are no network calls, no user input parsing, no authentication, no cryptography, no web endpoints, and no untrusted data deserialization introduced. ASVS categories are not applicable.

`security_enforcement` key is absent from config — treating as enabled, but no ASVS categories apply to a numerical optimization algorithm implementation.

---

## Sources

### Primary (HIGH confidence)
- `src/engines/de/engine.rs` — DeEngine pattern: struct layout, `new()`, `run()`, helpers [VERIFIED: codebase]
- `src/engines/de/gene.rs` — DeGene definition and Range<f64> impl to rename [VERIFIED: codebase]
- `src/engines/de/configuration.rs` — Configuration pattern: Default, builder methods [VERIFIED: codebase]
- `src/engines/gp/engine.rs` — Observer wiring pattern: `notify()`, WASM-gated `Instant`, `TerminationCause` [VERIFIED: codebase]
- `src/observe/observer/mod.rs` — Full `GaObserver<U>` trait: 12 hook signatures [VERIFIED: codebase]
- `src/stats.rs` — `GenerationStats::from_fitness_values()` factory [VERIFIED: codebase]
- `src/traits/linear_chromosome.rs` — `LinearChromosome` trait: `dna()`, `set_dna()`, `dna_mut()` [VERIFIED: codebase]
- `src/lib.rs` — Module re-export pattern: `#[path]` aliases for engine modules [VERIFIED: codebase]
- `tests/engines/de/test_de.rs` — DE test pattern: convergence tests, helper functions [VERIFIED: codebase]
- `tests/test_engines.rs` — Test module hierarchy for new engines [VERIFIED: codebase]

### Secondary (MEDIUM confidence)
- Hansen, N. "The CMA Evolution Strategy: A Tutorial." arXiv:1604.00772 — all default parameter formulas (λ, μ, weights, cs, ds, cc, c1, cmu, eigendecomposition interval) [CITED: arxiv.org/abs/1604.00772]

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — no new dependencies; all existing
- Architecture: HIGH — DeEngine + GpGa patterns are fully verified from codebase
- CMA-ES algorithm formulas: MEDIUM — derived from Hansen's tutorial; transcription should be verified line-by-line during implementation
- Pitfalls: HIGH — derived from codebase inspection and well-known CMA-ES numerics

**Research date:** 2026-06-01
**Valid until:** 2026-09-01 (algorithm is stable; formulas don't change)
