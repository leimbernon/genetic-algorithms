# Phase 61: Performance — Clone Reduction & Parallel Survivor - Research

**Researched:** 2026-06-08
**Domain:** Rust performance optimization — clone elimination, rayon parallel sort, Criterion benchmarking
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Clone Reduction — Crossover Fallback**
- D-01: Primary target is the crossover fallback at `src/engines/ga.rs` lines 2915-2917. When `crossover_probability > effective_crossover_prob` (no crossover fires), code currently clones `parent_1` and `parent_2` as fallback children. Replace by taking ownership of the parent chromosomes from the couple, eliminating both clones.
- D-02: Selection output collect at line 3091 (`indices.iter().map(|&i| chromosomes[i].clone()).collect()`) is NOT in scope.

**GaObserver Callback Signature**
- D-03: All `GaObserver<U>` callbacks that currently accept `U` (owned chromosome) are changed to `&U` (reference) uniformly across the trait. Breaking change — acceptable under v3.0.0.
- D-04: Change applies to ALL observer callbacks uniformly.
- D-05: All built-in observer implementations (`LogObserver`, `TracingObserver`, `MetricsObserver`, `CompositeObserver`) must be updated to match new `&U` signatures.

**Parallel Survivor Selection**
- D-06: Use `par_sort_unstable_by` (not score-precompute + sequential sort).
- D-07: Operators that receive `par_sort_unstable_by`: `fitness.rs` (both branches), `mu_plus_lambda.rs` (both branches), `age.rs` (`sort_by_key`), `mu_comma_lambda.rs` (the age==0 sub-vec sort only).
- D-08: `DeterministicCrowding` is explicitly excluded.
- D-09: All `par_sort_unstable_by` calls gated behind `#[cfg(not(target_arch = "wasm32"))]` with sequential `sort_unstable_by` fallback.

**Benchmark Harness**
- D-10: Create `benches/rastrigin.rs` as a new dedicated benchmark file. Do NOT add to `benches/ga_run.rs`.
- D-11: Use `RangeChromosome<f64>` with bounds `[-5.12, 5.12]` per gene. Dimensionalities: 10, 20, 50. Population size: 500 for all.
- D-12: Rastrigin fitness: `f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))` where `A=10`. Implemented inline.
- D-13: Benchmark run before and after changes to confirm ≥10% wall-time reduction.

### Claude's Discretion
- Whether `use rayon::prelude::*` is added to each survivor file or imported at the call site
- Internal variable name for the captured `fitness_target` in the parallel sort comparator closure
- Whether the benchmark uses `BatchSize::SmallInput` or `BatchSize::LargeInput`
- Exact `max_generations` parameter for rastrigin bench

### Deferred Ideas (OUT OF SCOPE)
- Selection output collect (line 3091) clone reduction
- `DeterministicCrowding` parallelism
- Observer async support
- Surrogate-assisted evaluation, batch fitness evaluation
</user_constraints>

---

## Summary

Phase 61 is a targeted performance optimization with three non-overlapping work streams: (1) eliminate two `clone()` calls in the crossover fallback path of `ga.rs`, (2) change `GaObserver<U>::on_new_best` from owned `U` to `&U` and propagate to all implementations, and (3) apply `par_sort_unstable_by` to four sort-based survivor operators behind WASM gates.

The code is already well-understood from the context session. All target sites have been located precisely. Rayon is already a dependency and already imported in `ga.rs`. The survivor files currently have no rayon import — each needs `#[cfg(not(target_arch = "wasm32"))] use rayon::prelude::*;` added. The `notify` helper in `ga.rs` uses a `FnOnce` closure — changing `on_new_best` to `&U` requires updating the closure at the single call site (`line 2285`) to pass a reference instead of a clone.

The benchmark harness needs `benches/rastrigin.rs` + a new `[[bench]]` entry in `Cargo.toml`. The existing `benches/ga_run.rs` provides the exact pattern to follow for `iter_batched`, `BenchmarkId`, and `BatchSize`.

