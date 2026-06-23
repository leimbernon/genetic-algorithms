# Phase 69: Build-perf M3 — Major Refactors - Pattern Map

**Mapped:** 2026-06-15
**Files analyzed:** 44 (13 bench files, 17 rayon-gating files, 11 ga submodules, Cargo.toml, feature-matrix.yml, lib.rs, docs)
**Analogs found:** 44 / 44

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `benches/metrics_observer.rs` | bench | request-response | `benches/metrics_observer.rs` (self — low complexity criterion) | self |
| `benches/de.rs` | bench | batch | `benches/de.rs` (self — low, feature-gated) | self |
| `benches/scatter.rs` | bench | batch | `benches/scatter.rs` (self — low) | self |
| `benches/alps.rs` | bench | batch | `benches/alps.rs` (self — low, loop-over-params) | self |
| `benches/cellular.rs` | bench | batch | `benches/alps.rs` (same pattern: `sample_size(10)`, `BenchmarkId` loop) | role-match |
| `benches/crossover.rs` | bench | batch | `benches/selection.rs` (parameterised BenchmarkId, PlotConfiguration) | role-match |
| `benches/nsga2.rs` | bench | batch | `benches/selection.rs` (Throughput::Elements, BenchmarkId, PlotConfiguration) | role-match |
| `benches/selection.rs` | bench | batch | `benches/selection.rs` (self — MED: Throughput, BenchmarkId, PlotConfiguration) | self |
| `benches/rastrigin.rs` | bench | batch | `benches/ga_run.rs` (iter_batched, BenchmarkId) | role-match |
| `benches/survivor.rs` | bench | batch | `benches/ga_run.rs` (iter_batched, BenchmarkId, PlotConfiguration) | role-match |
| `benches/mutation.rs` | bench | batch | `benches/ga_run.rs` (iter_batched, many BenchmarkId params, PlotConfiguration) | role-match |
| `benches/ga_run.rs` | bench | batch | `benches/ga_run.rs` (self — HIGH: iter_batched, PlotConfiguration, BatchSize) | self |
| `benches/island_ga.rs` | bench | batch | `benches/ga_run.rs` (most complex setup, iter_batched) | role-match |
| `Cargo.toml` | config | — | `Cargo.toml` (self — logging = ["dep:log"] pattern) | self |
| `.github/workflows/feature-matrix.yml` | config | — | `.github/workflows/feature-matrix.yml` (self — existing matrix entries) | self |
| `src/population.rs` (rayon gating) | utility | CRUD | `src/engines/ga.rs` lines 157–158, 1275–1295 (wasm32 gate → combined gate) | role-match |
| `src/traits/common.rs` (rayon gating) | utility | batch | `src/engines/ga.rs` lines 157–158, 1275–1295 | role-match |
| `src/engines/ga.rs` (rayon gating) | engine | event-driven | `src/engines/gp/engine.rs` lines 43–44, 166–176 (already-gated sites) | exact |
| `src/engines/nsga2/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/nsga3/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/spea2/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/ibea/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/moead/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/sms_emoa/mod.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` lines 43–44, 166–176 | exact |
| `src/engines/gp/engine.rs` (rayon gating) | engine | batch | `src/engines/gp/engine.rs` (self — 3 sites, already gated) | self |
| `src/engines/island/mod.rs` (rayon gating) | engine | batch | `src/engines/eda/engine.rs` lines 344–362 (local-block gate pattern) | exact |
| `src/engines/island/nsga2.rs` (rayon gating) | engine | batch | `src/engines/eda/engine.rs` lines 344–362 (ungated top-level import) | exact |
| `src/engines/eda/engine.rs` (rayon gating) | engine | batch | `src/engines/eda/engine.rs` (self — already uses block-level gate) | self |
| `src/operations/survivor/fitness.rs` (rayon gating) | operator | batch | `src/operations/survivor/fitness.rs` (self — par_sort_unstable_by pattern) | self |
| `src/operations/survivor/age.rs` (rayon gating) | operator | batch | `src/operations/survivor/fitness.rs` lines 12–13, 46–57 | exact |
| `src/operations/survivor/mu_comma_lambda.rs` (rayon gating) | operator | batch | `src/operations/survivor/fitness.rs` lines 12–13, 46–57 | exact |
| `src/operations/survivor/mu_plus_lambda.rs` (rayon gating) | operator | batch | `src/operations/survivor/fitness.rs` lines 12–13, 46–57 | exact |
| `src/operations/selection/tournament.rs` (rayon gating) | operator | batch | `src/operations/survivor/fitness.rs` lines 12–13, 46–57 | role-match |
| `src/engines/ga/mod.rs` | engine | event-driven | `src/engines/gp/mod.rs` (directory mod with pub use re-exports) | exact |
| `src/engines/ga/lifecycle.rs` | engine | event-driven | `src/engines/nsga2/mod.rs` (init/finalize functions) | role-match |
| `src/engines/ga/generation.rs` | engine | event-driven | `src/engines/nsga2/mod.rs` (per-generation loop body) | role-match |
| `src/engines/ga/adaptive.rs` | engine | event-driven | `src/engines/ga.rs` (scattered within generation loop) | self-split |
| `src/engines/ga/aos.rs` | engine | event-driven | `src/engines/ga.rs` (scattered within generation loop) | self-split |
| `src/engines/ga/extension.rs` | engine | event-driven | `src/engines/ga.rs` (~lines 2200–2280) | self-split |
| `src/engines/ga/cache.rs` | engine | CRUD | `src/engines/ga.rs` (~lines 2480–2682) | self-split |
| `src/engines/ga/batch.rs` | engine | batch | `src/engines/ga.rs` (~lines 2682–2778, fn batch_evaluate) | self-split |
| `src/engines/ga/stats.rs` | utility | batch | `src/stats.rs` (GenerationStats struct) | role-match |
| `src/engines/ga/observer.rs` | utility | event-driven | `src/engines/ga.rs` (notify calls, observer dispatch) | self-split |
| `src/engines/ga/stopping.rs` | utility | request-response | `src/engines/ga.rs` (lines 2778–2857, fn limit_reached) | self-split |
| `src/lib.rs` (#[path] update) | config | — | `src/lib.rs` lines 333–334 (self, one-line update) | self |

---

## Pattern Assignments

### `benches/*.rs` — Criterion → Divan Port

**Analog:** All 13 bench files are analogs for each other. The primary extraction source is the criterion API in the existing files.

#### Low-complexity bench pattern (replaces `criterion_group!` / `criterion_main!`)

**Source:** `benches/de.rs` (LOW complexity, feature-gated with `--features benchmarks`)

Current criterion structure (lines 1–4, 57–58):
```rust
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
// ...
criterion_group!(benches, bench_mutation_strategies);
criterion_main!(benches);
```

Divan replacement:
```rust
fn main() {
    divan::main();
}
```
- Remove all `criterion_group!` / `criterion_main!` macro calls
- Remove `use criterion::{...}` import
- Add `fn main() { divan::main(); }` at end of file
- No changes to `[[bench]]` declarations in Cargo.toml (`harness = false` stays)

#### Simple `b.iter(|| work())` → `bencher.bench(|| work())` pattern

**Source:** `benches/de.rs` lines 41–53 and `benches/scatter.rs` lines 32–45

Current pattern:
```rust
fn bench_mutation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("de_mutation_strategies");
    group.sample_size(10);
    for (name, strategy) in strategies {
        group.bench_with_input(BenchmarkId::new("sphere_5d", name), &strategy, |b, strat| {
            b.iter(|| {
                let config = DeConfiguration::default()/* ... */;
                let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
                engine.run()
            });
        });
    }
    group.finish();
}
```

Divan replacement:
```rust
#[divan::bench(
    args = [("rand1", DeMutationStrategy::Rand1), ("best1", DeMutationStrategy::Best1), ...],
    sample_count = 10
)]
fn bench_mutation_strategies(bencher: divan::Bencher, (name, strategy): (&str, DeMutationStrategy)) {
    bencher.bench(|| {
        let config = DeConfiguration::default()/* ... */;
        let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
        engine.run()
    });
}
```

Note: For iterator-over-enum-variants pattern, use a module-level grouping or multiple `#[divan::bench]` functions if `args` type constraints are awkward. Drop `group.finish()` — no equivalent needed.

#### `sample_size(10)` → `sample_count = 10` pattern

**Source:** `benches/alps.rs` lines 30–31, `benches/de.rs` line 39, `benches/scatter.rs` line 30

Current:
```rust
let mut group = c.benchmark_group("alps_vs_de");
group.sample_size(10);
```

Divan:
```rust
#[divan::bench(sample_count = 10)]
fn bench_alps_vs_de(bencher: divan::Bencher) { ... }
```

#### `plot_config(PlotConfiguration::...)` — DROP

**Source:** `benches/ga_run.rs` line 148, `benches/selection.rs` line 105

```rust
// REMOVE — no divan equivalent; loss of display only, not correctness:
group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
```

Remove the `AxisScale`, `PlotConfiguration` imports. Drop these lines entirely.

#### `iter_batched` → `with_inputs().bench()` pattern

**Source:** `benches/ga_run.rs` lines 167–175 (BatchSize::SmallInput pattern)

Current criterion:
```rust
b.iter_batched(
    || build_ga(ps, gl, mg),         // setup (returns new owned value)
    |mut ga| { let _ = ga.run(); },  // bench (consumes value)
    BatchSize::SmallInput,
);
```

Divan replacement:
```rust
bencher
    .with_inputs(|| build_ga(ps, gl, mg))
    .bench(|_b, mut ga| { let _ = ga.run(); });
```

The setup closure must return a new owned value each call. `BatchSize::SmallInput` semantics are preserved: setup is called once per iteration, bench closure receives the owned value.

#### `BenchmarkId::new(name, param)` parameterised bench → `args` pattern

**Source:** `benches/ga_run.rs` lines 151–177

Current criterion:
```rust
let configs: Vec<(usize, usize, usize)> = vec![(20,10,10), (50,10,10), (100,10,10), ...];
for &(pop_size, gene_len, max_gen) in &configs {
    group.bench_with_input(
        BenchmarkId::new("Ga::run", format!("pop_{}_genes_{}_gen_{}", pop_size, gene_len, max_gen)),
        &(pop_size, gene_len, max_gen),
        |b, &(ps, gl, mg)| {
            b.iter_batched(|| build_ga(ps, gl, mg), |mut ga| { let _ = ga.run(); }, BatchSize::SmallInput);
        },
    );
}
```

Divan replacement:
```rust
#[divan::bench(args = [(20,10,10usize), (50,10,10), (100,10,10), (50,50,10), (50,10,50)])]
fn benchmark_ga_run(bencher: divan::Bencher, (pop_size, gene_len, max_gen): (usize, usize, usize)) {
    bencher
        .with_inputs(|| build_ga(pop_size, gene_len, max_gen))
        .bench(|_b, mut ga| { let _ = ga.run(); });
}
```

#### `throughput(Throughput::Elements(n))` pattern

**Source:** `benches/selection.rs` line 112

Current:
```rust
group.throughput(Throughput::Elements(population_size as u64));
```

Divan: Drop the throughput annotation — divan does not have a direct equivalent to per-benchmark throughput in this version. Loss of elements/second display only; correctness unaffected. Remove `Throughput` from the `use criterion::` import.

#### `metrics_observer` bench (separate CI step)

**Source:** `benches/metrics_observer.rs` lines 1–60

Current criterion (simple `bench_function` + `b.iter(||...)`) pattern at lines 25–57:
```rust
fn bench_metrics_observer_island(c: &mut Criterion) {
    c.bench_function("metrics_observer_island_10gen", |b| {
        b.iter(|| {
            // setup + run inline (no batched setup needed)
            let observer = Arc::new(MetricsObserver::new("bench_run"));
            // ... build island_ga ...
            let _ = island_ga.run();
        });
    });
}
```

Divan replacement (no `with_inputs` needed since setup is cheap):
```rust
#[divan::bench]
fn bench_metrics_observer_island(bencher: divan::Bencher) {
    bencher.bench(|| {
        let observer = Arc::new(MetricsObserver::new("bench_run"));
        // ... build island_ga ...
        let _ = island_ga.run();
    });
}
```

---

### `Cargo.toml` — `parallel` feature addition

**Analog:** `Cargo.toml` itself — the `logging = ["dep:log"]` pattern at line 34.

**`logging` feature (Phase 68 pattern, lines 33–34):**
```toml
[features]
default = ["logging"]
logging = ["dep:log"]
```

**New `parallel` feature (identical `dep:` pattern):**
```toml
[features]
default = ["logging", "parallel"]    # add "parallel" to default
logging = ["dep:log"]
parallel = ["dep:rayon"]             # NEW — same dep: prefix pattern as logging

[dependencies]
# Change rayon from:
rayon = "1.10"
# To:
rayon = { version = "1.10", optional = true }   # add optional = true

[dev-dependencies]
# During plan 69-01: add divan alongside criterion
divan = "0.1.21"                     # NEW
criterion = "0.8.2"                  # remove after all benches ported
```

---

### `.github/workflows/feature-matrix.yml` — Add `parallel-off` combination + grep step

**Analog:** `.github/workflows/feature-matrix.yml` itself — the `logging-explicit` matrix entry (lines 43–45) and `no-default-features` entry (lines 41–42).

**Existing pattern (lines 43–45) to copy:**
```yaml
- name: "logging-explicit"
  features: "--no-default-features --features logging"
  cmd: "cargo test --quiet --no-default-features --features logging"
```

**New matrix entry (same pattern, parallel disabled):**
```yaml
- name: "parallel-off"
  features: "--no-default-features --features logging"
  cmd: "cargo test --quiet --no-default-features --features logging"
```

**New grep enforcement step — add after matrix `run` step:**
```yaml
- name: Enforce no unconditional rayon references
  run: |
    if grep -rn 'rayon::' src/ | grep -v '#\[cfg'; then
      echo "ERROR: unconditional rayon:: reference found in src/ — all rayon call-sites must be cfg-gated"
      exit 1
    fi
```

This step must come AFTER the compile step so compile errors fail first (D-13 + specifics).

---

### Rayon gating — all 17 files

Three distinct existing sub-patterns require different treatment. All converge to the same canonical gate (D-06).

#### Pattern A: Top-level import + individually-gated call-sites (13 files)

**Analog:** `src/engines/gp/engine.rs` lines 43–44 (import) and 166–176 (call-site pair).

**Before (current wasm32-only gate):**
```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```
```rust
#[cfg(not(target_arch = "wasm32"))]
let result: Vec<_> = items.par_iter().map(process).collect();
#[cfg(target_arch = "wasm32")]
let result: Vec<_> = items.iter().map(process).collect();
```

**After (combined gate — D-06):**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;
```
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
let result: Vec<_> = items.par_iter().map(process).collect();
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
let result: Vec<_> = items.iter().map(process).collect();
```

Files using Pattern A:
- `src/engines/ga.rs` (import line 157; 5 call-site pairs)
- `src/engines/nsga2/mod.rs` (import line 123–124; 4 call-site pairs)
- `src/engines/nsga3/mod.rs` (import + 2 sites)
- `src/engines/spea2/mod.rs` (import + 2 sites)
- `src/engines/ibea/mod.rs` (import + 2 sites)
- `src/engines/moead/mod.rs` (import + 1 site)
- `src/engines/sms_emoa/mod.rs` (import + 1 site)
- `src/engines/gp/engine.rs` (import line 43–44; 3 site pairs)
- `src/operations/survivor/age.rs` (import + 1 site)
- `src/operations/survivor/fitness.rs` (import lines 12–13; 2 site pairs — see Pattern A-sort below)
- `src/operations/survivor/mu_comma_lambda.rs` (import + 2 sites)
- `src/operations/survivor/mu_plus_lambda.rs` (import + 2 sites)
- `src/operations/selection/tournament.rs` (import + 1 site)

#### Pattern A-sort: `par_sort_unstable_by` sites (subset of Pattern A)

**Analog:** `src/operations/survivor/fitness.rs` lines 12–13, 46–57.

**Before:**
```rust
#[cfg(not(target_arch = "wasm32"))]
chromosomes.par_sort_unstable_by(|a, b| {
    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
});
#[cfg(target_arch = "wasm32")]
chromosomes.sort_unstable_by(|a, b| {
    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
});
```

**After:**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| {
    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
});
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| {
    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
});
```

Applied to all 4 survivor files and any other files using `par_sort_unstable_by`.

#### Pattern B: Ungated top-level import (3 highest-risk files)

**Analog:** `src/population.rs` lines 26, 135 (current ungated state — this IS the problem to fix).

**Before (BUGGY — no cfg gate):**
```rust
use rayon::prelude::*;                // line 26 — ungated!
// ...
self.chromosomes.par_iter_mut().for_each(|chromosome| {   // line 135 — ungated!
```

**After (combined gate applied to both):**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;
// ...
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
self.chromosomes.par_iter_mut().for_each(|chromosome| {
    if chromosome.fitness().is_nan() { chromosome.calculate_fitness(); }
});
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
self.chromosomes.iter_mut().for_each(|chromosome| {
    if chromosome.fitness().is_nan() { chromosome.calculate_fitness(); }
});
```

Files using Pattern B:
- `src/population.rs` (par_iter_mut at line 135)
- `src/traits/common.rs` (into_par_iter at line 148)
- `src/engines/island/nsga2.rs` (top-level import line 57 + function-level imports)

#### Pattern C: Block-level local import inside `#[cfg]` block

**Analog:** `src/engines/eda/engine.rs` lines 344–362.

**Before (cfg block contains local import):**
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    use rayon::prelude::*;
    let fitness_fn = Arc::clone(&self.fitness_fn);
    let fitnesses: Vec<f64> = new_pop.par_iter().map(|ind| fitness_fn(ind.dna())).collect();
    // ...
}
#[cfg(target_arch = "wasm32")]
{
    for ind in &mut new_pop { /* sequential */ }
}
```

**After (update cfg attribute only — block content unchanged):**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
{
    use rayon::prelude::*;
    let fitness_fn = Arc::clone(&self.fitness_fn);
    let fitnesses: Vec<f64> = new_pop.par_iter().map(|ind| fitness_fn(ind.dna())).collect();
    // ...
}
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
{
    for ind in &mut new_pop { /* sequential */ }
}
```

Files using Pattern C:
- `src/engines/eda/engine.rs` (lines 344–362 and 670–682 — update 2 block pairs)
- `src/engines/island/mod.rs` (line 511 local `use rayon::prelude::*;` inside fn body — wrap in combined-gate block)

---

### `src/engines/ga/mod.rs` — orchestrator after split

**Analog:** `src/engines/gp/mod.rs` (full file, 35 lines).

**Module declaration + re-export pattern:**
```rust
// src/engines/ga/mod.rs
// mod.rs contains: Ga<U> struct def, all impl ConfigurationT/SelectionConfig/… builder blocks,
// with_* methods, build(), run(), run_with_callback() orchestrator, stats(), hall_of_fame(),
// notify() — plus submodule declarations and pub(crate) re-exports for items used outside ga/

pub mod lifecycle;
pub(crate) mod generation;
pub(crate) mod adaptive;
pub(crate) mod aos;
pub(crate) mod extension;
pub(crate) mod cache;
pub(crate) mod batch;
pub(crate) mod stats;
pub(crate) mod observer;
pub(crate) mod stopping;

// Re-export the public Ga type at crate::ga::Ga (unchanged path):
pub use crate::ga::Ga;   // no change needed if Ga struct stays in mod.rs
```

The gp/mod.rs pattern of `pub use submodule::TypeName;` applies for any sub-items that external callers currently reach through `crate::ga::*`.

#### Visibility rules (D-11):

- Items accessed only by `mod.rs`: `pub(super)` — visible to parent mod.rs
- Items accessed by 2+ sibling submodules within `ga/`: `pub(crate)` — e.g., `batch_evaluate`, cache helpers, `limit_reached`
- Currently-public items (`Ga`, `run`, `build`, `stats`, `hall_of_fame`): remain `pub`

**Warning (Pitfall 4):** `pub(super)` in a submodule (e.g., `batch.rs`) is NOT visible to sibling submodules (`lifecycle.rs`). Use `pub(crate)` for any item that two or more ga submodules need. Reserve `pub(super)` only for items exclusively used by `mod.rs`.

---

### `src/lib.rs` — `#[path]` update

**Analog:** `src/lib.rs` lines 333–334 (self — one-line change).

**Before:**
```rust
#[path = "engines/ga.rs"]
pub mod ga;
```

**After:**
```rust
#[path = "engines/ga/mod.rs"]
pub mod ga;
```

This must be committed in the SAME commit that creates `src/engines/ga/mod.rs` and deletes `src/engines/ga.rs`. Never leave an intermediate state where both exist.

Other `#[path]` declarations in `src/lib.rs` (lines 351–362) show the established pattern for directory-based engine modules:
```rust
#[path = "engines/alps/mod.rs"]
pub mod alps;
#[path = "engines/cellular/mod.rs"]
pub mod cellular;
```
The updated ga declaration follows this exact same convention.

---

### `src/engines/ga/` submodule files (lifecycle, generation, adaptive, aos, extension, cache, batch, stats, observer, stopping)

**Analog:** `src/engines/nsga2/mod.rs` (multi-function engine with sub-helpers), `src/engines/gp/engine.rs` (split-friendly, single-function per conceptual role).

**File header pattern** (copy from any existing engine submodule):
```rust
//! [Short description of what this submodule owns]
//!
//! Extracted from `src/engines/ga.rs` in phase 69-04.

use super::*;    // or explicit imports of needed types from mod.rs via use super::Ga etc.
```

**Visibility rule for extracted free functions** (e.g., `stopping.rs` containing `limit_reached`):
```rust
// limit_reached is called from generation.rs AND potentially mod.rs:
pub(crate) fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &[U]) -> bool
where U: ChromosomeT { ... }
```

**Visibility rule for methods staying on `impl Ga<U>` block moved to a submodule:**
```rust
// In generation.rs — impl block continues; visibility of individual methods unchanged:
impl<U> Ga<U> where U: ... {
    pub(crate) fn run_generation(&mut self, ...) -> Result<(), GaError> { ... }
}
```

**Extraction order (dependency-first):**
1. `stopping.rs` — `limit_reached` is pure fn, no ga-internal deps
2. `cache.rs` — fitness cache helpers, external types only
3. `stats.rs` — GenerationStats collection, external types only
4. `observer.rs` — `notify` dispatcher, depends on GaObserver trait only
5. `batch.rs` — `batch_evaluate` helper, depends on cache.rs
6. `adaptive.rs` — probability recomputation, depends on configuration only
7. `aos.rs` — AOS credit/reward, depends on configuration + adaptive.rs
8. `extension.rs` — extension trigger, depends on external extension traits
9. `lifecycle.rs` — init functions, depends on cache.rs, batch.rs
10. `generation.rs` — per-gen loop, depends on all above submodules
11. `mod.rs` — everything remaining (Ga struct, builders, run, initialization call)

---

## Shared Patterns

### Feature `dep:` prefix in Cargo.toml
**Source:** `Cargo.toml` line 34
**Apply to:** Plan 69-02 (`parallel = ["dep:rayon"]` addition) and any future optional dependency.
```toml
logging = ["dep:log"]
parallel = ["dep:rayon"]   # same pattern
```

### Canonical rayon combined gate
**Source:** Decision D-06 (locked) — applies to every rayon call-site in all 17 files.
```rust
// Import — top-level:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

// Call-site pair (par_iter / iter):
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
let result: Vec<_> = items.par_iter().map(process).collect();
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
let result: Vec<_> = items.iter().map(process).collect();

// Sort pair (par_sort / sort):
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| cmp(a, b));
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| cmp(a, b));
```

**DO NOT TOUCH:** Non-rayon `#[cfg(not(target_arch = "wasm32"))]` gates for `Instant::now()` or other stdlib calls (D-02). Only rayon call-sites get the combined gate.

### Module declaration with pub use re-exports
**Source:** `src/engines/gp/mod.rs` lines 18–34
**Apply to:** `src/engines/ga/mod.rs` — list all submodules then `pub use` any items that must remain reachable at `crate::ga::*`
```rust
pub(crate) mod stopping;
// ... other submodule declarations ...
pub use self::SomePublicType;   // only if currently public
```

### `#[cfg(not(tarpaulin_include))]` on bench functions
**Source:** `benches/ga_run.rs` lines 99, 122, 145 (existing pattern)
**Apply to:** All ported bench files — preserve `#[cfg(not(tarpaulin_include))]` on setup helpers and bench functions where present in the original.

### Feature-isolated bench CI step
**Source:** `.github/workflows/feature-matrix.yml` lines 29–33 (de bench) and lines 122–126 in Cargo.toml (metrics_observer `required-features`)
**Apply to:** `metrics_observer` bench — keep as separate CI step with `--features observer-metrics`, same as `de` bench uses `--features benchmarks`. Both have `harness = false` in Cargo.toml `[[bench]]` entries — this does NOT change when porting to divan.

---

## No Analog Found

All files have close matches. No new patterns are required from RESEARCH.md alone.

| File | Role | Reason |
|------|------|--------|
| `docs/benchmarks.md` | doc | Update invocation snippets — content-only change, no code pattern needed |
| `docs/ARCHITECTURE.md` | doc | Update module map for ga/ split — content-only change |
| `CHANGELOG.md` | doc | Append new entries — follow existing CHANGELOG format in file |
| `CLAUDE.md` | doc | Update "WASM Compatibility" section — text update, pattern is D-06 canonical gate above |
| `.planning/intel/bench-harness.md` | intel | New AI-readable rationale file — no code pattern |
| `.planning/intel/parallel-feature.md` | intel | New AI-readable rationale file — no code pattern |
| `.planning/intel/ga-internals.md` | intel | New AI-readable rationale file — no code pattern |

---

## Metadata

**Analog search scope:** `src/engines/`, `src/operations/`, `src/traits/`, `src/`, `benches/`, `.github/workflows/`, `Cargo.toml`
**Files read:** 22 source files + 2 workflow files + Cargo.toml
**Pattern extraction date:** 2026-06-15