**Primary recommendation:** Three independent work tracks (crossover-fallback clone, observer signature, survivor parallelism) + one new file (rastrigin bench). Execute in this order: benchmark first (establishes baseline), then clone reduction + parallel sort (both measurable), then observer signature change (breaking, touches most files but is mechanical). CI gates: `cargo test`, `cargo test --features serde`, `cargo clippy`, `cargo check --target wasm32-unknown-unknown`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Clone elimination (crossover fallback) | GA engine (`ga.rs`) | — | The `process_pair` closure owns the parent references; restructuring happens there |
| Observer signature change (`&U`) | Observer trait (`observer/mod.rs`) | All GA engines + impls | Breaking change originates in trait; engines are call sites |
| Parallel survivor sort | Survivor operators (`operations/survivor/`) | — | Sort lives entirely in the operator functions |
| Benchmark harness | `benches/rastrigin.rs` | `Cargo.toml` | New bench file + registry entry |

---

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rayon` | 1.10 [VERIFIED: Cargo.toml] | `par_sort_unstable_by` in survivor ops | Already a project dependency; used in `ga.rs` for par crossover |
| `criterion` | 0.8.2 [VERIFIED: Cargo.toml] | Benchmark harness for `benches/rastrigin.rs` | Existing bench framework for this project |

### No New Dependencies

Rayon is already imported in `ga.rs` with `#[cfg(not(target_arch = "wasm32"))] use rayon::prelude::*;`. The survivor files need the same import added — no `Cargo.toml` dependency changes required. [VERIFIED: Cargo.toml line 42]

**Installation:** None required.

---

## Package Legitimacy Audit

No new packages are introduced in this phase — only existing dependencies (`rayon` 1.10, `criterion` 0.8.2) are used. No audit required.

---

## Architecture Patterns

### System Architecture Diagram

```
GA::run() per-generation loop
├── crossover inner loop (process_pair closure, rayon par_iter)
│   ├── [CURRENT]  else branch: child_1 = parent_1.clone(); child_2 = parent_2.clone()
│   └── [TARGET]   else branch: take parent_1/parent_2 by value (no clone)
│
├── on_new_best notification
│   ├── [CURRENT]  self.notify(|obs| obs.on_new_best(i, self.population.best_chromosome.clone()))
│   └── [TARGET]   self.notify(|obs| obs.on_new_best(i, &self.population.best_chromosome))
│
└── notify_stats observer call
    ├── [CURRENT]  let notify_stats = self.stats.last().unwrap().clone()   [GenerationStats, not U]
    └── [NOTE]     GenerationStats clone stays — it's not a chromosome clone; already &GenerationStats

Survivor operator functions (called from ga.rs dispatcher)
├── fitness_based()       sort_by → par_sort_unstable_by (both branches)
├── mu_plus_lambda()      sort_by → par_sort_unstable_by (both branches)
├── age_based()           sort_by_key → par_sort_unstable_by (equivalent)
└── mu_comma_lambda()     sort_by on age==0 sub-vec → par_sort_unstable_by

GaObserver<U> trait
├── on_new_best(&self, generation: usize, best: U)  →  best: &U
└── propagates to: LogObserver, CompositeObserver (fan-out clone also removed)
    TracingObserver [feature-gated], MetricsObserver [feature-gated]

Benchmark
└── benches/rastrigin.rs  (new)
    ├── RangeChromosome<f64>, dims=[10,20,50], pop=500
    └── Criterion::iter_batched / BatchSize::SmallInput pattern from ga_run.rs
```

### Recommended Project Structure

No structural changes. All modifications are in-place edits to existing files, plus one new bench file:
```
benches/
├── ga_run.rs          (unchanged — reference pattern only)
└── rastrigin.rs       (NEW — dedicated Rastrigin benchmark)
src/
├── engines/ga.rs      (crossover fallback clone, on_new_best call site)
├── observe/observer/
│   ├── mod.rs         (GaObserver trait: on_new_best U → &U)
│   ├── log.rs         (LogObserver impl: on_new_best signature)
│   ├── composite.rs   (CompositeObserver: on_new_best fan-out clone removed)
│   ├── tracing_observer.rs  [feature-gated]
│   └── metrics_observer.rs  [feature-gated]
└── operations/survivor/
    ├── fitness.rs      (par_sort_unstable_by + wasm gate + rayon import)
    ├── mu_plus_lambda.rs  (same)
    ├── age.rs          (same)
    └── mu_comma_lambda.rs (same)
Cargo.toml              (new [[bench]] entry for rastrigin)
```

### Pattern 1: WASM-Gated Parallel Sort in Survivor Ops

**What:** Duplicate only the sort expression behind cfg gates; keep truncation/drain logic shared.

**When to use:** Any `sort_by` / `sort_by_key` on a `Vec<U>` where `U: ChromosomeT + Send + Sync`.

**Example (fitness.rs — Maximization branch):**
```rust
// Source: CLAUDE.md WASM rules + existing ga.rs pattern
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

// In fitness_based():
#[cfg(not(target_arch = "wasm32"))]
chromosomes.par_sort_unstable_by(|a, b| {
    b.fitness()
        .partial_cmp(&a.fitness())
        .unwrap_or(std::cmp::Ordering::Equal)
});
#[cfg(target_arch = "wasm32")]
chromosomes.sort_unstable_by(|a, b| {
    b.fitness()
        .partial_cmp(&a.fitness())
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

**Note on `sort_by` → `sort_unstable_by`:** The existing `sort_by` calls are being replaced with `sort_unstable_by` (not just `par_sort_unstable_by`) in the WASM fallback too. This is intentional — `sort_unstable_by` is faster than `sort_by` in the sequential path as well (no merge allocation). D-06 says unstable sort is acceptable.

**Note on `age_based`:** Uses `sort_by_key(|a| Reverse(a.age()))`. The parallel equivalent is `par_sort_unstable_by(|a, b| b.age().cmp(&a.age()))` — `par_sort_unstable_by_key` exists in rayon but translating to `par_sort_unstable_by` with explicit comparison is equally valid. Either works; the cmp form avoids a secondary crate lookup.

### Pattern 2: Observer `&U` Signature Change

**What:** Mechanical signature change at the trait definition, call site, and all implementations.

**When to use:** Any observer that currently receives owned `U` — in this codebase only `on_new_best`.

**Example — trait definition:**
```rust
// Source: src/observe/observer/mod.rs (current → target)
// CURRENT:
fn on_new_best(&self, _generation: usize, _best: U) {}
// TARGET:
fn on_new_best(&self, _generation: usize, _best: &U) {}
```

**Example — call site in ga.rs:**
```rust
// Source: src/engines/ga.rs line 2285 (current → target)
// CURRENT:
self.notify(|obs| obs.on_new_best(i, self.population.best_chromosome.clone()));
// TARGET:
self.notify(|obs| obs.on_new_best(i, &self.population.best_chromosome));
```

**Example — CompositeObserver fan-out (clone also removed):**
```rust
// Source: src/observe/observer/composite.rs (current → target)
// CURRENT:
fn on_new_best(&self, generation: usize, best: U) {
    for obs in &self.observers {
        obs.on_new_best(generation, best.clone());  // clone per observer
    }
}
// TARGET:
fn on_new_best(&self, generation: usize, best: &U) {
    for obs in &self.observers {
        obs.on_new_best(generation, best);  // zero-copy fan-out
    }
}
```

**Note:** `LogObserver::on_new_best` currently has signature `fn on_new_best(&self, _generation: usize, _best: U)` and ignores `_best` entirely. The parameter rename to `_best: &U` is a no-op in its body. Same for all other impls that do not use the `best` value. [VERIFIED: log.rs line 111]

### Pattern 3: Crossover Fallback Clone Elimination (D-01)

**What:** The `else` branch of the crossover probability check currently clones both parents. The plan is to take ownership.

**Current code (ga.rs ~line 2914-2917):**
```rust
} else {
    child_1 = parent_1.clone();
    child_2 = parent_2.clone();
}
```

**Key constraint for the planner:** `parent_1` and `parent_2` are `&U` references obtained via `chromosomes.get(key)` and `chromosomes.get(value)`. Taking ownership requires restructuring how parents are accessed. The `process_pair` closure borrows `chromosomes: &[U]` from the outer scope, so direct ownership transfer is not straightforward. The planner must design this carefully — one viable approach is to clone `chromosomes[key]` and `chromosomes[value]` conditionally at the top of the closure (only when the else branch is taken), which avoids the unconditional clone only in the no-crossover path. Another is to defer fitness-fn injection to cover the else-branch children too (they come from the parent which already has the fn).

**Critical insight:** The comment at line 2980-2981 says "Children from `parent.clone()` (the else branch above) already carry the correct fitness fn from their parent." If ownership is taken rather than cloning, the fitness-fn injection block at line 2982-2987 must be updated to handle else-branch children. This is a behavioral correctness concern the planner must address explicitly in Task 1.

**Planner advisory:** D-01 says "taking ownership" but the closure captures `chromosomes: &[U]`. The literal implementation will likely be conditional extraction (clone only when else-branch fires, not unconditionally at top of closure), which still eliminates the always-present clone in favor of a conditional one. The planner should treat this as: "minimize clones in the else-branch path" rather than "zero-copy ownership transfer."

### Pattern 4: Benchmark Harness (benches/rastrigin.rs)

**What:** New criterion benchmark following `ga_run.rs` pattern.

**RangeChromosome<f64> construction:**

The benchmark needs to construct `Range<f64>` chromosomes with `RangeGenotype<f64>` genes. Looking at `ga_run.rs`, the `setup_population` helper creates chromosomes manually. For the rastrigin bench, initialization via `Ga::new().with_initialization_function(...)` is cleaner. Alternatively, direct population construction like `ga_run.rs` avoids the initialization phase overhead.

**Rastrigin fitness inline:**
```rust
// Source: D-12 from CONTEXT.md
fn rastrigin(genes: &[RangeGenotype<f64>]) -> f64 {
    let a = 10.0_f64;
    let n = genes.len() as f64;
    a * n + genes.iter().map(|g| {
        let x = g.value(); // or g.val or however RangeGenotype exposes its value
        x * x - a * (2.0 * std::f64::consts::PI * x).cos()
    }).sum::<f64>()
}
```

**Note:** The planner must verify how `RangeGenotype<f64>` exposes its value — check `src/types/genotypes/range.rs`. The fitness function signature is `Fn(&[RangeGenotype<f64>]) -> f64`.

**Cargo.toml entry:**
```toml
[[bench]]
name = "rastrigin"
harness = false
```

**Generation count recommendation (Claude's discretion):** For pop=500 with dims=10/20/50, a fixed `max_generations = 50` provides enough signal (25,000 fitness evaluations per run at pop=500×50gen) while keeping each benchmark iteration under ~1-2 seconds on modern hardware. Criterion will run multiple iterations automatically.

### Anti-Patterns to Avoid

- **Gating only `par_sort_unstable_by` but leaving `sort_by`:** The WASM fallback must use `sort_unstable_by`, not `sort_by`. Replace both.
- **Adding `use rayon::prelude::*;` unconditionally in survivor files:** Must be `#[cfg(not(target_arch = "wasm32"))] use rayon::prelude::*;`.
- **Forgetting `CompositeObserver` fan-out clone:** `composite.rs` line 150 does `best.clone()` inside the fan-out loop. This is a secondary clone site that D-03/D-05 eliminates alongside the trait signature change.
- **Forgetting `gp.rs` test:** `tests/gp.rs` line 480 has `impl GaObserver<GpChromosome<TestNode>> for StatsCollector`. After the `&U` change, this will fail to compile. The planner must include updating all test observer impls.
- **Forgetting `tests/observe/observer/test_observer.rs`:** `SpyObserver::on_new_best` and `CountingObserver::on_new_best` (lines 79 and ~257) both currently accept `_best: BinaryChromosome`. These must be updated to `_best: &BinaryChromosome`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parallel sort | Custom parallel merge sort | `rayon::slice::ParallelSliceMut::par_sort_unstable_by` | Already a dep; handles thread scheduling, cache effects, work stealing |
| Benchmark measurement | Manual timing loops with `std::time::Instant` | `criterion::iter_batched` | Criterion handles warmup, outlier rejection, statistical confidence |
| WASM detection | Runtime feature flags or env vars | `#[cfg(target_arch = "wasm32")]` | Zero-cost compile-time gate; the established pattern in this codebase |

**Key insight:** `par_sort_unstable_by` on a `Vec<U>` where `U: Send` works out-of-the-box — rayon's parallel sort is in-place on mutable slices. No ownership gymnastics needed beyond the existing `&mut Vec<U>` function signatures.

---

## Common Pitfalls

### Pitfall 1: `par_sort_unstable_by` requires `U: Send`

**What goes wrong:** Compile error: `U cannot be sent between threads safely`.
**Why it happens:** `par_sort_unstable_by` requires the element type to implement `Send`.
**How to avoid:** `ChromosomeT` already requires `U: Send + Sync` in the project's trait bounds. The existing `SurvivorOperator` trait bound `U: ChromosomeT` is sufficient — no additional bounds needed.
**Warning signs:** If a new chromosome type is introduced that is not `Send`, the parallel sort will not compile. This is correct behavior.

### Pitfall 2: `sort_by_key` vs `par_sort_unstable_by` for age_based

**What goes wrong:** Developer tries to use `par_sort_unstable_by_key` with `Reverse(a.age())`.
**Why it happens:** `par_sort_unstable_by_key` is less commonly known; `Reverse` requires the key to implement `Ord`.
**How to avoid:** `usize` implements `Ord`, so `par_sort_unstable_by_key(|a| Reverse(a.age()))` works. Alternatively, use `par_sort_unstable_by(|a, b| b.age().cmp(&a.age()))` which is more explicit. Either is correct.

### Pitfall 3: `CompositeObserver::on_new_best` secondary clone site

**What goes wrong:** Trait signature is updated to `&U` but `composite.rs` still has `best.clone()` inside the fan-out loop, causing a compile error (can't move out of `best: &U`).
**Why it happens:** The clone in `composite.rs` was needed when `best` was owned — each inner observer consumed it. With `&U`, `best` can be passed by reference to all inner observers without cloning.
**How to avoid:** Change `composite.rs::on_new_best` parameter to `best: &U` and remove the `.clone()` inside the loop.

### Pitfall 4: Crossover else-branch fitness-fn injection

**What goes wrong:** After D-01 refactor, children produced in the else-branch don't get the fitness fn injected, causing NaN fitness or panics.
**Why it happens:** The comment at ga.rs line 2980 says "Children from `parent.clone()` already carry the correct fitness fn." If the else-branch now takes ownership differently, this assumption may no longer hold.
**How to avoid:** The fitness-fn injection block at lines 2982-2987 runs unconditionally after both branches. If else-branch children are created by cloning parents (which do carry the fn), the `if let Some(ref ff) = fitness_fn` block re-overwrites with the same fn — harmless. The planner must verify the chosen implementation approach preserves this.

### Pitfall 5: Benchmark `RangeGenotype<f64>` value accessor

**What goes wrong:** Rastrigin fitness function calls a non-existent method on `RangeGenotype<f64>`.
**Why it happens:** The value field may be named `value`, `val`, or accessed via a method. Need to check the struct definition.
**How to avoid:** Read `src/types/genotypes/range.rs` before implementing the benchmark. [ASSUMED — field name not verified in this research session]

### Pitfall 6: Missing `[[bench]]` entry in Cargo.toml

**What goes wrong:** `cargo bench --bench rastrigin` fails with "no benchmark target named rastrigin".
**Why it happens:** Criterion benches require explicit registration in `Cargo.toml`.
**How to avoid:** Add `[[bench]] name = "rastrigin" harness = false` to `Cargo.toml`.

---

## Code Examples

### Parallel Sort — fitness.rs (complete replacement)

```rust
// Source: Based on CLAUDE.md WASM rules + ga.rs existing pattern (lines 157-158, 3026-3029)
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

pub fn fitness_based<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="fitness_based"; "Starting fitness based survivor method");
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        #[cfg(not(target_arch = "wasm32"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(target_arch = "wasm32")]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        let target = limit_configuration.fitness_target.unwrap_or(0.0);
        #[cfg(not(target_arch = "wasm32"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness_distance(&target).partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(target_arch = "wasm32")]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness_distance(&target).partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    // truncate/drain logic unchanged
    // ...
}
```

### Observer Trait Change — mod.rs

```rust
// Source: src/observe/observer/mod.rs (current line 112 → target)
fn on_new_best(&self, _generation: usize, _best: &U) {}
```

### Benchmark Structure — benches/rastrigin.rs

```rust
// Source: Pattern from benches/ga_run.rs (iter_batched, BenchmarkId, criterion_group)
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
// ... chromosome types, setup helpers, rastrigin fn inline ...

fn benchmark_rastrigin(c: &mut Criterion) {
    let mut group = c.benchmark_group("rastrigin");
    let dims: Vec<usize> = vec![10, 20, 50];
    for &dim in &dims {
        group.bench_with_input(
            BenchmarkId::new("Ga::run", format!("pop_500_dim_{}", dim)),
            &dim,
            |b, &d| {
                b.iter_batched(
                    || build_rastrigin_ga(500, d, 50),
                    |mut ga| { let _ = ga.run(); },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(rastrigin_benchmarks; config = Criterion::default(); targets = benchmark_rastrigin);
criterion_main!(rastrigin_benchmarks);
```

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `RangeGenotype<f64>` value field is named `value` or exposed via a `value()` method | Code Examples — rastrigin bench | Bench won't compile; fix by reading `src/types/genotypes/range.rs` before implementing |
| A2 | `par_sort_unstable_by` on `Vec<U: ChromosomeT>` compiles without additional trait bounds | Standard Stack, Code Examples | Would require adding `U: Send` bound (already in ChromosomeT supertrait) |

---

## Open Questions (RESOLVED)

1. **Crossover fallback ownership approach** — **RESOLVED: conditional clone approach accepted (user decision, 2026-06-08).** D-01 is relaxed. The literal zero-copy take-ownership transfer is not required. The acceptable deliverable is that the `parent_1.clone()` / `parent_2.clone()` at ga.rs lines 2916-2917 fire only when the else branch is taken (no-crossover path) and that no unconditional upstream clone of either parent exists earlier in the `process_pair` closure. Audit/verify in Plan 03 Task 2 (see acceptance criteria there: grep gates enforce the conditional structure).
   - What we know: `parent_1` and `parent_2` are `&U` from `chromosomes.get(key/value)`. The `process_pair` closure borrows `chromosomes: &[U]`. Current code already has the clones inside the else branch — audit confirms no upstream clone exists.
   - Original concern (now moot): The precise restructuring needed to eliminate both clones without changing the closure signature or borrowing model.

2. **TracingObserver and MetricsObserver signatures** — **RESOLVED: if the observer body uses `best`, it must clone internally; otherwise the `_best: &U` parameter is trivially renamed (no body change needed beyond the underscore).** The caller in `ga.rs` no longer pays the clone cost — the observer pays only if it needs ownership for internal state. Plan 03 Task 1 handles both files mechanically: change the signature to `&U`, then adapt the body only if it consumed `best`.
   - What we know: Both are feature-gated (`observer-tracing`, `observer-metrics`) and must be updated per D-05.
   - Original concern (now resolved by the rule above): Whether these observers use the `best` parameter in `on_new_best` body.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo bench` (criterion) | Rastrigin benchmark | ✓ | criterion 0.8.2 | — |
| `cargo check --target wasm32-unknown-unknown` | WASM gate verification | ✓ [ASSUMED] | — | CI enforces |
| `rayon` | Parallel sort in survivor ops | ✓ | 1.10 | Sequential (wasm32 path) |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + criterion |
| Config file | none (cargo test discovers automatically) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| — | Observer `&U` signature compiles for all impls | compile | `cargo test` | Update existing tests |
| — | `on_new_best` fires in integration test with new sig | integration | `cargo test -p genetic_algorithms --test observer` | ✅ `tests/observe/observer/test_observer.rs` |
| — | Survivor ops produce same ordered output | unit | `cargo test -p genetic_algorithms` | ✅ existing survivor tests implied by test_ga.rs |
| — | GP test compiles with new observer sig | compile | `cargo test --test gp` | ✅ `tests/gp.rs` |
| — | WASM target check passes | compile | `cargo check --target wasm32-unknown-unknown` | — |
| — | Rastrigin bench compiles and runs | smoke | `cargo bench --bench rastrigin -- --test` | ❌ Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `benches/rastrigin.rs` — new file; covers benchmark success criterion
- [ ] `Cargo.toml` — new `[[bench]]` entry for `rastrigin`
- [ ] Test updates in `tests/observe/observer/test_observer.rs` — `SpyObserver::on_new_best` and `CountingObserver::on_new_best` signatures
- [ ] Test update in `tests/gp.rs` — `StatsCollector::on_new_best` signature

---

## Security Domain

Step 2.6: SKIPPED — this phase has no external-facing inputs, authentication, or data persistence. It is purely an internal performance optimization and API refactor.

---

## Sources

### Primary (HIGH confidence)
- `src/engines/ga.rs` — crossover fallback at lines 2914-2917; `on_new_best` call at line 2285; `notify` helper at line 914; WASM gate pattern at lines 157-158, 3026-3029
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait; `on_new_best` signature at line 112
- `src/observe/observer/composite.rs` — fan-out `on_new_best` with `.clone()` at line 150
- `src/observe/observer/log.rs` — `LogObserver::on_new_best` no-op impl at line 111
- `src/operations/survivor/fitness.rs` — two `sort_by` calls; target for `par_sort_unstable_by`
- `src/operations/survivor/mu_plus_lambda.rs` — two `sort_by` calls; target for `par_sort_unstable_by`
- `src/operations/survivor/age.rs` — `sort_by_key(Reverse(age()))` call
- `src/operations/survivor/mu_comma_lambda.rs` — `sort_by` on age==0 sub-vec
- `benches/ga_run.rs` — `iter_batched`, `BatchSize::SmallInput`, `BenchmarkId` pattern
- `Cargo.toml` — rayon 1.10, criterion 0.8.2, existing `[[bench]]` entries
- `tests/observe/observer/test_observer.rs` — `SpyObserver::on_new_best` at line 79
- `tests/gp.rs` — `StatsCollector` observer impl at line 480

### Secondary (MEDIUM confidence)
- [CLAUDE.md WASM rules] — WASM cfg gate pattern: duplicate iterator expression, share closure body

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified directly from Cargo.toml and source files
- Architecture: HIGH — all call sites located precisely with line numbers
- Pitfalls: HIGH — derived from actual code read in this session
- Benchmark pattern: HIGH — ga_run.rs read in full; pattern is mechanical

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (stable — all Rust/rayon patterns; no external services)
